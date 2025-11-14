use crate::engine::executor::handler::{
    dispatch::{fetch_handler, Done, ExecutionOutcome},
    state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState},
};

pub fn execute_until_done(
    mut state: VmState,
    mut ip: Ip,
    mut sp: Sp,
    mut mem0: Mem0Ptr,
    mut mem0_len: Mem0Len,
    mut instance: Inst,
) -> Result<Sp, ExecutionOutcome> {
    let mut handler = fetch_handler(ip);
    'exec: loop {
        match handler(&mut state, ip, sp, mem0, mem0_len, instance) {
            Done::Continue(next) => {
                handler = fetch_handler(next.ip);
                ip = next.ip;
                sp = next.sp;
                mem0 = next.mem0;
                mem0_len = next.mem0_len;
                instance = next.instance;
                continue 'exec;
            }
            Done::Break(reason) => {
                if let Some(trap_code) = reason.trap_code() {
                    return Err(ExecutionOutcome::from(trap_code));
                }
                break 'exec;
            }
        }
    }
    state.into_execution_outcome()
}
