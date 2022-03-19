use super::{
    super::{bytecode::Register as ExecRegister, ExecInstruction, Provider as ExecProvider},
    locals_registry::LocalsRegistry,
    providers::{Providers, Stacks},
    OpaqueInstruction,
    Provider as OpaqueProvider,
    ProviderSliceArena,
    Register as OpaqueRegister,
};
use crate::Engine;
use wasmi_core::{TrapCode, Value};

/// Creates a closure taking 3 parameters and constructing a `wasmi` instruction.
macro_rules! make_op {
    ( $name:ident ) => {{
        |result, lhs, rhs| ExecInstruction::$name { result, lhs, rhs }
    }};
}

/// Creates a closure taking 2 parameters and constructing a `wasmi` instruction.
macro_rules! make_op2 {
    ( $name:ident ) => {{
        |result, input| ExecInstruction::$name { result, input }
    }};
}

pub struct CompileContext<'a> {
    engine: &'a Engine,
    reg_slices: &'a ProviderSliceArena,
    providers: &'a Providers,
}

impl OpaqueInstruction {
    fn compile_rrp<F>(
        self,
        ctx: &CompileContext,
        result: OpaqueRegister,
        lhs: OpaqueRegister,
        rhs: OpaqueProvider,
        make_op: F,
    ) -> ExecInstruction
    where
        F: FnOnce(ExecRegister, ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let result = ctx.providers.compile_register(result);
        let lhs = ctx.providers.compile_register(lhs);
        let rhs = ctx.providers.compile_provider(ctx.engine, rhs);
        make_op(result, lhs, rhs)
    }

    fn compile_rr<F>(
        self,
        ctx: &CompileContext,
        result: OpaqueRegister,
        input: OpaqueRegister,
        make_op: F,
    ) -> ExecInstruction
    where
        F: FnOnce(ExecRegister, ExecRegister) -> ExecInstruction,
    {
        let result = ctx.providers.compile_register(result);
        let input = ctx.providers.compile_register(input);
        make_op(result, input)
    }

    fn compile_rp<F>(
        self,
        ctx: &CompileContext,
        result: OpaqueRegister,
        input: OpaqueProvider,
        make_op: F,
    ) -> ExecInstruction
    where
        F: FnOnce(ExecRegister, ExecProvider) -> ExecInstruction,
    {
        let result = ctx.providers.compile_register(result);
        let input = ctx.providers.compile_provider(ctx.engine, input);
        make_op(result, input)
    }

