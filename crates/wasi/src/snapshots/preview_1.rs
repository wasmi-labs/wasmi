#[allow(unused_imports)]
use wasi_common::snapshots::preview_1::*;
use wasi_common::Error;
use wasmi::{core::Trap, AsContextMut, Caller, Extern, Func, Linker};
use wasmi_core::HostErrType;

/// Adapted from wasmtime's `wiggle` crate. To reuse this would have required
/// an implementation of `From<wasmtime::Trap>`
fn run_in_dummy_executor<F: std::future::Future>(future: F) -> Result<F::Output, Trap> {
    use std::{
        pin::Pin,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    let mut f = Pin::from(Box::new(future));
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    match f.as_mut().poll(&mut cx) {
        Poll::Ready(val) => return Ok(val),
        Poll::Pending =>
            return Err(Trap::from(HostErrType::new_with_reason("Cannot wait on pending future: must enable wiggle \"async\" future and execute on an async Store".to_string())))
    }

    fn dummy_waker() -> Waker {
        return unsafe { Waker::from_raw(clone(5 as *const _)) };

        unsafe fn clone(ptr: *const ()) -> RawWaker {
            assert_eq!(ptr as usize, 5);
            const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
            RawWaker::new(ptr, &VTABLE)
        }

        unsafe fn wake(ptr: *const ()) {
            assert_eq!(ptr as usize, 5);
        }

        unsafe fn wake_by_ref(ptr: *const ()) {
            assert_eq!(ptr as usize, 5);
        }

        unsafe fn drop(ptr: *const ()) {
            assert_eq!(ptr as usize, 5);
        }
    }
}

/// Creates the function item `add_wasi_snapshot_preview1_to_wasmi_linker` which when called adds all
/// `wasi preview_1` functions to the linker
macro_rules! impl_add_to_linker_for_funcs {
    ($($fname:ident ($( $arg:ident : $typ:ty ),* $(,)? ) => $ret:tt),+ $(,)?) => {
        fn add_wasi_snapshot_preview1_to_wasmi_linker<'a, T, U>(
            linker: &mut Linker<T>,
            mut store_ctx: impl AsContextMut<UserState = T>,
            wasi_ctx: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static)
        -> Result<(), Error>
        where U: wasi_common::snapshots::preview_1::wasi_snapshot_preview1::WasiSnapshotPreview1 +
                 wasi_common::snapshots::preview_1::wasi_snapshot_preview1::UserErrorConversion
        {
            let mut store = store_ctx.as_context_mut();
            $(linker.define(
                "wasi_snapshot_preview1",
                stringify!($fname),
                Func::wrap(&mut store, move|mut caller: Caller<'_, T>, $($arg : $typ,)*| {
                    let result = async {
                        let mem = match  caller.get_export("memory") {
                            Some(Extern::Memory(m)) => m,
                            _ => return Err(Trap::from(HostErrType::new_with_reason("missing required memory export".to_string()))),
                        };
                        let(mem, ctx) = mem.data_and_store_mut(&mut caller);
                        let ctx = wasi_ctx(ctx);
                        let mem = wiggle::wasmtime::WasmtimeGuestMemory::new(mem);

                        match wasi_common::snapshots::preview_1::wasi_snapshot_preview1::$fname(ctx, &mem, $($arg,)*).await {
                            Ok(r) => Ok(<$ret>::from(r)),
                            Err(wiggle::Trap::String(err)) => Err(Trap::from(HostErrType::new_with_reason(err))),
                            Err(wiggle::Trap::I32Exit(i)) => Err(Trap::from(HostErrType::new_132_exit(i))),

                        }
                    };
                    run_in_dummy_executor(result)?
                })
            )?;

        )+
    Ok(())
    }
}
}

impl_add_to_linker_for_funcs!(
    args_get (arg0: i32, arg1: i32) => i32,
    args_sizes_get(arg0: i32, arg1: i32) => i32,
    environ_get (arg0: i32, arg1: i32) => i32,
    environ_sizes_get (arg0: i32, arg1: i32) => i32,
    clock_res_get (arg0: i32, arg1: i32) => i32,
    clock_time_get (arg0 : i32, arg1 : i64, arg2 : i32) => i32,
    fd_advise(arg0: i32, arg1: i64, arg2: i64, arg3: i32) => i32,
    fd_allocate(arg0: i32, arg1: i64, arg2: i64) => i32,
    fd_close(arg0: i32,) => i32,
    fd_datasync(arg0: i32,) => i32,
    fd_fdstat_get(arg0: i32, arg1: i32) => i32,
    fd_fdstat_set_flags(arg0: i32, arg1: i32) => i32,
    fd_fdstat_set_rights(arg0: i32, arg1: i64, arg2: i64) => i32,
    fd_filestat_get(arg0: i32, arg1: i32) => i32,
    fd_filestat_set_size(arg0: i32, arg1: i64) => i32,
    fd_filestat_set_times(arg0: i32, arg1: i64, arg2: i64, arg3: i32) => i32,
    fd_pread(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) => i32,
    fd_prestat_get(arg0: i32, arg1: i32) => i32,
    fd_prestat_dir_name(arg0: i32, arg1: i32, arg2: i32) => i32,
    fd_pwrite(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) => i32,

    fd_read(arg0: i32, arg1: i32, arg2: i32, arg3: i32) => i32,
    fd_readdir(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) => i32,
    fd_renumber(arg0: i32, arg1: i32) => i32,
    fd_seek(arg0: i32, arg1: i64, arg2: i32, arg3: i32) => i32,
    fd_sync(arg0: i32) => i32,
    fd_tell(arg0: i32, arg1: i32) => i32,
    fd_write(arg0: i32, arg1: i32, arg2: i32, arg3: i32) => i32,
    path_create_directory(arg0: i32, arg1: i32, arg2: i32) => i32,
    path_filestat_get(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) => i32,
    path_filestat_set_times(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i64, arg5: i64, arg6: i32) => i32,
    path_link(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32, arg6: i32) => i32,
    path_open(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i64, arg6: i64, arg7: i32, arg8: i32) => i32,
    path_readlink(arg0: i32, rg1: i32, rg2: i32, rg3: i32, rg4: i32, rg5: i32) => i32,
    path_remove_directory(arg0: i32, arg1: i32, arg2: i32) => i32,
    path_rename(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) => i32,
    path_symlink(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) => i32,
    path_unlink_file(arg0: i32, arg1: i32, arg2: i32) => i32,
    poll_oneoff(arg0: i32, arg1: i32, arg2: i32, arg3: i32) => i32,
    proc_raise(arg0: i32) => i32,
    proc_exit(arg0: i32) => (),
    sched_yield() => i32,
    random_get(arg0: i32, arg1: i32) => i32,
    sock_accept(arg0: i32, arg1: i32, arg2: i32) => i32,
    sock_recv(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) => i32,
    sock_send(arg0: i32,arg1: i32,arg2: i32,arg3: i32,arg4: i32) => i32,
    sock_shutdown(arg0: i32, arg1: i32) => i32,

);

pub fn define_wasi<T, U>(
    linker: &mut Linker<T>,
    store_ctx: impl AsContextMut<UserState = T>,
    wasi_ctx: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
) -> Result<(), Error>
where
    U: wasi_common::snapshots::preview_1::wasi_snapshot_preview1::WasiSnapshotPreview1
        + wasi_common::snapshots::preview_1::wasi_snapshot_preview1::UserErrorConversion,
{
    add_wasi_snapshot_preview1_to_wasmi_linker(linker, store_ctx, wasi_ctx)
}
