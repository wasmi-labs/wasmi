use crate::{
    engine::executor::handler::{
        dispatch::{Break, Control, ExecutionOutcome, decode_handler, decode_op_code},
        exec,
        state::{Freg32, Freg64, Inst, Ip, Ireg, Mem0Len, Mem0Ptr, Sp, VmState},
    },
    ir,
    ir::OpCode,
};

#[inline(always)]
pub fn fetch_handler(ip: Ip) -> Handler {
    match cfg!(feature = "indirect-dispatch") {
        true => {
            let op_code = decode_op_code(ip);
            op_code_to_handler(op_code)
        }
        false => decode_handler(ip),
    }
}

pub enum Never {}
pub type Done = Control<Never, Break>;

#[cfg(not(target_arch = "x86_64"))]
pub type Handler = fn(
    &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> Done;

#[cfg(target_arch = "x86_64")]
#[allow(improper_ctypes_definitions)] // not used in FFI
pub type Handler = extern "sysv64" fn(
    &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> Done;

macro_rules! expand_op_code_to_handler {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        #[inline(always)]
        pub fn op_code_to_handler(code: OpCode) -> Handler {
            static HANDLERS: [Handler; ir::LEN_OPS] = [
                $( exec::$snake_case ),*
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

#[cfg(not(all(feature = "unstable", not(feature = "stable"))))]
macro_rules! dispatch {
    ( $state:expr, $args:expr $(,)? ) => {{
        let (ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64) = $args.into_parts();
        let handler = $crate::engine::executor::handler::dispatch::backend::fetch_handler(ip);
        return handler(
            $state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64,
        );
    }};
}

#[cfg(all(feature = "unstable", not(feature = "stable")))]
macro_rules! dispatch {
    ( $state:expr, $args:expr $(,)? ) => {{
        let (ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64) = $args.into_parts();
        let handler = $crate::engine::executor::handler::dispatch::backend::fetch_handler(ip);
        become handler(
            $state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64,
        );
    }};
}

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
    let handler = fetch_handler(ip);
    let Control::Break(reason) = handler(
        state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64,
    );
    if let Some(trap_code) = reason.trap_code() {
        return Err(ExecutionOutcome::from(trap_code));
    }
    state.execution_outcome()
}