    pub fn compile(
        self,
        engine: &Engine,
        reg_slices: &ProviderSliceArena,
        providers: &Providers,
    ) -> ExecInstruction {
        let ctx = CompileContext {
            engine,
            reg_slices,
            providers,
        };
        match self {
            Self::Trap { trap_code } => ExecInstruction::Trap { trap_code },
            Self::Return { results } => {
                let providers = reg_slices
                    .resolve(results)
                    .iter()
                    .copied()
                    .map(|provider| providers.compile_provider(engine, provider))
                    .collect::<Vec<_>>(); // TODO: replace collect
                let dedup = engine.alloc_provider_slice(providers);
                ExecInstruction::Return { results: dedup }
            }

            Self::Copy { result, input } => self.compile_rp(&ctx, result, input, make_op2!(Copy)),

            Self::I32Clz { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32Clz))
            }
            Self::I32Ctz { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32Ctz))
            }
            Self::I32Popcnt { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32Popcnt))
            }

            Self::I32Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Add))
            }
            Self::I32Sub { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Sub))
            }
            Self::I32Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Mul))
            }
            Self::I32DivS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32DivS))
            }
            Self::I32DivU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32DivU))
            }
            Self::I32RemS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32RemS))
            }
            Self::I32RemU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32RemU))
            }
            Self::I32Shl { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Shl))
            }
            Self::I32ShrS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32ShrS))
            }
            Self::I32ShrU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32ShrU))
            }
            Self::I32Rotl { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Rotl))
            }
            Self::I32Rotr { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Rotr))
            }
            Self::I32And { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32And))
            }
            Self::I32Or { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Or))
            }
            Self::I32Xor { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Xor))
            }

            Self::I64Clz { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64Clz))
            }
            Self::I64Ctz { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64Ctz))
            }
            Self::I64Popcnt { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64Popcnt))
            }

            Self::I64Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Add))
            }
            Self::I64Sub { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Sub))
            }
            Self::I64Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Mul))
            }
            Self::I64DivS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64DivS))
            }
            Self::I64DivU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64DivU))
            }
            Self::I64RemS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64RemS))
            }
            Self::I64RemU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64RemU))
            }
            Self::I64Shl { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Shl))
            }
            Self::I64ShrS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64ShrS))
            }
            Self::I64ShrU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64ShrU))
            }
            Self::I64Rotl { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Rotl))
            }
            Self::I64Rotr { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Rotr))
            }
            Self::I64And { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64And))
            }
            Self::I64Or { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Or))
            }
            Self::I64Xor { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Xor))
            }

            Self::F32Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Add))
            }
            Self::F32Sub { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Sub))
            }
            Self::F32Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Mul))
            }
            Self::F32Div { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Div))
            }
            Self::F32Min { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Min))
            }
            Self::F32Max { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Max))
            }
            Self::F32Copysign { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Copysign))
            }

            Self::F64Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Add))
            }
            Self::F64Sub { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Sub))
            }
            Self::F64Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Mul))
            }
            Self::F64Div { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Div))
            }
            Self::F64Min { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Min))
            }
            Self::F64Max { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Max))
            }
            Self::F64Copysign { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Copysign))
            }

            Self::I32Eq { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Eq))
            }
            Self::I32Ne { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Ne))
            }
            Self::I32LtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32LtS))
            }
            Self::I32LtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32LtU))
            }
            Self::I32LeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32LeS))
            }
            Self::I32LeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32LeU))
            }
            Self::I32GtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32GtS))
            }
            Self::I32GtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32GtU))
            }
            Self::I32GeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32GeS))
            }
            Self::I32GeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32GeU))
            }

            Self::I64Eq { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Eq))
            }
            Self::I64Ne { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Ne))
            }
            Self::I64LtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64LtS))
            }
            Self::I64LtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64LtU))
            }
            Self::I64LeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64LeS))
            }
            Self::I64LeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64LeU))
            }
            Self::I64GtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64GtS))
            }
            Self::I64GtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64GtU))
            }
            Self::I64GeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64GeS))
            }
            Self::I64GeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64GeU))
            }

            Self::F32Eq { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Eq))
            }
            Self::F32Ne { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Ne))
            }
            Self::F32Lt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Lt))
            }
            Self::F32Le { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Le))
            }
            Self::F32Gt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Gt))
            }
            Self::F32Ge { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Ge))
            }

            Self::F64Eq { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Eq))
            }
            Self::F64Ne { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Ne))
            }
            Self::F64Lt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Lt))
            }
            Self::F64Le { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Le))
            }
            Self::F64Gt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Gt))
            }
            Self::F64Ge { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Ge))
            }

            Self::F32Abs { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Abs))
            }
            Self::F32Neg { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Neg))
            }
            Self::F32Ceil { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Ceil))
            }
            Self::F32Floor { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Floor))
            }
            Self::F32Trunc { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Trunc))
            }
            Self::F32Nearest { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Nearest))
            }
            Self::F32Sqrt { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32Sqrt))
            }

            Self::F64Abs { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Abs))
            }
            Self::F64Neg { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Neg))
            }
            Self::F64Ceil { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Ceil))
            }
            Self::F64Floor { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Floor))
            }
            Self::F64Trunc { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Trunc))
            }
            Self::F64Nearest { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Nearest))
            }
            Self::F64Sqrt { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64Sqrt))
            }

            Self::I32WrapI64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32WrapI64))
            }
            Self::I32TruncSF32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncSF32))
            }
            Self::I32TruncUF32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncUF32))
            }
            Self::I32TruncSF64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncSF64))
            }
            Self::I32TruncUF64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncUF64))
            }
            Self::I64ExtendSI32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64ExtendSI32))
            }
            Self::I64ExtendUI32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64ExtendUI32))
            }
            Self::I64TruncSF32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncSF32))
            }
            Self::I64TruncUF32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncUF32))
            }
            Self::I64TruncSF64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncSF64))
            }
            Self::I64TruncUF64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncUF64))
            }
            Self::F32ConvertSI32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32ConvertSI32))
            }
            Self::F32ConvertUI32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32ConvertUI32))
            }
            Self::F32ConvertSI64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32ConvertSI64))
            }
            Self::F32ConvertUI64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32ConvertUI64))
            }
            Self::F32DemoteF64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F32DemoteF64))
            }
            Self::F64ConvertSI32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64ConvertSI32))
            }
            Self::F64ConvertUI32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64ConvertUI32))
            }
            Self::F64ConvertSI64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64ConvertSI64))
            }
            Self::F64ConvertUI64 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64ConvertUI64))
            }
            Self::F64PromoteF32 { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(F64PromoteF32))
            }

            Self::I32Extend8S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32Extend8S))
            }
            Self::I32Extend16S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32Extend16S))
            }
            Self::I64Extend8S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64Extend8S))
            }
            Self::I64Extend16S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64Extend16S))
            }
            Self::I64Extend32S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64Extend32S))
            }

            Self::I32TruncSatF32S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncSatF32S))
            }
            Self::I32TruncSatF32U { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncSatF32U))
            }
            Self::I32TruncSatF64S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncSatF64S))
            }
            Self::I32TruncSatF64U { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I32TruncSatF64U))
            }
            Self::I64TruncSatF32S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncSatF32S))
            }
            Self::I64TruncSatF32U { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncSatF32U))
            }
            Self::I64TruncSatF64S { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncSatF64S))
            }
            Self::I64TruncSatF64U { result, input } => {
                self.compile_rr(&ctx, result, input, make_op2!(I64TruncSatF64U))
            }

            _ => todo!(),
        }
    }
}
