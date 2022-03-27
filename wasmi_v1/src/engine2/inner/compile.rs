use super::{
    super::{IrProvider, IrRegister},
    EngineInner,
    EngineResources,
};
use crate::engine2::{
    func_builder::{CompileContext, IrProviderSlice, OpaqueInstruction},
    ConstPool,
    ExecInstruction,
    ExecProvider,
    ExecProviderSlice,
    ExecRegister,
    FuncBody,
    Instruction,
    Offset,
};

/// Creates a closure constructing a `wasmi` unary instruction.
macro_rules! unary_op {
    ( $name:ident ) => {{
        |result, input| ExecInstruction::$name { result, input }
    }};
}

/// Creates a closure constructing a `wasmi` binary instruction.
macro_rules! binary_op {
    ( $name:ident ) => {{
        |result, lhs, rhs| ExecInstruction::$name { result, lhs, rhs }
    }};
}

/// Creates a closure for constructing a `wasmi` load instruction.
macro_rules! load_op {
    ( $name:ident ) => {{
        |result, ptr, offset| ExecInstruction::$name {
            result,
            ptr,
            offset,
        }
    }};
}

/// Creates a closure for constructing a `wasmi` store instruction.
macro_rules! store_op {
    ( $name:ident ) => {{
        |ptr, offset, value| ExecInstruction::$name { ptr, offset, value }
    }};
}

