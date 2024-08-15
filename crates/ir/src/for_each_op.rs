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
            Trap { code: TrapCode },
            /// Consumes fuel for its associated control flow block.
            ///
            /// # Note
            ///
            /// This instruction type is only generated if fuel metering is enabled.
            #[snake_name(consume_fuel)]
            ConsumeFuel { fuel: BlockFuel },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns no value.
            #[snake_name(ret)]
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
            /// Returns a single 32-bit immediate value.
            #[snake_name(return_imm32)]
            ReturnImm32 {
                /// The returned value.
                value: Imm32,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single 64-bit immediate value.
            #[snake_name(return_imm64)]
            ReturnImm64 {
                /// The returned value.
                value: Imm64,
            },
            /// A Wasm `return` instruction.
            ///
            /// # Note
            ///
            /// Returns multiple values stored in a slice of registers.
            #[snake_name(return_regs)]
            ReturnMany<'op> {
                /// The returned value.
                values: Slice<'op, Reg>,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Returns no value.
            #[snake_name(return_nez)]
            ReturnNez {
                /// The register storing the condition value.
                condition: Reg,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single value stored in a register.
            #[snake_name(return_nez_reg)]
            ReturnNezReg {
                /// The register storing the condition value.
                condition: Reg,
                /// The returned value.
                value: Reg,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single 32-bit immediate value.
            #[snake_name(return_nez_imm32)]
            ReturnNezImm32 {
                /// The register storing the condition value.
                condition: Reg,
                /// The returned value.
                value: Imm32,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Returns a single 64-bit immediate value.
            #[snake_name(return_nez_imm64)]
            ReturnNezImm64 {
                /// The register storing the condition value.
                condition: Reg,
                /// The returned value.
                value: Imm64,
            },
            /// A conditional `return` instruction.
            ///
            /// # Note
            ///
            /// Returns multiple values stored in a slice of registers.
            #[snake_name(return_nez_regs)]
            ReturnNezMany<'op> {
                /// The register storing the condition value.
                condition: Reg,
                /// The returned value.
                values: Slice<'op, Reg>,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            #[snake_name(branch_table)]
            BranchTable<'op> {
                /// The index that selects which branch to take.
                index: Reg,
                /// The branching targets including the default target.
                targets: Slice<'op, BranchTableTarget>,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// Returns or copies a value to the branch destination.
            #[snake_name(branch_table_reg)]
            BranchTableReg<'op> {
                /// The index that selects which branch to take.
                index: Reg,
                /// The value that needs to be returned or copied to the branch destination.
                value: Reg,
                /// The branching targets including the default target.
                targets: Slice<'op, BranchTableTarget>,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// Returns or copies a 32-bit immediate value to the branch destination.
            #[snake_name(branch_table_imm32)]
            BranchTableImm32<'op> {
                /// The index that selects which branch to take.
                index: Reg,
                /// The 32-bit immediate value that needs to be returned or copied to the branch destination.
                value: Imm32,
                /// The branching targets including the default target.
                targets: Slice<'op, BranchTableTarget>,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// Returns or copies a 64-bit immediate value to the branch destination.
            #[snake_name(branch_table_imm64)]
            BranchTableImm64<'op> {
                /// The index that selects which branch to take.
                index: Reg,
                /// The 32-bit immediate value that needs to be returned or copied to the branch destination.
                value: Imm64,
                /// The branching targets including the default target.
                targets: Slice<'op, BranchTableTarget>,
            },
            /// A Wasm `br_table` equivalent Wasmi instruction.
            ///
            /// Returns or copies a some values to the branch destination.
            #[snake_name(branch_table_many)]
            BranchTableMany<'op> {
                /// The index that selects which branch to take.
                index: Reg,
                /// The values that need to be returned or copied to the branch destination.
                values: Slice<'op, Reg>,
                /// The branching targets including the default target.
                targets: Slice<'op, BranchTableTarget>,
            },
            /// A Wasm `br` instruction.
            #[snake_name(branch)]
            Branch {
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`==`) branching instruction.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_i32_eq)]
            BranchI32Eq {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`==`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_i32_eq_imm)]
            BranchI32EqImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_i32_ne)]
            BranchI32Ne {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_i32_ne_imm)]
            BranchI32NeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<`) branching instruction.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i32_lt_s)]
            BranchI32LtS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<`) branching instruction.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i32_lt_u)]
            BranchI32LtU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i32_lt_s_imm)]
            BranchI32LtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i32_lt_u_imm)]
            BranchI32LtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<=`) branching instruction.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i32_le_s)]
            BranchI32LeS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<=`) branching instruction.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i32_le_u)]
            BranchI32LeU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i32_le_s_imm)]
            BranchI32LeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i32_le_u_imm)]
            BranchI32LeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>`) branching instruction.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i32_gt_s)]
            BranchI32GtS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>`) branching instruction.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i32_gt_u)]
            BranchI32GtU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i32_gt_s_imm)]
            BranchI32GtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i32_gt_u_imm)]
            BranchI32GtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>=`) branching instruction.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i32_ge_s)]
            BranchI32GeS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>=`) branching instruction.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i32_ge_u)]
            BranchI32GeU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i32_ge_s_imm)]
            BranchI32GeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i32_ge_u_imm)]
            BranchI32GeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`==`) branching instruction.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_i64_eq)]
            BranchI64Eq {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`==`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_i64_eq_imm)]
            BranchI64EqImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_i64_ne)]
            BranchI64Ne {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_i64_ne_imm)]
            BranchI64NeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<`) branching instruction.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i64_lt_s)]
            BranchI64LtS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<`) branching instruction.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i64_lt_u)]
            BranchI64LtU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i64_lt_s_imm)]
            BranchI64LtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_i64_lt_u_imm)]
            BranchI64LtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<=`) branching instruction.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i64_le_s)]
            BranchI64LeS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<=`) branching instruction.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i64_le_u)]
            BranchI64LeU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`<=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i64_le_s_imm)]
            BranchI64LeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`<=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_i64_le_u_imm)]
            BranchI64LeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>`) branching instruction.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i64_gt_s)]
            BranchI64GtS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>`) branching instruction.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i64_gt_u)]
            BranchI64GtU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i64_gt_s_imm)]
            BranchI64GtSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_i64_gt_u_imm)]
            BranchI64GtUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>=`) branching instruction.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i64_ge_s)]
            BranchI64GeS {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>=`) branching instruction.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i64_ge_u)]
            BranchI64GeU {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A signed conditional (`>=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i64_ge_s_imm)]
            BranchI64GeSImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: i64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// An unsigned conditional (`>=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_i64_ge_u_imm)]
            BranchI64GeUImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: u64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },

            /// A conditional (`==`) branching instruction.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_f32_eq)]
            BranchF32Eq {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`==`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_f32_eq_imm)]
            BranchF32EqImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_f32_ne)]
            BranchF32Ne {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_f32_ne_imm)]
            BranchF32NeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<`) branching instruction.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_f32_lt)]
            BranchF32Lt {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_f32_lt_imm)]
            BranchF32LtImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<=`) branching instruction.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_f32_le)]
            BranchF32Le {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_f32_le_imm)]
            BranchF32LeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>`) branching instruction.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_f32_gt)]
            BranchF32Gt {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_f32_gt_imm)]
            BranchF32GtImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>=`) branching instruction.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_f32_ge)]
            BranchF32Ge {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_f32_ge_imm)]
            BranchF32GeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee32,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },

            /// A conditional (`==`) branching instruction.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_f64_eq)]
            BranchF64Eq {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`==`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs == rhs`
            #[snake_name(branch_f64_eq_imm)]
            BranchF64EqImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_f64_ne)]
            BranchF64Ne {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`!=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs != rhs`
            #[snake_name(branch_f64_ne_imm)]
            BranchF64NeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<`) branching instruction.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_f64_lt)]
            BranchF64Lt {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs < rhs`
            #[snake_name(branch_f64_lt_imm)]
            BranchF64LtImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<=`) branching instruction.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_f64_le)]
            BranchF64Le {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`<=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs <= rhs`
            #[snake_name(branch_f64_le_imm)]
            BranchF64LeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>`) branching instruction.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_f64_gt)]
            BranchF64Gt {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs > rhs`
            #[snake_name(branch_f64_gt_imm)]
            BranchF64GtImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>=`) branching instruction.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_f64_ge)]
            BranchF64Ge {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Reg,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// A conditional (`>=`) branching instruction with an immediate `rhs` value.
            ///
            /// Takes the branch if: `lhs >= rhs`
            #[snake_name(branch_f64_ge_imm)]
            BranchF64GeImm {
                /// The left-hand side operand to the conditional operator.
                lhs: Reg,
                /// The right-hand side operand to the conditional operator.
                rhs: Ieee64,
                /// The offset to the destination of the branch instruction.
                offset: BranchOffset,
            },
            /// Copies `value` to `result`.
            #[snake_name(copy)]
            Copy {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value to copy.
                value: Reg,
            },
            /// Copies the 32-bit immediate `value` to `result`.
            #[snake_name(copy_imm32)]
            CopyImm32 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The 32-bit immediate value to copy.
                value: Imm32,
            },
            /// Copies the 64-bit immediate `value` to `result`.
            #[snake_name(copy_imm64)]
            CopyImm64 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The 64-bit immediate value to copy.
                value: Imm64,
            },
            /// Copies many register values from `values` to `results`.
            #[snake_name(copy_many)]
            CopyMany<'op> {
                /// The register storing the result of the instruction.
                results: RegSpan,
                /// The 64-bit immediate value to copy.
                values: Slice<'op, Reg>,
            },
            /// Copies many register values from `values` to `results`.
            ///
            /// This instruction assumes that there are no overlapping copies.
            /// A copy is overlapping with another copy if it overwrites its result.
            #[snake_name(copy_many_non_overlapping)]
            CopyManyNonOverlapping<'op> {
                /// The register storing the result of the instruction.
                results: RegSpan,
                /// The 64-bit immediate value to copy.
                values: Slice<'op, Reg>,
            },
            /// Wasm `return_call` equivalent Wasmi instruction.
            #[snake_name(return_call_internal)]
            ReturnCallInternal<'op> {
                /// The internal function being tail called.
                func: InternalFunc,
                /// The parameters of the tail call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `return_call` equivalent Wasmi instruction.
            #[snake_name(return_call_imported)]
            ReturnCallImported<'op> {
                /// The function being tail called.
                func: Func,
                /// The parameters of the tail call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `return_call_indirect` equivalent Wasmi instruction.
            #[snake_name(return_call_indirect)]
            ReturnCallIndirect<'op> {
                /// The imported function being tail called.
                func_type: FuncType,
                /// The table on which to query the function for the indirect call.
                table: Table,
                /// The index within the queried table.
                index: Reg,
                /// The parameters of the tail call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `return_call_indirect` equivalent Wasmi instruction with immediate `index`.
            #[snake_name(return_call_indirect_imm)]
            ReturnCallIndirectImm<'op> {
                /// The imported function being tail called.
                func_type: FuncType,
                /// The table on which to query the function for the indirect call.
                table: Table,
                /// The index within the queried table.
                index: u32,
                /// The parameters of the tail call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `call` equivalent Wasmi instruction.
            #[snake_name(call_internal)]
            CallInternal<'op> {
                /// The registers storing the result of the function call.
                results: RegSpan,
                /// The internal function being called.
                func: InternalFunc,
                /// The parameters of the call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `call` equivalent Wasmi instruction.
            #[snake_name(call_imported)]
            CallImported<'op> {
                /// The registers storing the result of the function call.
                results: RegSpan,
                /// The function being called.
                func: Func,
                /// The parameters of the call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `call_indirect` equivalent Wasmi instruction.
            #[snake_name(call_indirect)]
            CallIndirect<'op> {
                /// The registers storing the result of the function call.
                results: RegSpan,
                /// The imported function being called.
                func_type: FuncType,
                /// The table on which to query the function for the indirect call.
                table: Table,
                /// The index within the queried table.
                index: Reg,
                /// The parameters of the call.
                params: Slice<'op, Reg>,
            },
            /// Wasm `call_indirect` equivalent Wasmi instruction with immediate `index`.
            #[snake_name(call_indirect_imm)]
            CallIndirectImm<'op> {
                /// The registers storing the result of the function call.
                results: RegSpan,
                /// The imported function being called.
                func_type: FuncType,
                /// The table on which to query the function for the indirect call.
                table: Table,
                /// The index within the queried table.
                index: u32,
                /// The parameters of the call.
                params: Slice<'op, Reg>,
            },
            /// A Wasm `select` or `select <ty>` instruction.
            ///
            /// Inspect `condition` and if `condition != 0`:
            ///
            /// - `true` : store `lhs` into `result`
            /// - `false`: store `rhs` into `result`
            #[snake_name(select)]
            Select {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The register holding the `lhs` value.
                lhs: Reg,
                /// The register holding the `rhs` value.
                rhs: Reg,
            },
            /// A Wasm `select` or `select <ty>` instruction with a 32-bit immediate `rhs` value.
            ///
            /// Read more about the semantics at [`Select`](crate::op::Select).
            #[snake_name(select_rhs_imm32)]
            SelectRhsImm32 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The register holding the `lhs` value.
                lhs: Reg,
                /// The 32-bit immediate `rhs` value.
                rhs: Imm32,
            },
            /// A Wasm `select` or `select <ty>` instruction with a 32-bit immediate `lhs` value.
            ///
            /// Read more about the semantics at [`Select`](crate::op::Select).
            #[snake_name(select_lhs_imm32)]
            SelectLhsImm32 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The 32-bit immediate `lhs` value.
                lhs: Imm32,
                /// The register holding the `rhs` value.
                rhs: Reg,
            },
            /// A Wasm `select` or `select <ty>` instruction with 32-bit immediate `lhs` and `rhs` values.
            ///
            /// Read more about the semantics at [`Select`](crate::op::Select).
            #[snake_name(select_imm32)]
            SelectImm32 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The 32-bit immediate `lhs` value.
                lhs: Imm32,
                /// The 32-bit immediate `rhs` value.
                rhs: Imm32,
            },
            /// A Wasm `select` or `select <ty>` instruction with a 64-bit immediate `rhs` value.
            ///
            /// Read more about the semantics at [`Select`](crate::op::Select).
            #[snake_name(select_rhs_imm64)]
            SelectRhsImm64 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The register holding the `lhs` value.
                lhs: Reg,
                /// The 64-bit immediate `rhs` value.
                rhs: Imm64,
            },
            /// A Wasm `select` or `select <ty>` instruction with a 64-bit immediate `lhs` value.
            ///
            /// Read more about the semantics at [`Select`](crate::op::Select).
            #[snake_name(select_lhs_imm64)]
            SelectLhsImm64 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The 64-bit immediate `lhs` value.
                lhs: Imm64,
                /// The register holding the `rhs` value.
                rhs: Reg,
            },
            /// A Wasm `select` or `select <ty>` instruction with 64-bit immediate `lhs` and `rhs` values.
            ///
            /// Read more about the semantics at [`Select`](crate::op::Select).
            #[snake_name(select_imm64)]
            SelectImm64 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register holding the `condition` value.
                condition: Reg,
                /// The 64-bit immediate `lhs` value.
                lhs: Imm64,
                /// The 64-bit immediate `rhs` value.
                rhs: Imm64,
            },
            /// A Wasm `ref.func` equivalent Wasmi instruction.
            #[snake_name(ref_func)]
            RefFunc {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The index of the referenced function.
                func: Func,
            },
            /// Wasm `global.get` equivalent Wasmi instruction.
            #[snake_name(global_get)]
            GlobalGet {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The index of the global variable which is being queried.
                global: Global,
            },
            /// Wasm `global.set` equivalent Wasmi instruction.
            ///
            /// Sets the value of the `global` variable to `input`.
            #[snake_name(global_set)]
            GlobalSet {
                /// The index of the global variable which is manipulated.
                global: Global,
                /// The register holding the `input` value.
                input: Reg,
            },
            /// Wasm `global.set` equivalent Wasmi instruction with a 32-bit immediate `input` value.
            ///
            /// Read more about the semantics at [`GlobalSet`](crate::op::GlobalSet).
            #[snake_name(global_set_imm32)]
            GlobalSetImm32 {
                /// The index of the global variable which is manipulated.
                global: Global,
                /// The 32-bit immediate `input` value.
                input: Imm32,
            },
            /// Wasm `global.set` equivalent Wasmi instruction with a 64-bit immediate `input` value.
            ///
            /// Read more about the semantics at [`GlobalSet`](crate::op::GlobalSet).
            #[snake_name(global_set_imm64)]
            GlobalSetImm64 {
                /// The index of the global variable which is manipulated.
                global: Global,
                /// The 64-bit immediate `input` value.
                input: Imm64,
            },

            /// Wasm `i32.load` equivalent Wasmi instruction.
            #[snake_name(i32_load)]
            I32Load {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i32.load` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i32_load_at)]
            I32LoadAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i32.load8_s` equivalent Wasmi instruction.
            #[snake_name(i32_load8_s)]
            I32Load8S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i32.load8_s` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i32_load8_s_at)]
            I32Load8SAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i32.load8_u` equivalent Wasmi instruction.
            #[snake_name(i32_load8_u)]
            I32Load8U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i32.load8_u` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i32_load8_u_at)]
            I32Load8UAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i32.load16_s` equivalent Wasmi instruction.
            #[snake_name(i32_load16_s)]
            I32Load16S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i32.load16_s` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i32_load16_s_at)]
            I32Load16SAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i32.load16_u` equivalent Wasmi instruction.
            #[snake_name(i32_load16_u)]
            I32Load16U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i32.load16_u` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i32_load16_u_at)]
            I32Load16UAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load` equivalent Wasmi instruction.
            #[snake_name(i64_load)]
            I64Load {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load_at)]
            I64LoadAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load8_s` equivalent Wasmi instruction.
            #[snake_name(i64_load8_s)]
            I64Load8S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load8_s` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load8_s_at)]
            I64Load8SAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load8_u` equivalent Wasmi instruction.
            #[snake_name(i64_load8_u)]
            I64Load8U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load8_u` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load8_u_at)]
            I64Load8UAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load16_s` equivalent Wasmi instruction.
            #[snake_name(i64_load16_s)]
            I64Load16S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load16_s` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load16_s_at)]
            I64Load16SAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load16_u` equivalent Wasmi instruction.
            #[snake_name(i64_load16_u)]
            I64Load16U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load16_u` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load16_u_at)]
            I64Load16UAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load32_s` equivalent Wasmi instruction.
            #[snake_name(i64_load32_s)]
            I64Load32S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load32_s` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load32_s_at)]
            I64Load32SAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i64.load32_u` equivalent Wasmi instruction.
            #[snake_name(i64_load32_u)]
            I64Load32U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `i64.load32_u` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(i64_load32_u_at)]
            I64Load32UAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },

            /// Wasm `f32.load` equivalent Wasmi instruction.
            #[snake_name(f32_load)]
            F32Load {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `f32.load` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(f32_load_at)]
            F32LoadAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `f64.load` equivalent Wasmi instruction.
            #[snake_name(f64_load)]
            F64Load {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the pointer of the `load` instruction.
                ptr: Reg,
                /// The byte offset for the `load` instruction.
                offset: ByteOffset,
            },
            /// Wasm `f64.load` equivalent Wasmi instruction with an immediate `ptr+offset` (address) value.
            #[snake_name(f64_load_at)]
            F64LoadAt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The address within the linear memory.
                address: Address,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction.
            #[snake_name(i32_store)]
            I32Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i32_store_at)]
            I32StoreAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i32_store_imm)]
            I32StoreImm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i32,
            },
            /// Wasm `i32.store` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i32_store_imm_at)]
            I32StoreImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i32,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction.
            #[snake_name(i32_store8)]
            I32Store8 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i32_store8_at)]
            I32Store8At {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i32_store8_imm)]
            I32Store8Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i8,
            },
            /// Wasm `i32.store8` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i32_store8_imm_at)]
            I32Store8ImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i8,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction.
            #[snake_name(i32_store16)]
            I32Store16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i32_store16_at)]
            I32Store16At {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i32_store16_imm)]
            I32Store16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i16,
            },
            /// Wasm `i32.store16` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i32_store16_imm_at)]
            I32Store16ImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i16,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction.
            #[snake_name(i64_store)]
            I64Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i64_store_at)]
            I64StoreAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i64_store_imm)]
            I64StoreImm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i64,
            },
            /// Wasm `i64.store` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i64_store_imm_at)]
            I64StoreImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i64,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction.
            #[snake_name(i64_store8)]
            I64Store8 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i64_store8_at)]
            I64Store8At {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i64_store8_imm)]
            I64Store8Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i8,
            },
            /// Wasm `i64.store8` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i64_store8_imm_at)]
            I64Store8ImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i8,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction.
            #[snake_name(i64_store16)]
            I64Store16 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i64_store16_at)]
            I64Store16At {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i64_store16_imm)]
            I64Store16Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i16,
            },
            /// Wasm `i64.store16` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i64_store16_imm_at)]
            I64Store16ImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i16,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction.
            #[snake_name(i64_store32)]
            I64Store32 {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(i64_store32_at)]
            I64Store32At {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(i64_store32_imm)]
            I64Store32Imm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: i32,
            },
            /// Wasm `i64.store32` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(i64_store32_imm_at)]
            I64Store32ImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: i32,
            },
            /// Wasm `f32.store` equivalent Wasmi instruction.
            #[snake_name(f32_store)]
            F32Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `f32.store` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(f32_store_at)]
            F32StoreAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `f32.store` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(f32_store_imm)]
            F32StoreImm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: Ieee32,
            },
            /// Wasm `f32.store` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(f32_store_imm_at)]
            F32StoreImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: Ieee32,
            },
            /// Wasm `f64.store` equivalent Wasmi instruction.
            #[snake_name(f64_store)]
            F64Store {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `f64.store` equivalent Wasmi instruction with immediate `address`.
            #[snake_name(f64_store_at)]
            F64StoreAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The register storing the value being stored.
                value: Reg,
            },
            /// Wasm `f64.store` equivalent Wasmi instruction with immediate `value`.
            #[snake_name(f64_store_imm)]
            F64StoreImm {
                /// The register storing the pointer of the `store` instruction.
                ptr: Reg,
                /// The register storing the byte offset of the `store` instruction.
                offset: ByteOffset,
                /// The immediate value being stored.
                value: Ieee64,
            },
            /// Wasm `f64.store` equivalent Wasmi instruction with immediate `address` and `value`.
            #[snake_name(f64_store_imm_at)]
            F64StoreImmAt {
                /// The address within the linear memory for the `store` instruction.
                address: Address,
                /// The immediate value being stored.
                value: Ieee64,
            },
            /// Wasm `i32.eq` equivalent Wasmi instruction.
            #[snake_name(i32_eq)]
            I32Eq {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.eq` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_eq_imm)]
            I32EqImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.ne` equivalent Wasmi instruction.
            #[snake_name(i32_ne)]
            I32Ne {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.ne` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_ne_imm)]
            I32NeImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.ge_s` equivalent Wasmi instruction.
            #[snake_name(i32_ge_s)]
            I32GeS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.ge_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_ge_s_imm)]
            I32GeSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.ge_u` equivalent Wasmi instruction.
            #[snake_name(i32_ge_u)]
            I32GeU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.ge_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_ge_u_imm)]
            I32GeUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.gt_s` equivalent Wasmi instruction.
            #[snake_name(i32_gt_s)]
            I32GtS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.gt_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_gt_s_imm)]
            I32GtSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.gt_u` equivalent Wasmi instruction.
            #[snake_name(i32_gt_u)]
            I32GtU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.gt_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_gt_u_imm)]
            I32GtUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.le_s` equivalent Wasmi instruction.
            #[snake_name(i32_le_s)]
            I32LeS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.le_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_le_s_imm)]
            I32LeSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.le_u` equivalent Wasmi instruction.
            #[snake_name(i32_le_u)]
            I32LeU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.le_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_le_u_imm)]
            I32LeUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.lt_s` equivalent Wasmi instruction.
            #[snake_name(i32_lt_s)]
            I32LtS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.lt_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_lt_s_imm)]
            I32LtSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.lt_u` equivalent Wasmi instruction.
            #[snake_name(i32_lt_u)]
            I32LtU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.lt_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i32_lt_u_imm)]
            I32LtUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i64.eq` equivalent Wasmi instruction.
            #[snake_name(i64_eq)]
            I64Eq {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.eq` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_eq_imm)]
            I64EqImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.ne` equivalent Wasmi instruction.
            #[snake_name(i64_ne)]
            I64Ne {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.ne` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_ne_imm)]
            I64NeImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.ge_s` equivalent Wasmi instruction.
            #[snake_name(i64_ge_s)]
            I64GeS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.ge_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_ge_s_imm)]
            I64GeSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.ge_u` equivalent Wasmi instruction.
            #[snake_name(i64_ge_u)]
            I64GeU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.ge_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_ge_u_imm)]
            I64GeUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.gt_s` equivalent Wasmi instruction.
            #[snake_name(i64_gt_s)]
            I64GtS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.gt_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_gt_s_imm)]
            I64GtSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.gt_u` equivalent Wasmi instruction.
            #[snake_name(i64_gt_u)]
            I64GtU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.gt_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_gt_u_imm)]
            I64GtUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.le_s` equivalent Wasmi instruction.
            #[snake_name(i64_le_s)]
            I64LeS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.le_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_le_s_imm)]
            I64LeSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.le_u` equivalent Wasmi instruction.
            #[snake_name(i64_le_u)]
            I64LeU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.le_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_le_u_imm)]
            I64LeUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.lt_s` equivalent Wasmi instruction.
            #[snake_name(i64_lt_s)]
            I64LtS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.lt_s` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_lt_s_imm)]
            I64LtSImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.lt_u` equivalent Wasmi instruction.
            #[snake_name(i64_lt_u)]
            I64LtU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.lt_u` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(i64_lt_u_imm)]
            I64LtUImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `f32.eq` equivalent Wasmi instruction.
            #[snake_name(f32_eq)]
            F32Eq {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.eq` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_eq_imm)]
            F32EqImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.ge` equivalent Wasmi instruction.
            #[snake_name(f32_ge)]
            F32Ge {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.ge` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_ge_imm)]
            F32GeImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.gt` equivalent Wasmi instruction.
            #[snake_name(f32_gt)]
            F32Gt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.gt` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_gt_imm)]
            F32GtImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.le` equivalent Wasmi instruction.
            #[snake_name(f32_le)]
            F32Le {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.le` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_le_imm)]
            F32LeImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.lt` equivalent Wasmi instruction.
            #[snake_name(f32_lt)]
            F32Lt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.lt` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_lt_imm)]
            F32LtImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f64.eq` equivalent Wasmi instruction.
            #[snake_name(f64_eq)]
            F64Eq {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.eq` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_eq_imm)]
            F64EqImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.ge` equivalent Wasmi instruction.
            #[snake_name(f64_ge)]
            F64Ge {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.ge` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_ge_imm)]
            F64GeImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.gt` equivalent Wasmi instruction.
            #[snake_name(f64_gt)]
            F64Gt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.gt` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_gt_imm)]
            F64GtImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.le` equivalent Wasmi instruction.
            #[snake_name(f64_le)]
            F64Le {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.le` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_le_imm)]
            F64LeImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.lt` equivalent Wasmi instruction.
            #[snake_name(f64_lt)]
            F64Lt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.lt` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_lt_imm)]
            F64LtImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `i32.clz` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_clz)]
            I32Clz {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the input value of the instruction.
                value: Reg,
            },
            /// Wasm `i32.ctz` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_ctz)]
            I32Ctz {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the input value of the instruction.
                value: Reg,
            },
            /// Wasm `i32.popcnt` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_popcnt)]
            I32Popcnt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the input value of the instruction.
                value: Reg,
            },
            /// Wasm `i32.add` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_add)]
            I32Add {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.add` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_add_imm)]
            I32AddImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.sub` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_sub)]
            I32Sub {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.sub` equivalent Wasmi bytecode instruction with immediate `lhs` value.
            #[snake_name(i32_sub_imm_lhs)]
            I32SubImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.mul` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_mul)]
            I32Mul {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.mul` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_mul_imm)]
            I32MulImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.div_s` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_div_s)]
            I32DivS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.div_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_div_s_imm_lhs)]
            I32DivSImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.div_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_div_s_imm_rhs)]
            I32DivSImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroI32,
            },
            /// Wasm `i32.div_u` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_div_u)]
            I32DivU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.div_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_div_u_imm_lhs)]
            I32DivUImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: u32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.div_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_div_u_imm_rhs)]
            I32DivUImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroU32,
            },
            /// Wasm `i32.rem_s` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_rem_s)]
            I32RemS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rem_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rem_s_imm_lhs)]
            I32RemSImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rem_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rem_s_imm_rhs)]
            I32RemSImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroI32,
            },
            /// Wasm `i32.rem_u` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_rem_u)]
            I32RemU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rem_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rem_u_imm_lhs)]
            I32RemUImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: u32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rem_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rem_u_imm_rhs)]
            I32RemUImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroU32,
            },
            /// Wasm `i32.and` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_and)]
            I32And {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.and` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_and_imm)]
            I32AndImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.or` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_or)]
            I32Or {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.or` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_or_imm)]
            I32OrImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.xor` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_xor)]
            I32Xor {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.xor` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_xor_imm)]
            I32XorImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.shl` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_shl)]
            I32Shl {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.shl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_shl_imm_lhs)]
            I32ShlImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.shl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_shl_imm_rhs)]
            I32ShlImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.shr_s` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_shr_s)]
            I32ShrS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.shr_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_shr_s_imm_lhs)]
            I32ShrSImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.shr_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_shr_s_imm_rhs)]
            I32ShrSImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.shr_u` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_shr_u)]
            I32ShrU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.shr_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_shr_u_imm_lhs)]
            I32ShrUImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.shr_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_shr_u_imm_rhs)]
            I32ShrUImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.rotl` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_rotl)]
            I32Rotl {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rotl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rotl_imm_lhs)]
            I32RotlImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rotl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rotl_imm_rhs)]
            I32RotlImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i32.rotr` equivalent Wasmi bytecode instruction.
            #[snake_name(i32_rotr)]
            I32Rotr {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rotr` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rotr_imm_lhs)]
            I32RotrImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i32.rotr` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i32_rotr_imm_rhs)]
            I32RotrImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i32,
            },
            /// Wasm `i64.clz` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_clz)]
            I64Clz {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the input value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.ctz` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_ctz)]
            I64Ctz {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the input value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.popcnt` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_popcnt)]
            I64Popcnt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the input value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.add` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_add)]
            I64Add {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.add` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_add_imm)]
            I64AddImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.sub` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_sub)]
            I64Sub {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.sub` equivalent Wasmi bytecode instruction with immediate `lhs` value.
            #[snake_name(i64_sub_imm_lhs)]
            I64SubImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.mul` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_mul)]
            I64Mul {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.mul` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_mul_imm)]
            I64MulImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.div_s` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_div_s)]
            I64DivS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.div_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_div_s_imm_lhs)]
            I64DivSImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.div_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_div_s_imm_rhs)]
            I64DivSImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroI64,
            },
            /// Wasm `i64.div_u` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_div_u)]
            I64DivU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.div_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_div_u_imm_lhs)]
            I64DivUImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: u64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.div_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_div_u_imm_rhs)]
            I64DivUImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroU64,
            },
            /// Wasm `i64.rem_s` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_rem_s)]
            I64RemS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rem_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rem_s_imm_lhs)]
            I64RemSImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rem_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rem_s_imm_rhs)]
            I64RemSImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroI32,
            },
            /// Wasm `i64.rem_u` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_rem_u)]
            I64RemU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rem_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rem_u_imm_lhs)]
            I64RemUImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: u64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rem_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rem_u_imm_rhs)]
            I64RemUImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: ::core::num::NonZeroU64,
            },
            /// Wasm `i64.and` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_and)]
            I64And {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.and` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_and_imm)]
            I64AndImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.or` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_or)]
            I64Or {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.or` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_or_imm)]
            I64OrImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.xor` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_xor)]
            I64Xor {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.xor` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_xor_imm)]
            I64XorImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.shl` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_shl)]
            I64Shl {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.shl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_shl_imm_lhs)]
            I64ShlImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.shl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_shl_imm_rhs)]
            I64ShlImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.shr_s` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_shr_s)]
            I64ShrS {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.shr_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_shr_s_imm_lhs)]
            I64ShrSImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.shr_s` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_shr_s_imm_rhs)]
            I64ShrSImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.shr_u` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_shr_u)]
            I64ShrU {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.shr_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_shr_u_imm_lhs)]
            I64ShrUImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.shr_u` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_shr_u_imm_rhs)]
            I64ShrUImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.rotl` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_rotl)]
            I64Rotl {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rotl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rotl_imm_lhs)]
            I64RotlImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rotl` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rotl_imm_rhs)]
            I64RotlImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i64.rotr` equivalent Wasmi bytecode instruction.
            #[snake_name(i64_rotr)]
            I64Rotr {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rotr` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rotr_imm_lhs)]
            I64RotrImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: i64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `i64.rotr` equivalent Wasmi bytecode instruction with immediate `rhs` value.
            #[snake_name(i64_rotr_imm_rhs)]
            I64RotrImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: i64,
            },
            /// Wasm `i32.extend8_s` equivalent Wasmi instruction.
            #[snake_name(i32_extend8_s)]
            I32Extend8S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `i32.extend16_s` equivalent Wasmi instruction.
            #[snake_name(i32_extend16_s)]
            I32Extend16S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `i64.extend8_s` equivalent Wasmi instruction.
            #[snake_name(i64_extend8_s)]
            I64Extend8S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `i64.extend16_s` equivalent Wasmi instruction.
            #[snake_name(i64_extend16_s)]
            I64Extend16S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `i64.extend32_s` equivalent Wasmi instruction.
            #[snake_name(i64_extend32_s)]
            I64Extend32S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },

            /// Wasm `f32.abs` equivalent Wasmi instruction.
            #[snake_name(f32_abs)]
            F32Abs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.neg` equivalent Wasmi instruction.
            #[snake_name(f32_neg)]
            F32Neg {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.ceil` equivalent Wasmi instruction.
            #[snake_name(f32_ceil)]
            F32Ceil {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.floor` equivalent Wasmi instruction.
            #[snake_name(f32_floor)]
            F32Floor {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.trunc` equivalent Wasmi instruction.
            #[snake_name(f32_trunc)]
            F32Trunc {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.nearest` equivalent Wasmi instruction.
            #[snake_name(f32_nearest)]
            F32Nearest {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.sqrt` equivalent Wasmi instruction.
            #[snake_name(f32_sqrt)]
            F32Sqrt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f32.add` equivalent Wasmi instruction.
            #[snake_name(f32_add)]
            F32Add {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.add` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f32_add_imm_lhs)]
            F32AddImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee32,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.add` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_add_imm_rhs)]
            F32AddImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.sub` equivalent Wasmi instruction.
            #[snake_name(f32_sub)]
            F32Sub{
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.sub` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f32_sub_imm_lhs)]
            F32SubImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: Ieee32,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.sub` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_sub_imm_rhs)]
            F32SubImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.mul` equivalent Wasmi instruction.
            #[snake_name(f32_mul)]
            F32Mul {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.mul` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f32_mul_imm_lhs)]
            F32MulImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee32,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.mul` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_mul_imm_rhs)]
            F32MulImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.div` equivalent Wasmi instruction.
            #[snake_name(f32_div)]
            F32Div {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.div` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f32_div_imm_lhs)]
            F32DivImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee32,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.div` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_div_imm_rhs)]
            F32DivImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.min` equivalent Wasmi instruction.
            #[snake_name(f32_min)]
            F32Min {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.min` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_min_imm)]
            F32MinImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.max` equivalent Wasmi instruction.
            #[snake_name(f32_max)]
            F32Max {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.max` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_max_imm)]
            F32MaxImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee32,
            },
            /// Wasm `f32.copysign` equivalent Wasmi instruction.
            #[snake_name(f32_copysign)]
            F32Copysign {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.copysign` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f32_copysign_imm_lhs)]
            F32CopysignImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee32,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f32.copysign` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f32_copysign_imm_rhs)]
            F32CopysignImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` sign of the instruction.
                rhs: Sign,
            },

            /// Wasm `f64.abs` equivalent Wasmi instruction.
            #[snake_name(f64_abs)]
            F64Abs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.neg` equivalent Wasmi instruction.
            #[snake_name(f64_neg)]
            F64Neg {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.ceil` equivalent Wasmi instruction.
            #[snake_name(f64_ceil)]
            F64Ceil {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.floor` equivalent Wasmi instruction.
            #[snake_name(f64_floor)]
            F64Floor {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.trunc` equivalent Wasmi instruction.
            #[snake_name(f64_trunc)]
            F64Trunc {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.nearest` equivalent Wasmi instruction.
            #[snake_name(f64_nearest)]
            F64Nearest {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.sqrt` equivalent Wasmi instruction.
            #[snake_name(f64_sqrt)]
            F64Sqrt {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `value` of the instruction.
                value: Reg,
            },
            /// Wasm `f64.add` equivalent Wasmi instruction.
            #[snake_name(f64_add)]
            F64Add {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.add` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f64_add_imm_lhs)]
            F64AddImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee64,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.add` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_add_imm_rhs)]
            F64AddImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.sub` equivalent Wasmi instruction.
            #[snake_name(f64_sub)]
            F64Sub{
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.sub` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f64_sub_imm_lhs)]
            F64SubImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `lhs` value of the instruction.
                lhs: Ieee64,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.sub` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_sub_imm_rhs)]
            F64SubImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.mul` equivalent Wasmi instruction.
            #[snake_name(f64_mul)]
            F64Mul {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.mul` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f64_mul_imm_lhs)]
            F64MulImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee64,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.mul` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_mul_imm_rhs)]
            F64MulImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.div` equivalent Wasmi instruction.
            #[snake_name(f64_div)]
            F64Div {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.div` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f64_div_imm_lhs)]
            F64DivImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee64,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.div` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_div_imm_rhs)]
            F64DivImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.min` equivalent Wasmi instruction.
            #[snake_name(f64_min)]
            F64Min {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.min` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_min_imm)]
            F64MinImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.max` equivalent Wasmi instruction.
            #[snake_name(f64_max)]
            F64Max {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.max` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_max_imm)]
            F64MaxImm {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` value of the instruction.
                rhs: Ieee64,
            },
            /// Wasm `f64.copysign` equivalent Wasmi instruction.
            #[snake_name(f64_copysign)]
            F64Copysign {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The register storing the `rhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.copysign` equivalent Wasmi instruction with immediate `lhs` value.
            #[snake_name(f64_copysign_imm_lhs)]
            F64CopysignImmLhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The immediate `rhs` value of the instruction.
                lhs: Ieee64,
                /// The register storing the `lhs` value of the instruction.
                rhs: Reg,
            },
            /// Wasm `f64.copysign` equivalent Wasmi instruction with immediate `rhs` value.
            #[snake_name(f64_copysign_imm_rhs)]
            F64CopysignImmRhs {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the `lhs` value of the instruction.
                lhs: Reg,
                /// The immediate `rhs` sign of the instruction.
                rhs: Sign,
            },

            /// Wasm `i32.trunc_f32_s` equivalent Wasmi instruction.
            #[snake_name(i32_trunc_f32_s)]
            I32TruncF32S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i32.trunc_f32_u` equivalent Wasmi instruction.
            #[snake_name(i32_trunc_f32_u)]
            I32TruncF32U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i32.trunc_f64_s` equivalent Wasmi instruction.
            #[snake_name(i32_trunc_f64_s)]
            I32TruncF64S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i32.trunc_f64_u` equivalent Wasmi instruction.
            #[snake_name(i32_trunc_f64_u)]
            I32TruncF64U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.trunc_f32_s` equivalent Wasmi instruction.
            #[snake_name(i64_trunc_f32_s)]
            I64TruncF32S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.trunc_f32_u` equivalent Wasmi instruction.
            #[snake_name(i64_trunc_f32_u)]
            I64TruncF32U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.trunc_f64_s` equivalent Wasmi instruction.
            #[snake_name(i64_trunc_f64_s)]
            I64TruncF64S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `i64.trunc_f64_u` equivalent Wasmi instruction.
            #[snake_name(i64_trunc_f64_u)]
            I64TruncF64U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },

            /// Wasm `f32.demote_f64` equivalent Wasmi instruction.
            #[snake_name(f32_demote_f64)]
            F32DemoteF64 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f64.promote_f32` equivalent Wasmi instruction.
            #[snake_name(f64_promote_f32)]
            F64PromoteF32 {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f32.convert_i32_s` equivalent Wasmi instruction.
            #[snake_name(f32_convert_i32_s)]
            F32ConvertI32S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f32.convert_i32_u` equivalent Wasmi instruction.
            #[snake_name(f32_convert_i32_u)]
            F32ConvertI32U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f32.convert_i64_s` equivalent Wasmi instruction.
            #[snake_name(f32_convert_i64_s)]
            F32ConvertI64S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f32.convert_i64_u` equivalent Wasmi instruction.
            #[snake_name(f32_convert_i64_u)]
            F32ConvertI64U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f64.convert_i32_s` equivalent Wasmi instruction.
            #[snake_name(f64_convert_i32_s)]
            F64ConvertI32S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f64.convert_i32_u` equivalent Wasmi instruction.
            #[snake_name(f64_convert_i32_u)]
            F64ConvertI32U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f64.convert_i64_s` equivalent Wasmi instruction.
            #[snake_name(f64_convert_i64_s)]
            F64ConvertI64S {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `f64.convert_i64_u` equivalent Wasmi instruction.
            #[snake_name(f64_convert_i64_u)]
            F64ConvertI64U {
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },

            /// Wasm `table.get` equivalent Wasmi instruction.
            #[snake_name(table_get)]
            TableGet {
                /// The table that is used by the instruction.
                table: Table,
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the index of the instruction.
                index: Reg,
            },
            /// Wasm `table.get` equivalent Wasmi instruction with an immediate `index` value.
            #[snake_name(table_get_imm)]
            TableGetImm {
                /// The table that is used by the instruction.
                table: Table,
                /// The register storing the result of the instruction.
                result: Reg,
                /// The register storing the index of the instruction.
                index: u32,
            },
            /// Wasm `table.set` equivalent Wasmi instruction.
            #[snake_name(table_set)]
            TableSet {
                /// The table that is used by the instruction.
                table: Table,
                /// The register storing the index of the instruction.
                index: Reg,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `table.set` equivalent Wasmi instruction with an immediate `index` value.
            #[snake_name(table_set_imm)]
            TableSetImm {
                /// The table that is used by the instruction.
                table: Table,
                /// The register storing the index of the instruction.
                index: u32,
                /// The register storing the value of the instruction.
                value: Reg,
            },
            /// Wasm `table.size` equivalent Wasmi instruction.
            #[snake_name(table_size)]
            TableSize {
                /// The table that is used by the instruction.
                table: Table,
                /// The register storing the result of the instruction.
                result: Reg,
            },

            /// Wasm `table.copy` equivalent Wasmi instruction.
            #[snake_name(table_copy)]
            TableCopy {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            #[snake_name(table_copy_to)]
            TableCopyTo {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            #[snake_name(table_copy_from)]
            TableCopyFrom {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            #[snake_name(table_copy_from_to)]
            TableCopyFromTo {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            #[snake_name(table_copy_exact)]
            TableCopyExact {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_copy_to_exact)]
            TableCopyToExact {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_copy_from_exact)]
            TableCopyFromExact {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `table.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_copy_from_to_exact)]
            TableCopyFromToExact {
                /// The destination table of the instruction.
                dst_table: Table,
                /// The source table of the instruction.
                src_table: Table,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },

            /// Wasm `table.init` equivalent Wasmi instruction.
            #[snake_name(table_init)]
            TableInit {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            #[snake_name(table_init_to)]
            TableInitTo {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            #[snake_name(table_init_from)]
            TableInitFrom {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            #[snake_name(table_init_from_to)]
            TableInitFromTo {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            #[snake_name(table_init_exact)]
            TableInitExact {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_init_to_exact)]
            TableInitToExact {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_init_from_exact)]
            TableInitFromExact {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `table.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_init_from_to_exact)]
            TableInitFromToExact {
                /// The destination table of the instruction.
                table: Table,
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },

            /// Wasm `table.fill` equivalent Wasmi instruction.
            #[snake_name(table_fill)]
            TableFill {
                /// The table used by the instruction.
                table: Table,
                /// The index in the destination table.
                index: Reg,
                /// The number of table elements to copy.
                len: Reg,
                /// The value that is used to fill the table.
                value: Reg,
            },
            /// Wasm `table.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            #[snake_name(table_fill_at)]
            TableFillAt {
                /// The table used by the instruction.
                table: Table,
                /// The index in the destination table.
                index: u32,
                /// The number of table elements to copy.
                len: Reg,
                /// The value that is used to fill the table.
                value: Reg,
            },
            /// Wasm `table.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            #[snake_name(table_fill_exact)]
            TableFillExact {
                /// The table used by the instruction.
                table: Table,
                /// The index in the destination table.
                index: Reg,
                /// The number of table elements to copy.
                len: u32,
                /// The value that is used to fill the table.
                value: Reg,
            },
            /// Wasm `table.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(table_fill_at_exact)]
            TableFillToExact {
                /// The table used by the instruction.
                table: Table,
                /// The index in the destination table.
                index: u32,
                /// The number of table elements to copy.
                len: u32,
                /// The value that is used to fill the table.
                value: Reg,
            },

            /// Wasm `table.grow` equivalent Wasmi instruction.
            #[snake_name(table_grow)]
            TableGrow {
                /// Register holding the result of the instruction.
                result: Reg,
                /// The number of elements to add to the table.
                delta: Reg,
                /// The value that is used to fill up the new cells.
                value: Reg,
            },
            /// Wasm `table.grow` equivalent Wasmi instruction with immediate `delta` value.
            #[snake_name(table_grow_imm)]
            TableGrowImm {
                /// Register holding the result of the instruction.
                result: Reg,
                /// The number of elements to add to the table.
                delta: u32,
                /// The value that is used to fill up the new cells.
                value: Reg,
            },

            /// A Wasm `elem.drop` equalivalent Wasmi instruction.
            #[snake_name(elem_drop)]
            ElemDrop {
                segment: ElementSegment,
            },
            /// A Wasm `data.drop` equalivalent Wasmi instruction.
            #[snake_name(data_drop)]
            DataDrop {
                segment: DataSegment,
            },

            /// Wasm `memory.size` equivalent Wasmi instruction.
            #[snake_name(memory_size)]
            MemorySize {
                /// The register storing the result of the instruction.
                result: Reg,
            },

            /// Wasm `memory.grow` equivalent Wasmi instruction.
            #[snake_name(memory_grow)]
            MemoryGrow {
                /// Register holding the result of the instruction.
                result: Reg,
                /// The number of elements to add to the table.
                delta: Reg,
                /// The value that is used to fill up the new cells.
                value: Reg,
            },
            /// Wasm `memory.grow` equivalent Wasmi instruction with immediate `delta` value.
            #[snake_name(memory_grow_imm)]
            MemoryGrowImm {
                /// Register holding the result of the instruction.
                result: Reg,
                /// The number of elements to add to the table.
                delta: u32,
                /// The value that is used to fill up the new cells.
                value: Reg,
            },

            /// Wasm `memory.copy` equivalent Wasmi instruction.
            #[snake_name(memory_copy)]
            MemoryCopy {
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            #[snake_name(memory_copy_to)]
            MemoryCopyTo {
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            #[snake_name(memory_copy_from)]
            MemoryCopyFrom {
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            #[snake_name(memory_copy_from_to)]
            MemoryCopyFromTo {
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            #[snake_name(memory_copy_exact)]
            MemoryCopyExact {
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_copy_to_exact)]
            MemoryCopyToExact {
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_copy_from_exact)]
            MemoryCopyFromExact {
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `memory.copy` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_copy_from_to_exact)]
            MemoryCopyFromToExact {
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },

            /// Wasm `memory.fill` equivalent Wasmi instruction.
            #[snake_name(memory_fill)]
            MemoryFill {
                /// The start index of the linear memory to fill.
                index: Reg,
                /// The number of bytes to fill.
                len: Reg,
                /// The byte value used to fill the linear memory.
                value: Reg,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            #[snake_name(memory_fill_at)]
            MemoryFillAt {
                /// The start index of the memory to fill.
                index: u32,
                /// The number of bytes to fill.
                len: Reg,
                /// The byte value used to fill the memory.
                value: Reg,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_fill_imm)]
            MemoryFillImm {
                /// The start index of the memory to fill.
                index: Reg,
                /// The number of bytes to fill.
                len: Reg,
                /// The byte value used to fill the memory.
                value: u8,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            #[snake_name(memory_fill_exact)]
            MemoryFillExact {
                /// The start index of the memory to fill.
                index: Reg,
                /// The number of bytes to fill.
                len: u32,
                /// The byte value used to fill the memory.
                value: Reg,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            /// - Uses an immediate `value`.
            #[snake_name(memory_fill_at_imm)]
            MemoryFillAtImm {
                /// The start index of the memory to fill.
                index: u32,
                /// The number of bytes to fill.
                len: Reg,
                /// The byte value used to fill the memory.
                value: u8,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_fill_at_exact)]
            MemoryFillAtExact {
                /// The start index of the memory to fill.
                index: u32,
                /// The number of bytes to fill.
                len: u32,
                /// The byte value used to fill the memory.
                value: Reg,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            /// - Uses an immediate `value`.
            #[snake_name(memory_fill_imm_exact)]
            MemoryFillImmExact {
                /// The start index of the memory to fill.
                index: Reg,
                /// The number of bytes to fill.
                len: u32,
                /// The byte value used to fill the memory.
                value: u8,
            },
            /// Wasm `memory.fill` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `index` index value.
            /// - Uses an immediate `len` value.
            /// - Uses an immediate `value`.
            #[snake_name(memory_fill_at_imm_exact)]
            MemoryFillAtImmExact {
                /// The start index of the memory to fill.
                index: u32,
                /// The number of bytes to fill.
                len: u32,
                /// The byte value used to fill the memory.
                value: u8,
            },

            /// Wasm `memory.init` equivalent Wasmi instruction.
            #[snake_name(memory_init)]
            MemoryInit {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            #[snake_name(memory_init_to)]
            MemoryInitTo {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            #[snake_name(memory_init_from)]
            MemoryInitFrom {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            #[snake_name(memory_init_from_to)]
            MemoryInitFromTo {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: Reg,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `len` value.
            #[snake_name(memory_init_exact)]
            MemoryInitExact {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_init_to_exact)]
            MemoryInitToExact {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: Reg,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_init_from_exact)]
            MemoryInitFromExact {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: Reg,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },
            /// Wasm `memory.init` equivalent Wasmi instruction.
            ///
            /// - Uses an immediate `dst_index` index value.
            /// - Uses an immediate `src_index` index value.
            /// - Uses an immediate `len` value.
            #[snake_name(memory_init_from_to_exact)]
            MemoryInitFromToExact {
                /// The source element segment of the instruction.
                segment: ElementSegment,
                /// The index in the destination table.
                dst_index: u32,
                /// The index in the source table.
                src_index: u32,
                /// The number of table elements to copy.
                len: u32,
            },
        }
    };
}
