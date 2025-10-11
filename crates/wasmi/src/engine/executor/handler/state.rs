use crate::{
    core::UntypedVal,
    engine::executor::{
        stack::{CallStack, ValueStack},
        CodeMap,
    },
    errors::HostError,
    instance::InstanceEntity,
    ir,
    ir::Slot,
    store::PrunedStore,
    TrapCode,
};
use alloc::boxed::Box;
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
        required: u64,
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
pub struct Sp {
    value: *mut UntypedVal,
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
