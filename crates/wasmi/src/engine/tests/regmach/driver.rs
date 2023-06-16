use super::{create_module, wat2wasm};
use crate::{
    engine::{bytecode2::Instruction, const_pool::ConstRef, CompiledFunc, DedupFuncType},
    Config,
    Engine,
    EngineBackend,
    Module,
};
use wasmi_core::UntypedValue;

/// A test driver for translation tests.
#[derive(Debug)]
pub struct TranslationTest {
    /// The input Wasm bytes.
    wasm: Vec<u8>,
    /// The config under which the engine is tested.
    config: Config,
    /// The expected functions and their instructions.
    expected_funcs: Vec<ExpectedFunc>,
    /// The expected constant values in the constant pool.
    expected_consts: Vec<ExpectedConst>,
}

/// An entry for an expected function body stored in the engine under test.
#[derive(Debug)]
pub struct ExpectedFunc {
    /// The instructions of the expected function.
    instrs: Vec<Instruction>,
}

impl ExpectedFunc {
    /// Create a new [`ExpectedFunc`] with the given `wasmi` bytecode [`Instruction`] sequence.
    pub fn new<I>(instrs: I) -> Self
    where
        I: IntoIterator<Item = Instruction>,
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        Self {
            instrs: instrs.into_iter().collect(),
        }
    }

    /// Returns the expected [`Instruction`] sequence of the [`ExpectedFunc`] as slice.
    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs
    }
}

/// An entry for an expected constant value stored in the pool of constant values.
#[derive(Debug)]
pub struct ExpectedConst {
    /// The [`ConstRef`] identifying the constant value under test.
    pub cref: ConstRef,
    /// The expected value of the constant value under test.
    pub value: UntypedValue,
}

impl ExpectedConst {
    /// Create a new [`ExpectedConst`] for `cref` expecting `value`.
    pub fn new(cref: ConstRef, value: UntypedValue) -> Self {
        Self { cref, value }
    }
}

impl TranslationTest {
    /// Creates a new [`TranslationTest`] for the given Webassembly `bytes`.
    pub fn new(bytes: impl AsRef<[u8]>) -> Self {
        let config = {
            let mut cfg = Config::default();
            cfg.set_engine_backend(EngineBackend::RegisterMachine);
            cfg
        };
        Self {
            wasm: bytes.as_ref().to_vec(),
            config,
            expected_funcs: Vec::new(),
            expected_consts: Vec::new(),
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

    /// Returns all [`ExpectedConst`] for the test case.
    fn expected_consts(&self) -> &[ExpectedConst] {
        &self.expected_consts
    }

    pub fn expect_func<I>(&mut self, instrs: I) -> &mut Self
    where
        I: IntoIterator<Item = Instruction>,
    {
        self.expected_funcs.push(ExpectedFunc {
            instrs: instrs.into_iter().collect(),
        });
        self
    }

    pub fn expect_const<T>(&mut self, cref: ConstRef, value: T) -> &mut Self
    where
        T: Into<UntypedValue>,
    {
        self.expected_consts.push(ExpectedConst {
            cref,
            value: value.into(),
        });
        self
    }

    pub fn run(&self) {
        let module = create_module(self.config(), self.wasm());
        let engine = module.engine();
        self.assert_funcs(engine, &module);
        self.assert_consts(engine);
    }

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
            self.assert_func(engine, func_type, compiled_func, expected_func);
        }
    }

    fn assert_func(
        &self,
        engine: &Engine,
        func_type: DedupFuncType,
        compiled_func: CompiledFunc,
        expected_func: &ExpectedFunc,
    ) {
        let expected_instrs = expected_func.instrs();
        let len_expected = expected_instrs.len();
        let func_type = engine.resolve_func_type(&func_type, Clone::clone);
        for (index, actual, expected) in
            expected_instrs
                .into_iter()
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

    fn assert_consts(&self, engine: &Engine) {
        for expected_const in self.expected_consts() {
            let cref = expected_const.cref;
            let actual_value = engine
                .get_const(cref)
                .unwrap_or_else(|| panic!("missing entry for expected constant value: {cref:?}"));
            let expected_value = expected_const.value;
            assert_eq!(
                actual_value, expected_value,
                "{cref:?} (= {actual_value:?}) fails to match expected value {expected_value:?}",
            );
        }
    }
}
