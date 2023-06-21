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
