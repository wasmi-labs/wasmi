macro_rules! handler_unary {
    ( $( fn $handler:ident($op:ident) = $eval:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (ip, $crate::ir::decode::$op { result, value }) = unsafe { decode_op(ip) };
                let value = get_value(value, sp);
                let value = $eval(value).into_control()?;
                set_value(sp, result, value);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}

macro_rules! handler_binary {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (ip, $crate::ir::decode::$decode { result, lhs, rhs }) = unsafe { decode_op(ip) };
                let lhs = get_value(lhs, sp);
                let rhs = get_value(rhs, sp);
                let value = $eval(lhs, rhs).into_control()?;
                set_value(sp, result, value);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}

macro_rules! handler_load_ss {
    ( $( fn $handler:ident($decode:ident) = $load:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        result,
                        ptr,
                        offset,
                        memory,
                    },
                ) = unsafe { decode_op(ip) };
                let ptr: u64 = get_value(ptr, sp);
                let offset: u64 = get_value(offset, sp);
                let mem_bytes = $crate::engine::executor::handler::utils::memory_bytes(memory, mem0, mem0_len, instance, state);
                let loaded = $load(mem_bytes, ptr, offset).into_control()?;
                set_value(sp, result, loaded);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}

macro_rules! handler_load_mem0_offset16_ss {
    ( $( fn $handler:ident($decode:ident) = $load:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        result,
                        ptr,
                        offset,
                    },
                ) = unsafe { decode_op(ip) };
                let ptr = get_value(ptr, sp);
                let offset = get_value(offset, sp);
                let mem_bytes = $crate::engine::executor::handler::state::mem0_bytes(mem0, mem0_len);
                let loaded = $load(mem_bytes, ptr, u64::from(u16::from(offset))).into_control()?;
                set_value(sp, result, loaded);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}

macro_rules! handler_store_sx {
    ( $( fn $handler:ident($decode:ident, $hint:ty) = $store:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        ptr,
                        offset,
                        value,
                        memory,
                    },
                ) = unsafe { decode_op(ip) };
                let ptr = get_value(ptr, sp);
                let offset = get_value(offset, sp);
                let value: $hint = get_value(value, sp);
                let mem_bytes = $crate::engine::executor::handler::utils::memory_bytes(memory, mem0, mem0_len, instance, state);
                $store(mem_bytes, ptr, offset, value.into()).into_control()?;
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}

macro_rules! handler_store_mem0_offset16_sx {
    ( $( fn $handler:ident($decode:ident, $hint:ty) = $store:expr );* $(;)? ) => {
        $(
            #[cfg_attr(feature = "portable-dispatch", inline(always))]
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: Mem0Ptr,
                mem0_len: Mem0Len,
                instance: Inst,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        ptr,
                        offset,
                        value,
                    },
                ) = unsafe { decode_op(ip) };
                let ptr = get_value(ptr, sp);
                let offset = get_value(offset, sp);
                let value: $hint = get_value(value, sp);
                let mem_bytes = $crate::engine::executor::handler::state::mem0_bytes(mem0, mem0_len);
                $store(mem_bytes, ptr, u64::from(u16::from(offset)), value.into()).into_control()?;
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
