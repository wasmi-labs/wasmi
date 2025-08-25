use crate::{
    engine::EngineFunc,
    ir::{index, Instruction, Reg, RegSpan, VisitRegs},
    module::ModuleHeader,
    Engine,
    Error,
    FuncType,
};

/// Extension trait for [`Instruction`] to conditionally relink result [`Reg`]s.
pub trait RelinkResult {
    /// Relinks the result [`Reg`] of `self` to `new_result` if its current `result` [`Reg`] equals `old_result`.
    ///
    /// # Note (Return Value)
    ///
    /// - `Ok(true)`: the result has been relinked
    /// - `Ok(false)`: the result has _not_ been relinked
    /// - `Err(_)`: translation error
    fn relink_result(
        &mut self,
        module: &ModuleHeader,
        new_result: Reg,
        old_result: Reg,
    ) -> Result<bool, Error>;
}

/// Visitor to implement [`RelinkResult`] for [`Instruction`].
struct Visitor {
    /// The new [`Reg`] that replaces the `old_result` [`Reg`].
    new_result: Reg,
    /// The old result [`Reg`].
    old_result: Reg,
    /// The return value of the visitation.
    ///
    /// For more information see docs of [`RelinkResult`].
    replaced: Result<bool, Error>,
}

impl Visitor {
    /// Creates a new [`Visitor`].
    fn new(new_result: Reg, old_result: Reg) -> Self {
        Self {
            new_result,
            old_result,
            replaced: Ok(false),
        }
    }
}

impl VisitRegs for Visitor {
    #[inline]
    fn visit_result_reg(&mut self, reg: &mut Reg) {
        if self.replaced.is_err() {
            return;
        }
        self.replaced = relink_simple(reg, self.new_result, self.old_result);
    }

    #[inline(always)]
    fn visit_result_regs(&mut self, _reg: &mut RegSpan, _len: Option<u16>) {}
}

impl RelinkResult for Instruction {
    fn relink_result(
        &mut self,
        module: &ModuleHeader,
        new_result: Reg,
        old_result: Reg,
    ) -> Result<bool, Error> {
        // Note: for call instructions we have to infer with special handling if they return
        //       a single value which allows us to relink the single result register.
        match self {
            Self::CallInternal0 { results, func } | Self::CallInternal { results, func } => {
                relink_call_internal(
                    results,
                    EngineFunc::from(*func),
                    module,
                    new_result,
                    old_result,
                )
            }
            Self::CallImported0 { results, func } | Self::CallImported { results, func } => {
                relink_call_imported(results, *func, module, new_result, old_result)
            }
            Self::CallIndirect0 { results, func_type }
            | Self::CallIndirect0Imm16 { results, func_type }
            | Self::CallIndirect { results, func_type }
            | Self::CallIndirectImm16 { results, func_type } => {
                relink_call_indirect(results, *func_type, module, new_result, old_result)
            }
            instr => {
                // Fallback: only relink results of instructions with statically known single results.
                let mut visitor = Visitor::new(new_result, old_result);
                instr.visit_regs(&mut visitor);
                visitor.replaced
            }
        }
    }
}

fn relink_simple(result: &mut Reg, new_result: Reg, old_result: Reg) -> Result<bool, Error> {
    if *result != old_result {
        // Note: This is a safeguard to prevent miscompilations.
        return Ok(false);
    }
    debug_assert_ne!(*result, new_result);
    *result = new_result;
    Ok(true)
}

fn get_engine(module: &ModuleHeader) -> Engine {
    module.engine().upgrade().unwrap_or_else(|| {
        panic!(
            "engine for result relinking does not exist: {:?}",
            module.engine()
        )
    })
}

fn relink_call_internal(
    results: &mut RegSpan,
    func: EngineFunc,
    module: &ModuleHeader,
    new_result: Reg,
    old_result: Reg,
) -> Result<bool, Error> {
    let Some(module_func) = module.get_func_index(func) else {
        panic!("missing module func for compiled func: {func:?}")
    };
    let engine = get_engine(module);
    let func_type = module.get_type_of_func(module_func);
    let len_results = engine.resolve_func_type(func_type, FuncType::len_results);
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

fn relink_call_imported(
    results: &mut RegSpan,
    func: index::Func,
    module: &ModuleHeader,
    new_result: Reg,
    old_result: Reg,
) -> Result<bool, Error> {
    let engine = get_engine(module);
    let func_idx = u32::from(func).into();
    let func_type = module.get_type_of_func(func_idx);
    let len_results = engine.resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

fn relink_call_indirect(
    results: &mut RegSpan,
    func_type: index::FuncType,
    module: &ModuleHeader,
    new_result: Reg,
    old_result: Reg,
) -> Result<bool, Error> {
    let engine = get_engine(module);
    let func_type_idx = u32::from(func_type).into();
    let func_type = module.get_func_type(func_type_idx);
    let len_results = engine.resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}
