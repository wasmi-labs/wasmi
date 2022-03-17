use wasmi_core::Value;

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
            Self::Unreachable => ExecInstruction::Unreachable,
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
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32Add { result, lhs, rhs }
                })
            }

            Self::I32Eq { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32Eq { result, lhs, rhs }
                })
            }
            Self::I32Ne { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32Ne { result, lhs, rhs }
                })
            }
            Self::I32LtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32LtS { result, lhs, rhs }
                })
            }
            Self::I32LtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32LtU { result, lhs, rhs }
                })
            }
            Self::I32LeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32LeS { result, lhs, rhs }
                })
            }
            Self::I32LeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32LeU { result, lhs, rhs }
                })
            }
            Self::I32GtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32GtS { result, lhs, rhs }
                })
            }
            Self::I32GtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32GtU { result, lhs, rhs }
                })
            }
            Self::I32GeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32GeS { result, lhs, rhs }
                })
            }
            Self::I32GeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I32GeU { result, lhs, rhs }
                })
            }

            Self::I64Eq { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64Eq { result, lhs, rhs }
                })
            }
            Self::I64Ne { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64Ne { result, lhs, rhs }
                })
            }
            Self::I64LtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64LtS { result, lhs, rhs }
                })
            }
            Self::I64LtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64LtU { result, lhs, rhs }
                })
            }
            Self::I64LeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64LeS { result, lhs, rhs }
                })
            }
            Self::I64LeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64LeU { result, lhs, rhs }
                })
            }
            Self::I64GtS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64GtS { result, lhs, rhs }
                })
            }
            Self::I64GtU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64GtU { result, lhs, rhs }
                })
            }
            Self::I64GeS { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64GeS { result, lhs, rhs }
                })
            }
            Self::I64GeU { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::I64GeU { result, lhs, rhs }
                })
            }

            Self::F32Lt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F32Lt { result, lhs, rhs }
                })
            }
            Self::F32Le { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F32Le { result, lhs, rhs }
                })
            }
            Self::F32Gt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F32Gt { result, lhs, rhs }
                })
            }
            Self::F32Ge { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F32Ge { result, lhs, rhs }
                })
            }

            Self::F64Lt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F64Lt { result, lhs, rhs }
                })
            }
            Self::F64Le { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F64Le { result, lhs, rhs }
                })
            }
            Self::F64Gt { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F64Gt { result, lhs, rhs }
                })
            }
            Self::F64Ge { result, lhs, rhs } => {
                self.compile_rrp(&ctx, result, lhs, rhs, |result, lhs, rhs| {
                    ExecInstruction::F64Ge { result, lhs, rhs }
                })
            }

            _ => todo!(),
        }
    }
}
