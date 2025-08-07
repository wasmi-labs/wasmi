use crate::{Config, Engine, Error, Linker, Module, Store, TrapCode, WasmParams, WasmResults};
use core::{fmt::Debug, mem};

/// Wasmi execution test runner.
pub struct ExecutionTest<T> {
    /// The underlying engine that is tested.
    engine: Engine,
    /// The store data which is used for the calls.
    data: T,
    /// The compiled Wasm module that is used for the calls.
    module: Option<Module>,
}

impl Default for ExecutionTest<()> {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            data: (),
            module: None,
        }
    }
}

impl<T> ExecutionTest<T>
where
    T: Default + PartialEq + Eq + 'static,
{
    /// Creates a new [`ExecutionTest`] with default initialized data.
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            data: T::default(),
            module: None,
        }
    }

    /// Creates a new [`ExecutionTest`] with the given `config` and default initialized data.
    pub fn with_config(config: Config) -> Self {
        let engine = Engine::new(&config);
        let data = <T as Default>::default();
        Self {
            engine,
            data,
            module: None,
        }
    }

    /// Sets the Wasm source that this [`ExecutionTest`] uses for its calls.
    pub fn wasm(&mut self, wasm: &str) -> &mut Self {
        assert!(self.module.is_none(), "already provided Wasm input");
        let module = Module::new(&self.engine, wasm).unwrap();
        self.module = Some(module);
        self
    }

    /// Calls the function named `func_name` with `args` and returns the result.
    ///
    /// # Panics
    ///
    /// If no Wasm source was set before.
    ///
    /// # Errors
    ///
    /// - If instantiation fails.
    /// - If starting the instance fails.
    /// - If there is no function named `func_name`.
    /// - If the call fails or traps.
    pub fn call<Args, Results>(&mut self, func_name: &str, args: Args) -> Result<Results, Error>
    where
        Args: WasmParams,
        Results: WasmResults,
    {
        let Some(module) = &self.module else {
            panic!("need Wasm before calling")
        };
        let data = mem::take(&mut self.data);
        let mut store = <Store<T>>::new(&self.engine, data);
        let linker = Linker::new(&self.engine);
        let results = linker
            .instantiate_and_start(&mut store, module)?
            .get_typed_func::<Args, Results>(&store, func_name)?
            .call(&mut store, args)?;
        _ = mem::replace(&mut self.data, store.into_data());
        Ok(results)
    }

    /// Sets the store data to `data`.
    pub fn data(&mut self, data: T) -> &mut Self {
        self.data = data;
        self
    }

    /// Asserts that the store data is as `expected`.
    pub fn assert_data(&mut self, expected: impl AsRef<T>) -> &mut Self {
        assert!(&self.data == expected.as_ref());
        self
    }
}

/// Convenience trait to assert that some call results are as expected.
pub trait AssertResults {
    /// The expected call result type.
    type Val;

    /// Panics if the `Result<Args, _>::Ok` value is not `expected`.
    fn assert_results(&self, expected: Self::Val);
}

impl<T, E> AssertResults for Result<T, E>
where
    T: PartialEq + Debug,
    Self: Debug,
{
    type Val = T;

    fn assert_results(&self, expected: T) {
        let results = match self {
            Ok(results) => results,
            Err(_) => panic!("must have Ok value but found: {self:?}"),
        };
        assert_eq!(results, &expected)
    }
}

/// Convenience trait to assert that some call traps are as expected.
pub trait AssertTrap {
    /// Panics if the `Result<Args, _>::Err` value is not `expected`.
    fn assert_trap(&self, expected: TrapCode);
}

impl<T> AssertTrap for Result<T, Error>
where
    Self: Debug,
{
    fn assert_trap(&self, expected: TrapCode) {
        let error = match self {
            Ok(_) => panic!("must have Err value but found: {self:?}"),
            Err(error) => error,
        };
        let Some(trap) = error.as_trap_code() else {
            panic!("must have trap code Err but found: {error:?}")
        };
        assert_eq!(trap, expected)
    }
}
