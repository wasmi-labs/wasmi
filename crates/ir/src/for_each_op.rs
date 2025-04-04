macro_rules! for_each_op_grouped {
    ($mac:ident) => {
        $mac! {
            @default {
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
                    value: Local,
                },
                /// A Wasm `return` instruction.
                ///
                /// # Note
                ///
                /// Returns two values stored in registers.
                #[snake_name(return_loc2)]
                ReturnReg2 {
                    /// The returned values.
                    values: [Local; 2],
                },
                /// A Wasm `return` instruction.
                ///
                /// # Note
                ///
                /// Returns three values stored in registers.
                #[snake_name(return_loc3)]
                ReturnReg3 {
                    /// The returned values.
                    values: [Local; 3],
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
                /// Returns values as stored in the bounded [`LocalSpan`].
                #[snake_name(return_span)]
                ReturnSpan {
                    /// The [`LocalSpan`] that represents the registers that store the returned values.
                    values: BoundedLocalSpan,
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
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(return_many)]
                ReturnMany {
                    /// The first three returned values.
                    values: [Local; 3],
                },

                /// A conditional `return` instruction.
                ///
                /// # Note
                ///
                /// This is used to translate certain conditional Wasm branches such as `br_if`.
                /// Returns back to the caller if and only if the `condition` value is non zero.
                #[snake_name(return_nez)]
                ReturnNez {
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
                },
                /// A conditional `return` instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::ReturnNez`] returning a single
                /// [`Local`] value if the `condition` evaluates to `true`.
                #[snake_name(return_nez_reg)]
                ReturnNezReg {
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
                    /// The returned value.
                    value: Local,
                },
                /// A conditional `return` instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::ReturnNez`] returning two
                /// [`Local`] value if the `condition` evaluates to `true`.
                #[snake_name(return_nez_loc2)]
                ReturnNezReg2 {
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
                    /// The returned value.
                    values: [Local; 2],
                },
                /// A conditional `return` instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::ReturnNez`] returning a single
                /// [`AnyConst32`] value if the `condition` evaluates to `true`.
                #[snake_name(return_nez_imm32)]
                ReturnNezImm32 {
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
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
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
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
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
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
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
                    /// The returned values.
                    values: BoundedLocalSpan,
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
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(return_nez_many)]
                ReturnNezMany {
                    /// The local holding the condition to evaluate against zero.
                    condition: Local,
                    /// The first returned value.
                    values: [Local; 2],
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
                    lhs: Local,
                    /// The right-hand side value for the comparison.
                    ///
                    /// # Note
                    ///
                    /// We allocate constant values as function local constant values and use
                    /// their register to only require a single fallback instruction variant.
                    rhs: Local,
                    /// The register that stores the [`ComparatorAndOffset`] of this instruction.
                    ///
                    /// # Note
                    ///
                    /// The [`ComparatorAndOffset`] is loaded from register as `u64` value and
                    /// decoded into a [`ComparatorAndOffset`] before access its comparator
                    /// and 32-bit branch offset fields.
                    ///
                    /// [`ComparatorAndOffset`]: crate::ComparatorAndOffset
                    params: Local,
                },
                /// A fused `i32.and` and branch instruction.
                #[snake_name(branch_i32_and)]
                BranchI32And {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.and` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_and_imm16)]
                BranchI32AndImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.or` and branch instruction.
                #[snake_name(branch_i32_or)]
                BranchI32Or {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.or` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_or_imm16)]
                BranchI32OrImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.xor` and branch instruction.
                #[snake_name(branch_i32_xor)]
                BranchI32Xor {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.xor` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_xor_imm16)]
                BranchI32XorImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `i32.eqz(i32.and)` and branch instruction.
                #[snake_name(branch_i32_and_eqz)]
                BranchI32AndEqz {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.eqz(i32.and)` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_and_eqz_imm16)]
                BranchI32AndEqzImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.eqz(i32.or)` and branch instruction.
                #[snake_name(branch_i32_or_eqz)]
                BranchI32OrEqz {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.eqz(i32.or)` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_or_eqz_imm16)]
                BranchI32OrEqzImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.eqz(i32.xor)` and branch instruction.
                #[snake_name(branch_i32_xor_eqz)]
                BranchI32XorEqz {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.eqz(i32.xor)` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_xor_eqz_imm16)]
                BranchI32XorEqzImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `i32.eq` and branch instruction.
                #[snake_name(branch_i32_eq)]
                BranchI32Eq {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.eq` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_eq_imm16)]
                BranchI32EqImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.ne` and branch instruction.
                #[snake_name(branch_i32_ne)]
                BranchI32Ne {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.ne` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_ne_imm16)]
                BranchI32NeImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `i32.lt_s` and branch instruction.
                #[snake_name(branch_i32_lt_s)]
                BranchI32LtS {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.lt_s` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i32_lt_s_imm16_lhs)]
                BranchI32LtSImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<i32>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.lt_s` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_lt_s_imm16_rhs)]
                BranchI32LtSImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.lt_u` and branch instruction.
                #[snake_name(branch_i32_lt_u)]
                BranchI32LtU {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.lt_u` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i32_lt_u_imm16_lhs)]
                BranchI32LtUImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<u32>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.lt_u` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_lt_u_imm16_rhs)]
                BranchI32LtUImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<u32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.le_s` and branch instruction.
                #[snake_name(branch_i32_le_s)]
                BranchI32LeS {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.le_s` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i32_le_s_imm16_lhs)]
                BranchI32LeSImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<i32>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.le_s` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_le_s_imm16_rhs)]
                BranchI32LeSImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.le_u` and branch instruction.
                #[snake_name(branch_i32_le_u)]
                BranchI32LeU {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.le_u` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i32_le_u_imm16_lhs)]
                BranchI32LeUImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<u32>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i32.le_u` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i32_le_u_imm16_rhs)]
                BranchI32LeUImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<u32>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `i64.eq` and branch instruction.
                #[snake_name(branch_i64_eq)]
                BranchI64Eq {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.eq` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i64_eq_imm16)]
                BranchI64EqImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i64>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.ne` and branch instruction.
                #[snake_name(branch_i64_ne)]
                BranchI64Ne {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.ne` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i64_ne_imm16)]
                BranchI64NeImm16 {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i64>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `i64.lt_s` and branch instruction.
                #[snake_name(branch_i64_lt_s)]
                BranchI64LtS {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.lt_s` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i64_lt_s_imm16_lhs)]
                BranchI64LtSImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<i64>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.lt_s` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i64_lt_s_imm16_rhs)]
                BranchI64LtSImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i64>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.lt_u` and branch instruction.
                #[snake_name(branch_i64_lt_u)]
                BranchI64LtU {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.lt_u` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i64_lt_u_imm16_lhs)]
                BranchI64LtUImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<u64>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.lt_u` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i64_lt_u_imm16_rhs)]
                BranchI64LtUImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<u64>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.le_s` and branch instruction.
                #[snake_name(branch_i64_le_s)]
                BranchI64LeS {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.le_s` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i64_le_s_imm16_lhs)]
                BranchI64LeSImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<i64>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.le_s` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i64_le_s_imm16_rhs)]
                BranchI64LeSImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<i64>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.le_u` and branch instruction.
                #[snake_name(branch_i64_le_u)]
                BranchI64LeU {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.le_u` and branch instruction with 16-bit immediate `lhs` value.
                #[snake_name(branch_i64_le_u_imm16_lhs)]
                BranchI64LeUImm16Lhs {
                    /// The right-hand side operand to the conditional operator.
                    lhs: Const16<u64>,
                    /// The left-hand side operand to the conditional operator.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `i64.le_u` and branch instruction with 16-bit immediate `rhs` value.
                #[snake_name(branch_i64_le_u_imm16_rhs)]
                BranchI64LeUImm16Rhs {
                    /// The left-hand side operand to the conditional operator.
                    lhs: Local,
                    /// The right-hand side operand to the conditional operator.
                    rhs: Const16<u64>,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `f32.eq` and branch instruction.
                #[snake_name(branch_f32_eq)]
                BranchF32Eq {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `f32.ne` and branch instruction.
                #[snake_name(branch_f32_ne)]
                BranchF32Ne {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `f32.lt` and branch instruction.
                #[snake_name(branch_f32_lt)]
                BranchF32Lt {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `f32.le` and branch instruction.
                #[snake_name(branch_f32_le)]
                BranchF32Le {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `f64.eq` and branch instruction.
                #[snake_name(branch_f64_eq)]
                BranchF64Eq {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `f64.ne` and branch instruction.
                #[snake_name(branch_f64_ne)]
                BranchF64Ne {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },

                /// A fused `f64.lt` and branch instruction.
                #[snake_name(branch_f64_lt)]
                BranchF64Lt {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
                    /// The 16-bit encoded branch offset.
                    offset: BranchOffset16,
                },
                /// A fused `f64.le` and branch instruction.
                #[snake_name(branch_f64_le)]
                BranchF64Le {
                    /// The left-hand side operand to the branch conditional.
                    lhs: Local,
                    /// The right-hand side operand to the branch conditional.
                    rhs: Local,
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
                    /// The local holding the index of the instruction.
                    index: Local,
                    /// The number of branch table targets including the default target.
                    len_targets: u32,
                },
                /// A Wasm `br_table` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// 1. Followed by one of
                ///
                /// - [`Instruction::Local`]
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
                    /// The local holding the index of the instruction.
                    index: Local,
                    /// The number of branch table targets including the default target.
                    len_targets: u32,
                },
                /// A Wasm `br_table` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// 1. Followed by [`Instruction::Local2`].
                /// 2. Followed `len_target` times by
                ///
                /// - [`Instruction::BranchTableTarget`]
                /// - [`Instruction::ReturnReg2`]
                #[snake_name(branch_table_2)]
                BranchTable2 {
                    /// The local holding the index of the instruction.
                    index: Local,
                    /// The number of branch table targets including the default target.
                    len_targets: u32,
                },
                /// A Wasm `br_table` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// 1. Followed by [`Instruction::Local3`].
                /// 2. Followed `len_target` times by
                ///
                /// - [`Instruction::BranchTableTarget`]
                /// - [`Instruction::ReturnReg3`]
                #[snake_name(branch_table_3)]
                BranchTable3 {
                    /// The local holding the index of the instruction.
                    index: Local,
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
                /// 1. Followed by one of [`Instruction::LocalSpan`].
                /// 2. Followed `len_target` times by
                ///
                /// - [`Instruction::BranchTableTarget`]
                /// - [`Instruction::BranchTableTargetNonOverlapping`]
                /// - [`Instruction::ReturnSpan`]
                #[snake_name(branch_table_span)]
                BranchTableSpan {
                    /// The local holding the index of the instruction.
                    index: Local,
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
                /// 1. Followed by [`Instruction::LocalList`] encoding.
                /// 2. Followed `len_target` times by
                ///
                /// - [`Instruction::BranchTableTarget`]
                /// - [`Instruction::BranchTableTargetNonOverlapping`]
                /// - [`Instruction::Return`]
                #[snake_name(branch_table_many)]
                BranchTableMany {
                    /// The local holding the index of the instruction.
                    index: Local,
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
                    @result: Local,
                    /// The local holding the value to copy.
                    value: Local,
                },
                /// Copies two [`Local`] values to `results`.
                ///
                /// # Note
                ///
                /// This is a Wasmi utility instruction used to translate Wasm control flow.
                #[snake_name(copy2)]
                Copy2 {
                    @results: FixedLocalSpan<2>,
                    /// The registers holding the values to copy.
                    values: [Local; 2],
                },
                /// Copies the 32-bit immediate `value` to `result`.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Copy`] for 32-bit encoded immediate values.
                /// Read [`Instruction::Copy`] for more information about this instruction.
                #[snake_name(copy_imm32)]
                CopyImm32 {
                    @result: Local,
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
                    @result: Local,
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
                    @result: Local,
                    /// The 32-bit encoded `i64` immediate value to copy.
                    value: Const32<f64>,
                },
                /// Copies `len` contiguous `values` [`LocalSpan`] into `results` [`LocalSpan`].
                ///
                /// Copies registers: `registers[results..results+len] <- registers[values..values+len]`
                ///
                /// # Note
                ///
                /// This [`Instruction`] serves as an optimization for cases were it is possible
                /// to copy whole spans instead of many individual register values bit by bit.
                #[snake_name(copy_span)]
                CopySpan {
                    @results: LocalSpan,
                    /// The contiguous registers holding the inputs of this instruction.
                    values: LocalSpan,
                    /// The amount of copied registers.
                    len: u16,
                },
                /// Variant of [`Instruction::CopySpan`] that assumes that `results` and `values` span do not overlap.
                #[snake_name(copy_span_non_overlapping)]
                CopySpanNonOverlapping {
                    @results: LocalSpan,
                    /// The contiguous registers holding the inputs of this instruction.
                    values: LocalSpan,
                    /// The amount of copied registers.
                    len: u16,
                },
                /// Copies some [`Local`] values into `results` [`LocalSpan`].
                ///
                /// # Encoding
                ///
                /// Must be followed by
                ///
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(copy_many)]
                CopyMany {
                    @results: LocalSpan,
                    /// The first two input registers to copy.
                    values: [Local; 2],
                },
                /// Variant of [`Instruction::CopyMany`] that assumes that `results` and `values` do not overlap.
                ///
                /// Must be followed by
                ///
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(copy_many_non_overlapping)]
                CopyManyNonOverlapping {
                    @results: LocalSpan,
                    /// The first two input registers to copy.
                    values: [Local; 2],
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
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
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
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
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
                /// 2. Zero or more [`Instruction::LocalList`]
                /// 3. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
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
                /// 2. Zero or more [`Instruction::LocalList`]
                /// 3. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
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
                    @results: LocalSpan,
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
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(call_internal)]
                CallInternal {
                    @results: LocalSpan,
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
                    @results: LocalSpan,
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
                /// 1. Zero or more [`Instruction::LocalList`]
                /// 2. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(call_imported)]
                CallImported {
                    @results: LocalSpan,
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
                    @results: LocalSpan,
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
                    @results: LocalSpan,
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
                /// 2. Zero or more [`Instruction::LocalList`]
                /// 3. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(call_indirect)]
                CallIndirect {
                    @results: LocalSpan,
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
                /// 2. Zero or more [`Instruction::LocalList`]
                /// 3. Followed by one of
                ///     - [`Instruction::Local`]
                ///     - [`Instruction::Local2`]
                ///     - [`Instruction::Local3`]
                #[snake_name(call_indirect_imm16)]
                CallIndirectImm16 {
                    @results: LocalSpan,
                    /// The called internal function.
                    func_type: FuncType,
                },

                /// A Wasm `select` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::Local2`] to encode `condition` and `rhs`.
                #[snake_name(select)]
                Select {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `rhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::LocalAndImm32`] to encode `condition` and `rhs`.
                #[snake_name(select_imm32_rhs)]
                SelectImm32Rhs {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `lhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::Local2`] to encode `condition` and `lhs`.
                #[snake_name(select_imm32_lhs)]
                SelectImm32Lhs {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: AnyConst32,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit immediate `lhs` and `rhs` values.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::LocalAndImm32`] to encode `condition` and `rhs`.
                #[snake_name(select_imm32)]
                SelectImm32 {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: AnyConst32,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::LocalAndImm32`] to encode `condition` and `rhs`.
                #[snake_name(select_i64imm32_rhs)]
                SelectI64Imm32Rhs {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::Local2`] to encode `condition` and `rhs`.
                #[snake_name(select_i64imm32_lhs)]
                SelectI64Imm32Lhs {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Const32<i64>,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `i64` immediate `lhs` and `rhs` values.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::LocalAndImm32`] to encode `condition` and `rhs`.
                #[snake_name(select_i64imm32)]
                SelectI64Imm32 {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Const32<i64>,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `rhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::LocalAndImm32`] to encode `condition` and `rhs`.
                #[snake_name(select_f64imm32_rhs)]
                SelectF64Imm32Rhs {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `lhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::Local2`] to encode `condition` and `rhs`.
                #[snake_name(select_f64imm32_lhs)]
                SelectF64Imm32Lhs {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Const32<f64>,
                },
                /// A Wasm `select` equivalent Wasmi instruction with 32-bit encoded `f64` immediate `lhs` and `rhs` value.
                ///
                /// # Encoding
                ///
                /// Must be followed by [`Instruction::LocalAndImm32`] to encode `condition` and `rhs`.
                #[snake_name(select_f64imm32)]
                SelectF64Imm32 {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Const32<f64>,
                },

                /// A Wasm `ref.func` equivalent Wasmi instruction.
                #[snake_name(ref_func)]
                RefFunc {
                    @result: Local,
                    /// The index of the referenced function.
                    func: Func,
                },

                /// Wasm `global.get` equivalent Wasmi instruction.
                #[snake_name(global_get)]
                GlobalGet {
                    @result: Local,
                    /// The index identifying the global variable for the `global.get` instruction.
                    global: Global,
                },
                /// Wasm `global.set` equivalent Wasmi instruction.
                #[snake_name(global_set)]
                GlobalSet {
                    /// The local holding the value to be stored in the global variable.
                    input: Local,
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

                /// Load instruction for 32-bit values.
                ///
                /// # Note
                ///
                /// Equivalent to Wasm `{i32,f32}.load` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(load32)]
                Load32 {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Load instruction for 32-bit values and a 32-bit encoded address.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Load32`] with a constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(load32_at)]
                Load32At {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Load instruction for 32-bit values.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::Load32`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(load32_offset16)]
                Load32Offset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Load instruction for 64-bit values.
                ///
                /// # Note
                ///
                /// Equivalent to Wasm `{i64,f64}.load` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(load64)]
                Load64 {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Load instruction for 64-bit values and a 32-bit encoded address.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Load32`] with a constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(load64_at)]
                Load64At {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Load instruction for 64-bit values.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::Load64`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(load64_offset16)]
                Load64Offset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i32.load8_s` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_load8_s)]
                I32Load8s {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.load8_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I32Load8s`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i32_load8_s_at)]
                I32Load8sAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i32.load8_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Load8s`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_load8_s_offset16)]
                I32Load8sOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i32.load8_u` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_load8_u)]
                I32Load8u {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.load8_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I32Load8u`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i32_load8_u_at)]
                I32Load8uAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i32.load8_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Load8u`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_load8_u_offset16)]
                I32Load8uOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i32.load16_s` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_load16_s)]
                I32Load16s {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.load16_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I32Load16s`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i32_load16_s_at)]
                I32Load16sAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i32.load16_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Load16s`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_load16_s_offset16)]
                I32Load16sOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i32.load16_u` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_load16_u)]
                I32Load16u {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.load16_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I32Load16u`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i32_load16_u_at)]
                I32Load16uAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i32.load16_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Load16u`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_load16_u_offset16)]
                I32Load16uOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i64.load8_s` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_load8_s)]
                I64Load8s {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.load8_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Load8s`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i64_load8_s_at)]
                I64Load8sAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i64.load8_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Load8s`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_load8_s_offset16)]
                I64Load8sOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i64.load8_u` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_load8_u)]
                I64Load8u {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.load8_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Load8u`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i64_load8_u_at)]
                I64Load8uAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i64.load8_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Load8u`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_load8_u_offset16)]
                I64Load8uOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i64.load16_s` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_load16_s)]
                I64Load16s {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.load16_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Load16s`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i64_load16_s_at)]
                I64Load16sAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i64.load16_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Load16s`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_load16_s_offset16)]
                I64Load16sOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i64.load16_u` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_load16_u)]
                I64Load16u {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.load16_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Load16u`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i64_load16_u_at)]
                I64Load16uAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i64.load16_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Load16u`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_load16_u_offset16)]
                I64Load16uOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i64.load32_s` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_load32_s)]
                I64Load32s {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.load32_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Load32s`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i64_load32_s_at)]
                I64Load32sAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i64.load32_s` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Load32s`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_load32_s_offset16)]
                I64Load32sOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `i64.load32_u` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `ptr` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_load32_u)]
                I64Load32u {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.load32_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Load32u`] with a 32-bit constant load address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(i64_load32_u_at)]
                I64Load32uAt {
                    @result: Local,
                    /// The `ptr+offset` address of the `load` instruction.
                    address: Address32,
                },
                /// Wasm `i64.load32_u` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Load32u`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_load32_u_offset16)]
                I64Load32uOffset16 {
                    @result: Local,
                    /// The local storing the pointer of the `load` instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Store instruction for 32-bit values.
                ///
                /// # Note
                ///
                /// Equivalent to Wasm `{i32,f32}.store` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(store32)]
                Store32 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Store instruction for 32-bit values.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::Store32`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(store32_offset16)]
                Store32Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Store instruction for 32-bit values.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Store32`] with an immediate `ptr+offset` address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(store32_at)]
                Store32At {
                    /// The value to be stored.
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
                },

                /// Store instruction for 64-bit values.
                ///
                /// # Note
                ///
                /// Equivalent to Wasm `{i64,f64}.store` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(store64)]
                Store64 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Store instruction for 64-bit values.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::Store64`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(store64_offset16)]
                Store64Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Store instruction for 64-bit values.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Store64`] with an immediate `ptr+offset` address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// - Operates on the default Wasm memory instance if missing.
                #[snake_name(store64_at)]
                Store64At {
                    /// The value to be stored.
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
                },

                /// Wasm `i32.store` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Store32`] with 16-bit immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_store_imm16)]
                I32StoreImm16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.store` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::Store32Offset16`] with 16-bit immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_store_offset16_imm16)]
                I32StoreOffset16Imm16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Const16<i32>,
                },
                /// Wasm `i32.store` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Store32At`] with 16-bit immediate `value`.
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
                    address: Address32,
                },

                /// Wasm `i32.store` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_store8)]
                I32Store8 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.store8` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I32Store8`] with immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_store8_imm)]
                I32Store8Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.store8` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Store8`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_store8_offset16)]
                I32Store8Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Wasm `i32.store8` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Store8Offset16`] with immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_store8_offset16_imm)]
                I32Store8Offset16Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
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
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
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
                    address: Address32,
                },

                /// Wasm `i32.store16` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_store16)]
                I32Store16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.store16` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I32Store16`] with immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i32_store16_imm)]
                I32Store16Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i32.store16` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Store16`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_store16_offset16)]
                I32Store16Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Wasm `i32.store16` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I32Store16Offset16`] with immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i32_store16_offset16_imm)]
                I32Store16Offset16Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
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
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
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
                    address: Address32,
                },

                /// Wasm `i64.store` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Store64`] with 16-bit immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store_imm16)]
                I64StoreImm16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::Store64Offset16`] with 16-bit immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store_offset16_imm16)]
                I64StoreOffset16Imm16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Const16<i64>,
                },
                /// Wasm `i64.store` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::Store64At`] with 16-bit immediate `value`.
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
                    address: Address32,
                },

                /// Wasm `i64.store8` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store8)]
                I64Store8 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store8` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Store8`] with immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store8_imm)]
                I64Store8Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store8` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Store8`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store8_offset16)]
                I64Store8Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Wasm `i64.store8` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Store8Offset16`] with immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store8_offset16_imm)]
                I64Store8Offset16Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
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
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
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
                    address: Address32,
                },

                /// Wasm `i64.store16` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store16)]
                I64Store16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store16` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Store16`] with immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store16_imm)]
                I64Store16Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store16` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Store16`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store16_offset16)]
                I64Store16Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Wasm `i64.store16` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Store16Offset16`] with immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store16_offset16_imm)]
                I64Store16Offset16Imm {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
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
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
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
                    address: Address32,
                },

                /// Wasm `i64.store32` equivalent Wasmi instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store32)]
                I64Store32 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store32` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// Variant of [`Instruction::I64Store32`] with 16-bit immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// 2. Optional [`Instruction::MemoryIndex`]: encoding `memory` index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(i64_store32_imm16)]
                I64Store32Imm16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load offset.
                    offset_lo: Offset64Lo,
                },
                /// Wasm `i64.store32` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Store32`] with a 16-bit `offset`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store32_offset16)]
                I64Store32Offset16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
                    /// The value to be stored.
                    value: Local,
                },
                /// Wasm `i64.store32` equivalent Wasmi instruction.
                ///
                /// # Note
                ///
                /// - Variant of [`Instruction::I64Store32Offset16`] with 16-bit immediate `value`.
                /// - Operates on the default Wasm memory instance.
                #[snake_name(i64_store32_offset16_imm16)]
                I64Store32Offset16Imm16 {
                    /// The local storing the pointer of the `store` instruction.
                    ptr: Local,
                    /// The local storing the pointer offset of the `store` instruction.
                    offset: Offset16,
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
                    value: Local,
                    /// The constant address to store the value.
                    address: Address32,
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
                    address: Address32,
                },

                /// Wasm `i32.eq` equivalent Wasmi instruction.
                #[snake_name(i32_eq)]
                I32Eq{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i32.eq` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_eq_imm16)]
                I32EqImm16{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// Wasm `i32.ne` equivalent Wasmi instruction.
                #[snake_name(i32_ne)]
                I32Ne{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i32.ne` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_ne_imm16)]
                I32NeImm16{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// Wasm `i32.lt_s` equivalent Wasmi instruction.
                #[snake_name(i32_lt_s)]
                I32LtS{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i32.lt_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_lt_s_imm16_lhs)]
                I32LtSImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i32.lt_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_lt_s_imm16_rhs)]
                I32LtSImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },
                /// Wasm `i32.lt_u` equivalent Wasmi instruction.
                #[snake_name(i32_lt_u)]
                I32LtU{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i32.lt_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_lt_u_imm16_lhs)]
                I32LtUImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i32.lt_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_lt_u_imm16_rhs)]
                I32LtUImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<u32>,
                },

                /// Wasm `i32.le_s` equivalent Wasmi instruction.
                #[snake_name(i32_le_s)]
                I32LeS{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i32.le_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_le_s_imm16_lhs)]
                I32LeSImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i32.le_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_le_s_imm16_rhs)]
                I32LeSImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },
                /// Wasm `i32.le_u` equivalent Wasmi instruction.
                #[snake_name(i32_le_u)]
                I32LeU{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i32.le_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_le_u_imm16_lhs)]
                I32LeUImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i32.le_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_le_u_imm16_rhs)]
                I32LeUImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<u32>,
                },

                /// Wasm `i64.eq` equivalent Wasmi instruction.
                #[snake_name(i64_eq)]
                I64Eq{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i64.eq` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_eq_imm16)]
                I64EqImm16{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// Wasm `i64.ne` equivalent Wasmi instruction.
                #[snake_name(i64_ne)]
                I64Ne{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i64.ne` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_ne_imm16)]
                I64NeImm16{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// Wasm `i64.lt_s` equivalent Wasmi instruction.
                #[snake_name(i64_lt_s)]
                I64LtS{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i64.lt_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_lt_s_imm16_lhs)]
                I64LtSImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i64.lt_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_lt_s_imm16_rhs)]
                I64LtSImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// Wasm `i64.lt_u` equivalent Wasmi instruction.
                #[snake_name(i64_lt_u)]
                I64LtU{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i64.lt_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_lt_u_imm16_lhs)]
                I64LtUImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i64.lt_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_lt_u_imm16_rhs)]
                I64LtUImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<u64>,
                },

                /// Wasm `i64.le_s` equivalent Wasmi instruction.
                #[snake_name(i64_le_s)]
                I64LeS{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i64.le_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_le_s_imm16_lhs)]
                I64LeSImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i64.le_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_le_s_imm16_rhs)]
                I64LeSImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// Wasm `i64.le_u` equivalent Wasmi instruction.
                #[snake_name(i64_le_u)]
                I64LeU{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `i64.le_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_le_u_imm16_lhs)]
                I64LeUImm16Lhs{
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },
                /// Wasm `i64.le_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_le_u_imm16_rhs)]
                I64LeUImm16Rhs{
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<u64>,
                },

                /// Wasm `f32.eq` equivalent Wasmi instruction.
                #[snake_name(f32_eq)]
                F32Eq{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.ne` equivalent Wasmi instruction.
                #[snake_name(f32_ne)]
                F32Ne{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.lt` equivalent Wasmi instruction.
                #[snake_name(f32_lt)]
                F32Lt{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.le` equivalent Wasmi instruction.
                #[snake_name(f32_le)]
                F32Le{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },

                /// Wasm `f64.eq` equivalent Wasmi instruction.
                #[snake_name(f64_eq)]
                F64Eq{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.ne` equivalent Wasmi instruction.
                #[snake_name(f64_ne)]
                F64Ne{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.lt` equivalent Wasmi instruction.
                #[snake_name(f64_lt)]
                F64Lt{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.le` equivalent Wasmi instruction.
                #[snake_name(f64_le)]
                F64Le{
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },

                /// `i32` count-leading-zeros (clz) instruction.
                #[snake_name(i32_clz)]
                I32Clz {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// `i32` count-trailing-zeros (ctz) instruction.
                #[snake_name(i32_ctz)]
                I32Ctz {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// `i32` pop-count instruction.
                #[snake_name(i32_popcnt)]
                I32Popcnt {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// `i32` add instruction: `r0 = r1 + r2`
                #[snake_name(i32_add)]
                I32Add {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i32` add (small) immediate instruction: `r0 = r1 + c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I32Add`] for 16-bit constant values.
                #[snake_name(i32_add_imm16)]
                I32AddImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// `i32` subtract instruction: `r0 = r1 - r2`
                #[snake_name(i32_sub)]
                I32Sub {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i32` subtract immediate instruction: `r0 = c0 - r1`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I32Sub`] for 16-bit constant values.
                /// - Required instruction since subtraction is not commutative.
                #[snake_name(i32_sub_imm16_lhs)]
                I32SubImm16Lhs {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i32` multiply instruction: `r0 = r1 * r2`
                #[snake_name(i32_mul)]
                I32Mul {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i32` multiply immediate instruction: `r0 = r1 * c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I32Mul`] for 16-bit constant values.
                #[snake_name(i32_mul_imm16)]
                I32MulImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// `i32` signed-division instruction: `r0 = r1 / r2`
                #[snake_name(i32_div_s)]
                I32DivS {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i32` signed-division immediate instruction: `r0 = r1 / c0`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I32DivS`] for 16-bit constant values.
                /// - Guarantees that the right-hand side operand is not zero.
                #[snake_name(i32_div_s_imm16_rhs)]
                I32DivSImm16Rhs {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i32` unsigned-division instruction: `r0 = r1 / r2`
                #[snake_name(i32_div_u)]
                I32DivU {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
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
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i32` signed-remainder instruction: `r0 = r1 % r2`
                #[snake_name(i32_rem_s)]
                I32RemS {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i32` signed-remainder immediate instruction: `r0 = r1 % c0`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I32RemS`] for 16-bit constant values.
                /// - Guarantees that the right-hand side operand is not zero.
                #[snake_name(i32_rem_s_imm16_rhs)]
                I32RemSImm16Rhs {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i32` unsigned-remainder instruction: `r0 = r1 % r2`
                #[snake_name(i32_rem_u)]
                I32RemU {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i32` signed-remainder immediate instruction: `r0 = r1 % c0`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I32RemU`] for 16-bit constant values.
                /// - Guarantees that the right-hand side operand is not zero.
                #[snake_name(i32_rem_u_imm16_rhs)]
                I32RemUImm16Rhs {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i32` bitwise-and instruction: `r0 = r1 & r2`
                #[snake_name(i32_and)]
                I32And {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`].
                #[snake_name(i32_and_eqz)]
                I32AndEqz {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Fused Wasm `i32.and` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
                #[snake_name(i32_and_eqz_imm16)]
                I32AndEqzImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// `i32` bitwise-or instruction: `r0 = r1 & r2`
                #[snake_name(i32_or)]
                I32Or {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`].
                #[snake_name(i32_or_eqz)]
                I32OrEqz {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Fused Wasm `i32.or` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
                #[snake_name(i32_or_eqz_imm16)]
                I32OrEqzImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// `i32` bitwise-or instruction: `r0 = r1 ^ r2`
                #[snake_name(i32_xor)]
                I32Xor {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`].
                #[snake_name(i32_xor_eqz)]
                I32XorEqz {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Fused Wasm `i32.xor` + `i32.eqz` [`Instruction`] with 16-bit encoded immediate.
                #[snake_name(i32_xor_eqz_imm16)]
                I32XorEqzImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i32>,
                },

                /// A Wasm `i32.shl` equivalent Wasmi instruction.
                #[snake_name(i32_shl)]
                I32Shl {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i32.shl` equivalent Wasmi instruction with 16-bit immediate `rhs` operand.
                #[snake_name(i32_shl_by)]
                I32ShlBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i32>,
                },
                /// A Wasm `i32.shl` equivalent Wasmi instruction with 16-bit immediate `lhs` operand.
                #[snake_name(i32_shl_imm16)]
                I32ShlImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i32.shr_u` equivalent Wasmi instruction.
                #[snake_name(i32_shr_u)]
                I32ShrU {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i32.shr_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_shr_u_by)]
                I32ShrUBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i32>,
                },
                /// A Wasm `i32.shr_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_shr_u_imm16)]
                I32ShrUImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i32.shr_s` equivalent Wasmi instruction.
                #[snake_name(i32_shr_s)]
                I32ShrS {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i32.shr_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_shr_s_by)]
                I32ShrSBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i32>,
                },
                /// A Wasm `i32.shr_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_shr_s_imm16)]
                I32ShrSImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i32.rotl` equivalent Wasmi instruction.
                #[snake_name(i32_rotl)]
                I32Rotl {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i32.rotl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_rotl_by)]
                I32RotlBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i32>,
                },
                /// A Wasm `i32.rotl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_rotl_imm16)]
                I32RotlImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i32.rotr` equivalent Wasmi instruction.
                #[snake_name(i32_rotr)]
                I32Rotr {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i32.rotr` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i32_rotr_by)]
                I32RotrBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i32>,
                },
                /// A Wasm `i32.rotr` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i32_rotr_imm16)]
                I32RotrImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i32>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i64` count-leading-zeros (clz) instruction.
                #[snake_name(i64_clz)]
                I64Clz {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// `i64` count-trailing-zeros (ctz) instruction.
                #[snake_name(i64_ctz)]
                I64Ctz {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// `i64` pop-count instruction.
                #[snake_name(i64_popcnt)]
                I64Popcnt {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// `i64` add instruction: `r0 = r1 + r2`
                #[snake_name(i64_add)]
                I64Add {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` add (small) immediate instruction: `r0 = r1 + c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I64Add`] for 16-bit constant values.
                #[snake_name(i64_add_imm16)]
                I64AddImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// `i64` subtract instruction: `r0 = r1 - r2`
                #[snake_name(i64_sub)]
                I64Sub {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` subtract immediate instruction: `r0 = c0 - r1`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I64Sub`] for 16-bit constant values.
                /// - Required instruction since subtraction is not commutative.
                #[snake_name(i64_sub_imm16_lhs)]
                I64SubImm16Lhs {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i64` multiply instruction: `r0 = r1 * r2`
                #[snake_name(i64_mul)]
                I64Mul {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` multiply immediate instruction: `r0 = r1 * c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I64Mul`] for 16-bit constant values.
                #[snake_name(i64_mul_imm16)]
                I64MulImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// `i64` signed-division instruction: `r0 = r1 / r2`
                #[snake_name(i64_div_s)]
                I64DivS {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` signed-division immediate instruction: `r0 = r1 / c0`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I64DivS`] for 16-bit constant values.
                /// - Guarantees that the right-hand side operand is not zero.
                #[snake_name(i64_div_s_imm16_rhs)]
                I64DivSImm16Rhs {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i64` unsigned-division instruction: `r0 = r1 / r2`
                #[snake_name(i64_div_u)]
                I64DivU {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
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
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i64` signed-remainder instruction: `r0 = r1 % r2`
                #[snake_name(i64_rem_s)]
                I64RemS {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` signed-remainder immediate instruction: `r0 = r1 % c0`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I64RemS`] for 16-bit constant values.
                /// - Guarantees that the right-hand side operand is not zero.
                #[snake_name(i64_rem_s_imm16_rhs)]
                I64RemSImm16Rhs {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i64` unsigned-remainder instruction: `r0 = r1 % r2`
                #[snake_name(i64_rem_u)]
                I64RemU {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` signed-remainder immediate instruction: `r0 = r1 % c0`
                ///
                /// # Note
                ///
                /// - Optimized variant of [`Instruction::I64RemU`] for 16-bit constant values.
                /// - Guarantees that the right-hand side operand is not zero.
                #[snake_name(i64_rem_u_imm16_rhs)]
                I64RemUImm16Rhs {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
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
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<u64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// `i64` bitwise-and instruction: `r0 = r1 & r2`
                #[snake_name(i64_and)]
                I64And {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` bitwise-and (small) immediate instruction: `r0 = r1 & c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I64And`] for 16-bit constant values.
                #[snake_name(i64_and_imm16)]
                I64AndImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// `i64` bitwise-or instruction: `r0 = r1 & r2`
                #[snake_name(i64_or)]
                I64Or {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 & c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I64Or`] for 16-bit constant values.
                #[snake_name(i64_or_imm16)]
                I64OrImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// `i64` bitwise-or instruction: `r0 = r1 ^ r2`
                #[snake_name(i64_xor)]
                I64Xor {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// `i64` bitwise-or (small) immediate instruction: `r0 = r1 ^ c0`
                ///
                /// # Note
                ///
                /// Optimized variant of [`Instruction::I64Xor`] for 16-bit constant values.
                #[snake_name(i64_xor_imm16)]
                I64XorImm16 {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: Const16<i64>,
                },

                /// A Wasm `i64.shl` equivalent Wasmi instruction.
                #[snake_name(i64_shl)]
                I64Shl {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i64.shl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_shl_by)]
                I64ShlBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i64>,
                },
                /// A Wasm `i64.shl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_shl_imm16)]
                I64ShlImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i64.shr_u` equivalent Wasmi instruction.
                #[snake_name(i64_shr_u)]
                I64ShrU {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i64.shr_u` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_shr_u_by)]
                I64ShrUBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i64>,
                },
                /// A Wasm `i64.shr_u` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_shr_u_imm16)]
                I64ShrUImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i64.shr_s` equivalent Wasmi instruction.
                #[snake_name(i64_shr_s)]
                I64ShrS {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i64.shr_s` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_shr_s_by)]
                I64ShrSBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i64>,
                },
                /// A Wasm `i64.shr_s` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_shr_s_imm16)]
                I64ShrSImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i64.rotl` equivalent Wasmi instruction.
                #[snake_name(i64_rotl)]
                I64Rotl {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i64.rotl` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_rotl_by)]
                I64RotlBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i64>,
                },
                /// A Wasm `i64.rotl` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_rotl_imm16)]
                I64RotlImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// A Wasm `i64.rotr` equivalent Wasmi instruction.
                #[snake_name(i64_rotr)]
                I64Rotr {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// A Wasm `i64.rotr` equivalent Wasmi instruction with 16-bit immediate `rhs` value.
                #[snake_name(i64_rotr_by)]
                I64RotrBy {
                    @result: Local,
                    /// The local holding one of the operands.
                    lhs: Local,
                    /// The 16-bit immediate value.
                    rhs: ShiftAmount<i64>,
                },
                /// A Wasm `i64.rotr` equivalent Wasmi instruction with 16-bit immediate `lhs` value.
                #[snake_name(i64_rotr_imm16)]
                I64RotrImm16 {
                    @result: Local,
                    /// The 16-bit immediate value.
                    lhs: Const16<i64>,
                    /// The local holding one of the operands.
                    rhs: Local,
                },

                /// Wasm `i32.wrap_i64` instruction.
                #[snake_name(i32_wrap_i64)]
                I32WrapI64 {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// Wasm `i64.add128` instruction.
                ///
                /// # Note
                ///
                /// This instruction is part of the Wasm `wide-arithmetic` proposal.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local3`] encoding `lhs_hi`, `rhs_lo` and `rhs_hi`
                #[snake_name(i64_add128)]
                I64Add128 {
                    // Note:
                    // - We are not using `FixedLocalSpan` to be able to change both results independently.
                    // - This allows for more `local.set` optimizations.
                    @results: [Local; 2],
                    /// The 64 hi-bits of the `lhs` input parameter.
                    lhs_lo: Local,
                },
                /// Wasm `i64.sub128` instruction.
                ///
                /// # Note
                ///
                /// This instruction is part of the Wasm `wide-arithmetic` proposal.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local3`] encoding `lhs_hi`, `rhs_lo` and `rhs_hi`
                #[snake_name(i64_sub128)]
                I64Sub128 {
                    // Note:
                    // - We are not using `FixedLocalSpan` to be able to change both results independently.
                    // - This allows for more `local.set` optimizations.
                    @results: [Local; 2],
                    /// The low 64-bits of the `lhs` input parameter.
                    lhs_lo: Local,
                },
                /// Wasm `i64.mul_wide_s` instruction.
                ///
                /// # Note
                ///
                /// This instruction is part of the Wasm `wide-arithmetic` proposal.
                #[snake_name(i64_mul_wide_s)]
                I64MulWideS {
                    @results: FixedLocalSpan<2>,
                    /// The `lhs` input value for the instruction.
                    lhs: Local,
                    /// The `rhs` input value for the instruction.
                    rhs: Local,
                },
                /// Wasm `i64.mul_wide_u` instruction.
                ///
                /// # Note
                ///
                /// This instruction is part of the Wasm `wide-arithmetic` proposal.
                #[snake_name(i64_mul_wide_u)]
                I64MulWideU {
                    @results: FixedLocalSpan<2>,
                    /// The `lhs` input value for the instruction.
                    lhs: Local,
                    /// The `rhs` input value for the instruction.
                    rhs: Local,
                },

                /// Wasm `i32.extend8_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `sign-extension` proposal.
                #[snake_name(i32_extend8_s)]
                I32Extend8S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.extend16_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `sign-extension` proposal.
                #[snake_name(i32_extend16_s)]
                I32Extend16S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.extend8_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `sign-extension` proposal.
                #[snake_name(i64_extend8_s)]
                I64Extend8S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm(UnaryInstr) `i64.extend16_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `sign-extension` proposal.
                #[snake_name(i64_extend16_s)]
                I64Extend16S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.extend32_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `sign-extension` proposal.
                #[snake_name(i64_extend32_s)]
                I64Extend32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// Wasm `f32.abs` equivalent Wasmi instruction.
                #[snake_name(f32_abs)]
                F32Abs {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.neg` equivalent Wasmi instruction.
                #[snake_name(f32_neg)]
                F32Neg {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.ceil` equivalent Wasmi instruction.
                #[snake_name(f32_ceil)]
                F32Ceil {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.floor` equivalent Wasmi instruction.
                #[snake_name(f32_floor)]
                F32Floor {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.trunc` equivalent Wasmi instruction.
                #[snake_name(f32_trunc)]
                F32Trunc {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.nearest` equivalent Wasmi instruction.
                #[snake_name(f32_nearest)]
                F32Nearest {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.sqrt` equivalent Wasmi instruction.
                #[snake_name(f32_sqrt)]
                F32Sqrt {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.add` equivalent Wasmi instruction.
                #[snake_name(f32_add)]
                F32Add {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.sub` equivalent Wasmi instruction.
                #[snake_name(f32_sub)]
                F32Sub {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.mul` equivalent Wasmi instruction.
                #[snake_name(f32_mul)]
                F32Mul {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.div` equivalent Wasmi instruction.
                #[snake_name(f32_div)]
                F32Div {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.min` equivalent Wasmi instruction.
                #[snake_name(f32_min)]
                F32Min {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.max` equivalent Wasmi instruction.
                #[snake_name(f32_max)]
                F32Max {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.copysign` equivalent Wasmi instruction.
                #[snake_name(f32_copysign)]
                F32Copysign {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f32.copysign` equivalent Wasmi instruction with NaN canonicalization.
                #[snake_name(f32_copysign_imm)]
                F32CopysignImm {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Sign<f32>,
                },

                /// Wasm `f64.abs` equivalent Wasmi instruction.
                #[snake_name(f64_abs)]
                F64Abs {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.neg` equivalent Wasmi instruction.
                #[snake_name(f64_neg)]
                F64Neg {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.ceil` equivalent Wasmi instruction.
                #[snake_name(f64_ceil)]
                F64Ceil {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.floor` equivalent Wasmi instruction.
                #[snake_name(f64_floor)]
                F64Floor {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.trunc` equivalent Wasmi instruction.
                #[snake_name(f64_trunc)]
                F64Trunc {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.nearest` equivalent Wasmi instruction.
                #[snake_name(f64_nearest)]
                F64Nearest {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.sqrt` equivalent Wasmi instruction.
                #[snake_name(f64_sqrt)]
                F64Sqrt {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.add` equivalent Wasmi instruction.
                #[snake_name(f64_add)]
                F64Add {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.sub` equivalent Wasmi instruction.
                #[snake_name(f64_sub)]
                F64Sub {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.mul` equivalent Wasmi instruction.
                #[snake_name(f64_mul)]
                F64Mul {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.div` equivalent Wasmi instruction.
                #[snake_name(f64_div)]
                F64Div {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.min` equivalent Wasmi instruction.
                #[snake_name(f64_min)]
                F64Min {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.max` equivalent Wasmi instruction.
                #[snake_name(f64_max)]
                F64Max {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.copysign` equivalent Wasmi instruction.
                #[snake_name(f64_copysign)]
                F64Copysign {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Local,
                },
                /// Wasm `f64.copysign` equivalent Wasmi instruction with imediate `rhs` value.
                #[snake_name(f64_copysign_imm)]
                F64CopysignImm {
                    @result: Local,
                    /// The local holding the left-hand side value.
                    lhs: Local,
                    /// The local holding the right-hand side value.
                    rhs: Sign<f64>,
                },

                /// Wasm `i32.trunc_f32_s` instruction.
                #[snake_name(i32_trunc_f32_s)]
                I32TruncF32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.trunc_f32_u` instruction.
                #[snake_name(i32_trunc_f32_u)]
                I32TruncF32U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.trunc_f64_s` instruction.
                #[snake_name(i32_trunc_f64_s)]
                I32TruncF64S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.trunc_f64_u` instruction.
                #[snake_name(i32_trunc_f64_u)]
                I32TruncF64U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_f32_s` instruction.
                #[snake_name(i64_trunc_f32_s)]
                I64TruncF32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_f32_u` instruction.
                #[snake_name(i64_trunc_f32_u)]
                I64TruncF32U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_f64_s` instruction.
                #[snake_name(i64_trunc_f64_s)]
                I64TruncF64S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_f64_u` instruction.
                #[snake_name(i64_trunc_f64_u)]
                I64TruncF64U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// Wasm `i32.trunc_sat_f32_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i32_trunc_sat_f32_s)]
                I32TruncSatF32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.trunc_sat_f32_u` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i32_trunc_sat_f32_u)]
                I32TruncSatF32U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.trunc_sat_f64_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i32_trunc_sat_f64_s)]
                I32TruncSatF64S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i32.trunc_sat_f64_u` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i32_trunc_sat_f64_u)]
                I32TruncSatF64U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_sat_f32_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i64_trunc_sat_f32_s)]
                I64TruncSatF32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_sat_f32_u` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i64_trunc_sat_f32_u)]
                I64TruncSatF32U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_sat_f64_s` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i64_trunc_sat_f64_s)]
                I64TruncSatF64S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `i64.trunc_sat_f64_u` instruction.
                ///
                /// # Note
                ///
                /// Instruction from the Wasm `non-trapping float-to-int conversions` proposal.
                #[snake_name(i64_trunc_sat_f64_u)]
                I64TruncSatF64U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// Wasm `f32.demote_f64` instruction.
                #[snake_name(f32_demote_f64)]
                F32DemoteF64 {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.promote_f32` instruction.
                #[snake_name(f64_promote_f32)]
                F64PromoteF32 {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// Wasm `f32.convert_i32_s` instruction.
                #[snake_name(f32_convert_i32_s)]
                F32ConvertI32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.convert_i32_u` instruction.
                #[snake_name(f32_convert_i32_u)]
                F32ConvertI32U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.convert_i64_s` instruction.
                #[snake_name(f32_convert_i64_s)]
                F32ConvertI64S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f32.convert_i64_u` instruction.
                #[snake_name(f32_convert_i64_u)]
                F32ConvertI64U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.convert_i32_s` instruction.
                #[snake_name(f64_convert_i32_s)]
                F64ConvertI32S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.convert_i32_u` instruction.
                #[snake_name(f64_convert_i32_u)]
                F64ConvertI32U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.convert_i64_s` instruction.
                #[snake_name(f64_convert_i64_s)]
                F64ConvertI64S {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },
                /// Wasm `f64.convert_i64_u` instruction.
                #[snake_name(f64_convert_i64_u)]
                F64ConvertI64U {
                    @result: Local,
                    /// The local holding the input of the instruction.
                    input: Local,
                },

                /// A Wasm `table.get` instruction: `result = table[index]`
                ///
                /// # Encoding
                ///
                /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
                #[snake_name(table_get)]
                TableGet {
                    @result: Local,
                    /// The local storing the index of the table element to get.
                    index: Local,
                },
                /// Variant of [`Instruction::TableGet`] with constant `index` value.
                ///
                /// # Encoding
                ///
                /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
                #[snake_name(table_get_imm)]
                TableGetImm {
                    @result: Local,
                    /// The constant `index` value of the table element to get.
                    index: Const32<u64>,
                },

                /// A Wasm `table.size` instruction.
                #[snake_name(table_size)]
                TableSize {
                    @result: Local,
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
                    /// The local holding the `index` of the instruction.
                    index: Local,
                    /// The local holding the `value` of the instruction.
                    value: Local,
                },
                /// Variant of [`Instruction::TableSet`] with constant `index` value.
                ///
                /// # Encoding
                ///
                /// This [`Instruction`] must be followed by an [`Instruction::TableIndex`].
                #[snake_name(table_set_at)]
                TableSetAt {
                    /// The local holding the `value` of the instruction.
                    value: Local,
                    /// The constant `index` of the instruction.
                    index: Const32<u64>,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Local,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Local,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Const16<u64>,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Const16<u64>,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Local,
                    /// The number of copied elements.
                    len: Const16<u64>,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Local,
                    /// The number of copied elements.
                    len: Const16<u64>,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Const16<u64>,
                    /// The number of copied elements.
                    len: Const16<u64>,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Const16<u64>,
                    /// The number of copied elements.
                    len: Const16<u64>,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Local,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Local,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Const16<u32>,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Const16<u32>,
                    /// The number of copied elements.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` table.
                    src: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` table.
                    src: Local,
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
                    dst: Local,
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
                    dst: Const16<u64>,
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
                    dst: Local,
                    /// The number of elements to fill.
                    len: Local,
                    /// The value of the filled elements.
                    value: Local,
                },
                /// Variant of [`Instruction::TableFill`] with 16-bit constant `dst` index.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
                #[snake_name(table_fill_at)]
                TableFillAt {
                    /// The start index of the table to fill.
                    dst: Const16<u64>,
                    /// The number of elements to fill.
                    len: Local,
                    /// The value of the filled elements.
                    value: Local,
                },
                /// Variant of [`Instruction::TableFill`] with 16-bit constant `len` index.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
                #[snake_name(table_fill_exact)]
                TableFillExact {
                    /// The start index of the table to fill.
                    dst: Local,
                    /// The number of elements to fill.
                    len: Const16<u64>,
                    /// The value of the filled elements.
                    value: Local,
                },
                /// Variant of [`Instruction::TableFill`] with 16-bit constant `dst` and `len` fields.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
                #[snake_name(table_fill_at_exact)]
                TableFillAtExact {
                    /// The start index of the table to fill.
                    dst: Const16<u64>,
                    /// The number of elements to fill.
                    len: Const16<u64>,
                    /// The value of the filled elements.
                    value: Local,
                },

                /// Wasm `table.grow <table>` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
                #[snake_name(table_grow)]
                TableGrow {
                    @result: Local,
                    /// The number of elements to add to the table.
                    delta: Local,
                    /// The value that is used to fill up the new cells.
                    value: Local,
                },
                /// Variant of [`Instruction::TableGrow`] with 16-bit constant `delta`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::TableIndex`] encoding the Wasm `table` instance.
                #[snake_name(table_grow_imm)]
                TableGrowImm {
                    @result: Local,
                    /// The number of elements to add to the table.
                    delta: Const16<u64>,
                    /// The value that is used to fill up the new cells.
                    value: Local,
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
                    @result: Local,
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
                    @result: Local,
                    /// The number of pages to add to the memory.
                    delta: Local,
                },
                /// Variant of [`Instruction::MemoryGrow`] with 16-bit constant `delta`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_grow_by)]
                MemoryGrowBy {
                    @result: Local,
                    /// The number of pages to add to the memory.
                    delta: Const32<u64>,
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
                    dst: Local,
                    /// The start index of the `src` memory.
                    src: Local,
                    /// The number of copied bytes.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` memory.
                    src: Local,
                    /// The number of copied bytes.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` memory.
                    src: Const16<u64>,
                    /// The number of copied bytes.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` memory.
                    src: Const16<u64>,
                    /// The number of copied bytes.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` memory.
                    src: Local,
                    /// The number of copied bytes.
                    len: Const16<u64>,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` memory.
                    src: Local,
                    /// The number of copied bytes.
                    len: Const16<u64>,
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
                    dst: Local,
                    /// The start index of the `src` memory.
                    src: Const16<u64>,
                    /// The number of copied bytes.
                    len: Const16<u64>,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` memory.
                    src: Const16<u64>,
                    /// The number of copied bytes.
                    len: Const16<u64>,
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
                    dst: Local,
                    /// The byte value used to fill the memory.
                    value: Local,
                    /// The number of bytes to fill.
                    len: Local,
                },
                /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `dst` index.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_at)]
                MemoryFillAt {
                    /// The start index of the memory to fill.
                    dst: Const16<u64>,
                    /// The byte value used to fill the memory.
                    value: Local,
                    /// The number of bytes to fill.
                    len: Local,
                },
                /// Variant of [`Instruction::MemoryFill`] with constant fill `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_imm)]
                MemoryFillImm {
                    /// The start index of the memory to fill.
                    dst: Local,
                    /// The byte value used to fill the memory.
                    value: u8,
                    /// The number of bytes to fill.
                    len: Local,
                },
                /// Variant of [`Instruction::MemoryFill`] with 16-bit constant `len` value.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_exact)]
                MemoryFillExact {
                    /// The start index of the memory to fill.
                    dst: Local,
                    /// The byte value used to fill the memory.
                    value: Local,
                    /// The number of bytes to fill.
                    len: Const16<u64>,
                },
                /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_at_imm)]
                MemoryFillAtImm {
                    /// The start index of the memory to fill.
                    dst: Const16<u64>,
                    /// The byte value used to fill the memory.
                    value: u8,
                    /// The number of bytes to fill.
                    len: Local,
                },
                /// Variant of [`Instruction::MemoryFill`] with constant `dst` index and `len`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_at_exact)]
                MemoryFillAtExact {
                    /// The start index of the memory to fill.
                    dst: Const16<u64>,
                    /// The byte value used to fill the memory.
                    value: Local,
                    /// The number of bytes to fill.
                    len: Const16<u64>,
                },
                /// Variant of [`Instruction::MemoryFill`] with constant fill `value` and `len`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_imm_exact)]
                MemoryFillImmExact {
                    /// The start index of the memory to fill.
                    dst: Local,
                    /// The byte value used to fill the memory.
                    value: u8,
                    /// The number of bytes to fill.
                    len: Const16<u64>,
                },
                /// Variant of [`Instruction::MemoryFill`] with constant `dst` index, fill `value` and `len`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::MemoryIndex`] encoding the Wasm `memory` instance.
                #[snake_name(memory_fill_at_imm_exact)]
                MemoryFillAtImmExact {
                    /// The start index of the memory to fill.
                    dst: Const16<u64>,
                    /// The byte value used to fill the memory.
                    value: u8,
                    /// The number of bytes to fill.
                    len: Const16<u64>,
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
                    dst: Local,
                    /// The start index of the `src` data segment.
                    src: Local,
                    /// The number of bytes to initialize.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` data segment.
                    src: Local,
                    /// The number of initialized bytes.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` data segment.
                    src: Const16<u32>,
                    /// The number of initialized bytes.
                    len: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` data segment.
                    src: Const16<u32>,
                    /// The number of initialized bytes.
                    len: Local,
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
                    dst: Local,
                    /// The start index of the `src` data segment.
                    src: Local,
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
                    dst: Const16<u64>,
                    /// The start index of the `src` data segment.
                    src: Local,
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
                    dst: Local,
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
                    dst: Const16<u64>,
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
                    results: LocalSpan,
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
                    results: LocalSpan,
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
                /// An instruction parameter with a [`Local`] and a 32-bit immediate value.
                #[snake_name(local_and_imm32)]
                LocalAndImm32 {
                    /// The [`Local`] parameter value.
                    reg: Local,
                    /// The 32-bit immediate value.
                    imm: AnyConst32,
                },
                /// A bounded [`LocalSpan`] instruction parameter.
                #[snake_name(local_span)]
                LocalSpan { span: BoundedLocalSpan },
                /// A [`Local`] instruction parameter.
                ///
                /// # Note
                ///
                /// This [`Instruction`] only acts as a parameter to another
                /// one and will never be executed itself directly.
                #[snake_name(local)]
                Local {
                    reg: Local
                },
                /// Two [`Local`] instruction parameters.
                ///
                /// # Note
                ///
                /// This [`Instruction`] only acts as a parameter to another
                /// one and will never be executed itself directly.
                #[snake_name(local2)]
                Local2 {
                    regs: [Local; 2]
                },
                /// Three [`Local`] instruction parameters.
                ///
                /// # Note
                ///
                /// This [`Instruction`] only acts as a parameter to another
                /// one and will never be executed itself directly.
                #[snake_name(local3)]
                Local3 {
                    regs: [Local; 3]
                },
                /// [`Local`] slice parameters.
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
                /// - [`Instruction::Local`]
                /// - [`Instruction::Local2`]
                /// - [`Instruction::Local3`]
                #[snake_name(local_list)]
                LocalList {
                    regs: [Local; 3]
                },
                /// Auxiliary [`Instruction`] to encode table access information for indirect call instructions.
                #[snake_name(call_indirect_params)]
                CallIndirectParams {
                    /// The index of the called function in the table.
                    index: Local,
                    /// The table which holds the called function at the index.
                    table: Table,
                },
                /// Variant of [`Instruction::CallIndirectParams`] for 16-bit constant `index` parameter.
                #[snake_name(call_indirect_params_imm16)]
                CallIndirectParamsImm16 {
                    /// The index of the called function in the table.
                    index: Const16<u64>,
                    /// The table which holds the called function at the index.
                    table: Table,
                },
            },

            @simd {
                /// Wasm `i8x16.splat` instruction.
                #[snake_name(i8x16_splat)]
                I8x16Splat {
                    @result: Local,
                    /// The value to be splatted.
                    value: Local,
                },
                /// Wasm `i16x8.splat` instruction.
                #[snake_name(i16x8_splat)]
                I16x8Splat {
                    @result: Local,
                    /// The value to be splatted.
                    value: Local,
                },
                /// Wasm `i32x4.splat` instruction.
                #[snake_name(i32x4_splat)]
                I32x4Splat {
                    @result: Local,
                    /// The value to be splatted.
                    value: Local,
                },
                /// Wasm `i64x2.splat` instruction.
                #[snake_name(i64x2_splat)]
                I64x2Splat {
                    @result: Local,
                    /// The value to be splatted.
                    value: Local,
                },
                /// Wasm `f32x4.splat` instruction.
                #[snake_name(f32x4_splat)]
                F32x4Splat {
                    @result: Local,
                    /// The value to be splatted.
                    value: Local,
                },
                /// Wasm `f64x2.splat` instruction.
                #[snake_name(f64x2_splat)]
                F64x2Splat {
                    @result: Local,
                    /// The value to be splatted.
                    value: Local,
                },

                /// Wasm `i8x16.extract_lane_s` instruction.
                #[snake_name(i8x16_extract_lane_s)]
                I8x16ExtractLaneS {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx16,
                },
                /// Wasm `i8x16.extract_lane_u` instruction.
                #[snake_name(i8x16_extract_lane_u)]
                I8x16ExtractLaneU {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx16,
                },
                /// Wasm `i16x8.extract_lane_s` instruction.
                #[snake_name(i16x8_extract_lane_s)]
                I16x8ExtractLaneS {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx8,
                },
                /// Wasm `i16x8.extract_lane_u` instruction.
                #[snake_name(i16x8_extract_lane_u)]
                I16x8ExtractLaneU {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx8,
                },
                /// Wasm `i32x4.extract_lane` instruction.
                #[snake_name(i32x4_extract_lane)]
                I32x4ExtractLane {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx4,
                },
                /// Wasm `i64x2.extract_lane` instruction.
                #[snake_name(i64x2_extract_lane)]
                I64x2ExtractLane {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx2,
                },
                /// Wasm `f32x4.extract_lane` instruction.
                #[snake_name(f32x4_extract_lane)]
                F32x4ExtractLane {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx4,
                },
                /// Wasm `f64x2.extract_lane` instruction.
                #[snake_name(f64x2_extract_lane)]
                F64x2ExtractLane {
                    @result: Local,
                    /// The input [`V128`].
                    value: Local,
                    /// The lane to extract the value.
                    lane: ImmLaneIdx2,
                },

                /// Wasm `i8x16.replace_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding `value`.
                #[snake_name(i8x16_replace_lane)]
                I8x16ReplaceLane {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx16,
                },
                /// Variant of [`Instruction::I8x16ReplaceLane`] with imediate `value`.
                #[snake_name(i8x16_replace_lane_imm)]
                I8x16ReplaceLaneImm {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx16,
                    /// The value replacing the `lane` in `input`.
                    value: i8,
                },
                /// Wasm `i16x8.replace_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding `value`.
                #[snake_name(i16x8_replace_lane)]
                I16x8ReplaceLane {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx8,
                },
                /// Variant of [`Instruction::I16x8ReplaceLane`] with imediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Const32`] encoding the immediate `value` of type `i16`.
                #[snake_name(i16x8_replace_lane_imm)]
                I16x8ReplaceLaneImm {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx8,
                },
                /// Wasm `i32x4.replace_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding `value`.
                #[snake_name(i32x4_replace_lane)]
                I32x4ReplaceLane {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx4,
                },
                /// Variant of [`Instruction::I32x4ReplaceLaneImm`] with imediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding the immediate `value` of type `i32`.
                #[snake_name(i32x4_replace_lane_imm)]
                I32x4ReplaceLaneImm {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx4,
                },
                /// Wasm `i64x2.replace_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding `value`.
                #[snake_name(i64x2_replace_lane)]
                I64x2ReplaceLane {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx2,
                },
                /// Variant of [`Instruction::I64x2ReplaceLane`] with imediate 32-bit `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::I64Const32`] encoding the 32-bit `value`.
                #[snake_name(i64x2_replace_lane_imm32)]
                I64x2ReplaceLaneImm32 {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx2,
                },
                /// Wasm `f32x4.replace_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding `value`.
                #[snake_name(f32x4_replace_lane)]
                F32x4ReplaceLane {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx4,
                },
                /// Variant of [`Instruction::F32x4ReplaceLane`] with immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Const32`] encoding `value` of type `f32`.
                #[snake_name(f32x4_replace_lane_imm)]
                F32x4ReplaceLaneImm {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx4,
                },
                /// Wasm `f64x2.replace_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding `value`.
                #[snake_name(f64x2_replace_lane)]
                F64x2ReplaceLane {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx2,
                },
                /// Variant of [`Instruction::F64x2ReplaceLane`] with 32-bit immediate `value`.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::F64Const32`] encoding the 32-bit immediate `value`.
                #[snake_name(f64x2_replace_lane_imm32)]
                F64x2ReplaceLaneImm32 {
                    @result: Local,
                    /// The input [`V128`] that gets a value replaced.
                    input: Local,
                    /// The lane of the replaced value.
                    lane: ImmLaneIdx2,
                },

                /// Wasm `i8x16.shuffle` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding the `selector` of type [`V128`].
                #[snake_name(i8x16_shuffle)]
                I8x16Shuffle {
                    @result: Local,
                    /// The local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.swizzle` instruction.
                #[snake_name(i8x16_swizzle)]
                I8x16Swizzle {
                    @result: Local,
                    /// The local holding the `input` of the instruction.
                    input: Local,
                    /// The local holding the `selector` of the instruction.
                    selector: Local,
                },

                /// Wasm `i8x16.add` instruction.
                #[snake_name(i8x16_add)]
                I8x16Add {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.add` instruction.
                #[snake_name(i16x8_add)]
                I16x8Add {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.add` instruction.
                #[snake_name(i32x4_add)]
                I32x4Add {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.add` instruction.
                #[snake_name(i64x2_add)]
                I64x2Add {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.sub` instruction.
                #[snake_name(i8x16_sub)]
                I8x16Sub {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.sub` instruction.
                #[snake_name(i16x8_sub)]
                I16x8Sub {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.sub` instruction.
                #[snake_name(i32x4_sub)]
                I32x4Sub {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.sub` instruction.
                #[snake_name(i64x2_sub)]
                I64x2Sub {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.mul` instruction.
                #[snake_name(i16x8_mul)]
                I16x8Mul {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.mul` instruction.
                #[snake_name(i32x4_mul)]
                I32x4Mul {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.mul` instruction.
                #[snake_name(i64x2_mul)]
                I64x2Mul {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i32x4.dot_i16x8_s` instruction.
                #[snake_name(i32x4_dot_i16x8_s)]
                I32x4DotI16x8S {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.relaxed_dot_i8x16_i7x8_s` instruction.
                #[snake_name(i16x8_relaxed_dot_i8x16_i7x16_s)]
                I16x8RelaxedDotI8x16I7x16S {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.relaxed_dot_i8x16_i7x16_add_s` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Local`] encoding `c`.
                #[snake_name(i32x4_relaxed_dot_i8x16_i7x16_add_s)]
                I32x4RelaxedDotI8x16I7x16AddS {
                    @result: Local,
                    /// The local storing the `lhs` of the instruction.
                    lhs: Local,
                    /// The local storing the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `f32x4.relaxed_madd` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Local`] encoding `c`.
                #[snake_name(f32x4_relaxed_madd)]
                F32x4RelaxedMadd {
                    @result: Local,
                    /// The local storing the `a` of the instruction.
                    a: Local,
                    /// The local storing the `b` of the instruction.
                    b: Local,
                },
                /// Wasm `f32x4.relaxed_nmadd` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Local`] encoding `c`.
                #[snake_name(f32x4_relaxed_nmadd)]
                F32x4RelaxedNmadd {
                    @result: Local,
                    /// The local storing the `a` of the instruction.
                    a: Local,
                    /// The local storing the `b` of the instruction.
                    b: Local,
                },
                /// Wasm `f64x2.relaxed_madd` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Local`] encoding `c`.
                #[snake_name(f64x2_relaxed_madd)]
                F64x2RelaxedMadd {
                    @result: Local,
                    /// The local storing the `a` of the instruction.
                    a: Local,
                    /// The local storing the `b` of the instruction.
                    b: Local,
                },
                /// Wasm `f64x2.relaxed_nmadd` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Local`] encoding `c`.
                #[snake_name(f64x2_relaxed_nmadd)]
                F64x2RelaxedNmadd {
                    @result: Local,
                    /// The local storing the `a` of the instruction.
                    a: Local,
                    /// The local storing the `b` of the instruction.
                    b: Local,
                },

                /// Wasm `i8x16.neg` instruction.
                #[snake_name(i8x16_neg)]
                I8x16Neg {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.neg` instruction.
                #[snake_name(i16x8_neg)]
                I16x8Neg {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.neg` instruction.
                #[snake_name(i32x4_neg)]
                I32x4Neg {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.neg` instruction.
                #[snake_name(i64x2_neg)]
                I64x2Neg {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `i16x8.extmul_low_i8x16_s` instruction.
                #[snake_name(i16x8_extmul_low_i8x16_s)]
                I16x8ExtmulLowI8x16S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.extmul_high_i8x16_s` instruction.
                #[snake_name(i16x8_extmul_high_i8x16_s)]
                I16x8ExtmulHighI8x16S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.extmul_low_i8x16_u` instruction.
                #[snake_name(i16x8_extmul_low_i8x16_u)]
                I16x8ExtmulLowI8x16U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.extmul_high_i8x16_u` instruction.
                #[snake_name(i16x8_extmul_high_i8x16_u)]
                I16x8ExtmulHighI8x16U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.extmul_low_i16x8_s` instruction.
                #[snake_name(i32x4_extmul_low_i16x8_s)]
                I32x4ExtmulLowI16x8S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.extmul_high_i16x8_s` instruction.
                #[snake_name(i32x4_extmul_high_i16x8_s)]
                I32x4ExtmulHighI16x8S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.extmul_low_i16x8_u` instruction.
                #[snake_name(i32x4_extmul_low_i16x8_u)]
                I32x4ExtmulLowI16x8U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.extmul_high_i16x8_u` instruction.
                #[snake_name(i32x4_extmul_high_i16x8_u)]
                I32x4ExtmulHighI16x8U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.extmul_low_i32x4_s` instruction.
                #[snake_name(i64x2_extmul_low_i32x4_s)]
                I64x2ExtmulLowI32x4S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.extmul_high_i32x4_s` instruction.
                #[snake_name(i64x2_extmul_high_i32x4_s)]
                I64x2ExtmulHighI32x4S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.extmul_low_i32x4_u` instruction.
                #[snake_name(i64x2_extmul_low_i32x4_u)]
                I64x2ExtmulLowI32x4U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.extmul_high_i32x4_u` instruction.
                #[snake_name(i64x2_extmul_high_i32x4_u)]
                I64x2ExtmulHighI32x4U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i16x8.extadd_pairwise_i8x16_s` instruction.
                #[snake_name(i16x8_extadd_pairwise_i8x16_s)]
                I16x8ExtaddPairwiseI8x16S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.extadd_pairwise_i8x16_u` instruction.
                #[snake_name(i16x8_extadd_pairwise_i8x16_u)]
                I16x8ExtaddPairwiseI8x16U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.extadd_pairwise_i16x8_s` instruction.
                #[snake_name(i32x4_extadd_pairwise_i16x8_s)]
                I32x4ExtaddPairwiseI16x8S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.extadd_pairwise_i16x8_u` instruction.
                #[snake_name(i32x4_extadd_pairwise_i16x8_u)]
                I32x4ExtaddPairwiseI16x8U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `i8x16.add_sat_s` instruction.
                #[snake_name(i8x16_add_sat_s)]
                I8x16AddSatS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.add_sat_u` instruction.
                #[snake_name(i8x16_add_sat_u)]
                I8x16AddSatU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.add_sat_s` instruction.
                #[snake_name(i16x8_add_sat_s)]
                I16x8AddSatS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.add_sat_u` instruction.
                #[snake_name(i16x8_add_sat_u)]
                I16x8AddSatU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.sub_sat_s` instruction.
                #[snake_name(i8x16_sub_sat_s)]
                I8x16SubSatS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.sub_sat_u` instruction.
                #[snake_name(i8x16_sub_sat_u)]
                I8x16SubSatU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.sub_sat_s` instruction.
                #[snake_name(i16x8_sub_sat_s)]
                I16x8SubSatS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.sub_sat_u` instruction.
                #[snake_name(i16x8_sub_sat_u)]
                I16x8SubSatU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i16x8.q15mulr_sat_s` instruction.
                #[snake_name(i16x8_q15mulr_sat_s)]
                I16x8Q15MulrSatS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i8x16.min_s` instruction.
                #[snake_name(i8x16_min_s)]
                I8x16MinS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.min_u` instruction.
                #[snake_name(i8x16_min_u)]
                I8x16MinU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.min_s` instruction.
                #[snake_name(i16x8_min_s)]
                I16x8MinS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.min_u` instruction.
                #[snake_name(i16x8_min_u)]
                I16x8MinU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.min_s` instruction.
                #[snake_name(i32x4_min_s)]
                I32x4MinS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.min_u` instruction.
                #[snake_name(i32x4_min_u)]
                I32x4MinU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.max_s` instruction.
                #[snake_name(i8x16_max_s)]
                I8x16MaxS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.max_u` instruction.
                #[snake_name(i8x16_max_u)]
                I8x16MaxU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.max_s` instruction.
                #[snake_name(i16x8_max_s)]
                I16x8MaxS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.max_u` instruction.
                #[snake_name(i16x8_max_u)]
                I16x8MaxU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.max_s` instruction.
                #[snake_name(i32x4_max_s)]
                I32x4MaxS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.max_u` instruction.
                #[snake_name(i32x4_max_u)]
                I32x4MaxU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i8x16.avgr_u` instruction.
                #[snake_name(i8x16_avgr_u)]
                I8x16AvgrU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.avgr_u` instruction.
                #[snake_name(i16x8_avgr_u)]
                I16x8AvgrU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i8x16.abs` instruction.
                #[snake_name(i8x16_abs)]
                I8x16Abs {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.abs` instruction.
                #[snake_name(i16x8_abs)]
                I16x8Abs {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.abs` instruction.
                #[snake_name(i32x4_abs)]
                I32x4Abs {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.abs` instruction.
                #[snake_name(i64x2_abs)]
                I64x2Abs {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `i8x16.shl` instruction.
                #[snake_name(i8x16_shl)]
                I8x16Shl {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I8x16Shl`] with immediate shift amount.
                #[snake_name(i8x16_shl_by)]
                I8x16ShlBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i16x8.shl` instruction.
                #[snake_name(i16x8_shl)]
                I16x8Shl {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I16x8Shl`] with immediate shift amount.
                #[snake_name(i16x8_shl_by)]
                I16x8ShlBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i32x4.shl` instruction.
                #[snake_name(i32x4_shl)]
                I32x4Shl {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I32x4Shl`] with immediate shift amount.
                #[snake_name(i32x4_shl_by)]
                I32x4ShlBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i64x2.shl` instruction.
                #[snake_name(i64x2_shl)]
                I64x2Shl {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I64x2Shl`] with immediate shift amount.
                #[snake_name(i64x2_shl_by)]
                I64x2ShlBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i8x16.shr_s` instruction.
                #[snake_name(i8x16_shr_s)]
                I8x16ShrS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I8x16ShrS`] with immediate shift amount.
                #[snake_name(i8x16_shr_s_by)]
                I8x16ShrSBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i8x16.shr_u` instruction.
                #[snake_name(i8x16_shr_u)]
                I8x16ShrU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I8x16ShrU`] with immediate shift amount.
                #[snake_name(i8x16_shr_u_by)]
                I8x16ShrUBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i16x8.shr_s` instruction.
                #[snake_name(i16x8_shr_s)]
                I16x8ShrS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I16x8ShrS`] with immediate shift amount.
                #[snake_name(i16x8_shr_s_by)]
                I16x8ShrSBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i16x8.shr_u` instruction.
                #[snake_name(i16x8_shr_u)]
                I16x8ShrU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I16x8ShrU`] with immediate shift amount.
                #[snake_name(i16x8_shr_u_by)]
                I16x8ShrUBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i32x4.shr_s` instruction.
                #[snake_name(i32x4_shr_s)]
                I32x4ShrS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I32x4ShrS`] with immediate shift amount.
                #[snake_name(i32x4_shr_s_by)]
                I32x4ShrSBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i32x4.shr_u` instruction.
                #[snake_name(i32x4_shr_u)]
                I32x4ShrU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I32x4ShrU`] with immediate shift amount.
                #[snake_name(i32x4_shr_u_by)]
                I32x4ShrUBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i64x2.shr_s` instruction.
                #[snake_name(i64x2_shr_s)]
                I64x2ShrS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I64x2ShrS`] with immediate shift amount.
                #[snake_name(i64x2_shr_s_by)]
                I64x2ShrSBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },
                /// Wasm `i64x2.shr_u` instruction.
                #[snake_name(i64x2_shr_u)]
                I64x2ShrU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Variants of [`Instruction::I64x2ShrU`] with immediate shift amount.
                #[snake_name(i64x2_shr_u_by)]
                I64x2ShrUBy {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// The 16-bit encoded shift amount.
                    rhs: ShiftAmount<u32>,
                },

                /// Wasm `v128.and` instruction.
                #[snake_name(v128_and)]
                V128And {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `v128.or` instruction.
                #[snake_name(v128_or)]
                V128Or {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `v128.xor` instruction.
                #[snake_name(v128_xor)]
                V128Xor {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `v128.andnot` instruction.
                #[snake_name(v128_andnot)]
                V128Andnot {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `v128.not` instruction.
                #[snake_name(v128_not)]
                V128Not {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `v128.bitselect` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by [`Instruction::Local`] encoding the `selector`.
                #[snake_name(v128_bitselect)]
                V128Bitselect {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i8x16.popcnt` instruction.
                #[snake_name(i8x16_popcnt)]
                I8x16Popcnt {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `v128.any_true` instruction.
                #[snake_name(v128_any_true)]
                V128AnyTrue {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i8x16.all_true` instruction.
                #[snake_name(i8x16_all_true)]
                I8x16AllTrue {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.all_true` instruction.
                #[snake_name(i16x8_all_true)]
                I16x8AllTrue {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.all_true` instruction.
                #[snake_name(i32x4_all_true)]
                I32x4AllTrue {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.all_true` instruction.
                #[snake_name(i64x2_all_true)]
                I64x2AllTrue {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i8x16.bitmask` instruction.
                #[snake_name(i8x16_bitmask)]
                I8x16Bitmask {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.bitmask` instruction.
                #[snake_name(i16x8_bitmask)]
                I16x8Bitmask {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.bitmask` instruction.
                #[snake_name(i32x4_bitmask)]
                I32x4Bitmask {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.bitmask` instruction.
                #[snake_name(i64x2_bitmask)]
                I64x2Bitmask {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `i8x16.eq` instruction.
                #[snake_name(i8x16_eq)]
                I8x16Eq {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.eq` instruction.
                #[snake_name(i16x8_eq)]
                I16x8Eq {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.eq` instruction.
                #[snake_name(i32x4_eq)]
                I32x4Eq {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.eq` instruction.
                #[snake_name(i64x2_eq)]
                I64x2Eq {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.eq` instruction.
                #[snake_name(f32x4_eq)]
                F32x4Eq {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.eq` instruction.
                #[snake_name(f64x2_eq)]
                F64x2Eq {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.ne` instruction.
                #[snake_name(i8x16_ne)]
                I8x16Ne {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.ne` instruction.
                #[snake_name(i16x8_ne)]
                I16x8Ne {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.ne` instruction.
                #[snake_name(i32x4_ne)]
                I32x4Ne {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.ne` instruction.
                #[snake_name(i64x2_ne)]
                I64x2Ne {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.ne` instruction.
                #[snake_name(f32x4_ne)]
                F32x4Ne {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.ne` instruction.
                #[snake_name(f64x2_ne)]
                F64x2Ne {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.lt_s` instruction.
                #[snake_name(i8x16_lt_s)]
                I8x16LtS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.lt_u` instruction.
                #[snake_name(i8x16_lt_u)]
                I8x16LtU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.lt_s` instruction.
                #[snake_name(i16x8_lt_s)]
                I16x8LtS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.lt_u` instruction.
                #[snake_name(i16x8_lt_u)]
                I16x8LtU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.lt_s` instruction.
                #[snake_name(i32x4_lt_s)]
                I32x4LtS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.lt_u` instruction.
                #[snake_name(i32x4_lt_u)]
                I32x4LtU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.lt_s` instruction.
                #[snake_name(i64x2_lt_s)]
                I64x2LtS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.lt` instruction.
                #[snake_name(f32x4_lt)]
                F32x4Lt {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.lt` instruction.
                #[snake_name(f64x2_lt)]
                F64x2Lt {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.le_s` instruction.
                #[snake_name(i8x16_le_s)]
                I8x16LeS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.le_u` instruction.
                #[snake_name(i8x16_le_u)]
                I8x16LeU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.le_s` instruction.
                #[snake_name(i16x8_le_s)]
                I16x8LeS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.le_u` instruction.
                #[snake_name(i16x8_le_u)]
                I16x8LeU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.le_s` instruction.
                #[snake_name(i32x4_le_s)]
                I32x4LeS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i32x4.le_u` instruction.
                #[snake_name(i32x4_le_u)]
                I32x4LeU {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i64x2.le_s` instruction.
                #[snake_name(i64x2_le_s)]
                I64x2LeS {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.le` instruction.
                #[snake_name(f32x4_le)]
                F32x4Le {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.le` instruction.
                #[snake_name(f64x2_le)]
                F64x2Le {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `f32x4.neg` instruction.
                #[snake_name(f32x4_neg)]
                F32x4Neg {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.neg` instruction.
                #[snake_name(f64x2_neg)]
                F64x2Neg {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.abs` instruction.
                #[snake_name(f32x4_abs)]
                F32x4Abs {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.abs` instruction.
                #[snake_name(f64x2_abs)]
                F64x2Abs {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `f32x4.min` instruction.
                #[snake_name(f32x4_min)]
                F32x4Min {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.min` instruction.
                #[snake_name(f64x2_min)]
                F64x2Min {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.max` instruction.
                #[snake_name(f32x4_max)]
                F32x4Max {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.max` instruction.
                #[snake_name(f64x2_max)]
                F64x2Max {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.pmin` instruction.
                #[snake_name(f32x4_pmin)]
                F32x4Pmin {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.pmin` instruction.
                #[snake_name(f64x2_pmin)]
                F64x2Pmin {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.pmax` instruction.
                #[snake_name(f32x4_pmax)]
                F32x4Pmax {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.pmax` instruction.
                #[snake_name(f64x2_pmax)]
                F64x2Pmax {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.add` instruction.
                #[snake_name(f32x4_add)]
                F32x4Add {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.add` instruction.
                #[snake_name(f64x2_add)]
                F64x2Add {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.sub` instruction.
                #[snake_name(f32x4_sub)]
                F32x4Sub {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.sub` instruction.
                #[snake_name(f64x2_sub)]
                F64x2Sub {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.div` instruction.
                #[snake_name(f32x4_div)]
                F32x4Div {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.div` instruction.
                #[snake_name(f64x2_div)]
                F64x2Div {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f32x4.mul` instruction.
                #[snake_name(f32x4_mul)]
                F32x4Mul {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `f64x2.mul` instruction.
                #[snake_name(f64x2_mul)]
                F64x2Mul {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Local holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `f32x4.sqrt` instruction.
                #[snake_name(f32x4_sqrt)]
                F32x4Sqrt {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.sqrt` instruction.
                #[snake_name(f64x2_sqrt)]
                F64x2Sqrt {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.ceil` instruction.
                #[snake_name(f32x4_ceil)]
                F32x4Ceil {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.ceil` instruction.
                #[snake_name(f64x2_ceil)]
                F64x2Ceil {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.floor` instruction.
                #[snake_name(f32x4_floor)]
                F32x4Floor {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.floor` instruction.
                #[snake_name(f64x2_floor)]
                F64x2Floor {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.trunc` instruction.
                #[snake_name(f32x4_trunc)]
                F32x4Trunc {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.trunc` instruction.
                #[snake_name(f64x2_trunc)]
                F64x2Trunc {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.nearest` instruction.
                #[snake_name(f32x4_nearest)]
                F32x4Nearest {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.nearest` instruction.
                #[snake_name(f64x2_nearest)]
                F64x2Nearest {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `f32x4.convert_i32x4_s` instruction.
                #[snake_name(f32x4_convert_i32x4_s)]
                F32x4ConvertI32x4S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.convert_i32x4_u` instruction.
                #[snake_name(f32x4_convert_i32x4_u)]
                F32x4ConvertI32x4U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.convert_low_i32x4_s` instruction.
                #[snake_name(f64x2_convert_low_i32x4_s)]
                F64x2ConvertLowI32x4S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.convert_low_i32x4_u` instruction.
                #[snake_name(f64x2_convert_low_i32x4_u)]
                F64x2ConvertLowI32x4U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.trunc_sat_f32x4_s` instruction.
                #[snake_name(i32x4_trunc_sat_f32x4_s)]
                I32x4TruncSatF32x4S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.trunc_sat_f32x4_u` instruction.
                #[snake_name(i32x4_trunc_sat_f32x4_u)]
                I32x4TruncSatF32x4U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.trunc_sat_f64x2_s_zero` instruction.
                #[snake_name(i32x4_trunc_sat_f64x2_s_zero)]
                I32x4TruncSatF64x2SZero {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.trunc_sat_f64x2_u_zero` instruction.
                #[snake_name(i32x4_trunc_sat_f64x2_u_zero)]
                I32x4TruncSatF64x2UZero {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f32x4.demote_f64x2_zero` instruction.
                #[snake_name(f32x4_demote_f64x2_zero)]
                F32x4DemoteF64x2Zero {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `f64x2.promote_low_f32x4` instruction.
                #[snake_name(f64x2_promote_low_f32x4)]
                F64x2PromoteLowF32x4 {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `i8x16.narrow_i16x8_s` instruction.
                #[snake_name(i8x16_narrow_i16x8_s)]
                I8x16NarrowI16x8S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Regstier holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i8x16.narrow_i16x8_u` instruction.
                #[snake_name(i8x16_narrow_i16x8_u)]
                I8x16NarrowI16x8U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Regstier holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.narrow_i32x4_s` instruction.
                #[snake_name(i16x8_narrow_i32x4_s)]
                I16x8NarrowI32x4S {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Regstier holding the `rhs` of the instruction.
                    rhs: Local,
                },
                /// Wasm `i16x8.narrow_i32x4_u` instruction.
                #[snake_name(i16x8_narrow_i32x4_u)]
                I16x8NarrowI32x4U {
                    @result: Local,
                    /// Local holding the `lhs` of the instruction.
                    lhs: Local,
                    /// Regstier holding the `rhs` of the instruction.
                    rhs: Local,
                },

                /// Wasm `i16x8.extend_low_i8x16_s` instruction.
                #[snake_name(i16x8_extend_low_i8x16_s)]
                I16x8ExtendLowI8x16S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.extend_high_i8x16_s` instruction.
                #[snake_name(i16x8_extend_high_i8x16_s)]
                I16x8ExtendHighI8x16S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.extend_low_i8x16_u` instruction.
                #[snake_name(i16x8_extend_low_i8x16_u)]
                I16x8ExtendLowI8x16U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i16x8.extend_high_i8x16_u` instruction.
                #[snake_name(i16x8_extend_high_i8x16_u)]
                I16x8ExtendHighI8x16U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.extend_low_i16x8_s` instruction.
                #[snake_name(i32x4_extend_low_i16x8_s)]
                I32x4ExtendLowI16x8S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.extend_high_i16x8_s` instruction.
                #[snake_name(i32x4_extend_high_i16x8_s)]
                I32x4ExtendHighI16x8S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.extend_low_i16x8_u` instruction.
                #[snake_name(i32x4_extend_low_i16x8_u)]
                I32x4ExtendLowI16x8U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i32x4.extend_high_i16x8_u` instruction.
                #[snake_name(i32x4_extend_high_i16x8_u)]
                I32x4ExtendHighI16x8U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.extend_low_i32x4_s` instruction.
                #[snake_name(i64x2_extend_low_i32x4_s)]
                I64x2ExtendLowI32x4S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.extend_high_i32x4_s` instruction.
                #[snake_name(i64x2_extend_high_i32x4_s)]
                I64x2ExtendHighI32x4S {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.extend_low_i32x4_u` instruction.
                #[snake_name(i64x2_extend_low_i32x4_u)]
                I64x2ExtendLowI32x4U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },
                /// Wasm `i64x2.extend_high_i32x4_u` instruction.
                #[snake_name(i64x2_extend_high_i32x4_u)]
                I64x2ExtendHighI32x4U {
                    @result: Local,
                    /// Local holding the `input` of the instruction.
                    input: Local,
                },

                /// Wasm `v128.store` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// - [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// - Optional [`Instruction::MemoryIndex`]: encoding memory index used
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_store)]
                V128Store {
                    /// The local storing the `pointer` of the store instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Store`] with 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_store_offset16)]
                V128StoreOffset16 {
                    /// The local storing the `pointer` of the store instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                    /// The local storing the `value` of the store instruction.
                    value: Local,
                },
                /// Variant of [`Instruction::V128Store`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding `memory`.
                ///
                /// Operates on the default Wasm memory instance if missing.
                #[snake_name(v128_store_at)]
                V128StoreAt {
                    /// The value to be stored.
                    value: Local,
                    /// The 32-bit constant address to store the value.
                    address: Address32,
                },

                /// Wasm `v128.store8_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// - [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// - [`Instruction::Imm16AndImm32`]: encoding `lane_index` and `memory_index` respectively
                ///
                /// The `lane_index` is of type [`ImmLaneIdx16`].
                #[snake_name(v128_store8_lane)]
                V128Store8Lane {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit store `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Store8Lane`] with an 8-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_store8_lane_offset8)]
                V128Store8LaneOffset8 {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// Local storing the `value` of the instruction.
                    value: Local,
                    /// The 8-bit store `offset`.
                    offset: Offset8,
                    /// The lane of the stored [`V128`] `value`.
                    lane: ImmLaneIdx16,
                },
                /// Variant of [`Instruction::V128Store8Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Imm16AndImm32`] encoding `lane_index` and `memory_index` respectively.
                #[snake_name(v128_store8_lane_at)]
                V128Store8LaneAt {
                    /// The value to be stored.
                    value: Local,
                    /// The 32-bit constant address to store the value.
                    address: Address32,
                },

                /// Wasm `v128.store16_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// - [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// - [`Instruction::Imm16AndImm32`]: encoding `lane_index` and `memory_index` respectively
                ///
                /// The `lane_index` is of type [`ImmLaneIdx8`].
                #[snake_name(v128_store16_lane)]
                V128Store16Lane {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit store `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Store16Lane`] with an 8-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_store16_lane_offset8)]
                V128Store16LaneOffset8 {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// Local storing the `value` of the instruction.
                    value: Local,
                    /// The 8-bit store `offset`.
                    offset: Offset8,
                    /// The lane of the stored [`V128`] `value`.
                    lane: ImmLaneIdx8,
                },
                /// Variant of [`Instruction::V128Store16Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Imm16AndImm32`] encoding `lane_index` and `memory_index` respectively.
                #[snake_name(v128_store16_lane_at)]
                V128Store16LaneAt {
                    /// The value to be stored.
                    value: Local,
                    /// The 32-bit constant address to store the value.
                    address: Address32,
                },

                /// Wasm `v128.store32_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// - [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// - [`Instruction::Imm16AndImm32`]: encoding `lane_index` and `memory_index` respectively
                ///
                /// The `lane_index` is of type [`ImmLaneIdx4`].
                #[snake_name(v128_store32_lane)]
                V128Store32Lane {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit store `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Store32Lane`] with an 8-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_store32_lane_offset8)]
                V128Store32LaneOffset8 {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// Local storing the `value` of the instruction.
                    value: Local,
                    /// The 8-bit store `offset`.
                    offset: Offset8,
                    /// The lane of the stored [`V128`] `value`.
                    lane: ImmLaneIdx4,
                },
                /// Variant of [`Instruction::V128Store32Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Imm16AndImm32`] encoding `lane_index` and `memory_index` respectively.
                #[snake_name(v128_store32_lane_at)]
                V128Store32LaneAt {
                    /// The value to be stored.
                    value: Local,
                    /// The 32-bit constant address to store the value.
                    address: Address32,
                },

                /// Wasm `v128.store64_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// - [`Instruction::LocalAndImm32`]: encoding `value` and `offset_hi`
                /// - [`Instruction::Imm16AndImm32`]: encoding `lane_index` and `memory_index` respectively
                ///
                /// The `lane_index` is of type [`ImmLaneIdx2`].
                #[snake_name(v128_store64_lane)]
                V128Store64Lane {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// The lower 32-bit of the 64-bit store `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Store64Lane`] with an 8-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_store64_lane_offset8)]
                V128Store64LaneOffset8 {
                    /// Local storing the `ptr` of the instruction.
                    ptr: Local,
                    /// Local storing the `value` of the instruction.
                    value: Local,
                    /// The 8-bit store `offset`.
                    offset: Offset8,
                    /// The lane of the stored [`V128`] `value`.
                    lane: ImmLaneIdx2,
                },
                /// Variant of [`Instruction::V128Store64Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by an [`Instruction::Imm16AndImm32`] encoding `lane_index` and `memory_index` respectively.
                #[snake_name(v128_store64_lane_at)]
                V128Store64LaneAt {
                    /// The value to be stored.
                    value: Local,
                    /// The 32-bit constant address to store the value.
                    address: Address32,
                },

                /// Wasm `v128.load` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load)]
                V128Load {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load_at)]
                V128LoadAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load_offset16)]
                V128LoadOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load32_zero` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32_zero)]
                V128Load32Zero {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load32Zero`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32_zero_at)]
                V128Load32ZeroAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load32Zero`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load32_zero_offset16)]
                V128Load32ZeroOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load64_zero` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load64_zero)]
                V128Load64Zero {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load64Zero`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load64_zero_at)]
                V128Load64ZeroAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load64Zero`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load64_zero_offset16)]
                V128Load64ZeroOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load8_splat` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8_splat)]
                V128Load8Splat {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load8Splat`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8_splat_at)]
                V128Load8SplatAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load8Splat`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load8_splat_offset16)]
                V128Load8SplatOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load16_splat` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16_splat)]
                V128Load16Splat {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load16Splat`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16_splat_at)]
                V128Load16SplatAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load16Splat`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load16_splat_offset16)]
                V128Load16SplatOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load32_splat` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32_splat)]
                V128Load32Splat {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load32Splat`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32_splat_at)]
                V128Load32SplatAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load32Splat`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load32_splat_offset16)]
                V128Load32SplatOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load64_splat` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load64_splat)]
                V128Load64Splat {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load64Splat`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load64_splat_at)]
                V128Load64SplatAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load64Splat`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load64_splat_offset16)]
                V128Load64SplatOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load8x8_s` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8x8_s)]
                V128Load8x8S {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load8x8S`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8x8_s_at)]
                V128Load8x8SAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load8x8S`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load8x8_s_offset16)]
                V128Load8x8SOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load8x8_u` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8x8_u)]
                V128Load8x8U {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load8x8U`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8x8_u_at)]
                V128Load8x8UAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load8x8U`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load8x8_u_offset16)]
                V128Load8x8UOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load16x4_s` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16x4_s)]
                V128Load16x4S {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load16x4S`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16x4_s_at)]
                V128Load16x4SAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load16x4S`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load16x4_s_offset16)]
                V128Load16x4SOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load16x4_u` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16x4_u)]
                V128Load16x4U {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load16x4U`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16x4_u_at)]
                V128Load16x4UAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load16x4U`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load16x4_u_offset16)]
                V128Load16x4UOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load32x2_s` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32x2_s)]
                V128Load32x2S {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load32x2S`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32x2_s_at)]
                V128Load32x2SAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load32x2S`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load32x2_s_offset16)]
                V128Load32x2SOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load32x2_u` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32x2_u)]
                V128Load32x2U {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load32x2U`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Optionally followed by an [`Instruction::MemoryIndex`] encoding the `memory_index`.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32x2_u_at)]
                V128Load32x2UAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
                /// Variant of [`Instruction::V128Load32x2U`] with a 16-bit offset.
                ///
                /// # Note
                ///
                /// Operates on the default Wasm memory instance.
                #[snake_name(v128_load32x2_u_offset16)]
                V128Load32x2UOffset16 {
                    @result: Local,
                    /// Local holding the `ptr` of the instruction.
                    ptr: Local,
                    /// The 16-bit encoded offset of the `load` instruction.
                    offset: Offset16,
                },

                /// Wasm `v128.load8_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. [`Instruction::LocalAndImm32`] encoding `input` and `lane` index.
                /// 3. Optional [`Instruction::MemoryIndex`] encoding the `memory` index used.
                ///
                /// # Note
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                ///
                /// The `lane_index` is of type [`ImmLaneIdx16`].
                #[snake_name(v128_load8_lane)]
                V128Load8Lane {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load8Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `input` and `lane_index`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding the `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load8_lane_at)]
                V128Load8LaneAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },

                /// Wasm `v128.load16_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. [`Instruction::LocalAndImm32`] encoding `input` and `lane` index.
                /// 3. Optional [`Instruction::MemoryIndex`] encoding the `memory` index used.
                ///
                /// # Note
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                ///
                /// The `lane_index` is of type [`ImmLaneIdx8`].
                #[snake_name(v128_load16_lane)]
                V128Load16Lane {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load16Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `input` and `lane_index`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding the `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load16_lane_at)]
                V128Load16LaneAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },

                /// Wasm `v128.load32_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. [`Instruction::LocalAndImm32`] encoding `input` and `lane` index.
                /// 3. Optional [`Instruction::MemoryIndex`] encoding the `memory` index used.
                ///
                /// # Note
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                ///
                /// The `lane_index` is of type [`ImmLaneIdx4`].
                #[snake_name(v128_load32_lane)]
                V128Load32Lane {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load32Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `input` and `lane_index`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding the `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load32_lane_at)]
                V128Load32LaneAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },

                /// Wasm `v128.load64_lane` instruction.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `ptr` and `offset_hi`.
                /// 2. [`Instruction::LocalAndImm32`] encoding `input` and `lane` index.
                /// 3. Optional [`Instruction::MemoryIndex`] encoding the `memory` index used.
                ///
                /// # Note
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                ///
                /// The `lane_index` is of type [`ImmLaneIdx2`].
                #[snake_name(v128_load64_lane)]
                V128Load64Lane {
                    @result: Local,
                    /// The lower 32-bit of the 64-bit load `offset`.
                    offset_lo: Offset64Lo,
                },
                /// Variant of [`Instruction::V128Load64Lane`] with 32-bit immediate address.
                ///
                /// # Encoding
                ///
                /// Followed by
                ///
                /// 1. [`Instruction::LocalAndImm32`] encoding `input` and `lane_index`.
                /// 2. Optional [`Instruction::MemoryIndex`] encoding the `memory_index` used.
                ///
                /// If [`Instruction::MemoryIndex`] is missing the default memory is used.
                #[snake_name(v128_load64_lane_at)]
                V128Load64LaneAt {
                    @result: Local,
                    /// The `ptr+offset` address of the load instruction.
                    address: Address32,
                },
            }
        }
    };
}

#[cfg(not(feature = "simd"))]
macro_rules! generate_for_each_op {
    (
        @default {
            $(
                $( #[doc = $doc:literal] )*
                #[snake_name($snake_name:ident)]
                $name:ident
                $(
                    {
                        $(
                            @ $result_name:ident: $result_ty:ty,
                        )?
                        $(
                            $( #[$field_docs:meta] )*
                            $field_name:ident: $field_ty:ty
                        ),*
                        $(,)?
                    }
                )?
            ),* $(,)?
        },
        @simd $rest:tt
    ) => {
        #[macro_export]
        macro_rules! for_each_op {
            ($mac:ident) => {
                $mac! {
                    $(
                        $( #[doc = $doc] )*
                        #[snake_name($snake_name)]
                        $name
                        $(
                            {
                                $(
                                    @ $result_name: $result_ty,
                                )?
                                $(
                                    $( #[$field_docs] )*
                                    $field_name: $field_ty,
                                )*
                            }
                        )?,
                    )*
                }
            };
        }
    };
}

#[cfg(feature = "simd")]
macro_rules! generate_for_each_op {
    (
        $(
            @$group:ident {
                $(
                    $( #[doc = $doc:literal] )*
                    #[snake_name($snake_name:ident)]
                    $name:ident
                    $(
                        {
                            $(
                                @ $result_name:ident: $result_ty:ty,
                            )?
                            $(
                                $( #[$field_docs:meta] )*
                                $field_name:ident: $field_ty:ty
                            ),*
                            $(,)?
                        }
                    )?
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        #[macro_export]
        macro_rules! for_each_op {
            ($mac:ident) => {
                $mac! {
                    $(
                        // for each group
                        $(
                            // for each instruction in group
                            $( #[doc = $doc] )*
                            #[snake_name($snake_name)]
                            $name
                            $(
                                {
                                    $(
                                        @ $result_name: $result_ty,
                                    )?
                                    $(
                                        $( #[$field_docs] )*
                                        $field_name: $field_ty,
                                    )*
                                }
                            )?,
                        )*
                    )*
                }
            };
        }
    }
}

for_each_op_grouped!(generate_for_each_op);
pub use for_each_op;