impl EngineInner {
    pub fn compile<I>(&mut self, context: &CompileContext, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = OpaqueInstruction>,
    {
        let insts = insts
            .into_iter()
            .map(|inst| Self::compile_inst(&mut self.res, context, inst));
        self.code_map.alloc(insts)
    }

    fn compile_register(context: &CompileContext, register: IrRegister) -> ExecRegister {
        context.compile_register(register)
    }

    fn compile_provider_impl(
        const_pool: &mut ConstPool,
        context: &CompileContext,
        provider: IrProvider,
    ) -> ExecProvider {
        match provider {
            IrProvider::Register(register) => {
                ExecProvider::from_register(Self::compile_register(context, register))
            }
            IrProvider::Immediate(value) => {
                ExecProvider::from_immediate(const_pool.alloc_const(value))
            }
        }
    }

    fn compile_provider(
        res: &mut EngineResources,
        context: &CompileContext,
        provider: IrProvider,
    ) -> ExecProvider {
        Self::compile_provider_impl(&mut res.const_pool, context, provider)
    }

    fn compile_provider_slice(
        res: &mut EngineResources,
        context: &CompileContext,
        provider: IrProviderSlice,
    ) -> ExecProviderSlice {
        let providers = context
            .resolve_provider_slice(provider)
            .iter()
            .copied()
            .map(|provider| Self::compile_provider_impl(&mut res.const_pool, context, provider));
        res.provider_slices.alloc(providers)
    }

    fn compile_inst_rr(
        context: &CompileContext,
        result: IrRegister,
        input: IrRegister,
        make_op: fn(ExecRegister, ExecRegister) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let input = Self::compile_register(context, input);
        make_op(result, input)
    }

    fn compile_inst_rp(
        res: &mut EngineResources,
        context: &CompileContext,
        result: IrRegister,
        input: IrProvider,
        make_op: fn(ExecRegister, ExecProvider) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let input = Self::compile_provider(res, context, input);
        make_op(result, input)
    }

    fn compile_inst_rrp(
        res: &mut EngineResources,
        context: &CompileContext,
        result: IrRegister,
        lhs: IrRegister,
        rhs: IrProvider,
        make_op: fn(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let lhs = Self::compile_register(context, lhs);
        let rhs = Self::compile_provider(res, context, rhs);
        make_op(result, lhs, rhs)
    }

    fn compile_load(
        context: &CompileContext,
        result: IrRegister,
        ptr: IrRegister,
        offset: Offset,
        make_op: fn(ExecRegister, ExecRegister, Offset) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let ptr = Self::compile_register(context, ptr);
        make_op(result, ptr, offset)
    }

    fn compile_store(
        res: &mut EngineResources,
        context: &CompileContext,
        ptr: IrRegister,
        offset: Offset,
        value: IrProvider,
        make_op: fn(ExecRegister, Offset, ExecProvider) -> ExecInstruction,
    ) -> ExecInstruction {
        let ptr = Self::compile_register(context, ptr);
        let value = Self::compile_provider(res, context, value);
        make_op(ptr, offset, value)
    }

    fn compile_inst(
        res: &mut EngineResources,
        context: &CompileContext,
        inst: OpaqueInstruction,
    ) -> ExecInstruction {
        match inst {
            Instruction::Trap { trap_code } => ExecInstruction::Trap { trap_code },
            Instruction::Return { results } => {
                let results = Self::compile_provider_slice(res, context, results);
                ExecInstruction::Return { results }
            }

            Instruction::Copy { result, input } => {
                Self::compile_inst_rp(res, context, result, input, unary_op!(Copy))
            }
            Instruction::Select {
                result,
                condition,
                if_true,
                if_false,
            } => {
                let result = Self::compile_register(context, result);
                let condition = Self::compile_register(context, condition);
                let if_true = Self::compile_provider(res, context, if_true);
                let if_false = Self::compile_provider(res, context, if_false);
                ExecInstruction::Select {
                    result,
                    condition,
                    if_true,
                    if_false,
                }
            }

            Instruction::MemorySize { result } => {
                let result = Self::compile_register(context, result);
                ExecInstruction::MemorySize { result }
            }
            Instruction::MemoryGrow { result, amount } => {
                let result = Self::compile_register(context, result);
                let amount = Self::compile_provider(res, context, amount);
                ExecInstruction::MemoryGrow { result, amount }
            }

            Instruction::GlobalGet { result, global } => {
                let result = Self::compile_register(context, result);
                ExecInstruction::GlobalGet { result, global }
            }
            Instruction::GlobalSet { global, value } => {
                let value = Self::compile_provider(res, context, value);
                ExecInstruction::GlobalSet { global, value }
            }

            Instruction::I32Load {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I32Load)),
            Instruction::I64Load {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load)),
            Instruction::F32Load {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(F32Load)),
            Instruction::F64Load {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(F64Load)),
            Instruction::I32Load8S {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I32Load8S)),
            Instruction::I32Load8U {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I32Load8U)),
            Instruction::I32Load16S {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I32Load16S)),
            Instruction::I32Load16U {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I32Load16U)),
            Instruction::I64Load8S {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load8S)),
            Instruction::I64Load8U {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load8U)),
            Instruction::I64Load16S {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load16S)),
            Instruction::I64Load16U {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load16U)),
            Instruction::I64Load32S {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load32S)),
            Instruction::I64Load32U {
                result,
                ptr,
                offset,
            } => Self::compile_load(context, result, ptr, offset, load_op!(I64Load32U)),

            Instruction::I32Store { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I32Store))
            }
            Instruction::I64Store { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I64Store))
            }
            Instruction::F32Store { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(F32Store))
            }
            Instruction::F64Store { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(F64Store))
            }
            Instruction::I32Store8 { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I32Store8))
            }
            Instruction::I32Store16 { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I32Store16))
            }
            Instruction::I64Store8 { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I64Store8))
            }
            Instruction::I64Store16 { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I64Store16))
            }
            Instruction::I64Store32 { ptr, offset, value } => {
                Self::compile_store(res, context, ptr, offset, value, store_op!(I64Store32))
            }

            Instruction::I32Clz { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32Clz))
            }
            Instruction::I32Ctz { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32Ctz))
            }
            Instruction::I32Popcnt { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32Popcnt))
            }

            Instruction::I32Add { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Add))
            }
            Instruction::I32Sub { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Sub))
            }
            Instruction::I32Mul { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Mul))
            }
            Instruction::I32DivS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32DivS))
            }
            Instruction::I32DivU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32DivU))
            }
            Instruction::I32RemS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32RemS))
            }
            Instruction::I32RemU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32RemU))
            }
            Instruction::I32Shl { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Shl))
            }
            Instruction::I32ShrS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32ShrS))
            }
            Instruction::I32ShrU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32ShrU))
            }
            Instruction::I32Rotl { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Rotl))
            }
            Instruction::I32Rotr { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Rotr))
            }
            Instruction::I32And { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32And))
            }
            Instruction::I32Or { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Or))
            }
            Instruction::I32Xor { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Xor))
            }

            Instruction::I64Clz { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64Clz))
            }
            Instruction::I64Ctz { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64Ctz))
            }
            Instruction::I64Popcnt { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64Popcnt))
            }

            Instruction::I64Add { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Add))
            }
            Instruction::I64Sub { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Sub))
            }
            Instruction::I64Mul { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Mul))
            }
            Instruction::I64DivS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64DivS))
            }
            Instruction::I64DivU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64DivU))
            }
            Instruction::I64RemS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64RemS))
            }
            Instruction::I64RemU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64RemU))
            }
            Instruction::I64Shl { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Shl))
            }
            Instruction::I64ShrS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64ShrS))
            }
            Instruction::I64ShrU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64ShrU))
            }
            Instruction::I64Rotl { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Rotl))
            }
            Instruction::I64Rotr { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Rotr))
            }
            Instruction::I64And { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64And))
            }
            Instruction::I64Or { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Or))
            }
            Instruction::I64Xor { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Xor))
            }

            Instruction::F32Add { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Add))
            }
            Instruction::F32Sub { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Sub))
            }
            Instruction::F32Mul { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Mul))
            }
            Instruction::F32Div { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Div))
            }
            Instruction::F32Min { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Min))
            }
            Instruction::F32Max { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Max))
            }
            Instruction::F32Copysign { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Copysign))
            }

            Instruction::F64Add { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Add))
            }
            Instruction::F64Sub { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Sub))
            }
            Instruction::F64Mul { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Mul))
            }
            Instruction::F64Div { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Div))
            }
            Instruction::F64Min { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Min))
            }
            Instruction::F64Max { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Max))
            }
            Instruction::F64Copysign { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Copysign))
            }

            Instruction::I32Eq { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Eq))
            }
            Instruction::I32Ne { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32Ne))
            }
            Instruction::I32LtS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32LtS))
            }
            Instruction::I32LtU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32LtU))
            }
            Instruction::I32LeS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32LeS))
            }
            Instruction::I32LeU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32LeU))
            }
            Instruction::I32GtS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32GtS))
            }
            Instruction::I32GtU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32GtU))
            }
            Instruction::I32GeS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32GeS))
            }
            Instruction::I32GeU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I32GeU))
            }

            Instruction::I64Eq { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Eq))
            }
            Instruction::I64Ne { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64Ne))
            }
            Instruction::I64LtS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64LtS))
            }
            Instruction::I64LtU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64LtU))
            }
            Instruction::I64LeS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64LeS))
            }
            Instruction::I64LeU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64LeU))
            }
            Instruction::I64GtS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64GtS))
            }
            Instruction::I64GtU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64GtU))
            }
            Instruction::I64GeS { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64GeS))
            }
            Instruction::I64GeU { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(I64GeU))
            }

            Instruction::F32Eq { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Eq))
            }
            Instruction::F32Ne { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Ne))
            }
            Instruction::F32Lt { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Lt))
            }
            Instruction::F32Le { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Le))
            }
            Instruction::F32Gt { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Gt))
            }
            Instruction::F32Ge { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F32Ge))
            }

            Instruction::F64Eq { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Eq))
            }
            Instruction::F64Ne { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Ne))
            }
            Instruction::F64Lt { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Lt))
            }
            Instruction::F64Le { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Le))
            }
            Instruction::F64Gt { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Gt))
            }
            Instruction::F64Ge { result, lhs, rhs } => {
                Self::compile_inst_rrp(res, context, result, lhs, rhs, binary_op!(F64Ge))
            }

            Instruction::F32Abs { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Abs))
            }
            Instruction::F32Neg { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Neg))
            }
            Instruction::F32Ceil { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Ceil))
            }
            Instruction::F32Floor { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Floor))
            }
            Instruction::F32Trunc { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Trunc))
            }
            Instruction::F32Nearest { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Nearest))
            }
            Instruction::F32Sqrt { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32Sqrt))
            }

            Instruction::F64Abs { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Abs))
            }
            Instruction::F64Neg { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Neg))
            }
            Instruction::F64Ceil { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Ceil))
            }
            Instruction::F64Floor { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Floor))
            }
            Instruction::F64Trunc { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Trunc))
            }
            Instruction::F64Nearest { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Nearest))
            }
            Instruction::F64Sqrt { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64Sqrt))
            }

            Instruction::I32WrapI64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32WrapI64))
            }
            Instruction::I32TruncSF32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncSF32))
            }
            Instruction::I32TruncUF32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncUF32))
            }
            Instruction::I32TruncSF64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncSF64))
            }
            Instruction::I32TruncUF64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncUF64))
            }
            Instruction::I64ExtendSI32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64ExtendSI32))
            }
            Instruction::I64ExtendUI32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64ExtendUI32))
            }
            Instruction::I64TruncSF32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncSF32))
            }
            Instruction::I64TruncUF32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncUF32))
            }
            Instruction::I64TruncSF64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncSF64))
            }
            Instruction::I64TruncUF64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncUF64))
            }
            Instruction::F32ConvertSI32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32ConvertSI32))
            }
            Instruction::F32ConvertUI32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32ConvertUI32))
            }
            Instruction::F32ConvertSI64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32ConvertSI64))
            }
            Instruction::F32ConvertUI64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32ConvertUI64))
            }
            Instruction::F32DemoteF64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F32DemoteF64))
            }
            Instruction::F64ConvertSI32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64ConvertSI32))
            }
            Instruction::F64ConvertUI32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64ConvertUI32))
            }
            Instruction::F64ConvertSI64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64ConvertSI64))
            }
            Instruction::F64ConvertUI64 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64ConvertUI64))
            }
            Instruction::F64PromoteF32 { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(F64PromoteF32))
            }

            Instruction::I32Extend8S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32Extend8S))
            }
            Instruction::I32Extend16S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32Extend16S))
            }
            Instruction::I64Extend8S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64Extend8S))
            }
            Instruction::I64Extend16S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64Extend16S))
            }
            Instruction::I64Extend32S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64Extend32S))
            }

            Instruction::I32TruncSatF32S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncSatF32S))
            }
            Instruction::I32TruncSatF32U { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncSatF32U))
            }
            Instruction::I32TruncSatF64S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncSatF64S))
            }
            Instruction::I32TruncSatF64U { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I32TruncSatF64U))
            }
            Instruction::I64TruncSatF32S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncSatF32S))
            }
            Instruction::I64TruncSatF32U { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncSatF32U))
            }
            Instruction::I64TruncSatF64S { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncSatF64S))
            }
            Instruction::I64TruncSatF64U { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(I64TruncSatF64U))
            }

            _ => todo!(),
        }
    }
}
