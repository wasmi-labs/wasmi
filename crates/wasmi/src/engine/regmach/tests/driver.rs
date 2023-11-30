use super::create_module;
use crate::{
    engine::{regmach::bytecode::Instruction, CompiledFunc, DedupFuncType},
    Config,
    Engine,
    Module,
};
use core::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use wasmi_core::UntypedValue;

/// A test driver for translation tests.
#[derive(Debug)]
pub struct TranslationTest {
    /// The input Wasm bytes.
    wasm: Box<[u8]>,
    /// The config under which the engine is tested.
    config: Config,
    /// The expected functions and their instructions.
    expected_funcs: Vec<ExpectedFunc>,
    /// Is `true` if the [`TranslationTest`] has been run at least once.
    has_run: AtomicBool,
}

impl Drop for TranslationTest {
    fn drop(&mut self) {
        if !self.has_run.load(Ordering::SeqCst) {
            panic!("TranslationTest did not run at least once. This is probably a bug!")
        }
    }
}

/// An entry for an expected function body stored in the engine under test.
#[derive(Debug, Clone)]
pub struct ExpectedFunc {
    /// The instructions of the expected function.
    instrs: Vec<Instruction>,
    /// The function local constant values.
    consts: Vec<UntypedValue>,
}

impl ExpectedFunc {
    /// Create a new [`ExpectedFunc`] with the given `wasmi` bytecode [`Instruction`] sequence.
    pub fn new<I>(instrs: I) -> Self
    where
        I: IntoIterator<Item = Instruction>,
    {
        let instrs: Vec<_> = instrs.into_iter().collect();
        assert!(
            !instrs.is_empty(),
            "an expected function must have instructions"
        );
        Self {
            instrs,
            consts: Vec::new(),
        }
    }

    /// Add expected function local constant values to this [`ExpectedFunc`].
    ///
    /// # Note
    ///
    /// The function local constant values are in the order of their expected
    /// and deduplicated allocations during the translation phase.
    ///
    /// # Panics
    ///
    /// - If the `consts` iterator yields no values.
    /// - If this [`ExpectedFunc`] already expects some function local constant values.
    pub fn consts<I, T>(mut self, consts: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<UntypedValue>,
    {
        assert!(
            self.expected_consts().is_empty(),
            "tried to call `expect_consts` twice"
        );
        self.consts.extend(consts.into_iter().map(Into::into));
        assert!(
            !self.expected_consts().is_empty(),
            "called `expect_consts` with empty set"
        );
        self
    }

    /// Returns the expected [`Instruction`] sequence of the [`ExpectedFunc`] as slice.
    fn expected_instrs(&self) -> &[Instruction] {
        &self.instrs
    }

    /// Returns the expected function local constant values of the [`ExpectedFunc`] as slice.
    fn expected_consts(&self) -> &[UntypedValue] {
        &self.consts
    }

    /// Asserts that properties of the [`ExpectedFunc`] have been translated as expected.
    fn assert_func(&self, engine: &Engine, func_type: DedupFuncType, compiled_func: CompiledFunc) {
        self.assert_instrs(engine, compiled_func, func_type);
        self.assert_consts(engine, compiled_func);
    }

    /// Asserts that the instructions of the [`ExpectedFunc`] have been translated as expected.
    fn assert_instrs(
        &self,
        engine: &Engine,
        compiled_func: CompiledFunc,
        func_type: DedupFuncType,
    ) {
        let expected_instrs = self.expected_instrs();
        let len_expected = expected_instrs.len();
        let func_type = engine.resolve_func_type(&func_type, Clone::clone);
        for (index, actual, expected) in
            expected_instrs
                .iter()
                .copied()
                .enumerate()
                .map(|(index, expected_instr)| {
                    let actual_instr =
                        engine
                            .resolve_instr_2(compiled_func, index)
                            .unwrap_or_else(|| {
                                panic!("missing instruction at index {index} for {compiled_func:?} ({func_type:?})")
                            });
                    (index, actual_instr, expected_instr)
                })
        {
            assert!(
                actual == expected,
                "instruction mismatch at index {index} for {compiled_func:?} ({func_type:?})\n    \
                    - expected: {expected:?}\n    \
                    - found: {actual:?}",
            );
        }
        if let Some(unexpected) = engine.resolve_instr_2(compiled_func, len_expected) {
            panic!("unexpected instruction at index {len_expected}: {unexpected:?}");
        }
    }

    /// Asserts that the function local constant values of the [`ExpectedFunc`] have been translated as expected.
    fn assert_consts(&self, engine: &Engine, func: CompiledFunc) {
        let expected_consts = self.expected_consts();
        for (index, expected_value) in expected_consts.iter().copied().enumerate() {
            let actual_value = engine.get_func_const_2(func, index).unwrap_or_else(|| {
                panic!("missing function local constant value of for {func:?} at index {index}")
            });
            assert_eq!(
                actual_value, expected_value,
                "function local constant value mismatch for {func:?} at index {index}"
            );
        }
        // Check that there are not more function local constants than we already expected.
        let len_consts = expected_consts.len();
        if let Some(unexpected) = engine.get_func_const_2(func, len_consts) {
            panic!("unexpected function local constant value (= {unexpected:?}) for {func:?} at index {len_consts}")
        }
    }
}

impl TranslationTest {
    /// Creates a new [`TranslationTest`] for the given Webassembly `bytes`.
    #[must_use]
    pub fn new(bytes: impl AsRef<[u8]>) -> Self {
        let config = {
            let mut cfg = Config::default();
            cfg.wasm_tail_call(true);
            cfg
        };
        Self {
            wasm: bytes.as_ref().into(),
            config,
            expected_funcs: Vec::new(),
            has_run: AtomicBool::from(false),
        }
    }

    /// Returns the [`Config`] used for the test case.
    fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the WebAssembly bytes used for the test case.
    fn wasm(&self) -> &[u8] {
        &self.wasm
    }

    /// Returns the sequence of [`ExpectedFunc`] for the test case.
    fn expected_funcs(&self) -> &[ExpectedFunc] {
        &self.expected_funcs
    }

    /// Add an expected function with its instructions.
    ///
    /// # Note
    ///
    /// This is a convenience method for [`TranslationTest::expect_func_ext`].
    pub fn expect_func_instrs<I>(&mut self, instrs: I) -> &mut Self
    where
        I: IntoIterator<Item = Instruction>,
    {
        self.expect_func(ExpectedFunc::new(instrs))
    }

    /// Add an [`ExpectedFunc`].
    pub fn expect_func(&mut self, func: ExpectedFunc) -> &mut Self {
        self.expected_funcs.push(func);
        self
    }

    /// Runs the [`TranslationTest`].
    ///
    /// # Panics
    ///
    /// If the translation test was not successful.
    pub fn run(&self) {
        self.has_run.store(true, Ordering::SeqCst);
        let module = create_module(self.config(), self.wasm());
        let engine = module.engine();
        self.assert_funcs(engine, &module);
    }

    /// Asserts that all expected functions of the translated Wasm module are as expected.
    fn assert_funcs(&self, engine: &Engine, module: &Module) {
        {
            let len_compiled = module.internal_funcs().len();
            let len_expected = self.expected_funcs().len();
            assert_eq!(
                len_compiled,
                len_expected,
                "number of compiled functions (={len_compiled}) do not match the expected amount (= {len_expected})",
            );
        }
        for ((func_type, compiled_func), expected_func) in
            module.internal_funcs().zip(self.expected_funcs())
        {
            expected_func.assert_func(engine, func_type, compiled_func);
        }
    }
}
