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

            Self::I32Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Add))
            }
            Self::I32Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I32Mul))
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

            Self::I64Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Add))
            }
            Self::I64Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(I64Mul))
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
            Self::F32Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Mul))
            }
            Self::F32Min { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Min))
            }
            Self::F32Max { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F32Max))
            }

            Self::F64Add { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Add))
            }
            Self::F64Mul { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Mul))
            }
            Self::F64Min { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Min))
            }
            Self::F64Max { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, make_op!(F64Max))
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

            _ => todo!(),
        }
    }
}
