use crate::{
    engine::EngineFunc,
    ir::{index, Op, Slot, SlotSpan},
    module::ModuleHeader,
    Engine,
    Error,
    FuncType,
};

/// Extension trait for [`Op`] to conditionally relink result [`Slot`]s.
pub trait RelinkResult {
    /// Relinks the result [`Slot`] of `self` to `new_result` if its current `result` [`Slot`] equals `old_result`.
    ///
    /// # Note (Return Value)
    ///
    /// - `Ok(true)`: the result has been relinked
    /// - `Ok(false)`: the result has _not_ been relinked
    /// - `Err(_)`: translation error
    fn relink_result(
        &mut self,
        module: &ModuleHeader,
        new_result: Slot,
        old_result: Slot,
    ) -> Result<bool, Error>;
}

/// Visitor to implement [`RelinkResult`] for [`Op`].
struct Visitor {
    /// The new [`Slot`] that replaces the `old_result` [`Slot`].
    new_result: Slot,
    /// The old result [`Slot`].
    old_result: Slot,
    /// The return value of the visitation.
    ///
    /// For more information see docs of [`RelinkResult`].
    replaced: Result<bool, Error>,
}

impl Visitor {
    /// Creates a new [`Visitor`].
    fn new(new_result: Slot, old_result: Slot) -> Self {
        Self {
            new_result,
            old_result,
            replaced: Ok(false),
        }
    }
}

impl RelinkResult for Op {
    fn relink_result(
        &mut self,
        module: &ModuleHeader,
        new_result: Slot,
        old_result: Slot,
    ) -> Result<bool, Error> {
        let mut visitor = Visitor::new(new_result, old_result);
        self.visit_results(&mut visitor);
        visitor.replaced
    }
}

fn relink_simple(result: &mut Slot, new_result: Slot, old_result: Slot) -> Result<bool, Error> {
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
    results: &mut SlotSpan,
    func: EngineFunc,
    module: &ModuleHeader,
    new_result: Slot,
    old_result: Slot,
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
    results: &mut SlotSpan,
    func: index::Func,
    module: &ModuleHeader,
    new_result: Slot,
    old_result: Slot,
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
    results: &mut SlotSpan,
    func_type: index::FuncType,
    module: &ModuleHeader,
    new_result: Slot,
    old_result: Slot,
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
