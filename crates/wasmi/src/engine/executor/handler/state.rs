use crate::{
    collections::HeadVec,
    core::UntypedVal,
    engine::executor::CodeMap,
    errors::HostError,
    instance::InstanceEntity,
    ir,
    ir::{Slot, SlotSpan},
    store::PrunedStore,
    Instance,
    TrapCode,
};
use alloc::{boxed::Box, vec::Vec};
use core::ptr::{self, NonNull};

#[derive(Debug, Default, Copy, Clone)]
pub struct Done {
    _priv: (),
}

pub struct VmState<'vm> {
    pub store: &'vm mut PrunedStore,
    pub frames: &'vm mut CallStack,
    pub stack: &'vm mut ValueStack,
    pub code: &'vm CodeMap,
    pub done_reason: DoneReason,
}

#[derive(Debug)]
pub enum DoneReason {
    Trap(TrapCode),
    OutOfFuel {
        required_fuel: u64,
    },
    Host(Box<dyn HostError>),
    Return,
    Continue {
        ip: Ip,
        sp: Sp,
        mem0: *mut u8,
        mem0_len: usize,
        instance: NonNull<InstanceEntity>,
    },
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Ip {
    value: *const u8,
}

struct IpDecoder(Ip);
impl ir::Decoder for IpDecoder {
    fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), ir::DecodeError> {
        unsafe { ptr::copy_nonoverlapping(self.0.value, buffer.as_mut_ptr(), buffer.len()) };
        Ok(())
    }
}

impl Ip {
    pub unsafe fn decode<T: ir::Decode>(self) -> (Ip, T) {
        let mut ip = IpDecoder(self);
        let decoded = <T as ir::Decode>::decode(&mut ip).unwrap();
        (ip.0, decoded)
    }

    pub unsafe fn offset(self, delta: isize) -> Self {
        let value = unsafe { self.value.byte_offset(delta) };
        Self { value }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Sp {
    value: *mut UntypedVal,
}

impl<'a> From<&'a mut [UntypedVal]> for Sp {
    fn from(value: &'a mut [UntypedVal]) -> Self {
        Self {
            value: value.as_mut_ptr(),
        }
    }
}

impl Sp {
    pub fn get<T>(self, slot: Slot) -> T
    where
        T: From<UntypedVal>,
    {
        let index = usize::from(u16::from(slot));
        let value = unsafe { *self.value.add(index) };
        T::from(value)
    }

    pub fn set<T>(self, slot: Slot, value: T)
    where
        T: Into<UntypedVal>,
    {
        let index = usize::from(u16::from(slot));
        let cell = unsafe { &mut *self.value.add(index) };
        *cell = value.into();
    }
}

pub struct Stack {
    values: ValueStack,
    frames: CallStack,
}

impl Stack {
    pub fn push_frame(
        &mut self,
        ip: Ip,
        params: SlotSpan,
        size: usize,
        state: &VmState,
        instance: Option<Instance>,
    ) -> Result<Sp, TrapCode> {
        let delta = usize::from(u16::from(params.head()));
        let Some(start) = self.frames.top_start().checked_add(delta) else {
            return Err(TrapCode::StackOverflow);
        };
        let sp = self.values.push(start, size)?;
        self.frames.push(ip, start, state, instance)?;
        Ok(sp)
    }

    pub fn pop_frame(
        &mut self,
        instance: NonNull<InstanceEntity>,
    ) -> (Sp, NonNull<InstanceEntity>) {
        let (start, changed_instance) = self.frames.pop();
        let sp = self.values.sp(start);
        if let Some(instance) = changed_instance {
            return (sp, instance);
        }
        (sp, instance)
    }
}

pub struct ValueStack {
    cells: Vec<UntypedVal>,
    max_height: usize,
}

impl ValueStack {
    fn sp(&mut self, start: usize) -> Sp {
        Sp::from(&mut self.cells[start..]) // TODO: maybe avoid bounds check if necessary for performance
    }

    fn push(&mut self, start: usize, size: usize) -> Result<Sp, TrapCode> {
        let Some(end) = start.checked_add(size) else {
            return Err(TrapCode::StackOverflow);
        };
        if end > self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        self.cells.resize(end, UntypedVal::default());
        let sp = self.sp(start);
        Ok(sp)
    }
}

pub struct CallStack {
    frames: Vec<Frame>,
    instances: HeadVec<NonNull<InstanceEntity>>,
    max_height: usize,
}

impl CallStack {
    fn top_start(&self) -> usize {
        let Some(top) = self.top() else { return 0 };
        top.start
    }

    fn top(&self) -> Option<&Frame> {
        self.frames.last()
    }

    fn push(
        &mut self,
        ip: Ip,
        start: usize,
        state: &VmState,
        instance: Option<Instance>,
    ) -> Result<(), TrapCode> {
        if self.frames.len() == self.max_height {
            return Err(TrapCode::StackOverflow);
        }
        let changes_instance = instance.is_some();
        if let Some(instance) = instance {
            let entity = state.store.inner().resolve_instance(&instance);
            self.instances.push(entity.into());
        }
        self.frames.push(Frame {
            ip,
            start,
            changes_instance,
        });
        Ok(())
    }

    fn pop(&mut self) -> (usize, Option<NonNull<InstanceEntity>>) {
        let Some(popped) = self.frames.pop() else {
            panic!("unexpected empty frame stack") // TODO: return `Result` instead of panicking
        };
        let start = self.top_start();
        if popped.changes_instance {
            self.instances.pop().expect("must have an instance if changed");
            // Note: it is expected to return `None` for the instance when the last frame is popped
            //       since that means that the execution is finished anyways. We might even want to expect
            //       this at the caller site.
            let instance = self.instances.last().copied();
            return (start, instance);
        }
        (start, None)
    }
}

pub struct Frame {
    pub ip: Ip,
    start: usize,
    changes_instance: bool,
}
