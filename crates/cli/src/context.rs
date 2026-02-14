use anyhow::{Error, anyhow};
use std::{fs, path::Path};
use wasmi::{CompilationMode, Config, ExternType, Func, FuncType, Instance, Module, Store};

#[cfg(feature = "wasi")]
pub type StoreContext = wasmi_wasi::WasiCtx;

#[cfg(not(feature = "wasi"))]
pub type StoreContext = ();

/// The [`Context`] for the Wasmi CLI application.
///
/// This simply stores all the necessary data.
pub struct Context {
    /// The given Wasm module.
    module: Module,
    /// The used Wasm store.
    store: Store<StoreContext>,
    /// The Wasm module instance to operate on.
    instance: Instance,
}

impl Context {
    /// Creates a new [`Context`].
    ///
    /// # Errors
    ///
    /// - If parsing, validating, compiling or instantiating the Wasm module failed.
    /// - If adding WASI definitions to the linker failed.
    pub fn new(
        wasm_file: &Path,
        store_ctx: StoreContext,
        fuel: Option<u64>,
        compilation_mode: CompilationMode,
    ) -> Result<Self, Error> {
        let mut config = Config::default();
        if fuel.is_some() {
            config.consume_fuel(true);
        }
        config.compilation_mode(compilation_mode);
        config.wasm_custom_page_sizes(true);
        config.wasm_wide_arithmetic(true);
        let engine = wasmi::Engine::new(&config);
        let wasm =
            fs::read(wasm_file).map_err(|_| anyhow!("failed to read Wasm file {wasm_file:?}"))?;
        let module = wasmi::Module::new(&engine, wasm).map_err(|error| {
            anyhow!("failed to parse and validate Wasm module {wasm_file:?}: {error}")
        })?;
        let mut store = wasmi::Store::new(&engine, store_ctx);
        if let Some(fuel) = fuel {
            store.set_fuel(fuel).unwrap_or_else(|error| {
                panic!("error: fuel metering is enabled but encountered: {error}")
            });
        }
        #[cfg_attr(not(feature = "wasi"), allow(unused_mut))]
        let mut linker = <wasmi::Linker<StoreContext>>::new(&engine);
        #[cfg(feature = "wasi")]
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx)
            .map_err(|error| anyhow!("failed to add WASI definitions to the linker: {error}"))?;
        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|error| anyhow!("failed to instantiate and start the Wasm module: {error}"))?;
        Ok(Self {
            module,
            store,
            instance,
        })
    }

    /// Returns the exported named functions of the Wasm [`Module`].
    ///
    /// [`Module`]: wasmi::Module
    pub fn exported_funcs(&self) -> impl Iterator<Item = (&str, FuncType)> {
        self.module.exports().filter_map(|export| {
            let name = export.name();
            match export.ty() {
                ExternType::Func(func_type) => Some((name, func_type.clone())),
                _ => None,
            }
        })
    }

    /// Returns a shared reference to the [`Store`] of the [`Context`].
    pub fn store(&self) -> &Store<StoreContext> {
        &self.store
    }

    /// Returns an exclusive reference to the [`Store`] of the [`Context`].
    pub fn store_mut(&mut self) -> &mut Store<StoreContext> {
        &mut self.store
    }

    /// Returns the exported function named `name` if any.
    pub fn get_func(&self, name: &str) -> Result<Func, Error> {
        self.instance
            .get_func(&self.store, name)
            .ok_or_else(|| anyhow!("failed to find function named {name:?} in the Wasm module"))
    }
}
