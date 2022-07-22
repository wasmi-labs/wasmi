use super::{
    super::{ExecRegisterSlice, IrProvider, IrRegister},
    EngineInner,
    EngineResources,
};
use crate::engine::{
    func_builder::{CompileContext, IrInstruction, IrProviderSlice, IrRegisterSlice},
    ConstPool,
    ExecInstruction,
    ExecProvider,
    ExecProviderSlice,
    ExecRegister,
    FuncBody,
    Instruction,
    Offset,
};
use wasmi_core::UntypedValue;

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
        I: IntoIterator<Item = IrInstruction>,
    {
        let len_regs = context.len_registers();
        let insts = insts
            .into_iter()
            .map(|inst| Self::compile_inst(&mut self.res, context, inst));
        self.code_map.alloc(insts, len_regs)
    }

    fn compile_register(context: &CompileContext, register: IrRegister) -> ExecRegister {
        context.compile_register(register)
    }

    fn compile_register_slice(
        context: &CompileContext,
        slice: IrRegisterSlice,
    ) -> ExecRegisterSlice {
        match slice.first() {
            Some(first) => {
                let first = context.compile_register(first);
                let len = slice.len();
                ExecRegisterSlice::new(first, len)
            }
            None => ExecRegisterSlice::empty(),
        }
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

    fn compile_immediate(immediate: UntypedValue) -> UntypedValue {
        immediate
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
        res.provider_pool.alloc(providers)
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

    fn compile_inst_ri(
        context: &CompileContext,
        result: IrRegister,
        input: UntypedValue,
        make_op: fn(ExecRegister, UntypedValue) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let input = Self::compile_immediate(input);
        make_op(result, input)
    }

    fn compile_inst_rrr(
        context: &CompileContext,
        result: IrRegister,
        lhs: IrRegister,
        rhs: IrRegister,
        make_op: fn(ExecRegister, ExecRegister, ExecRegister) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let lhs = Self::compile_register(context, lhs);
        let rhs = Self::compile_register(context, rhs);
        make_op(result, lhs, rhs)
    }

    fn compile_inst_rri(
        context: &CompileContext,
        result: IrRegister,
        lhs: IrRegister,
        rhs: UntypedValue,
        make_op: fn(ExecRegister, ExecRegister, UntypedValue) -> ExecInstruction,
    ) -> ExecInstruction {
        let result = Self::compile_register(context, result);
        let lhs = Self::compile_register(context, lhs);
        let rhs = Self::compile_immediate(rhs);
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
        context: &CompileContext,
        ptr: IrRegister,
        offset: Offset,
        value: IrRegister,
        make_op: fn(ExecRegister, Offset, ExecRegister) -> ExecInstruction,
    ) -> ExecInstruction {
        let ptr = Self::compile_register(context, ptr);
        let value = Self::compile_register(context, value);
        make_op(ptr, offset, value)
    }

    fn compile_store_imm(
        context: &CompileContext,
        ptr: IrRegister,
        offset: Offset,
        value: UntypedValue,
        make_op: fn(ExecRegister, Offset, UntypedValue) -> ExecInstruction,
    ) -> ExecInstruction {
        let ptr = Self::compile_register(context, ptr);
        let value = Self::compile_immediate(value);
        make_op(ptr, offset, value)
    }

    fn compile_inst(
        res: &mut EngineResources,
        context: &CompileContext,
        inst: IrInstruction,
    ) -> ExecInstruction {
        match inst {
            Instruction::Trap { trap_code } => ExecInstruction::Trap { trap_code },
            Instruction::Br { target } => {
                let target = context.compile_label(target);
                ExecInstruction::Br { target }
            }
            Instruction::BrCopyMulti {
                target,
                results,
                returned,
            } => {
                let target = context.compile_label(target);
                let results = Self::compile_register_slice(context, results);
                let returned = Self::compile_provider_slice(res, context, returned);
                ExecInstruction::BrCopyMulti {
                    target,
                    results,
                    returned,
                }
            }
            Instruction::BrEqz { target, condition } => {
                let target = context.compile_label(target);
                let condition = Self::compile_register(context, condition);
                Instruction::BrEqz { target, condition }
            }
            Instruction::BrNez { target, condition } => {
                let target = context.compile_label(target);
                let condition = Self::compile_register(context, condition);
                Instruction::BrNez { target, condition }
            }
            Instruction::BrNezSingle {
                target,
                condition,
                result,
                returned,
            } => {
                let target = context.compile_label(target);
                let condition = Self::compile_register(context, condition);
                let result = Self::compile_register(context, result);
                let returned = Self::compile_provider(res, context, returned);
                Instruction::BrNezSingle {
                    target,
                    condition,
                    result,
                    returned,
                }
            }
            Instruction::BrNezMulti {
                target,
                condition,
                results,
                returned,
            } => {
                let target = context.compile_label(target);
                let condition = Self::compile_register(context, condition);
                let results = Self::compile_register_slice(context, results);
                let returned = Self::compile_provider_slice(res, context, returned);
                Instruction::BrNezMulti {
                    target,
                    condition,
                    results,
                    returned,
                }
            }
            Instruction::Return { result } => {
                let result = Self::compile_register(context, result);
                ExecInstruction::Return { result }
            }
            Instruction::ReturnImm { result } => {
                let result = Self::compile_immediate(result);
                ExecInstruction::ReturnImm { result }
            }
            Instruction::ReturnMulti { results } => {
                let results = Self::compile_provider_slice(res, context, results);
                ExecInstruction::ReturnMulti { results }
            }
            Instruction::ReturnNez { result, condition } => {
                let result = Self::compile_register(context, result);
                let condition = Self::compile_register(context, condition);
                ExecInstruction::ReturnNez { result, condition }
            }
            Instruction::ReturnNezImm { result, condition } => {
                let result = Self::compile_immediate(result);
                let condition = Self::compile_register(context, condition);
                ExecInstruction::ReturnNezImm { result, condition }
            }
            Instruction::ReturnNezMulti { results, condition } => {
                let results = Self::compile_provider_slice(res, context, results);
                let condition = Self::compile_register(context, condition);
                ExecInstruction::ReturnNezMulti { results, condition }
            }
            Instruction::BrTable { case, len_targets } => {
                let case = Self::compile_register(context, case);
                ExecInstruction::BrTable { case, len_targets }
            }

            Instruction::Call {
                func_idx,
                results,
                params,
            } => {
                let results = Self::compile_register_slice(context, results);
                let params = Self::compile_provider_slice(res, context, params);
                ExecInstruction::Call {
                    func_idx,
                    results,
                    params,
                }
            }
            Instruction::CallIndirect {
                func_type_idx,
                results,
                index,
                params,
            } => {
                let results = Self::compile_register_slice(context, results);
                let index = Self::compile_provider(res, context, index);
                let params = Self::compile_provider_slice(res, context, params);
                ExecInstruction::CallIndirect {
                    func_type_idx,
                    results,
                    index,
                    params,
                }
            }

            Instruction::Copy { result, input } => {
                Self::compile_inst_rr(context, result, input, unary_op!(Copy))
            }
            Instruction::CopyImm { result, input } => {
                Self::compile_inst_ri(context, result, input, unary_op!(CopyImm))
            }
            Instruction::CopyMany { results, inputs } => {
                let results = Self::compile_register_slice(context, results);
                let inputs = Self::compile_provider_slice(res, context, inputs);
                ExecInstruction::CopyMany { results, inputs }
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
                Self::compile_store(context, ptr, offset, value, store_op!(I32Store))
            }
            Instruction::I32StoreImm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I32StoreImm))
            }
            Instruction::I64Store { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(I64Store))
            }
            Instruction::I64StoreImm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I64StoreImm))
            }
            Instruction::F32Store { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(F32Store))
            }
            Instruction::F32StoreImm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(F32StoreImm))
            }
            Instruction::F64Store { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(F64Store))
            }
            Instruction::F64StoreImm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(F64StoreImm))
            }
            Instruction::I32Store8 { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(I32Store8))
            }
            Instruction::I32Store8Imm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I32Store8Imm))
            }
            Instruction::I32Store16 { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(I32Store16))
            }
            Instruction::I32Store16Imm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I32Store16Imm))
            }
            Instruction::I64Store8 { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(I64Store8))
            }
            Instruction::I64Store8Imm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I64Store8Imm))
            }
            Instruction::I64Store16 { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(I64Store16))
            }
            Instruction::I64Store16Imm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I64Store16Imm))
            }
            Instruction::I64Store32 { ptr, offset, value } => {
                Self::compile_store(context, ptr, offset, value, store_op!(I64Store32))
            }
            Instruction::I64Store32Imm { ptr, offset, value } => {
                Self::compile_store_imm(context, ptr, offset, value, store_op!(I64Store32Imm))
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
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Add))
            }
            Instruction::I32AddImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32AddImm))
            }
            Instruction::I32Sub { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Sub))
            }
            Instruction::I32SubImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32SubImm))
            }
            Instruction::I32Mul { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Mul))
            }
            Instruction::I32MulImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32MulImm))
            }
            Instruction::I32DivS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32DivS))
            }
            Instruction::I32DivSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32DivSImm))
            }
            Instruction::I32DivU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32DivU))
            }
            Instruction::I32DivUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32DivUImm))
            }
            Instruction::I32RemS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32RemS))
            }
            Instruction::I32RemSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32RemSImm))
            }
            Instruction::I32RemU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32RemU))
            }
            Instruction::I32RemUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32RemUImm))
            }
            Instruction::I32Shl { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Shl))
            }
            Instruction::I32ShlImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32ShlImm))
            }
            Instruction::I32ShrS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32ShrS))
            }
            Instruction::I32ShrSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32ShrSImm))
            }
            Instruction::I32ShrU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32ShrU))
            }
            Instruction::I32ShrUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32ShrUImm))
            }
            Instruction::I32Rotl { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Rotl))
            }
            Instruction::I32RotlImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32RotlImm))
            }
            Instruction::I32Rotr { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Rotr))
            }
            Instruction::I32RotrImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32RotrImm))
            }
            Instruction::I32And { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32And))
            }
            Instruction::I32AndImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32AndImm))
            }
            Instruction::I32Or { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Or))
            }
            Instruction::I32OrImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32OrImm))
            }
            Instruction::I32Xor { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Xor))
            }
            Instruction::I32XorImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32XorImm))
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
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Add))
            }
            Instruction::I64AddImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64AddImm))
            }
            Instruction::I64Sub { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Sub))
            }
            Instruction::I64SubImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64SubImm))
            }
            Instruction::I64Mul { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Mul))
            }
            Instruction::I64MulImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64MulImm))
            }
            Instruction::I64DivS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64DivS))
            }
            Instruction::I64DivSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64DivSImm))
            }
            Instruction::I64DivU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64DivU))
            }
            Instruction::I64DivUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64DivUImm))
            }
            Instruction::I64RemS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64RemS))
            }
            Instruction::I64RemSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64RemSImm))
            }
            Instruction::I64RemU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64RemU))
            }
            Instruction::I64RemUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64RemUImm))
            }
            Instruction::I64Shl { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Shl))
            }
            Instruction::I64ShlImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64ShlImm))
            }
            Instruction::I64ShrS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64ShrS))
            }
            Instruction::I64ShrSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64ShrSImm))
            }
            Instruction::I64ShrU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64ShrU))
            }
            Instruction::I64ShrUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64ShrUImm))
            }
            Instruction::I64Rotl { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Rotl))
            }
            Instruction::I64RotlImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64RotlImm))
            }
            Instruction::I64Rotr { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Rotr))
            }
            Instruction::I64RotrImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64RotrImm))
            }
            Instruction::I64And { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64And))
            }
            Instruction::I64AndImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64AndImm))
            }
            Instruction::I64Or { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Or))
            }
            Instruction::I64OrImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64OrImm))
            }
            Instruction::I64Xor { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Xor))
            }
            Instruction::I64XorImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64XorImm))
            }

            Instruction::F32Add { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Add))
            }
            Instruction::F32AddImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32AddImm))
            }
            Instruction::F32Sub { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Sub))
            }
            Instruction::F32SubImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32SubImm))
            }
            Instruction::F32Mul { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Mul))
            }
            Instruction::F32MulImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32MulImm))
            }
            Instruction::F32Div { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Div))
            }
            Instruction::F32DivImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32DivImm))
            }
            Instruction::F32Min { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Min))
            }
            Instruction::F32MinImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32MinImm))
            }
            Instruction::F32Max { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Max))
            }
            Instruction::F32MaxImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32MaxImm))
            }
            Instruction::F32Copysign { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Copysign))
            }
            Instruction::F32CopysignImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32CopysignImm))
            }

            Instruction::F64Add { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Add))
            }
            Instruction::F64AddImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64AddImm))
            }
            Instruction::F64Sub { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Sub))
            }
            Instruction::F64SubImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64SubImm))
            }
            Instruction::F64Mul { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Mul))
            }
            Instruction::F64MulImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64MulImm))
            }
            Instruction::F64Div { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Div))
            }
            Instruction::F64DivImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64DivImm))
            }
            Instruction::F64Min { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Min))
            }
            Instruction::F64MinImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64MinImm))
            }
            Instruction::F64Max { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Max))
            }
            Instruction::F64MaxImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64MaxImm))
            }
            Instruction::F64Copysign { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Copysign))
            }
            Instruction::F64CopysignImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64CopysignImm))
            }

            Instruction::I32Eq { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Eq))
            }
            Instruction::I32EqImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32EqImm))
            }
            Instruction::I32Ne { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32Ne))
            }
            Instruction::I32NeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32NeImm))
            }
            Instruction::I32LtS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32LtS))
            }
            Instruction::I32LtSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32LtSImm))
            }
            Instruction::I32LtU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32LtU))
            }
            Instruction::I32LtUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32LtUImm))
            }
            Instruction::I32LeS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32LeS))
            }
            Instruction::I32LeSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32LeSImm))
            }
            Instruction::I32LeU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32LeU))
            }
            Instruction::I32LeUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32LeUImm))
            }
            Instruction::I32GtS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32GtS))
            }
            Instruction::I32GtSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32GtSImm))
            }
            Instruction::I32GtU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32GtU))
            }
            Instruction::I32GtUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32GtUImm))
            }
            Instruction::I32GeS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32GeS))
            }
            Instruction::I32GeSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32GeSImm))
            }
            Instruction::I32GeU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I32GeU))
            }
            Instruction::I32GeUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I32GeUImm))
            }

            Instruction::I64Eq { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Eq))
            }
            Instruction::I64EqImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64EqImm))
            }
            Instruction::I64Ne { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64Ne))
            }
            Instruction::I64NeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64NeImm))
            }
            Instruction::I64LtS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64LtS))
            }
            Instruction::I64LtSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64LtSImm))
            }
            Instruction::I64LtU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64LtU))
            }
            Instruction::I64LtUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64LtUImm))
            }
            Instruction::I64LeS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64LeS))
            }
            Instruction::I64LeSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64LeSImm))
            }
            Instruction::I64LeU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64LeU))
            }
            Instruction::I64LeUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64LeUImm))
            }
            Instruction::I64GtS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64GtS))
            }
            Instruction::I64GtSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64GtSImm))
            }
            Instruction::I64GtU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64GtU))
            }
            Instruction::I64GtUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64GtUImm))
            }
            Instruction::I64GeS { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64GeS))
            }
            Instruction::I64GeSImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64GeSImm))
            }
            Instruction::I64GeU { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(I64GeU))
            }
            Instruction::I64GeUImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(I64GeUImm))
            }

            Instruction::F32Eq { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Eq))
            }
            Instruction::F32EqImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32EqImm))
            }
            Instruction::F32Ne { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Ne))
            }
            Instruction::F32NeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32NeImm))
            }
            Instruction::F32Lt { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Lt))
            }
            Instruction::F32LtImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32LtImm))
            }
            Instruction::F32Le { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Le))
            }
            Instruction::F32LeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32LeImm))
            }
            Instruction::F32Gt { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Gt))
            }
            Instruction::F32GtImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32GtImm))
            }
            Instruction::F32Ge { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F32Ge))
            }
            Instruction::F32GeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F32GeImm))
            }

            Instruction::F64Eq { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Eq))
            }
            Instruction::F64EqImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64EqImm))
            }
            Instruction::F64Ne { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Ne))
            }
            Instruction::F64NeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64NeImm))
            }
            Instruction::F64Lt { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Lt))
            }
            Instruction::F64LtImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64LtImm))
            }
            Instruction::F64Le { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Le))
            }
            Instruction::F64LeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64LeImm))
            }
            Instruction::F64Gt { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Gt))
            }
            Instruction::F64GtImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64GtImm))
            }
            Instruction::F64Ge { result, lhs, rhs } => {
                Self::compile_inst_rrr(context, result, lhs, rhs, binary_op!(F64Ge))
            }
            Instruction::F64GeImm { result, lhs, rhs } => {
                Self::compile_inst_rri(context, result, lhs, rhs, binary_op!(F64GeImm))
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
        }
    }
}
