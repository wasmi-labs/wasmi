use super::*;

mod i32_clz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i32>("clz", Instruction::i32_clz);
    }

    #[test]
    fn imm() {
        unary_imm::<i32>("clz", 42, |input| input.leading_zeros() as _);
    }
}

mod i64_clz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i64>("clz", Instruction::i64_clz);
    }

    #[test]
    fn imm() {
        unary_imm::<i64>("clz", 42, |input| i64::from(input.leading_zeros()));
    }
}

mod i32_ctz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i32>("ctz", Instruction::i32_ctz);
    }

    #[test]
    fn imm() {
        unary_imm::<i32>("ctz", 42, |input| input.trailing_zeros() as _);
    }
}

mod i64_ctz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i64>("ctz", Instruction::i64_ctz);
    }

    #[test]
    fn imm() {
        unary_imm::<i64>("ctz", 42, |input| i64::from(input.trailing_zeros()));
    }
}

mod i32_popcnt {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i32>("popcnt", Instruction::i32_popcnt);
    }

    #[test]
    fn imm() {
        unary_imm::<i32>("popcnt", 42, |input| input.count_ones() as _);
    }
}

mod i64_popcnt {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i64>("popcnt", Instruction::i64_popcnt);
    }

    #[test]
    fn imm() {
        unary_imm::<i64>("popcnt", 42, |input| i64::from(input.count_ones()));
    }
}

mod f32_abs {
    use super::*;

    const OP_NAME: &str = "abs";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_abs);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::abs);
        unary_imm::<f32>(OP_NAME, -42.5, f32::abs);
    }
}

mod f32_neg {
    use super::*;

    const OP_NAME: &str = "neg";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_neg);
    }

    #[test]
    fn imm() {
        use core::ops::Neg as _;
        unary_imm::<f32>(OP_NAME, 42.5, f32::neg);
        unary_imm::<f32>(OP_NAME, -42.5, f32::neg);
    }
}

mod f32_ceil {
    use super::*;

    const OP_NAME: &str = "ceil";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_ceil);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::ceil);
        unary_imm::<f32>(OP_NAME, -42.5, f32::ceil);
    }
}

mod f32_floor {
    use super::*;

    const OP_NAME: &str = "floor";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_floor);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::floor);
        unary_imm::<f32>(OP_NAME, -42.5, f32::floor);
    }
}

mod f32_trunc {
    use super::*;

    const OP_NAME: &str = "trunc";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_trunc);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::trunc);
        unary_imm::<f32>(OP_NAME, -42.5, f32::trunc);
    }
}

mod f32_nearest {
    use super::*;
    use wasmi_core::UntypedValue;

    const OP_NAME: &str = "nearest";

    /// We simply use the `f32_nearest` implementation from the `wasmi_core` crate.
    ///
    /// # Note
    ///
    /// Rust currently does not ship with a proper rounding function for floats
    /// that has the same behavior as mandated by the WebAssembly specification.
    /// There is an issue to add a proper `round_ties_even` to Rust and we should
    /// use it once it is stabilized.
    ///
    /// More information here: https://github.com/rust-lang/rust/issues/96710
    fn f32_nearest(input: f32) -> f32 {
        f32::from(UntypedValue::f32_nearest(UntypedValue::from(input)))
    }

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_nearest);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32_nearest);
        unary_imm::<f32>(OP_NAME, -42.5, f32_nearest);
    }
}

mod f32_sqrt {
    use super::*;

    const OP_NAME: &str = "sqrt";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_sqrt);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::sqrt);
        unary_imm::<f32>(OP_NAME, -42.5, f32::sqrt);
    }
}

mod f64_abs {
    use super::*;

    const OP_NAME: &str = "abs";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_abs);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::abs);
        unary_imm::<f64>(OP_NAME, -42.5, f64::abs);
    }
}

mod f64_neg {
    use super::*;

    const OP_NAME: &str = "neg";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_neg);
    }

    #[test]
    fn imm() {
        use core::ops::Neg as _;
        unary_imm::<f64>(OP_NAME, 42.5, f64::neg);
        unary_imm::<f64>(OP_NAME, -42.5, f64::neg);
    }
}

mod f64_ceil {
    use super::*;

    const OP_NAME: &str = "ceil";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_ceil);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::ceil);
        unary_imm::<f64>(OP_NAME, -42.5, f64::ceil);
    }
}

