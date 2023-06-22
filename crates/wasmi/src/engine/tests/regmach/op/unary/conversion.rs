use super::*;

mod i32_wrap_i64 {
    use super::*;
    const OP: &str = "wrap_i64";

    #[test]
    fn reg() {
        conversion_reg::<i64, i32>(OP, Instruction::i32_wrap_i64);
    }

    #[test]
    fn imm() {
        fn consteval(input: i64) -> i32 {
            input as i32
        }

        conversion_imm::<i64, i32>(OP, 0, consteval);
        conversion_imm::<i64, i32>(OP, 42, consteval);
        conversion_imm::<i64, i32>(OP, -42, consteval);
        conversion_imm::<i64, i32>(OP, i64::MIN, consteval);
        conversion_imm::<i64, i32>(OP, i64::MAX, consteval);
    }
}

mod i64_extend_i32_s {
    use super::*;
    const OP: &str = "extend_i32_s";

    #[test]
    fn reg() {
        conversion_reg::<i32, i64>(OP, Instruction::i64_extend_i32_s);
    }

    #[test]
    fn imm() {
        fn consteval(input: i32) -> i64 {
            i64::from(input)
        }

        conversion_imm::<i32, i64>(OP, 0, consteval);
        conversion_imm::<i32, i64>(OP, 42, consteval);
        conversion_imm::<i32, i64>(OP, -42, consteval);
        conversion_imm::<i32, i64>(OP, i32::MIN, consteval);
        conversion_imm::<i32, i64>(OP, i32::MAX, consteval);
    }
}

mod i64_extend_i32_u {
    use super::*;
    const OP: &str = "extend_i32_u";

    #[test]
    fn reg() {
        conversion_reg::<i32, i64>(OP, Instruction::i64_extend_i32_u);
    }

    #[test]
    fn imm() {
        fn consteval(input: i32) -> i64 {
            i64::from(input as u32)
        }

        conversion_imm::<i32, i64>(OP, 0, consteval);
        conversion_imm::<i32, i64>(OP, 42, consteval);
        conversion_imm::<i32, i64>(OP, -42, consteval);
        conversion_imm::<i32, i64>(OP, i32::MIN, consteval);
        conversion_imm::<i32, i64>(OP, i32::MAX, consteval);
    }
}

macro_rules! signed_trunc_tests {
    (
        $(
            fn $name:ident($op_name:literal, $input_ty:ty, $output_ty:ty);
        )*
    ) => {
        $(
            mod $name {
                use super::*;
                const OP: &str = $op_name;

                #[test]
                fn reg() {
                    conversion_reg::<$input_ty, $output_ty>(OP, Instruction::$name);
                }

                #[test]
                fn imm_ok() {
                    fn consteval_ok(input: $input_ty) -> $output_ty {
                        <$output_ty>::from(UntypedValue::from(input).$name().expect("testcase expects Ok result"))
                    }

                    conversion_imm::<$input_ty, $output_ty>(OP, 0.0, consteval_ok);
                    conversion_imm::<$input_ty, $output_ty>(OP, -0.0, consteval_ok);
                    conversion_imm::<$input_ty, $output_ty>(OP, 42.5, consteval_ok);
                    conversion_imm::<$input_ty, $output_ty>(OP, -42.5, consteval_ok);
                }

                #[test]
                fn imm_err() {
                    fn consteval_err(input: $input_ty) -> TrapCode {
                        UntypedValue::from(input).$name().expect_err("testcase expects Err result")
                    }

                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, <$input_ty>::NAN, consteval_err);
                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, <$input_ty>::INFINITY, consteval_err);
                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, <$input_ty>::NEG_INFINITY, consteval_err);
                }
            }
        )*
    };
}
signed_trunc_tests! {
    fn i32_trunc_f32_s("trunc_f32_s", f32, i32);
    fn i32_trunc_f64_s("trunc_f64_s", f64, i32);
    fn i64_trunc_f32_s("trunc_f32_s", f32, i64);
    fn i64_trunc_f64_s("trunc_f64_s", f64, i64);
}

