use crate::engine::executor::handler::{
    dispatch::{fetch_handler, Control, ExecutionOutcome},
    state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
};

pub fn execute_until_done(
    state: VmState,
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
    state.into_execution_outcome()
}
