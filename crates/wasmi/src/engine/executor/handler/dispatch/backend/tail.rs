use crate::engine::executor::handler::{
    dispatch::{fetch_handler, Break, Control, ExecutionOutcome},
    state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
};

pub enum Never {}
pub type Done = Control<Never, Break>;

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