macro_rules! unsigned_trunc_tests {
    (
        $(
            fn $name:ident($op_name:literal, $input_ty:ty, $output_ty:ty);
        )*
    ) => {
        $(
            mod $name {
                use super::*;
                const OP: &str = $op_name;

                #[test]
                fn reg() {
                    conversion_reg::<$input_ty, $output_ty>(OP, Instruction::$name);
                }

                #[test]
                fn imm_ok() {
                    fn consteval_ok(input: $input_ty) -> $output_ty {
                        <$output_ty>::from(UntypedValue::from(input).$name().expect("testcase expects Ok result"))
                    }

                    conversion_imm::<$input_ty, $output_ty>(OP, 0.0, consteval_ok);
                    conversion_imm::<$input_ty, $output_ty>(OP, -0.0, consteval_ok);
                    conversion_imm::<$input_ty, $output_ty>(OP, 42.5, consteval_ok);
                }

                #[test]
                fn imm_err() {
                    fn consteval_err(input: $input_ty) -> TrapCode {
                        UntypedValue::from(input).$name().expect_err("testcase expects Err result")
                    }

                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, -42.5, consteval_err);
                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, <$input_ty>::NAN, consteval_err);
                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, <$input_ty>::INFINITY, consteval_err);
                    fallible_conversion_imm_err::<$input_ty, $output_ty>(OP, <$input_ty>::NEG_INFINITY, consteval_err);
                }
            }
        )*
    };
}
unsigned_trunc_tests! {
    fn i32_trunc_f32_u("trunc_f32_u", f32, i32);
    fn i32_trunc_f64_u("trunc_f64_u", f64, i32);
    fn i64_trunc_f32_u("trunc_f32_u", f32, i64);
    fn i64_trunc_f64_u("trunc_f64_u", f64, i64);
}

macro_rules! trunc_sat_tests {
    (
        $(
            fn $name:ident($op_name:literal, $input_ty:ty, $output_ty:ty);
        )*
    ) => {
        $(
            mod $name {
                use super::*;
                const OP: &str = $op_name;

                #[test]
                fn reg() {
                    conversion_reg::<$input_ty, $output_ty>(OP, Instruction::$name);
                }

                #[test]
                fn imm() {
                    fn consteval(input: $input_ty) -> $output_ty {
                        <$output_ty>::from(UntypedValue::$name(input.into()))
                    }

                    conversion_imm::<$input_ty, $output_ty>(OP, 0.0, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, 42.5, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, -42.5, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::NAN, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::INFINITY, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::NEG_INFINITY, consteval);
                }
            }
        )*
    };
}
trunc_sat_tests! {
    fn i32_trunc_sat_f32_s("trunc_sat_f32_s", f32, i32);
    fn i32_trunc_sat_f32_u("trunc_sat_f32_u", f32, i32);
    fn i32_trunc_sat_f64_s("trunc_sat_f64_s", f64, i32);
    fn i32_trunc_sat_f64_u("trunc_sat_f64_u", f64, i32);
    fn i64_trunc_sat_f32_s("trunc_sat_f32_s", f32, i64);
    fn i64_trunc_sat_f32_u("trunc_sat_f32_u", f32, i64);
    fn i64_trunc_sat_f64_s("trunc_sat_f64_s", f64, i64);
    fn i64_trunc_sat_f64_u("trunc_sat_f64_u", f64, i64);
}

macro_rules! convert_tests {
    (
        $(
            fn $name:ident($op_name:literal, $input_ty:ty, $output_ty:ty);
        )*
    ) => {
        $(
            mod $name {
                use super::*;
                const OP: &str = $op_name;

                #[test]
                fn reg() {
                    conversion_reg::<$input_ty, $output_ty>(OP, Instruction::$name);
                }

                #[test]
                fn imm() {
                    fn consteval(input: $input_ty) -> $output_ty {
                        <$output_ty>::from(UntypedValue::$name(input.into()))
                    }

                    conversion_imm::<$input_ty, $output_ty>(OP, 0, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, 42, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, -42, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::MIN, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::MAX, consteval);
                }
            }
        )*
    };
}
convert_tests! {
    fn f32_convert_i32_s("convert_i32_s", i32, f32);
    fn f32_convert_i32_u("convert_i32_u", i32, f32);
    fn f32_convert_i64_s("convert_i64_s", i64, f32);
    fn f32_convert_i64_u("convert_i64_u", i64, f32);
    fn f64_convert_i32_s("convert_i32_s", i32, f64);
    fn f64_convert_i32_u("convert_i32_u", i32, f64);
    fn f64_convert_i64_s("convert_i64_s", i64, f64);
    fn f64_convert_i64_u("convert_i64_u", i64, f64);
}

