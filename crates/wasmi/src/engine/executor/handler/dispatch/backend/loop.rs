use crate::{
    engine::executor::handler::{
        dispatch::{Break, Control, ExecutionOutcome},
        exec,
        state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
    },
    ir,
    ir::OpCode,
};

#[derive(Debug, Copy, Clone)]
pub struct NextState {
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
}

pub type Done = Control<NextState, Break>;

#[inline(always)]
pub fn control_continue(ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Done {
    Done::Continue(NextState {
        ip,
        sp,
        mem0,
        mem0_len,
        instance,
    })
}

macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let _: &mut VmState = $state;
        return $crate::engine::executor::handler::dispatch::backend::control_continue(
            $ip, $sp, $mem0, $mem0_len, $instance,
        );
    }};
}

pub type Handler = fn(&mut Executor, state: &mut VmState) -> Control<(), Break>;

macro_rules! expand_op_code_to_handler {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        #[inline(always)]
        pub fn op_code_to_handler(code: OpCode) -> Handler {
            match code {
                $( OpCode::$camel_case => Executor::$snake_case, )*
            }
        }
    };
}
ir::for_each_op!(expand_op_code_to_handler);

pub fn execute_until_done(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Result<Sp, ExecutionOutcome> {
    let mut executor = Executor {
        ip,
        sp,
        mem0,
        mem0_len,
        instance,
    };
    executor.execute_until_done(state)
}

#[derive(Debug)]
pub struct Executor {
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
}

impl Executor {
    #[inline(never)]
    fn execute_until_done(&mut self, state: &mut VmState) -> Result<Sp, ExecutionOutcome> {
        let Control::Break(reason) = self.execute_until_break(state);
        Self::handle_break(state, reason)
    }

    #[cold]
    #[inline(never)]
    fn handle_break(state: &mut VmState, reason: Break) -> Result<Sp, ExecutionOutcome> {
        if let Some(trap_code) = reason.trap_code() {
            return Err(ExecutionOutcome::from(trap_code));
        }
        state.execution_outcome()
    }

    #[cold]
    #[inline]
    fn forward_break(reason: Break) -> Control<(), Break> {
        Control::Break(reason)
    }
}

/// Represents the never type `!`.
pub enum Never {}

#[cfg(feature = "indirect-dispatch")]
impl Executor {
    #[inline(always)]
    fn execute_until_break(&mut self, state: &mut VmState) -> Control<Never, Break> {
        macro_rules! impl_body {
            ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
                let mut op_code = super::decode_op_code(self.ip);
                loop {
                    op_code = 'next: {
                        match op_code {
                            $(
                                OpCode::$camel_case => {
                                    self.$snake_case(state)?;
                                    op_code = super::decode_op_code(self.ip);
                                    break 'next op_code
                                },
                            )*
                        }
                    }
                }
            };
        }
        ir::for_each_op! { impl_body }
    }
}

#[cfg(not(feature = "indirect-dispatch"))]
impl Executor {
    fn execute_until_break(&mut self, state: &mut VmState) -> Control<Never, Break> {
        loop {
            self.dispatch_handler(state)?
        }
    }

    #[inline]
    fn dispatch_handler(&mut self, state: &mut VmState) -> Control<(), Break> {
        let handler = super::decode_handler(self.ip);
        handler(self, state)
    }
}

macro_rules! impl_executor_handlers {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        $(
            #[cfg_attr(feature = "indirect-dispatch", inline(always))]
            #[cfg_attr(not(feature = "indirect-dispatch"), inline(never))]
            fn $snake_case(&mut self, state: &mut VmState) -> Control<(), Break> {
                match exec::$snake_case(state, self.ip, self.sp, self.mem0, self.mem0_len, self.instance) {
                    Done::Continue(NextState { ip, sp, mem0, mem0_len, instance }) => {
                        self.ip = ip;
                        self.sp = sp;
                        self.mem0 = mem0;
                        self.mem0_len = mem0_len;
                        self.instance = instance;
                        Control::Continue(())
                    }
                    Done::Break(reason) => Self::forward_break(reason),
                }
            }
        )*
    };
}
impl Executor {
    ir::for_each_op!(impl_executor_handlers);
}
