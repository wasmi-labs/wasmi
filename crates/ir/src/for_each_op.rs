#[macro_export]
macro_rules! for_each_op {
    ($mac:ident) => {
        $mac! {
            /// Traps the execution with the given [`TrapCode`].
            ///
            /// # Note
            ///
            /// Used to represent Wasm `unreachable` instruction
            /// as well as code paths that are determined to always
            /// lead to traps during execution. For example division
            /// by constant zero.
            #[snake_name(trap)]
            Trap {
                trap_code: TrapCode
            },
            /// Instruction generated to consume fuel for its associated basic block.
            ///
            /// # Note
            ///
            /// These instructions are only generated if fuel metering is enabled.
            #[snake_name(consume_fuel)]
            ConsumeFuel {
                block_fuel: BlockFuel
            },

            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns nothing.
            #[snake_name(r#return)]
            Return,
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single value stored in a register.
            #[snake_name(return_reg)]
            ReturnReg {
                /// The returned value.
                value: Reg,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns two values stored in registers.
            #[snake_name(return_reg2)]
            ReturnReg2 {
                /// The returned values.
                values: [Reg; 2],
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns three values stored in registers.
            #[snake_name(return_reg3)]
            ReturnReg3 {
                /// The returned values.
                values: [Reg; 3],
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single 32-bit constant value.
            #[snake_name(return_imm32)]
            ReturnImm32 {
                /// The returned 32-bit constant value.
                value: AnyConst32,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single 32-bit encoded `i64` constant value.
            #[snake_name(return_i64imm32)]
            ReturnI64Imm32 {
                /// The returned constant value.
                value: Const32<i64>,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single 32-bit encoded `f64` constant value.
            #[snake_name(return_f64imm32)]
            ReturnF64Imm32 {
                /// The returned constant value.
                value: Const32<f64>,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns values as stored in the bounded [`RegSpan`].
            #[snake_name(return_span)]
            ReturnSpan {
                /// The [`RegSpan`] that represents the registers that store the returned values.
                values: BoundedRegSpan,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns many values accessed by registers.
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(return_many)]
            ReturnMany {
                /// The first three returned values.
                values: [Reg; 3],
            },

            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// This is used to translate certain conditional Wasm branches such as `br_if`.
            /// Returns back to the caller if and only if the `condition` value is non zero.
            #[snake_name(return_nez)]
            ReturnNez {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning a single
            /// [`Reg`] value if the `condition` evaluates to `true`.
            #[snake_name(return_nez_reg)]
            ReturnNezReg {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The returned value.
                value: Reg,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning two
            /// [`Reg`] value if the `condition` evaluates to `true`.
            #[snake_name(return_nez_reg2)]
            ReturnNezReg2 {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The returned value.
                values: [Reg; 2],
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning a single
            /// [`AnyConst32`] value if the `condition` evaluates to `true`.
            #[snake_name(return_nez_imm32)]
            ReturnNezImm32 {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The returned value.
                value: AnyConst32,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning a single
            /// 32-bit encoded [`i64`] value if the `condition` evaluates to `true`.
            #[snake_name(return_nez_i64imm32)]
            ReturnNezI64Imm32 {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The returned value.
                value: Const32<i64>,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning a single
            /// 32-bit encoded [`f64`] value if the `condition` evaluates to `true`.
            #[snake_name(return_nez_f64imm32)]
            ReturnNezF64Imm32 {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The returned value.
                value: Const32<f64>,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning two or more values.
            #[snake_name(return_nez_span)]
            ReturnNezSpan {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The returned values.
                values: BoundedRegSpan,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::ReturnNez`] returning multiple register values.
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(return_nez_many)]
            ReturnNezMany {
                /// The register holding the condition to evaluate against zero.
                condition: Reg,
                /// The first returned value.
                values: [Reg; 2],
            },

            /// A Wasm `br` instruction.
            #[snake_name(branch)]
            Branch {
                /// The branching offset for the instruction pointer.
                offset: BranchOffset,
            },

            /// A fallback instruction for cmp+branch instructions with branch offsets that cannot be 16-bit encoded.
            ///
            /// # Note
            ///
            /// This instruction fits in a single instruction word but arguably executes slower than
            /// cmp+branch instructions with a 16-bit encoded branch offset. It only ever gets encoded
            /// and used whenever a branch offset of a cmp+branch instruction cannot be 16-bit encoded.
            #[snake_name(branch_cmp_fallback)]
            BranchCmpFallback {
                /// The left-hand side value for the comparison.
                lhs: Reg,
                /// The right-hand side value for the comparison.
                ///
                /// # Note
                ///
                /// We allocate constant values as function local constant values and use
                /// their register to only require a single fallback instruction variant.
                rhs: Reg,
                /// The register that stores the [`ComparatorAndOffset`] of this instruction.
                ///
                /// # Note
                ///
                /// The [`ComparatorAndOffset`] is loaded from register as `u64` value and
                /// decoded into a [`ComparatorAndOffset`] before access its comparator
                /// and 32-bit branch offset fields.
                ///
                /// [`ComparatorAndOffset`]: crate::ComparatorAndOffset
                params: Reg,
            },
            /// A fused [`Instruction::I32And`] and Wasm branch instruction.
            #[snake_name(branch_i32_and)]
            BranchI32And {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32And`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32And`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_and_imm)]
            BranchI32AndImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Or`] and Wasm branch instruction.
            #[snake_name(branch_i32_or)]
            BranchI32Or {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Or`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32Or`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_or_imm)]
            BranchI32OrImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Xor`] and Wasm branch instruction.
            #[snake_name(branch_i32_xor)]
            BranchI32Xor {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Xor`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32Xor`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_xor_imm)]
            BranchI32XorImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused not-[`Instruction::I32And`] and Wasm branch instruction.
            #[snake_name(branch_i32_and_eqz)]
            BranchI32AndEqz {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused not-[`Instruction::I32And`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32AndEqz`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_and_eqz_imm)]
            BranchI32AndEqzImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused not-[`Instruction::I32Or`] and Wasm branch instruction.
            #[snake_name(branch_i32_or_eqz)]
            BranchI32OrEqz {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused not-[`Instruction::I32Or`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32OrEqz`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_or_eqz_imm)]
            BranchI32OrEqzImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused not-[`Instruction::I32Xor`] and Wasm branch instruction.
            #[snake_name(branch_i32_xor_eqz)]
            BranchI32XorEqz {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused not-[`Instruction::I32Xor`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32XorEqz`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_xor_eqz_imm)]
            BranchI32XorEqzImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::I32Eq`] and Wasm branch instruction.
            #[snake_name(branch_i32_eq)]
            BranchI32Eq {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Eq`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32Eq`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_eq_imm)]
            BranchI32EqImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Ne`] and Wasm branch instruction.
            #[snake_name(branch_i32_ne)]
            BranchI32Ne {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32Ne`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32Ne`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_ne_imm)]
            BranchI32NeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::I32LtS`] and Wasm branch instruction.
            #[snake_name(branch_i32_lt_s)]
            BranchI32LtS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LtS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32LtS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_lt_s_imm)]
            BranchI32LtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LtU`] and Wasm branch instruction.
            #[snake_name(branch_i32_lt_u)]
            BranchI32LtU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LtU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32LtU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_lt_u_imm)]
            BranchI32LtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LeS`] and Wasm branch instruction.
            #[snake_name(branch_i32_le_s)]
            BranchI32LeS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LeS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32LeS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_le_s_imm)]
            BranchI32LeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LeU`] and Wasm branch instruction.
            #[snake_name(branch_i32_le_u)]
            BranchI32LeU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32LeU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32LeU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_le_u_imm)]
            BranchI32LeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GtS`] and Wasm branch instruction.
            #[snake_name(branch_i32_gt_s)]
            BranchI32GtS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GtS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32GtS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_gt_s_imm)]
            BranchI32GtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GtU`] and Wasm branch instruction.
            #[snake_name(branch_i32_gt_u)]
            BranchI32GtU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GtU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32GtU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_gt_u_imm)]
            BranchI32GtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GeS`] and Wasm branch instruction.
            #[snake_name(branch_i32_ge_s)]
            BranchI32GeS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GeS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32GeS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_ge_s_imm)]
            BranchI32GeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GeU`] and Wasm branch instruction.
            #[snake_name(branch_i32_ge_u)]
            BranchI32GeU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I32GeU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI32GeU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i32_ge_u_imm)]
            BranchI32GeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u32>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::I64Eq`] and Wasm branch instruction.
            #[snake_name(branch_i64_eq)]
            BranchI64Eq {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64Eq`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64Eq`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_eq_imm)]
            BranchI64EqImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64Ne`] and Wasm branch instruction.
            #[snake_name(branch_i64_ne)]
            BranchI64Ne {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64Ne`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64Ne`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_ne_imm)]
            BranchI64NeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::I64LtS`] and Wasm branch instruction.
            #[snake_name(branch_i64_lt_s)]
            BranchI64LtS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LtS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64LtS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_lt_s_imm)]
            BranchI64LtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LtU`] and Wasm branch instruction.
            #[snake_name(branch_i64_lt_u)]
            BranchI64LtU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LtU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64LtU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_lt_u_imm)]
            BranchI64LtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LeS`] and Wasm branch instruction.
            #[snake_name(branch_i64_le_s)]
            BranchI64LeS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LeS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64LeS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_le_s_imm)]
            BranchI64LeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LeU`] and Wasm branch instruction.
            #[snake_name(branch_i64_le_u)]
            BranchI64LeU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64LeU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64LeU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_le_u_imm)]
            BranchI64LeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GtS`] and Wasm branch instruction.
            #[snake_name(branch_i64_gt_s)]
            BranchI64GtS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GtS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64GtS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_gt_s_imm)]
            BranchI64GtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GtU`] and Wasm branch instruction.
            #[snake_name(branch_i64_gt_u)]
            BranchI64GtU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GtU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64GtU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_gt_u_imm)]
            BranchI64GtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GeS`] and Wasm branch instruction.
            #[snake_name(branch_i64_ge_s)]
            BranchI64GeS {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GeS`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64GeS`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_ge_s_imm)]
            BranchI64GeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<i64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GeU`] and Wasm branch instruction.
            #[snake_name(branch_i64_ge_u)]
            BranchI64GeU {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::I64GeU`] and Wasm branch instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::BranchI64GeU`] with 16-bit encoded constant `rhs`.
            #[snake_name(branch_i64_ge_u_imm)]
            BranchI64GeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Const16<u64>,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::F32Eq`] and Wasm branch instruction.
            #[snake_name(branch_f32_eq)]
            BranchF32Eq {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F32Ne`] and Wasm branch instruction.
            #[snake_name(branch_f32_ne)]
            BranchF32Ne {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::F32Lt`] and Wasm branch instruction.
            #[snake_name(branch_f32_lt)]
            BranchF32Lt {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F32Le`] and Wasm branch instruction.
            #[snake_name(branch_f32_le)]
            BranchF32Le {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F32Gt`] and Wasm branch instruction.
            #[snake_name(branch_f32_gt)]
            BranchF32Gt {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F32Ge`] and Wasm branch instruction.
            #[snake_name(branch_f32_ge)]
            BranchF32Ge {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::F64Eq`] and Wasm branch instruction.
            #[snake_name(branch_f64_eq)]
            BranchF64Eq {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F64Ne`] and Wasm branch instruction.
            #[snake_name(branch_f64_ne)]
            BranchF64Ne {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A fused [`Instruction::F64Lt`] and Wasm branch instruction.
            #[snake_name(branch_f64_lt)]
            BranchF64Lt {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F64Le`] and Wasm branch instruction.
            #[snake_name(branch_f64_le)]
            BranchF64Le {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F64Gt`] and Wasm branch instruction.
            #[snake_name(branch_f64_gt)]
            BranchF64Gt {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },
            /// A fused [`Instruction::F64Ge`] and Wasm branch instruction.
            #[snake_name(branch_f64_ge)]
            BranchF64Ge {
                /// The left-hand side operand to the branch conditional.
                lhs: Reg,
                /// The right-hand side operand to the branch conditional.
                rhs: Reg,
                /// The 16-bit encoded branch offset.
                offset: BranchOffset16,
            },

            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed `len_target` times by
            ///
            /// - [`Instruction::Branch`]
            /// - [`Instruction::Return`]
            #[snake_name(branch_table_0)]
            BranchTable0 {
                /// The register holding the index of the instruction.
                index: Reg,
                /// The number of branch table targets including the default target.
                len_targets: u32,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// 1. Followed by one of
            ///
            /// - [`Instruction::Register`]
            /// - [`Instruction::Const32`]
            /// - [`Instruction::I64Const32`]
            /// - [`Instruction::F64Const32`]
            ///
            /// 2. Followed `len_target` times by
            ///
            /// - [`Instruction::BranchTableTarget`]
            /// - [`Instruction::ReturnReg`]
            /// - [`Instruction::ReturnImm32`]
            /// - [`Instruction::ReturnI64Imm32`]
            /// - [`Instruction::ReturnF64Imm32`]
            #[snake_name(branch_table_1)]
            BranchTable1 {
                /// The register holding the index of the instruction.
                index: Reg,
                /// The number of branch table targets including the default target.
                len_targets: u32,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// 1. Followed by [`Instruction::Register2`].
            /// 2. Followed `len_target` times by
            ///
            /// - [`Instruction::BranchTableTarget`]
            /// - [`Instruction::ReturnReg2`]
            #[snake_name(branch_table_2)]
            BranchTable2 {
                /// The register holding the index of the instruction.
                index: Reg,
                /// The number of branch table targets including the default target.
                len_targets: u32,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// 1. Followed by [`Instruction::Register3`].
            /// 2. Followed `len_target` times by
            ///
            /// - [`Instruction::BranchTableTarget`]
            /// - [`Instruction::ReturnReg3`]
            #[snake_name(branch_table_3)]
            BranchTable3 {
                /// The register holding the index of the instruction.
                index: Reg,
                /// The number of branch table targets including the default target.
                len_targets: u32,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// All branch table targets must share the same destination registers.
            ///
            /// # Encoding
            ///
            /// 1. Followed by one of [`Instruction::RegisterSpan`].
            /// 2. Followed `len_target` times by
            ///
            /// - [`Instruction::BranchTableTarget`]
            /// - [`Instruction::BranchTableTargetNonOverlapping`]
            /// - [`Instruction::ReturnSpan`]
            #[snake_name(branch_table_span)]
            BranchTableSpan {
                /// The register holding the index of the instruction.
                index: Reg,
                /// The number of branch table targets including the default target.
                len_targets: u32,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// All branch table targets must share the same destination registers.
            ///
            /// # Encoding
            ///
            /// 1. Followed by [`Instruction::RegisterList`] encoding.
            /// 2. Followed `len_target` times by
            ///
            /// - [`Instruction::BranchTableTarget`]
            /// - [`Instruction::BranchTableTargetNonOverlapping`]
            /// - [`Instruction::Return`]
            #[snake_name(branch_table_many)]
            BranchTableMany {
                /// The register holding the index of the instruction.
                index: Reg,
                /// The number of branch table targets including the default target.
                len_targets: u32,
            },

            /// Copies `value` to `result`.
            ///
            /// # Note
            ///
            /// This is a Wasmi utility instruction used to translate Wasm control flow.
            #[snake_name(copy)]
            Copy {
                @result: Reg,
                /// The register holding the value to copy.
                value: Reg,
            },
            /// Copies two [`Reg`] values to `results`.
            ///
            /// # Note
            ///
            /// This is a Wasmi utility instruction used to translate Wasm control flow.
            #[snake_name(copy2)]
            Copy2 {
                @results: FixedRegSpan<2>,
                /// The registers holding the values to copy.
                values: [Reg; 2],
            },
            /// Copies the 32-bit immediate `value` to `result`.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::Copy`] for 32-bit encoded immediate values.
            /// Read [`Instruction::Copy`] for more information about this instruction.
            #[snake_name(copy_imm32)]
            CopyImm32 {
                @result: Reg,
                /// The 32-bit encoded immediate value to copy.
                value: AnyConst32,
            },
            /// Copies the 32-bit encoded `i64` immediate `value` to `result`.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::Copy`] for 32-bit encodable `i64` immediate values.
            /// - Upon execution the 32-bit encoded `i32` `value` is sign extended to `i64` and copied into `result`.
            /// - Read [`Instruction::Copy`] for more information about this instruction.
            #[snake_name(copy_i64imm32)]
            CopyI64Imm32 {
                @result: Reg,
                /// The 32-bit encoded `i64` immediate value to copy.
                value: Const32<i64>,
            },
            /// Copies the 32-bit encoded `f64` immediate `value` to `result`.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::Copy`] for 32-bit encodable `f64` immediate values.
            /// - Upon execution the 32-bit encoded `f32` `value` is promoted to `f64` and copied into `result`.
            /// - Read [`Instruction::Copy`] for more information about this instruction.
            #[snake_name(copy_f64imm32)]
            CopyF64Imm32 {
                @result: Reg,
                /// The 32-bit encoded `i64` immediate value to copy.
                value: Const32<f64>,
            },
            /// Copies `len` contiguous `values` [`RegSpan`] into `results` [`RegSpan`].
            ///
            /// Copies registers: `registers[results..results+len] <- registers[values..values+len]`
            ///
            /// # Note
            ///
            /// This [`Instruction`] serves as an optimization for cases were it is possible
            /// to copy whole spans instead of many individual register values bit by bit.
            #[snake_name(copy_span)]
            CopySpan {
                @results: RegSpan,
                /// The contiguous registers holding the inputs of this instruction.
                values: RegSpan,
                /// The amount of copied registers.
                len: u16,
            },
            /// Variant of [`Instruction::CopySpan`] that assumes that `results` and `values` span do not overlap.
            #[snake_name(copy_span_non_overlapping)]
            CopySpanNonOverlapping {
                @results: RegSpan,
                /// The contiguous registers holding the inputs of this instruction.
                values: RegSpan,
                /// The amount of copied registers.
                len: u16,
            },
            /// Copies some [`Reg`] values into `results` [`RegSpan`].
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(copy_many)]
            CopyMany {
                @results: RegSpan,
                /// The first two input registers to copy.
                values: [Reg; 2],
            },
            /// Variant of [`Instruction::CopyMany`] that assumes that `results` and `values` do not overlap.
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(copy_many_non_overlapping)]
            CopyManyNonOverlapping {
                @results: RegSpan,
                /// The first two input registers to copy.
                values: [Reg; 2],
            },

            /// Wasm `return_call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for tail calling internally compiled Wasm functions without parameters.
            #[snake_name(return_call_internal_0)]
            ReturnCallInternal0 {
                /// The called internal function.
                func: InternalFunc,
            },
            /// Wasm `return_call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for tail calling internally compiled Wasm functions with parameters.
            ///
            /// # Encoding (Parameters)
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(return_call_internal)]
            ReturnCallInternal {
                /// The called internal function.
                func: InternalFunc,
            },

            /// Wasm `return_call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for tail calling imported Wasm functions without parameters.
            #[snake_name(return_call_imported_0)]
            ReturnCallImported0 {
                /// The called imported function.
                func: Func,
            },
            /// Wasm `return_call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for tail calling imported Wasm functions with parameters.
            ///
            /// # Encoding (Parameters)
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(return_call_imported)]
            ReturnCallImported {
                /// The called imported function.
                func: Func,
            },

            /// Wasm `return_call_indirect` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions without parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::CallIndirectParams`] encoding `table` and `index`.
            #[snake_name(return_call_indirect_0)]
            ReturnCallIndirect0 {
                /// The called internal function.
                func_type: FuncType,
            },
            /// Wasm `return_call_indirect` equivalent Wasmi instruction with 16-bit immediate `index`.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions without parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::CallIndirectParamsImm16`] encoding `table` and `index`.
            #[snake_name(return_call_indirect_0_imm16)]
            ReturnCallIndirect0Imm16 {
                /// The called internal function.
                func_type: FuncType,
            },
            /// Wasm `return_call_indirect` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions with parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. [`Instruction::CallIndirectParams`]: encoding `table` and `index`
            /// 2. Zero or more [`Instruction::RegisterList`]
            /// 3. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(return_call_indirect)]
            ReturnCallIndirect {
                /// The called internal function.
                func_type: FuncType,
            },
            /// Wasm `return_call_indirect` equivalent Wasmi instruction with 16-bit immediate `index`.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions with parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. [`Instruction::CallIndirectParamsImm16`]: encoding `table` and `index`
            /// 2. Zero or more [`Instruction::RegisterList`]
            /// 3. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(return_call_indirect_imm16)]
            ReturnCallIndirectImm16 {
                /// The called internal function.
                func_type: FuncType,
            },

            /// Wasm `call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for calling internally compiled Wasm functions without parameters.
            #[snake_name(call_internal_0)]
            CallInternal0 {
                @results: RegSpan,
                /// The called internal function.
                func: InternalFunc,
            },
            /// Wasm `call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for calling internally compiled Wasm functions with parameters.
            ///
            /// # Encoding (Parameters)
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(call_internal)]
            CallInternal {
                @results: RegSpan,
                /// The called internal function.
                func: InternalFunc,
            },

            /// Wasm `call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for calling imported Wasm functions without parameters.
            #[snake_name(call_imported_0)]
            CallImported0 {
                @results: RegSpan,
                /// The called imported function.
                func: Func,
            },
            /// Wasm `call` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for calling imported Wasm functions with parameters.
            ///
            /// # Encoding (Parameters)
            ///
            /// Must be followed by
            ///
            /// 1. Zero or more [`Instruction::RegisterList`]
            /// 2. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(call_imported)]
            CallImported {
                @results: RegSpan,
                /// The called imported function.
                func: Func,
            },

            /// Wasm `call_indirect` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions without parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::CallIndirectParams`] encoding `table` and `index`.
            #[snake_name(call_indirect_0)]
            CallIndirect0 {
                @results: RegSpan,
                /// The called internal function.
                func_type: FuncType,
            },
            /// Wasm `call_indirect` equivalent Wasmi instruction with 16-bit immediate `inde` value.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions without parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::CallIndirectParamsImm16`] encoding `table` and `index`.
            #[snake_name(call_indirect_0_imm16)]
            CallIndirect0Imm16 {
                @results: RegSpan,
                /// The called internal function.
                func_type: FuncType,
            },
            /// Wasm `call_indirect` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions with parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. [`Instruction::CallIndirectParams`]: encoding `table` and `index`
            /// 2. Zero or more [`Instruction::RegisterList`]
            /// 3. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(call_indirect)]
            CallIndirect {
                @results: RegSpan,
                /// The called internal function.
                func_type: FuncType,
            },
            /// Wasm `call_indirect` equivalent Wasmi instruction with 16-bit immediate `index` value.
            ///
            /// # Note
            ///
            /// Used for indirectly calling Wasm functions with parameters.
            ///
            /// # Encoding
            ///
            /// Must be followed by
            ///
            /// 1. [`Instruction::CallIndirectParamsImm16`]: encoding `table` and `index`
            /// 2. Zero or more [`Instruction::RegisterList`]
            /// 3. Followed by one of
            ///     - [`Instruction::Register`]
            ///     - [`Instruction::Register2`]
            ///     - [`Instruction::Register3`]
            #[snake_name(call_indirect_imm16)]
            CallIndirectImm16 {
                @results: RegSpan,
                /// The called internal function.
                func_type: FuncType,
            },

            /// A Wasm `select` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::Register2`] to encode `condition` and `rhs`.
            #[snake_name(select)]
            Select {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `rhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
            #[snake_name(select_imm32_rhs)]
            SelectImm32Rhs {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `lhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::Register2`] to encode `condition` and `lhs`.
            #[snake_name(select_imm32_lhs)]
            SelectImm32Lhs {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: AnyConst32,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `lhs` and `rhs` values.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
            #[snake_name(select_imm32)]
            SelectImm32 {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: AnyConst32,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
            #[snake_name(select_i64imm32_rhs)]
            SelectI64Imm32Rhs {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::Register2`] to encode `condition` and `rhs`.
            #[snake_name(select_i64imm32_lhs)]
            SelectI64Imm32Lhs {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Const32<i64>,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` and `rhs` values.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
            #[snake_name(select_i64imm32)]
            SelectI64Imm32 {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Const32<i64>,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `rhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
            #[snake_name(select_f64imm32_rhs)]
            SelectF64Imm32Rhs {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `lhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::Register2`] to encode `condition` and `rhs`.
            #[snake_name(select_f64imm32_lhs)]
            SelectF64Imm32Lhs {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Const32<f64>,
            },
            /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `lhs` and `rhs` value.
            ///
            /// # Encoding
            ///
            /// Must be followed by [`Instruction::RegisterAndImm32`] to encode `condition` and `rhs`.
            #[snake_name(select_f64imm32)]
            SelectF64Imm32 {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Const32<f64>,
            },

            /// A Wasm `ref.func` equivalent Wasmi instruction.
            #[snake_name(ref_func)]
            RefFunc {
                @result: Reg,
                /// The index of the referenced function.
                func: Func,
            },

            /// Wasm `global.get` equivalent Wasmi instruction.
            #[snake_name(global_get)]
            GlobalGet {
                @result: Reg,
                /// The index identifying the global variable for the `global.get` instruction.
                global: Global,
            },
            /// Wasm `global.set` equivalent Wasmi instruction.
            #[snake_name(global_set)]
            GlobalSet {
                /// The register holding the value to be stored in the global variable.
                input: Reg,
                /// The index identifying the global variable for the `global.set` instruction.
                global: Global,
            },
            /// Wasm `global.set` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::GlobalSet`] for 16-bit encoded `i32` immutable `input` values.
            #[snake_name(global_set_i32imm16)]
            GlobalSetI32Imm16 {
                /// The 16-bit encoded `i32` value.
                input: Const16<i32>,
                /// The index identifying the global variable for the `global.set` instruction.
                global: Global,
            },
            /// Wasm `global.set` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::GlobalSet`] for 16-bit encoded `i64` immutable `input` values.
            #[snake_name(global_set_i64imm16)]
            GlobalSetI64Imm16 {
                /// The 16-bit encoded `i64` value.
                input: Const16<i64>,
                /// The index identifying the global variable for the `global.set` instruction.
                global: Global,
            },

            /// Wasm `i32.load` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i32_load)]
            I32Load {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Load`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_load_at)]
            I32LoadAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i32.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Load`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_load_offset16)]
            I32LoadOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load)]
            I64Load {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load_at)]
            I64LoadAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load_offset16)]
            I64LoadOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `f32.load` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(f32_load)]
            F32Load {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `f32.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::F32Load`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(f32_load_at)]
            F32LoadAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `f32.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::F32Load`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(f32_load_offset16)]
            F32LoadOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `f64.load` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(f64_load)]
            F64Load {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `f64.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::F64Load`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(f64_load_at)]
            F64LoadAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `f64.load` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::F64Load`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(f64_load_offset16)]
            F64LoadOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i32.load8_s` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i32_load8_s)]
            I32Load8s {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.load8_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Load8s`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_load8_s_at)]
            I32Load8sAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i32.load8_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Load8s`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_load8_s_offset16)]
            I32Load8sOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i32.load8_u` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i32_load8_u)]
            I32Load8u {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.load8_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Load8u`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_load8_u_at)]
            I32Load8uAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i32.load8_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Load8u`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_load8_u_offset16)]
            I32Load8uOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i32.load16_s` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i32_load16_s)]
            I32Load16s {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.load16_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Load16s`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_load16_s_at)]
            I32Load16sAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i32.load16_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Load16s`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_load16_s_offset16)]
            I32Load16sOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i32.load16_u` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i32_load16_u)]
            I32Load16u {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.load16_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Load16u`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_load16_u_at)]
            I32Load16uAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i32.load16_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Load16u`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_load16_u_offset16)]
            I32Load16uOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load8_s` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load8_s)]
            I64Load8s {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load8_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load8s`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load8_s_at)]
            I64Load8sAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load8_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load8s`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load8_s_offset16)]
            I64Load8sOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load8_u` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load8_u)]
            I64Load8u {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load8_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load8u`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load8_u_at)]
            I64Load8uAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load8_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load8u`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load8_u_offset16)]
            I64Load8uOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load16_s` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load16_s)]
            I64Load16s {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load16_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load16s`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load16_s_at)]
            I64Load16sAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load16_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load16s`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load16_s_offset16)]
            I64Load16sOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load16_u` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load16_u)]
            I64Load16u {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load16_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load16u`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load16_u_at)]
            I64Load16uAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load16_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load16u`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load16_u_offset16)]
            I64Load16uOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load32_s` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load32_s)]
            I64Load32s {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load32_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load32s`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load32_s_at)]
            I64Load32sAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load32_s` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load32s`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load32_s_offset16)]
            I64Load32sOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i64.load32_u` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `ptr` and `offset`.
            #[snake_name(i64_load32_u)]
            I64Load32u {
                @result: Reg,
                /// The linear memory index for which the load instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.load32_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Load32u`] with a constant load address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_load32_u_at)]
            I64Load32uAt {
                @result: Reg,
                /// The `ptr+offset` address of the `load` instruction.
                address: u32,
            },
            /// Wasm `i64.load32_u` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Load32u`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_load32_u_offset16)]
            I64Load32uOffset16 {
                @result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The 16-bit encoded offset of the `load` instruction.
                offset: Const16<u32>,
            },

            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i32_store)]
            I32Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store`] with 16-bit immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i32_store_imm16)]
            I32StoreImm16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Store`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_store_offset16)]
            I32StoreOffset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32StoreOffset16`] with 16-bit immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_store_offset16_imm16)]
            I32StoreOffset16Imm16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Const16<i32>,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_store_at)]
            I32StoreAt {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32StoreAt`] with 16-bit immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_store_at_imm16)]
            I32StoreAtImm16 {
                /// The value to be stored.
                value: Const16<i32>,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `i32.store` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i32_store8)]
            I32Store8 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store8`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i32_store8_imm)]
            I32Store8Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Store8`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_store8_offset16)]
            I32Store8Offset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Store8Offset16`] with immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_store8_offset16_imm)]
            I32Store8Offset16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: i8,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store8`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_store8_at)]
            I32Store8At {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store8At`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_store8_at_imm)]
            I32Store8AtImm {
                /// The value to be stored.
                value: i8,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `i32.store16` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i32_store16)]
            I32Store16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store16`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i32_store16_imm)]
            I32Store16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Store16`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_store16_offset16)]
            I32Store16Offset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I32Store16Offset16`] with immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i32_store16_offset16_imm)]
            I32Store16Offset16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: i16,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store16`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_store16_at)]
            I32Store16At {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I32Store16At`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i32_store16_at_imm)]
            I32Store16AtImm {
                /// The value to be stored.
                value: i16,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `i64.store` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store)]
            I64Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store`] with 16-bit immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store_imm16)]
            I64StoreImm16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store_offset16)]
            I64StoreOffset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64StoreOffset16`] with 16-bit immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store_offset16_imm16)]
            I64StoreOffset16Imm16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Const16<i64>,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store_at)]
            I64StoreAt {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64StoreAt`] with 16-bit immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store_at_imm16)]
            I64StoreAtImm16 {
                /// The value to be stored.
                value: Const16<i64>,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `i64.store8` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store8)]
            I64Store8 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store8`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store8_imm)]
            I64Store8Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store8`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store8_offset16)]
            I64Store8Offset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store8Offset16`] with immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store8_offset16_imm)]
            I64Store8Offset16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: i8,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store8`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store8_at)]
            I64Store8At {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store8At`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store8_at_imm)]
            I64Store8AtImm {
                /// The value to be stored.
                value: i8,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `i64.store16` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store16)]
            I64Store16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store16`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store16_imm)]
            I64Store16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store16`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store16_offset16)]
            I64Store16Offset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store16Offset16`] with immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store16_offset16_imm)]
            I64Store16Offset16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: i16,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store16`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store16_at)]
            I64Store16At {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store16At`] with immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store16_at_imm)]
            I64Store16AtImm {
                /// The value to be stored.
                value: i16,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `i64.store32` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store32)]
            I64Store32 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store32`] with 16-bit immediate `value`.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::Imm16AndImm32`] encoding `value` and `offset`.
            #[snake_name(i64_store32_imm16)]
            I64Store32Imm16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store32`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store32_offset16)]
            I64Store32Offset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::I64Store32Offset16`] with 16-bit immediate `value`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(i64_store32_offset16_imm16)]
            I64Store32Offset16Imm16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Const16<i32>,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store32`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store32_at)]
            I64Store32At {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::I64Store32At`] with 16-bit immediate `value`.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(i64_store32_at_imm16)]
            I64Store32AtImm16 {
                /// The value to be stored.
                value: Const16<i32>,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `f32.store` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(f32_store)]
            F32Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `f32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::F32Store`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(f32_store_offset16)]
            F32StoreOffset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `f32.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::F32Store`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(f32_store_at)]
            F32StoreAt {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },

            /// Wasm `f32.store` equivalent Wasmi instruction.
            ///
            /// # Encoding
            ///
            /// Followed by an [`Instruction::RegisterAndImm32`] encoding `value` and `offset`.
            #[snake_name(f64_store)]
            F64Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The linear memory index for which the store instruction is executed.
                memory: Memory,
            },
            /// Wasm `f64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// - Variant of [`Instruction::F64Store`] with a 16-bit `offset`.
            /// - Operates on the default Wasm memory instance.
            #[snake_name(f64_store_offset16)]
            F64StoreOffset16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the pointer offset of the `store` instruction.
                offset: Const16<u32>,
                /// The value to be stored.
                value: Reg,
            },
            /// Wasm `f64.store` equivalent Wasmi instruction.
            ///
            /// # Note
            ///
            /// Variant of [`Instruction::F64Store`] with an immediate `ptr+offset` address.
            ///
            /// # Encoding
            ///
            /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
            ///
            /// - Operates on the default Wasm memory instance if missing.
            #[snake_name(f64_store_at)]
            F64StoreAt {
                /// The value to be stored.
                value: Reg,
                /// The constant address to store the value.
                address: u32,
            },

            /// `i32` equality comparison instruction: `r0 = r1 == r2`
            #[snake_name(i32_eq)]
            I32Eq{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` equality comparison instruction with immediate: `r0 = r1 == c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32Eq`]
            /// for 16-bit right-hand side constant values.
            #[snake_name(i32_eq_imm16)]
            I32EqImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// `i32` inequality comparison instruction: `r0 = r1 != r2`
            #[snake_name(i32_ne)]
            I32Ne{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` inequality comparison instruction with immediate: `r0 = r1 != c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32Ne`]
            /// for 16-bit right-hand side constant values.
            #[snake_name(i32_ne_imm16)]
            I32NeImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// `i32` signed less-than comparison instruction: `r0 = r1 < r2`
            #[snake_name(i32_lt_s)]
            I32LtS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32LtS`]
            /// for small right-hand side constant values.
            #[snake_name(i32_lt_s_imm16)]
            I32LtSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` unsigned less-than comparison instruction: `r0 = r1 < r2`
            #[snake_name(i32_lt_u)]
            I32LtU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32LtU`]
            /// for small right-hand side constant values.
            #[snake_name(i32_lt_u_imm16)]
            I32LtUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u32>,
            },

            /// `i32` signed greater-than comparison instruction: `r0 = r1 > r2`
            #[snake_name(i32_gt_s)]
            I32GtS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32GtS`]
            /// for small right-hand side constant values.
            #[snake_name(i32_gt_s_imm16)]
            I32GtSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` unsigned greater-than comparison instruction: `r0 = r1 > r2`
            #[snake_name(i32_gt_u)]
            I32GtU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32GtU`]
            /// for small right-hand side constant values.
            #[snake_name(i32_gt_u_imm16)]
            I32GtUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u32>,
            },

            /// `i32` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
            #[snake_name(i32_le_s)]
            I32LeS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32LeS`]
            /// for small right-hand side constant values.
            #[snake_name(i32_le_s_imm16)]
            I32LeSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
            #[snake_name(i32_le_u)]
            I32LeU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32LeU`]
            /// for small right-hand side constant values.
            #[snake_name(i32_le_u_imm16)]
            I32LeUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u32>,
            },

            /// `i32` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
            #[snake_name(i32_ge_s)]
            I32GeS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32GeS`]
            /// for small right-hand side constant values.
            #[snake_name(i32_ge_s_imm16)]
            I32GeSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
            #[snake_name(i32_ge_u)]
            I32GeU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I32GeU`]
            /// for small right-hand side constant values.
            #[snake_name(i32_ge_u_imm16)]
            I32GeUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u32>,
            },

            /// `i64` equality comparison instruction: `r0 = r1 == r2`
            #[snake_name(i64_eq)]
            I64Eq{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` equality comparison instruction with immediate: `r0 = r1 == c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64Eq`]
            /// for 16-bit right-hand side constant values.
            #[snake_name(i64_eq_imm16)]
            I64EqImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` inequality comparison instruction: `r0 = r1 != r2`
            #[snake_name(i64_ne)]
            I64Ne{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` inequality comparison instruction with immediate: `r0 = r1 != c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64Ne`]
            /// for 16-bit right-hand side constant values.
            #[snake_name(i64_ne_imm16)]
            I64NeImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` signed less-than comparison instruction: `r0 = r1 < r2`
            #[snake_name(i64_lt_s)]
            I64LtS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed less-than comparison instruction with immediate: `r0 = r1 < c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64LtS`]
            /// for small right-hand side constant values.
            #[snake_name(i64_lt_s_imm16)]
            I64LtSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` unsigned less-than comparison instruction: `r0 = r1 < r2`
            #[snake_name(i64_lt_u)]
            I64LtU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` unsigned less-than comparison instruction with immediate: `r0 = r1 < c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64LtU`]
            /// for small right-hand side constant values.
            #[snake_name(i64_lt_u_imm16)]
            I64LtUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u64>,
            },

            /// `i64` signed greater-than comparison instruction: `r0 = r1 > r2`
            #[snake_name(i64_gt_s)]
            I64GtS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed greater-than comparison instruction with immediate: `r0 = r1 > c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64GtS`]
            /// for small right-hand side constant values.
            #[snake_name(i64_gt_s_imm16)]
            I64GtSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` unsigned greater-than comparison instruction: `r0 = r1 > r2`
            #[snake_name(i64_gt_u)]
            I64GtU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` unsigned greater-than comparison instruction with immediate: `r0 = r1 > c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64GtU`]
            /// for small right-hand side constant values.
            #[snake_name(i64_gt_u_imm16)]
            I64GtUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u64>,
            },

            /// `i64` signed less-than or equals comparison instruction: `r0 = r1 <= r2`
            #[snake_name(i64_le_s)]
            I64LeS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64LeS`]
            /// for small right-hand side constant values.
            #[snake_name(i64_le_s_imm16)]
            I64LeSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` unsigned less-than or equals comparison instruction: `r0 = r1 <= r2`
            #[snake_name(i64_le_u)]
            I64LeU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` unsigned less-than or equals comparison instruction with immediate: `r0 = r1 <= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64LeU`]
            /// for small right-hand side constant values.
            #[snake_name(i64_le_u_imm16)]
            I64LeUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u64>,
            },

            /// `i64` signed greater-than or equals comparison instruction: `r0 = r1 >= r2`
            #[snake_name(i64_ge_s)]
            I64GeS{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64GeS`]
            /// for small right-hand side constant values.
            #[snake_name(i64_ge_s_imm16)]
            I64GeSImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` unsigned greater-than or equals comparison instruction: `r0 = r1 >= r2`
            #[snake_name(i64_ge_u)]
            I64GeU{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` unsigned greater-than or equals comparison instruction with immediate: `r0 = r1 >= c0`
            ///
            /// # Note
            ///
            /// This is an optimization of [`Instruction::I64GeU`]
            /// for small right-hand side constant values.
            #[snake_name(i64_ge_u_imm16)]
            I64GeUImm16{
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<u64>,
            },

            /// `f32` equality comparison instruction: `r0 = r1 == r2`
            #[snake_name(f32_eq)]
            F32Eq{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f32` inequality comparison instruction: `r0 = r1 != r2`
            #[snake_name(f32_ne)]
            F32Ne{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f32` less-than comparison instruction: `r0 = r1 < r2`
            #[snake_name(f32_lt)]
            F32Lt{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f32` less-than or equals comparison instruction: `r0 = r1 <= r2`
            #[snake_name(f32_le)]
            F32Le{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f32` greater-than comparison instruction: `r0 = r1 > r2`
            #[snake_name(f32_gt)]
            F32Gt{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f32` greater-than or equals comparison instruction: `r0 = r1 >= r2`
            #[snake_name(f32_ge)]
            F32Ge{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },

            /// `f64` equality comparison instruction: `r0 = r1 == r2`
            #[snake_name(f64_eq)]
            F64Eq{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f64` inequality comparison instruction: `r0 = r1 != r2`
            #[snake_name(f64_ne)]
            F64Ne{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f64` less-than comparison instruction: `r0 = r1 < r2`
            #[snake_name(f64_lt)]
            F64Lt{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f64` less-than or equals comparison instruction: `r0 = r1 <= r2`
            #[snake_name(f64_le)]
            F64Le{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f64` greater-than comparison instruction: `r0 = r1 > r2`
            #[snake_name(f64_gt)]
            F64Gt{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `f64` greater-than or equals comparison instruction: `r0 = r1 >= r2`
            #[snake_name(f64_ge)]
            F64Ge{
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },

            /// `i32` count-leading-zeros (clz) instruction.
            #[snake_name(i32_clz)]
            I32Clz {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// `i32` count-trailing-zeros (ctz) instruction.
            #[snake_name(i32_ctz)]
            I32Ctz {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// `i32` pop-count instruction.
            #[snake_name(i32_popcnt)]
            I32Popcnt {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// `i32` add instruction: `r0 = r1 + r2`
            #[snake_name(i32_add)]
            I32Add {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` add (small) immediate instruction: `r0 = r1 + c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I32Add`] for 16-bit constant values.
            #[snake_name(i32_add_imm16)]
            I32AddImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// `i32` subtract instruction: `r0 = r1 - r2`
            #[snake_name(i32_sub)]
            I32Sub {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` subtract immediate instruction: `r0 = c0 - r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32Sub`] for 16-bit constant values.
            /// - Required instruction since subtraction is not commutative.
            #[snake_name(i32_sub_imm16_lhs)]
            I32SubImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i32` multiply instruction: `r0 = r1 * r2`
            #[snake_name(i32_mul)]
            I32Mul {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` multiply immediate instruction: `r0 = r1 * c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I32Mul`] for 16-bit constant values.
            #[snake_name(i32_mul_imm16)]
            I32MulImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// `i32` signed-division instruction: `r0 = r1 / r2`
            #[snake_name(i32_div_s)]
            I32DivS {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed-division immediate instruction: `r0 = r1 / c0`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32DivS`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            #[snake_name(i32_div_s_imm16_rhs)]
            I32DivSImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroI32>,
            },
            /// `i32` signed-division immediate instruction: `r0 = c0 / r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since signed-division is not commutative.
            #[snake_name(i32_div_s_imm16_lhs)]
            I32DivSImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i32` unsigned-division instruction: `r0 = r1 / r2`
            #[snake_name(i32_div_u)]
            I32DivU {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` unsigned-division immediate instruction: `r0 = r1 / c0`
            ///
            /// # Note
            ///
            /// Guarantees that the right-hand side operand is not zero.
            ///
            /// # Encoding
            ///
            /// Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
            #[snake_name(i32_div_u_imm16_rhs)]
            I32DivUImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroU32>,
            },
            /// `i32` unsigned-division immediate instruction: `r0 = c0 / r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32DivU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since `i32` unsigned-division is not commutative.
            #[snake_name(i32_div_u_imm16_lhs)]
            I32DivUImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<u32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i32` signed-remainder instruction: `r0 = r1 % r2`
            #[snake_name(i32_rem_s)]
            I32RemS {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed-remainder immediate instruction: `r0 = r1 % c0`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            #[snake_name(i32_rem_s_imm16_rhs)]
            I32RemSImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroI32>,
            },
            /// `i32` signed-remainder immediate instruction: `r0 = c0 % r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since `i32` signed-remainder is not commutative.
            #[snake_name(i32_rem_s_imm16_lhs)]
            I32RemSImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
            #[snake_name(i32_rem_u)]
            I32RemU {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i32` signed-remainder immediate instruction: `r0 = r1 % c0`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            #[snake_name(i32_rem_u_imm16_rhs)]
            I32RemUImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroU32>,
            },
            /// `i32` unsigned-remainder immediate instruction: `r0 = c0 % r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since unsigned-remainder is not commutative.
            #[snake_name(i32_rem_u_imm16_lhs)]
            I32RemUImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<u32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i32` bitwise-and instruction: `r0 = r1 & r2`
            #[snake_name(i32_and)]
            I32And {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`].
            #[snake_name(i32_and_eqz)]
            I32AndEqz {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
            #[snake_name(i32_and_eqz_imm16)]
            I32AndEqzImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I32And`] for 16-bit constant values.
            #[snake_name(i32_and_imm16)]
            I32AndImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// `i32` bitwise-or instruction: `r0 = r1 & r2`
            #[snake_name(i32_or)]
            I32Or {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`].
            #[snake_name(i32_or_eqz)]
            I32OrEqz {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
            #[snake_name(i32_or_eqz_imm16)]
            I32OrEqzImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I32Or`] for 16-bit constant values.
            #[snake_name(i32_or_imm16)]
            I32OrImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
            #[snake_name(i32_xor)]
            I32Xor {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`].
            #[snake_name(i32_xor_eqz)]
            I32XorEqz {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
            #[snake_name(i32_xor_eqz_imm16)]
            I32XorEqzImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },
            /// `i32` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I32Xor`] for 16-bit constant values.
            #[snake_name(i32_xor_imm16)]
            I32XorImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i32>,
            },

            /// A Wasm `i32.shl` equivalent Wasmi instruction.
            #[snake_name(i32_shl)]
            I32Shl {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i32.shl` equivalent Wasmi instruction with 16-bit immediate `rhs` operand.
            #[snake_name(i32_shl_by)]
            I32ShlBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i32>,
            },
            /// A Wasm `i32.shl` equivalent Wasmi instruction with 16-bit immediate `lhs` operand.
            #[snake_name(i32_shl_imm16)]
            I32ShlImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i32.shr_u` equivalent Wasmi instruction.
            #[snake_name(i32_shr_u)]
            I32ShrU {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i32.shr_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i32_shr_u_by)]
            I32ShrUBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i32>,
            },
            /// A Wasm `i32.shr_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i32_shr_u_imm16)]
            I32ShrUImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i32.shr_s` equivalent Wasmi instruction.
            #[snake_name(i32_shr_s)]
            I32ShrS {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i32.shr_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i32_shr_s_by)]
            I32ShrSBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i32>,
            },
            /// A Wasm `i32.shr_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i32_shr_s_imm16)]
            I32ShrSImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i32.rotl` equivalent Wasmi instruction.
            #[snake_name(i32_rotl)]
            I32Rotl {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i32.rotl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i32_rotl_by)]
            I32RotlBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i32>,
            },
            /// A Wasm `i32.rotl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i32_rotl_imm16)]
            I32RotlImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i32.rotr` equivalent Wasmi instruction.
            #[snake_name(i32_rotr)]
            I32Rotr {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i32.rotr` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i32_rotr_by)]
            I32RotrBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i32>,
            },
            /// A Wasm `i32.rotr` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i32_rotr_imm16)]
            I32RotrImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i32>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i64` count-leading-zeros (clz) instruction.
            #[snake_name(i64_clz)]
            I64Clz {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// `i64` count-trailing-zeros (ctz) instruction.
            #[snake_name(i64_ctz)]
            I64Ctz {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// `i64` pop-count instruction.
            #[snake_name(i64_popcnt)]
            I64Popcnt {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// `i64` add instruction: `r0 = r1 + r2`
            #[snake_name(i64_add)]
            I64Add {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` add (small) immediate instruction: `r0 = r1 + c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I64Add`] for 16-bit constant values.
            #[snake_name(i64_add_imm16)]
            I64AddImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` subtract instruction: `r0 = r1 - r2`
            #[snake_name(i64_sub)]
            I64Sub {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` subtract immediate instruction: `r0 = c0 - r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64Sub`] for 16-bit constant values.
            /// - Required instruction since subtraction is not commutative.
            #[snake_name(i64_sub_imm16_lhs)]
            I64SubImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i64` multiply instruction: `r0 = r1 * r2`
            #[snake_name(i64_mul)]
            I64Mul {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` multiply immediate instruction: `r0 = r1 * c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I64Mul`] for 16-bit constant values.
            #[snake_name(i64_mul_imm16)]
            I64MulImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` signed-division instruction: `r0 = r1 / r2`
            #[snake_name(i64_div_s)]
            I64DivS {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed-division immediate instruction: `r0 = r1 / c0`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64DivS`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            #[snake_name(i64_div_s_imm16_rhs)]
            I64DivSImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroI64>,
            },
            /// `i32` signed-division immediate instruction: `r0 = c0 / r1`
            ///
            /// # Note
            ///
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since signed-division is not commutative.
            /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
            #[snake_name(i64_div_s_imm16_lhs)]
            I64DivSImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i64` unsigned-division instruction: `r0 = r1 / r2`
            #[snake_name(i64_div_u)]
            I64DivU {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` unsigned-division immediate instruction: `r0 = r1 / c0`
            ///
            /// # Note
            ///
            /// Guarantees that the right-hand side operand is not zero.
            ///
            /// # Encoding
            ///
            /// Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
            #[snake_name(i64_div_u_imm16_rhs)]
            I64DivUImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroU64>,
            },
            /// `i64` unsigned-division immediate instruction: `r0 = c0 / r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64DivU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since unsigned-division is not commutative.
            #[snake_name(i64_div_u_imm16_lhs)]
            I64DivUImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<u64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i64` signed-remainder instruction: `r0 = r1 % r2`
            #[snake_name(i64_rem_s)]
            I64RemS {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed-remainder immediate instruction: `r0 = r1 % c0`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            #[snake_name(i64_rem_s_imm16_rhs)]
            I64RemSImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroI64>,
            },
            /// `i64` signed-remainder immediate instruction: `r0 = c0 % r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since signed-remainder is not commutative.
            #[snake_name(i64_rem_s_imm16_lhs)]
            I64RemSImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i64` unsigned-remainder instruction: `r0 = r1 % r2`
            #[snake_name(i64_rem_u)]
            I64RemU {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` signed-remainder immediate instruction: `r0 = r1 % c0`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            #[snake_name(i64_rem_u_imm16_rhs)]
            I64RemUImm16Rhs {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<NonZeroU64>,
            },
            /// `i64` unsigned-remainder immediate instruction: `r0 = c0 % r1`
            ///
            /// # Note
            ///
            /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
            /// - Guarantees that the right-hand side operand is not zero.
            /// - Required instruction since unsigned-remainder is not commutative.
            #[snake_name(i64_rem_u_imm16_lhs)]
            I64RemUImm16Lhs {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<u64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// `i64` bitwise-and instruction: `r0 = r1 & r2`
            #[snake_name(i64_and)]
            I64And {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I64And`] for 16-bit constant values.
            #[snake_name(i64_and_imm16)]
            I64AndImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` bitwise-or instruction: `r0 = r1 & r2`
            #[snake_name(i64_or)]
            I64Or {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I64Or`] for 16-bit constant values.
            #[snake_name(i64_or_imm16)]
            I64OrImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// `i64` bitwise-or instruction: `r0 = r1 ^ r2`
            #[snake_name(i64_xor)]
            I64Xor {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
            ///
            /// # Note
            ///
            /// Optimized variant of [`Instruction::I64Xor`] for 16-bit constant values.
            #[snake_name(i64_xor_imm16)]
            I64XorImm16 {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: Const16<i64>,
            },

            /// A Wasm `i64.shl` equivalent Wasmi instruction.
            #[snake_name(i64_shl)]
            I64Shl {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i64.shl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i64_shl_by)]
            I64ShlBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i64>,
            },
            /// A Wasm `i64.shl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i64_shl_imm16)]
            I64ShlImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i64.shr_u` equivalent Wasmi instruction.
            #[snake_name(i64_shr_u)]
            I64ShrU {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i64.shr_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i64_shr_u_by)]
            I64ShrUBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i64>,
            },
            /// A Wasm `i64.shr_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i64_shr_u_imm16)]
            I64ShrUImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i64.shr_s` equivalent Wasmi instruction.
            #[snake_name(i64_shr_s)]
            I64ShrS {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i64.shr_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i64_shr_s_by)]
            I64ShrSBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i64>,
            },
            /// A Wasm `i64.shr_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i64_shr_s_imm16)]
            I64ShrSImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i64.rotl` equivalent Wasmi instruction.
            #[snake_name(i64_rotl)]
            I64Rotl {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i64.rotl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i64_rotl_by)]
            I64RotlBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i64>,
            },
            /// A Wasm `i64.rotl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i64_rotl_imm16)]
            I64RotlImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// A Wasm `i64.rotr` equivalent Wasmi instruction.
            #[snake_name(i64_rotr)]
            I64Rotr {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// A Wasm `i64.rotr` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
            #[snake_name(i64_rotr_by)]
            I64RotrBy {
                @result: Reg,
                /// The register holding one of the operands.
                lhs: Reg,
                /// The 16-bit immediate value.
                rhs: ShiftAmount<i64>,
            },
            /// A Wasm `i64.rotr` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
            #[snake_name(i64_rotr_imm16)]
            I64RotrImm16 {
                @result: Reg,
                /// The 16-bit immediate value.
                lhs: Const16<i64>,
                /// The register holding one of the operands.
                rhs: Reg,
            },

            /// Wasm `i32.wrap_i64` instruction.
            #[snake_name(i32_wrap_i64)]
            I32WrapI64 {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// Wasm `i32.extend8_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `sign-extension` proposal.
            #[snake_name(i32_extend8_s)]
            I32Extend8S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.extend16_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `sign-extension` proposal.
            #[snake_name(i32_extend16_s)]
            I32Extend16S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.extend8_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `sign-extension` proposal.
            #[snake_name(i64_extend8_s)]
            I64Extend8S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm(UnaryInstr) `i64.extend16_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `sign-extension` proposal.
            #[snake_name(i64_extend16_s)]
            I64Extend16S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.extend32_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `sign-extension` proposal.
            #[snake_name(i64_extend32_s)]
            I64Extend32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// Wasm `f32.abs` instruction.
            #[snake_name(f32_abs)]
            F32Abs {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.neg` instruction.
            #[snake_name(f32_neg)]
            F32Neg {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.ceil` instruction.
            #[snake_name(f32_ceil)]
            F32Ceil {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.floor` instruction.
            #[snake_name(f32_floor)]
            F32Floor {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.trunc` instruction.
            #[snake_name(f32_trunc)]
            F32Trunc {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.nearest` instruction.
            #[snake_name(f32_nearest)]
            F32Nearest {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.sqrt` instruction.
            #[snake_name(f32_sqrt)]
            F32Sqrt {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.add` instruction: `r0 = r1 + r2`
            #[snake_name(f32_add)]
            F32Add {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.sub` instruction: `r0 = r1 - r2`
            #[snake_name(f32_sub)]
            F32Sub {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.mul` instruction: `r0 = r1 * r2`
            #[snake_name(f32_mul)]
            F32Mul {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.div` instruction: `r0 = r1 / r2`
            #[snake_name(f32_div)]
            F32Div {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.min` instruction: `r0 = min(r1, r2)`
            #[snake_name(f32_min)]
            F32Min {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.max` instruction: `r0 = max(r1, r2)`
            #[snake_name(f32_max)]
            F32Max {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.copysign` instruction: `r0 = copysign(r1, r2)`
            #[snake_name(f32_copysign)]
            F32Copysign {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f32.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
            #[snake_name(f32_copysign_imm)]
            F32CopysignImm {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Sign<f32>,
            },

            /// Wasm `f64.abs` instruction.
            #[snake_name(f64_abs)]
            F64Abs {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.neg` instruction.
            #[snake_name(f64_neg)]
            F64Neg {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.ceil` instruction.
            #[snake_name(f64_ceil)]
            F64Ceil {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.floor` instruction.
            #[snake_name(f64_floor)]
            F64Floor {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.trunc` instruction.
            #[snake_name(f64_trunc)]
            F64Trunc {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.nearest` instruction.
            #[snake_name(f64_nearest)]
            F64Nearest {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.sqrt` instruction.
            #[snake_name(f64_sqrt)]
            F64Sqrt {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.add` instruction: `r0 = r1 + r2`
            #[snake_name(f64_add)]
            F64Add {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.sub` instruction: `r0 = r1 - r2`
            #[snake_name(f64_sub)]
            F64Sub {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.mul` instruction: `r0 = r1 * r2`
            #[snake_name(f64_mul)]
            F64Mul {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.div` instruction: `r0 = r1 / r2`
            #[snake_name(f64_div)]
            F64Div {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.min` instruction: `r0 = min(r1, r2)`
            #[snake_name(f64_min)]
            F64Min {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.max` instruction: `r0 = max(r1, r2)`
            #[snake_name(f64_max)]
            F64Max {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.copysign` instruction: `r0 = copysign(r1, r2)`
            #[snake_name(f64_copysign)]
            F64Copysign {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Reg,
            },
            /// Wasm `f64.copysign` instruction with immediate: `r0 = copysign(r1, c0)`
            #[snake_name(f64_copysign_imm)]
            F64CopysignImm {
                @result: Reg,
                /// The register holding the left-hand side value.
                lhs: Reg,
                /// The register holding the right-hand side value.
                rhs: Sign<f64>,
            },

            /// Wasm `i32.trunc_f32_s` instruction.
            #[snake_name(i32_trunc_f32_s)]
            I32TruncF32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.trunc_f32_u` instruction.
            #[snake_name(i32_trunc_f32_u)]
            I32TruncF32U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.trunc_f64_s` instruction.
            #[snake_name(i32_trunc_f64_s)]
            I32TruncF64S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.trunc_f64_u` instruction.
            #[snake_name(i32_trunc_f64_u)]
            I32TruncF64U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_f32_s` instruction.
            #[snake_name(i64_trunc_f32_s)]
            I64TruncF32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_f32_u` instruction.
            #[snake_name(i64_trunc_f32_u)]
            I64TruncF32U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_f64_s` instruction.
            #[snake_name(i64_trunc_f64_s)]
            I64TruncF64S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_f64_u` instruction.
            #[snake_name(i64_trunc_f64_u)]
            I64TruncF64U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// Wasm `i32.trunc_sat_f32_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i32_trunc_sat_f32_s)]
            I32TruncSatF32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.trunc_sat_f32_u` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i32_trunc_sat_f32_u)]
            I32TruncSatF32U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.trunc_sat_f64_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i32_trunc_sat_f64_s)]
            I32TruncSatF64S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i32.trunc_sat_f64_u` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i32_trunc_sat_f64_u)]
            I32TruncSatF64U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_sat_f32_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i64_trunc_sat_f32_s)]
            I64TruncSatF32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_sat_f32_u` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i64_trunc_sat_f32_u)]
            I64TruncSatF32U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_sat_f64_s` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i64_trunc_sat_f64_s)]
            I64TruncSatF64S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `i64.trunc_sat_f64_u` instruction.
            ///
            /// # Note
            ///
            /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
            #[snake_name(i64_trunc_sat_f64_u)]
            I64TruncSatF64U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// Wasm `f32.demote_f64` instruction.
            #[snake_name(f32_demote_f64)]
            F32DemoteF64 {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.promote_f32` instruction.
            #[snake_name(f64_promote_f32)]
            F64PromoteF32 {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// Wasm `f32.convert_i32_s` instruction.
            #[snake_name(f32_convert_i32_s)]
            F32ConvertI32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.convert_i32_u` instruction.
            #[snake_name(f32_convert_i32_u)]
            F32ConvertI32U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.convert_i64_s` instruction.
            #[snake_name(f32_convert_i64_s)]
            F32ConvertI64S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f32.convert_i64_u` instruction.
            #[snake_name(f32_convert_i64_u)]
            F32ConvertI64U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.convert_i32_s` instruction.
            #[snake_name(f64_convert_i32_s)]
            F64ConvertI32S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.convert_i32_u` instruction.
            #[snake_name(f64_convert_i32_u)]
            F64ConvertI32U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.convert_i64_s` instruction.
            #[snake_name(f64_convert_i64_s)]
            F64ConvertI64S {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },
            /// Wasm `f64.convert_i64_u` instruction.
            #[snake_name(f64_convert_i64_u)]
            F64ConvertI64U {
                @result: Reg,
                /// The register holding the input of the instruction.
                input: Reg,
            },

            /// A Wasm `table.get` instruction: `result = table[index]`
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
            #[snake_name(table_get)]
            TableGet {
                @result: Reg,
                /// The register storing the index of the table element to get.
                index: Reg,
            },
            /// Variant of [`Instruction::TableGet`] with constant `index` value.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
            #[snake_name(table_get_imm)]
            TableGetImm {
                @result: Reg,
                /// The constant `index` value of the table element to get.
                index: u32,
            },

            /// A Wasm `table.size` instruction.
            #[snake_name(table_size)]
            TableSize {
                @result: Reg,
                /// The index identifying the table for the instruction.
                table: Table,
            },

            /// A Wasm `table.set` instruction: `table[index] = value`
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
            #[snake_name(table_set)]
            TableSet {
                /// The register holding the `index` of the instruction.
                index: Reg,
                /// The register holding the `value` of the instruction.
                value: Reg,
            },
            /// Variant of [`Instruction::TableSet`] with constant `index` value.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
            #[snake_name(table_set_at)]
            TableSetAt {
                /// The register holding the `value` of the instruction.
                value: Reg,
                /// The constant `index` of the instruction.
                index: u32,
            },

            /// Wasm `table.copy <dst> <src>` instruction.
            ///
            /// Copies elements from `table<src>[src..src+len]` to `table<dst>[dst..dst+len]`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy)]
            TableCopy {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `dst` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_to)]
            TableCopyTo {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `src` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_from)]
            TableCopyFrom {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `dst` and `src` indices.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_from_to)]
            TableCopyFromTo {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` field.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_exact)]
            TableCopyExact {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` and `dst`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_to_exact)]
            TableCopyToExact {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` and `src`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_from_exact)]
            TableCopyFromExact {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::TableCopy`] with a constant 16-bit `len` and `src`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the `dst` Wasm table instance
            /// 2. [`Instruction::TableIndex`]: the `src` Wasm table instance
            #[snake_name(table_copy_from_to_exact)]
            TableCopyFromToExact {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Const16<u32>,
            },

            /// Wasm `table.init <table> <elem>` instruction.
            ///
            /// Copies elements from `table[src..src+len]` to `table[dst..dst+len]`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init)]
            TableInit {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `dst` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_to)]
            TableInitTo {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `src` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_from)]
            TableInitFrom {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `dst` and `src` indices.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_from_to)]
            TableInitFromTo {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Reg,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` field.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_exact)]
            TableInitExact {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` and `dst`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_to_exact)]
            TableInitToExact {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Reg,
                /// The number of copied elements.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` and `src`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_from_exact)]
            TableInitFromExact {
                /// The start index of the `dst` table.
                dst: Reg,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::TableInit`] with a constant 16-bit `len` and `src`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the tables.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::TableIndex`]: the Wasm `table` instance
            /// 2. [`Instruction::ElemIndex`]: the Wasm `element` segment instance
            #[snake_name(table_init_from_to_exact)]
            TableInitFromToExact {
                /// The start index of the `dst` table.
                dst: Const16<u32>,
                /// The start index of the `src` table.
                src: Const16<u32>,
                /// The number of copied elements.
                len: Const16<u32>,
            },

            /// Wasm `table.fill <table>` instruction: `table[dst..dst+len] = value`
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
            #[snake_name(table_fill)]
            TableFill {
                /// The start index of the table to fill.
                dst: Reg,
                /// The number of elements to fill.
                len: Reg,
                /// The value of the filled elements.
                value: Reg,
            },
            /// Variant of [`Instruction::TableFill`] with 16-bit constant `dst` index.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
            #[snake_name(table_fill_at)]
            TableFillAt {
                /// The start index of the table to fill.
                dst: Const16<u32>,
                /// The number of elements to fill.
                len: Reg,
                /// The value of the filled elements.
                value: Reg,
            },
            /// Variant of [`Instruction::TableFill`] with 16-bit constant `len` index.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
            #[snake_name(table_fill_exact)]
            TableFillExact {
                /// The start index of the table to fill.
                dst: Reg,
                /// The number of elements to fill.
                len: Const16<u32>,
                /// The value of the filled elements.
                value: Reg,
            },
            /// Variant of [`Instruction::TableFill`] with 16-bit constant `dst` and `len` fields.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
            #[snake_name(table_fill_at_exact)]
            TableFillAtExact {
                /// The start index of the table to fill.
                dst: Const16<u32>,
                /// The number of elements to fill.
                len: Const16<u32>,
                /// The value of the filled elements.
                value: Reg,
            },

            /// Wasm `table.grow <table>` instruction.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
            #[snake_name(table_grow)]
            TableGrow {
                @result: Reg,
                /// The number of elements to add to the table.
                delta: Reg,
                /// The value that is used to fill up the new cells.
                value: Reg,
            },
            /// Variant of [`Instruction::TableGrow`] with 16-bit constant `delta`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
            #[snake_name(table_grow_imm)]
            TableGrowImm {
                @result: Reg,
                /// The number of elements to add to the table.
                delta: Const16<u32>,
                /// The value that is used to fill up the new cells.
                value: Reg,
            },

            /// A Wasm `elem.drop` equalivalent Wasmi instruction.
            #[snake_name(elem_drop)]
            ElemDrop {
                index: Elem,
            },
            /// A Wasm `data.drop` equalivalent Wasmi instruction.
            #[snake_name(data_drop)]
            DataDrop {
                index: Data,
            },

            /// Wasm `memory.size` instruction.
            #[snake_name(memory_size)]
            MemorySize {
                @result: Reg,
                /// The index identifying the Wasm linear memory for the instruction.
                memory: Memory,
            },

            /// Wasm `memory.grow` instruction.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_grow)]
            MemoryGrow {
                @result: Reg,
                /// The number of pages to add to the memory.
                delta: Reg,
            },
            /// Variant of [`Instruction::MemoryGrow`] with 16-bit constant `delta`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_grow_by)]
            MemoryGrowBy {
                @result: Reg,
                /// The number of pages to add to the memory.
                delta: u32,
            },

            /// Wasm `memory.copy` instruction.
            ///
            /// Copies elements from `memory[src..src+len]` to `memory[dst..dst+len]`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy)]
            MemoryCopy {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` memory.
                src: Reg,
                /// The number of copied bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `dst` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_to)]
            MemoryCopyTo {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` memory.
                src: Reg,
                /// The number of copied bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `src` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_from)]
            MemoryCopyFrom {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` memory.
                src: Const16<u32>,
                /// The number of copied bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `dst` and `src` indices.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_from_to)]
            MemoryCopyFromTo {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` memory.
                src: Const16<u32>,
                /// The number of copied bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` field.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the memories.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_exact)]
            MemoryCopyExact {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` memory.
                src: Reg,
                /// The number of copied bytes.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` and `dst`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the memories.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_to_exact)]
            MemoryCopyToExact {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` memory.
                src: Reg,
                /// The number of copied bytes.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` and `src`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the memories.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_from_exact)]
            MemoryCopyFromExact {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` memory.
                src: Const16<u32>,
                /// The number of copied bytes.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryCopy`] with a constant 16-bit `len` and `src`.
            ///
            /// # Note
            ///
            /// This instruction copies _exactly_ `len` elements between the memories.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the `dst` Wasm linear memory instance
            /// 2. [`Instruction::MemoryIndex`]: the `src` Wasm linear memory instance
            #[snake_name(memory_copy_from_to_exact)]
            MemoryCopyFromToExact {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` memory.
                src: Const16<u32>,
                /// The number of copied bytes.
                len: Const16<u32>,
            },

            /// Wasm `memory.fill` instruction.
            ///
            /// Sets bytes of `memory[dst..dst+len]` to `value`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill)]
            MemoryFill {
                /// The start index of the memory to fill.
                dst: Reg,
                /// The byte value used to fill the memory.
                value: Reg,
                /// The number of bytes to fill.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `dst` index.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_at)]
            MemoryFillAt {
                /// The start index of the memory to fill.
                dst: Const16<u32>,
                /// The byte value used to fill the memory.
                value: Reg,
                /// The number of bytes to fill.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryFill`] with constant fill `value`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_imm)]
            MemoryFillImm {
                /// The start index of the memory to fill.
                dst: Reg,
                /// The byte value used to fill the memory.
                value: u8,
                /// The number of bytes to fill.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `len` value.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_exact)]
            MemoryFillExact {
                /// The start index of the memory to fill.
                dst: Reg,
                /// The byte value used to fill the memory.
                value: Reg,
                /// The number of bytes to fill.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `value`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_at_imm)]
            MemoryFillAtImm {
                /// The start index of the memory to fill.
                dst: Const16<u32>,
                /// The byte value used to fill the memory.
                value: u8,
                /// The number of bytes to fill.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `len`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_at_exact)]
            MemoryFillAtExact {
                /// The start index of the memory to fill.
                dst: Const16<u32>,
                /// The byte value used to fill the memory.
                value: Reg,
                /// The number of bytes to fill.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryFill`] with constant fill `value` and `len`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_imm_exact)]
            MemoryFillImmExact {
                /// The start index of the memory to fill.
                dst: Reg,
                /// The byte value used to fill the memory.
                value: u8,
                /// The number of bytes to fill.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryFill`] with constant `dst` index, fill `value` and `len`.
            ///
            /// # Encoding
            ///
            /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
            #[snake_name(memory_fill_at_imm_exact)]
            MemoryFillAtImmExact {
                /// The start index of the memory to fill.
                dst: Const16<u32>,
                /// The byte value used to fill the memory.
                value: u8,
                /// The number of bytes to fill.
                len: Const16<u32>,
            },

            /// Wasm `memory.init <data>` instruction.
            ///
            /// Initializes bytes of `memory[dst..dst+len]` from `data[src..src+len]`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init)]
            MemoryInit {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` data segment.
                src: Reg,
                /// The number of bytes to initialize.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `dst` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_to)]
            MemoryInitTo {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` data segment.
                src: Reg,
                /// The number of initialized bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `src` index.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_from)]
            MemoryInitFrom {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` data segment.
                src: Const16<u32>,
                /// The number of initialized bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `dst` and `src` indices.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_from_to)]
            MemoryInitFromTo {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` data segment.
                src: Const16<u32>,
                /// The number of initialized bytes.
                len: Reg,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` field.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_exact)]
            MemoryInitExact {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` data segment.
                src: Reg,
                /// The number of initialized bytes.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` and `dst`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_to_exact)]
            MemoryInitToExact {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` data segment.
                src: Reg,
                /// The number of initialized bytes.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` and `src`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_from_exact)]
            MemoryInitFromExact {
                /// The start index of the `dst` memory.
                dst: Reg,
                /// The start index of the `src` data segment.
                src: Const16<u32>,
                /// The number of initialized bytes.
                len: Const16<u32>,
            },
            /// Variant of [`Instruction::MemoryInit`] with a constant 16-bit `len` and `src`.
            ///
            /// # Encoding
            ///
            /// This [`Instruction`] must be followed by
            ///
            /// 1. [`Instruction::MemoryIndex`]: the Wasm `memory` instance
            /// 1. [`Instruction::DataIndex`]: the `data` segment to initialize the memory
            #[snake_name(memory_init_from_to_exact)]
            MemoryInitFromToExact {
                /// The start index of the `dst` memory.
                dst: Const16<u32>,
                /// The start index of the `src` data segment.
                src: Const16<u32>,
                /// The number of initialized bytes.
                len: Const16<u32>,
            },

            /// A [`Table`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(table_index)]
            TableIndex {
                index: Table,
            },
            /// A [`Memory`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(memory_index)]
            MemoryIndex {
                index: Memory,
            },
            /// A [`Data`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(data_index)]
            DataIndex {
                index: Data,
            },
            /// An [`Elem`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(elem_index)]
            ElemIndex {
                index: Elem,
            },
            /// A [`AnyConst32`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(const32)]
            Const32 {
                value: AnyConst32
            },
            /// A [`Const32<i64>`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(i64const32)]
            I64Const32 {
                value: Const32<i64>
            },
            /// A [`Const32<f64>`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(f64const32)]
            F64Const32 {
                value: Const32<f64>
            },
            /// A Wasm `br_table` branching target which copies values before branching.
            ///
            /// # Encoding
            ///
            /// This always follows
            ///
            /// - [`Instruction::BranchTable1`]
            /// - [`Instruction::BranchTable2`]
            /// - [`Instruction::BranchTableSpan`]
            /// - [`Instruction::BranchTableMany`]
            #[snake_name(branch_table_target)]
            BranchTableTarget {
                /// The registers where the values are going to be copied.
                results: RegSpan,
                /// The branching offset of the branch table target.
                offset: BranchOffset,
            },
            /// A Wasm `br_table` branching target which copies overlapping values before branching.
            ///
            /// # Encoding
            ///
            /// This always follows
            ///
            /// - [`Instruction::BranchTableSpan`]
            /// - [`Instruction::BranchTableMany`]
            #[snake_name(branch_table_target_non_overlapping)]
            BranchTableTargetNonOverlapping {
                /// The registers where the values are going to be copied.
                results: RegSpan,
                /// The branching offset of the branch table target.
                offset: BranchOffset,
            },
            /// An instruction parameter with 16-bit and 32-bit immediate values.
            #[snake_name(imm16_and_imm32)]
            Imm16AndImm32 {
                /// The 16-bit immediate value.
                imm16: AnyConst16,
                /// The 32-bit immediate value.
                imm32: AnyConst32,
            },
            /// An instruction parameter with a [`Reg`] and a 32-bit immediate value.
            #[snake_name(register_and_imm32)]
            RegisterAndImm32 {
                /// The [`Reg`] parameter value.
                reg: Reg,
                /// The 32-bit immediate value.
                imm: AnyConst32,
            },
            /// A bounded [`RegSpan`] instruction parameter.
            #[snake_name(register_span)]
            RegisterSpan { span: BoundedRegSpan },
            /// A [`Reg`] instruction parameter.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(register)]
            Register {
                reg: Reg
            },
            /// Two [`Reg`] instruction parameters.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(register2)]
            Register2 {
                regs: [Reg; 2]
            },
            /// Three [`Reg`] instruction parameters.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            #[snake_name(register3)]
            Register3 {
                regs: [Reg; 3]
            },
            /// [`Reg`] slice parameters.
            ///
            /// # Note
            ///
            /// This [`Instruction`] only acts as a parameter to another
            /// one and will never be executed itself directly.
            ///
            /// # Encoding
            ///
            /// This must always be followed by one of
            ///
            /// - [`Instruction::Register`]
            /// - [`Instruction::Register2`]
            /// - [`Instruction::Register3`]
            #[snake_name(register_list)]
            RegisterList {
                regs: [Reg; 3]
            },
            /// Auxiliary [`Instruction`] to encode table access information for indirect call instructions.
            #[snake_name(call_indirect_params)]
            CallIndirectParams {
                /// The index of the called function in the table.
                index: Reg,
                /// The table which holds the called function at the index.
                table: Table,
            },
            /// Variant of [`Instruction::CallIndirectParams`] for 16-bit constant `index` parameter.
            #[snake_name(call_indirect_params_imm16)]
            CallIndirectParamsImm16 {
                /// The index of the called function in the table.
                index: Const16<u32>,
                /// The table which holds the called function at the index.
                table: Table,
            },
        }
    };
}
pub use for_each_op;
