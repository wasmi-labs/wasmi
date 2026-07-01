#[cfg(target_arch = "x86_64")]
macro_rules! execution_handler {
    (
        fn $name:ident(
            $state:ident   : $state_ty:ty,
            $ip:ident      : $ip_ty:ty,
            $sp:ident      : $sp_ty:ty,
            $mem0_ptr:ident: $mem0_ptr_ty:ty,
            $mem0_len:ident: $mem0_len_ty:ty,
            $instance:ident: $instance_ty:ty,
            $ireg:ident    : $ireg_ty:ty,
            $freg32:ident  : $freg32_ty:ty,
            $freg64:ident  : $freg64_ty:ty,
        ) -> $done:ty = $body:tt
    ) => {
        #[cfg_attr(feature = "portable-dispatch", inline(always))]
        #[cfg_attr(not(feature = "portable-dispatch"), inline(never))]
        #[allow(improper_ctypes_definitions)] // not used in FFI
        #[allow(clippy::too_many_arguments)] // extern fns are ignored
        pub extern "sysv64" fn $name(
            $state: $state_ty,
            $ip: $ip_ty,
            $sp: $sp_ty,
            $mem0_ptr: $mem0_ptr_ty,
            $mem0_len: $mem0_len_ty,
            $instance: $instance_ty,
            $ireg: $ireg_ty,
            $freg32: $freg32_ty,
            $freg64: $freg64_ty,
        ) -> $done $body
    };
}

#[cfg(not(target_arch = "x86_64"))]
macro_rules! execution_handler {
    (
        fn $name:ident(
            $state:ident   : $state_ty:ty,
            $ip:ident      : $ip_ty:ty,
            $sp:ident      : $sp_ty:ty,
            $mem0_ptr:ident: $mem0_ptr_ty:ty,
            $mem0_len:ident: $mem0_len_ty:ty,
            $instance:ident: $instance_ty:ty,
            $ireg:ident    : $ireg_ty:ty,
            $freg32:ident  : $freg32_ty:ty,
            $freg64:ident  : $freg64_ty:ty,
        ) -> $done:ty = $body:tt
    ) => {
        #[cfg_attr(feature = "portable-dispatch", inline(always))]
        #[cfg_attr(not(feature = "portable-dispatch"), inline(never))]
        #[allow(improper_ctypes_definitions)] // not used in FFI
        #[expect(clippy::too_many_arguments)]
        pub fn $name(
            $state: $state_ty,
            $ip: $ip_ty,
            $sp: $sp_ty,
            $mem0_ptr: $mem0_ptr_ty,
            $mem0_len: $mem0_len_ty,
            $instance: $instance_ty,
            $ireg: $ireg_ty,
            $freg32: $freg32_ty,
            $freg64: $freg64_ty,
        ) -> $done $body
    };
}

macro_rules! handler_unary {
    ( $( fn $handler:ident($op:ident) = $eval:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let mut args = Args::from_parts(ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64);
                    let $crate::ir::decode::$op { result, value } = unsafe { args.decode_op() };
                    let value = args.get(value);
                    let value = $eval(value).into_control()?;
                    args.set(result, value);
                    dispatch_v2!(state, args)
                }
            }
        )*
    };
}

macro_rules! handler_binary {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let mut args = Args::from_parts(ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64);
                    let $crate::ir::decode::$decode { result, lhs, rhs } = unsafe { args.decode_op() };
                    let lhs = args.get(lhs);
                    let rhs = args.get(rhs);
                    let value = $eval(lhs, rhs).into_control()?;
                    args.set(result, value);
                    dispatch_v2!(state, args)
                }
            }
        )*
    };
}

macro_rules! handler_load {
    ( $( fn $handler:ident($decode:ident) = $load:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let mut args = Args::from_parts(ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64);
                    let crate::ir::decode::$decode {
                        result,
                        ptr,
                        offset,
                        memory,
                    } = unsafe { args.decode_op() };
                    let ptr: u64 = args.get(ptr);
                    let offset: u64 = args.get(offset);
                    let bytes = args.fetch_memory(state, memory);
                    let loaded = $load(bytes, ptr, offset).into_control()?;
                    args.set(result, loaded);
                    dispatch_v2!(state, args)
                }
            }
        )*
    };
}

macro_rules! handler_load_mem0_offset16 {
    ( $( fn $handler:ident($decode:ident) = $load:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let mut args = Args::from_parts(ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64);
                    let crate::ir::decode::$decode {
                        result,
                        ptr,
                        offset,
                    } = unsafe { args.decode_op() };
                    let ptr = args.get(ptr);
                    let offset = args.get(offset);
                    let bytes = args.fetch_default_memory();
                    let loaded = $load(bytes, ptr, u64::from(offset)).into_control()?;
                    args.set(result, loaded);
                    dispatch_v2!(state, args)
                }
            }
        )*
    };
}

macro_rules! handler_store {
    ( $( fn $handler:ident($decode:ident, $hint:ty) = $store:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let mut args = Args::from_parts(ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64);
                    let crate::ir::decode::$decode {
                        ptr,
                        offset,
                        value,
                        memory,
                    } = unsafe { args.decode_op() };
                    let ptr = args.get(ptr);
                    let offset = args.get(offset);
                    let value: $hint = args.get(value);
                    let bytes = args.fetch_memory(state, memory);
                    $store(bytes, ptr, offset, value.into()).into_control()?;
                    dispatch_v2!(state, args)
                }
            }
        )*
    };
}

macro_rules! handler_store_mem0_offset16 {
    ( $( fn $handler:ident($decode:ident, $hint:ty) = $store:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let mut args = Args::from_parts(ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64);
                    let crate::ir::decode::$decode {
                        ptr,
                        offset,
                        value,
                    } = unsafe { args.decode_op() };
                    let ptr = args.get(ptr);
                    let offset = args.get(offset);
                    let value: $hint = args.get(value);
                    let bytes = args.fetch_default_memory();
                    $store(bytes, ptr, u64::from(offset), value.into()).into_control()?;
                    dispatch_v2!(state, args)
                }
            }
        )*
    };
}
