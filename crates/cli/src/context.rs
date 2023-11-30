use crate::utils;
use anyhow::{anyhow, Error};
use std::path::Path;
use wasmi::{Config, ExternType, Func, FuncType, Instance, Module, Store};
use wasmi_wasi::WasiCtx;

/// The [`Context`] for the `wasmi` CLI application.
///
/// This simply stores all the necessary data.
pub struct Context {
    /// The given Wasm module.
    module: Module,
    /// The used Wasm store.
    store: Store<WasiCtx>,
    /// The Wasm module instance to operate on.
    instance: Instance,
}

impl Context {
    /// Creates a new [`Context`].
    ///
    /// # Errors
    ///
    /// - If parsing, validating, compiling or instantiating the Wasm module failed.
    /// - If adding WASI defintions to the linker failed.
    pub fn new(wasm_file: &Path, wasi_ctx: WasiCtx, fuel: Option<u64>) -> Result<Self, Error> {
        let mut config = Config::default();
        if fuel.is_some() {
            config.consume_fuel(true);
        }
        let engine = wasmi::Engine::new(&config);
        let wasm_bytes = utils::read_wasm_or_wat(wasm_file)?;
        let module = wasmi::Module::new(&engine, &mut &wasm_bytes[..]).map_err(|error| {
            anyhow!("failed to parse and validate Wasm module {wasm_file:?}: {error}")
        })?;
        let mut store = wasmi::Store::new(&engine, wasi_ctx);
        if let Some(fuel) = fuel {
            store.add_fuel(fuel).unwrap_or_else(|error| {
                panic!("error: fuel metering is enabled but encountered: {error}")
            });
        }
        let mut linker = <wasmi::Linker<WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx)
            .map_err(|error| anyhow!("failed to add WASI definitions to the linker: {error}"))?;
        let instance = linker
            .instantiate(&mut store, &module)
            .and_then(|pre| pre.start(&mut store))
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
    pub fn store(&self) -> &Store<WasiCtx> {
        &self.store
    }

    /// Returns an exclusive reference to the [`Store`] of the [`Context`].
    pub fn store_mut(&mut self) -> &mut Store<WasiCtx> {
        &mut self.store
    }

    /// Returns the exported function named `name` if any.
    pub fn get_func(&self, name: &str) -> Result<Func, Error> {
        self.instance
            .get_func(&self.store, name)
            .ok_or_else(|| anyhow!("failed to find function named {name:?} in the Wasm module"))
    }
}
