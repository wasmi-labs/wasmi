use crate::{
    engine::executor::handler::{
        dispatch::{fetch_handler, Break, Control, ExecutionOutcome},
        exec,
        state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
    },
    ir,
    ir::OpCode,
};

pub enum Never {}
pub type Done = Control<Never, Break>;

pub type Handler =
    fn(&mut VmState, ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Done;

macro_rules! expand_op_code_to_handler {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        #[inline(always)]
        pub fn op_code_to_handler(code: OpCode) -> Handler {
            match code {
                $( ir::OpCode::$camel_case => exec::$snake_case, )*
            }
        }
    };
}
ir::for_each_op!(expand_op_code_to_handler);

macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let handler = $crate::engine::executor::handler::dispatch::fetch_handler($ip);
        return handler($state, $ip, $sp, $mem0, $mem0_len, $instance);
    }};
}

pub fn execute_until_done(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Result<Sp, ExecutionOutcome> {
    let mut state = state;
    let handler = fetch_handler(ip);
    let Control::Break(reason) = handler(&mut state, ip, sp, mem0, mem0_len, instance);
    if let Some(trap_code) = reason.trap_code() {
        return Err(ExecutionOutcome::from(trap_code));
    }
    state.execution_outcome()
}
