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