mod f32_demote_f64 {
    use super::*;
    const OP: &str = "demote_f64";

    #[test]
    fn reg() {
        conversion_reg::<f64, f32>(OP, Instruction::f32_demote_f64);
    }

    #[test]
    fn imm() {
        fn consteval(input: f64) -> f32 {
            f32::from(UntypedValue::from(input).f32_demote_f64())
        }

        conversion_imm::<f64, f32>(OP, 0.0, consteval);
        conversion_imm::<f64, f32>(OP, 42.5, consteval);
        conversion_imm::<f64, f32>(OP, -42.5, consteval);
        conversion_imm::<f64, f32>(OP, f64::NAN, consteval);
        conversion_imm::<f64, f32>(OP, f64::INFINITY, consteval);
        conversion_imm::<f64, f32>(OP, f64::NEG_INFINITY, consteval);
    }
}

mod f64_promote_f32 {
    use super::*;
    const OP: &str = "promote_f32";

    #[test]
    fn reg() {
        conversion_reg::<f32, f64>(OP, Instruction::f64_promote_f32);
    }

    #[test]
    fn imm() {
        fn consteval(input: f32) -> f64 {
            f64::from(UntypedValue::from(input).f64_promote_f32())
        }

        conversion_imm::<f32, f64>(OP, 0.0, consteval);
        conversion_imm::<f32, f64>(OP, 42.5, consteval);
        conversion_imm::<f32, f64>(OP, -42.5, consteval);
        conversion_imm::<f32, f64>(OP, f32::NAN, consteval);
        conversion_imm::<f32, f64>(OP, f32::INFINITY, consteval);
        conversion_imm::<f32, f64>(OP, f32::NEG_INFINITY, consteval);
    }
}

macro_rules! iN_reinterpret_fN_tests {
    ( $( fn $name:ident($op:literal, $input_ty:ty, $output_ty:ty); )* ) => {
        $(
            mod $name {
                use super::*;
                const OP: &str = $op;

                #[test]
                fn reg() {
                    conversion_reg_with::<$input_ty, $output_ty, _>(OP, [Instruction::return_reg(Register::from(0))]);
                }

                #[test]
                fn imm() {
                    fn consteval(input: $input_ty) -> $output_ty {
                        <$output_ty>::from(UntypedValue::from(input))
                    }

                    conversion_imm::<$input_ty, $output_ty>(OP, 0.0, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, 42.5, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, -42.5, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::NAN, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::INFINITY, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::NEG_INFINITY, consteval);
                }
            }
        )*
    }
}
iN_reinterpret_fN_tests! {
    fn i32_reinterpret_f32("reinterpret_f32", f32, i32);
    fn i64_reinterpret_f64("reinterpret_f64", f64, i64);
}

macro_rules! fN_reinterpret_iN_tests {
    ( $( fn $name:ident($op:literal, $input_ty:ty, $output_ty:ty); )* ) => {
        $(
            mod $name {
                use super::*;
                const OP: &str = $op;

                #[test]
                fn reg() {
                    conversion_reg_with::<$input_ty, $output_ty, _>(OP, [Instruction::return_reg(Register::from(0))]);
                }

                #[test]
                fn imm() {
                    fn consteval(input: $input_ty) -> $output_ty {
                        <$output_ty>::from(UntypedValue::from(input))
                    }

                    conversion_imm::<$input_ty, $output_ty>(OP, 0, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, 42, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, -42, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::MIN, consteval);
                    conversion_imm::<$input_ty, $output_ty>(OP, <$input_ty>::MAX, consteval);
                }
            }
        )*
    }
}
fN_reinterpret_iN_tests! {
    fn f32_reinterpret_i32("reinterpret_i32", i32, f32);
    fn f64_reinterpret_i64("reinterpret_i64", i64, f64);
}
