use crate::{
    engine::executor::handler::{
        dispatch::{Break, Control, ExecutionOutcome},
        exec,
        state::{Freg32, Freg64, Inst, Ip, Ireg, Mem0Len, Mem0Ptr, Sp, VmState},
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
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
}

pub type Done = Control<NextState, Break>;

#[inline(always)]
#[expect(clippy::too_many_arguments)]
pub fn control_continue(
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> Done {
    Done::Continue(NextState {
        ip,
        sp,
        mem0,
        mem0_len,
        instance,
        ireg,
        freg32,
        freg64,
    })
}

macro_rules! dispatch {
    ( $state:expr, $args:expr $(,)? ) => {{
        let _: &mut VmState = $state;
        let (ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64) = $args.into_parts();
        return $crate::engine::executor::handler::dispatch::backend::control_continue(
            ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64,
        );
    }};
}

pub type Handler = fn(&mut Executor, state: &mut VmState) -> Control<(), Break>;

macro_rules! expand_op_code_to_handler {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        #[inline(always)]
        pub fn op_code_to_handler(code: OpCode) -> Handler {
            static HANDLERS: [Handler; ir::LEN_OPS] = [
                $( Executor::$snake_case ),*
            ];
            // SAFETY: the `HANDLERS` table has exactly the same size as `LEN_OPS`
            //         which represents the number of [`Op`] and thus [`OpCode`]
            //         variants. Since [`OpCode`] is contiguously defined, all [`OpCode`]
            //         values are represented in the table, thus using their values as
            //         unchecked index into the `HANDLERS` table is safe.
            unsafe { *HANDLERS.get_unchecked(usize::from(u16::from(code))) }
        }
    };
}
ir::for_each_op!(expand_op_code_to_handler);

#[expect(clippy::too_many_arguments)]
pub fn execute_until_done(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> Result<Sp, ExecutionOutcome> {
    let mut executor = Executor {
        ip,
        sp,
        mem0,
        mem0_len,
        instance,
        ireg,
        freg32,
        freg64,
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
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
}

impl Executor {
    #[inline(always)]
    fn execute_until_done(&mut self, state: &mut VmState) -> Result<Sp, ExecutionOutcome> {
        let Control::Break(reason) = self.execute_until_break(state);
        Self::handle_break(state, reason)
    }

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

    #[inline(always)]
    fn dispatch_handler(&mut self, state: &mut VmState) -> Control<(), Break> {
        let handler = super::decode_handler(self.ip);
        handler(self, state)
    }
}

macro_rules! impl_executor_handlers {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        $(
            // Note we only enable `inline(always)` if `debug_assertions` is `false`.
            // The rational is that `debug_assertions` usually indicate a test run usually without optimizations.
            // This particular configuration with `inline(always)` bloated the outer `execute_until_done`
            // stack size so much that it caused a stackoverflow on the GitHub Actions CI runner for windows.
            #[cfg_attr(all(feature = "indirect-dispatch", not(debug_assertions)), inline(always))]
            #[cfg_attr(all(feature = "indirect-dispatch", debug_assertions), inline(never))]
            #[cfg_attr(not(feature = "indirect-dispatch"), inline(never))]
            fn $snake_case(&mut self, state: &mut VmState) -> Control<(), Break> {
                match exec::$snake_case(state, self.ip, self.sp, self.mem0, self.mem0_len, self.instance, self.ireg, self.freg32, self.freg64) {
                    Done::Continue(NextState { ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64 }) => {
                        self.ip = ip;
                        self.sp = sp;
                        self.mem0 = mem0;
                        self.mem0_len = mem0_len;
                        self.instance = instance;
                        self.ireg = ireg;
                        self.freg32 = freg32;
                        self.freg64 = freg64;
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