mod f64_floor {
    use super::*;

    const OP_NAME: &str = "floor";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_floor);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::floor);
        unary_imm::<f64>(OP_NAME, -42.5, f64::floor);
    }
}

mod f64_trunc {
    use super::*;

    const OP_NAME: &str = "trunc";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_trunc);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::trunc);
        unary_imm::<f64>(OP_NAME, -42.5, f64::trunc);
    }
}

mod f64_nearest {
    use super::*;
    use wasmi_core::UntypedValue;

    const OP_NAME: &str = "nearest";

    /// We simply use the `f32_nearest` implementation from the `wasmi_core` crate.
    ///
    /// # Note
    ///
    /// Rust currently does not ship with a proper rounding function for floats
    /// that has the same behavior as mandated by the WebAssembly specification.
    /// There is an issue to add a proper `round_ties_even` to Rust and we should
    /// use it once it is stabilized.
    ///
    /// More information here: https://github.com/rust-lang/rust/issues/96710
    fn f64_nearest(input: f64) -> f64 {
        f64::from(UntypedValue::f64_nearest(UntypedValue::from(input)))
    }

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_nearest);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64_nearest);
        unary_imm::<f64>(OP_NAME, -42.5, f64_nearest);
    }
}

mod f64_sqrt {
    use super::*;

    const OP_NAME: &str = "sqrt";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_sqrt);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::sqrt);
        unary_imm::<f64>(OP_NAME, -42.5, f64::sqrt);
    }
}

macro_rules! wrap_untyped {
    ($name:ident) => {
        |input| <_>::from(UntypedValue::$name(UntypedValue::from(input)))
    };
}

mod i32_extend8_s {
    use super::*;

    const OP_NAME: &str = "extend8_s";

    #[test]
    fn reg() {
        unary_reg::<i32>(OP_NAME, Instruction::i32_extend8_s);
    }

    #[test]
    fn imm() {
        let consteval = wrap_untyped!(i32_extend8_s);
        unary_imm::<i32>(OP_NAME, 0xFF, consteval);
        unary_imm::<i32>(OP_NAME, 42, consteval);
        unary_imm::<i32>(OP_NAME, -42, consteval);
    }
}

mod i32_extend16_s {
    use super::*;

    const OP_NAME: &str = "extend16_s";

    #[test]
    fn reg() {
        unary_reg::<i32>(OP_NAME, Instruction::i32_extend16_s);
    }

    #[test]
    fn imm() {
        let consteval = wrap_untyped!(i32_extend16_s);
        unary_imm::<i32>(OP_NAME, 0xFFFF, consteval);
        unary_imm::<i32>(OP_NAME, 42, consteval);
        unary_imm::<i32>(OP_NAME, -42, consteval);
    }
}

mod i64_extend8_s {
    use super::*;

    const OP_NAME: &str = "extend8_s";

    #[test]
    fn reg() {
        unary_reg::<i64>(OP_NAME, Instruction::i64_extend8_s);
    }

    #[test]
    fn imm() {
        let consteval = wrap_untyped!(i64_extend8_s);
        unary_imm::<i64>(OP_NAME, 0xFF, consteval);
        unary_imm::<i64>(OP_NAME, 42, consteval);
        unary_imm::<i64>(OP_NAME, -42, consteval);
    }
}

mod i64_extend16_s {
    use super::*;

    const OP_NAME: &str = "extend16_s";

    #[test]
    fn reg() {
        unary_reg::<i64>(OP_NAME, Instruction::i64_extend16_s);
    }

    #[test]
    fn imm() {
        let consteval = wrap_untyped!(i64_extend16_s);
        unary_imm::<i64>(OP_NAME, 0xFFFF, consteval);
        unary_imm::<i64>(OP_NAME, 42, consteval);
        unary_imm::<i64>(OP_NAME, -42, consteval);
    }
}

mod i64_extend32_s {
    use super::*;

    const OP_NAME: &str = "extend32_s";

    #[test]
    fn reg() {
        unary_reg::<i64>(OP_NAME, Instruction::i64_extend32_s);
    }

    #[test]
    fn imm() {
        let consteval = wrap_untyped!(i64_extend32_s);
        unary_imm::<i64>(OP_NAME, 0xFFFF_FFFF, consteval);
        unary_imm::<i64>(OP_NAME, 42, consteval);
        unary_imm::<i64>(OP_NAME, -42, consteval);
    }
}
