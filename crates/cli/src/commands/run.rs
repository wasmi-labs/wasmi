use crate::{
    commands::Command,
    context::{Context, StoreContext},
    display::{DisplayExportedFuncs, DisplaySequence, DisplayValue},
    utils,
};
#[cfg(feature = "wasi")]
use anyhow::Context as _;
use anyhow::{Error, Result, anyhow, bail};
use clap::{Parser, ValueEnum};
#[cfg(feature = "wasi")]
use std::path::PathBuf;
#[cfg(feature = "wasi")]
use std::{net::SocketAddr, str::FromStr};
use std::{path::Path, process};
use wasmi::{Func, Val};
#[cfg(feature = "wasi")]
use wasmi_wasi::{Dir, TcpListener, WasiCtxBuilder, ambient_authority};

/// Executes a Wasm module.
#[derive(Parser)]
pub struct RunCommand {
    /// The host directory to pre-open for the `guest` to use.
    #[clap(
        long = "dir",
        value_name = "DIRECTORY",
        action = clap::ArgAction::Append,
        value_hint = clap::ValueHint::DirPath,
    )]
    #[cfg(feature = "wasi")]
    dirs: Vec<PathBuf>,

    /// The socket address provided to the module. Allows it to perform socket-related `WASI` ops.
    #[clap(
        long = "tcplisten",
        value_name = "SOCKET ADDRESS",
        action = clap::ArgAction::Append,
    )]
    #[cfg(feature = "wasi")]
    tcplisten: Vec<SocketAddr>,

    /// The environment variable pair made available for the program.
    #[clap(
        long = "env",
        value_name = "NAME=VAL",
        value_parser(KeyValue::from_str),
        action = clap::ArgAction::Append,
    )]
    #[cfg(feature = "wasi")]
    envs: Vec<KeyValue>,

    /// The function to invoke.
    ///
    /// If this argument is missing, Wasmi CLI will try to run `""` or `_start`.
    ///
    /// If neither are exported the Wasmi CLI will display out all exported
    /// functions of the Wasm module and return with an error.
    #[clap(long = "invoke", value_name = "FUNCTION")]
    invoke: Option<String>,

    /// Select Wasmi's mode of compilation.
    #[clap(long = "compilation-mode", value_enum, default_value_t=CompilationMode::LazyTranslation)]
    compilation_mode: CompilationMode,

    /// Enable execution fuel metering with N units of fuel.
    ///
    /// The execution will trap after running out of the N units of fuel.
    #[clap(long = "fuel", value_name = "N")]
    fuel: Option<u64>,

    /// Enable informational messages beyond warnings or errors.
    #[clap(long = "verbose")]
    verbose: bool,

    /// Wasm module and arguments given to the invoked function or to WASI.
    ///
    /// If the `--invoke` CLI argument has been passed these arguments
    /// will be provided to the invoked function.
    ///
    /// Otherwise these arguments will be passed as WASI CLI arguments.
    ///
    /// Usage:
    ///
    /// - wasmi foo.wasm
    ///
    /// - wasmi foo.wasm a b c
    ///
    /// - wasmi foo.wasm --invoke bar a b c
    #[clap(value_name = "ARGS", trailing_var_arg = true)]
    module_and_args: Vec<String>,
}

impl Command for RunCommand {
    fn execute(self) -> Result<(), Error> {
        let wasm_file = self.module();
        let wasi_ctx = self.store_context()?;
        let mut ctx = Context::new(wasm_file, wasi_ctx, self.fuel(), self.compilation_mode())?;
        let (func_name, func) = self.invoked_name_and_func(&ctx)?;
        let ty = func.ty(ctx.store());
        let func_args = utils::decode_func_args(&ty, self.args())?;
        let mut func_results = utils::prepare_func_results(&ty);
        utils::typecheck_args(&func_name, &ty, &func_args)?;
        self.print_execution_start(&func_name, &func_args);
        match func.call(ctx.store_mut(), &func_args, &mut func_results) {
            Ok(()) => {
                self.print_remaining_fuel(&ctx);
                Self::print_pretty_results(&func_results);
                Ok(())
            }
            Err(error) => {
                if let Some(exit_code) = error.i32_exit_status() {
                    // We received an exit code from the WASI program,
                    // therefore we exit with the same exit code after
                    // pretty printing the results.
                    self.print_remaining_fuel(&ctx);
                    Self::print_pretty_results(&func_results);
                    process::exit(exit_code)
                }
                bail!("failed during execution of {func_name}: {error}")
            }
        }
    }
}

impl RunCommand {
    /// Returns the invoked named function or the WASI entry point to the Wasm module if any.
    ///
    /// # Errors
    ///
    /// - If the function given via `--invoke` could not be found in the Wasm module.
    /// - If `--invoke` was not given and no WASI entry points were exported.
    fn invoked_name_and_func(&self, ctx: &Context) -> Result<(String, Func), Error> {
        match self.invoked() {
            Some(func_name) => {
                let func = ctx
                    .get_func(func_name)
                    .map_err(|error| anyhow!("{error}\n\n{}", DisplayExportedFuncs::from(ctx)))?;
                let func_name = func_name.into();
                Ok((func_name, func))
            }
            None => {
                // No `--invoke` flag was provided so we try to find
                // the conventional WASI entry points `""` and `"_start"`.
                if let Ok(func) = ctx.get_func("") {
                    Ok(("".into(), func))
                } else if let Ok(func) = ctx.get_func("_start") {
                    Ok(("_start".into(), func))
                } else {
                    bail!(
                        "did not specify `--invoke` and could not find exported WASI entry point functions\n\n{}",
                        DisplayExportedFuncs::from(ctx)
                    )
                }
            }
        }
    }

    /// Prints a signalling text that Wasm execution has started.
    fn print_execution_start(&self, func_name: &str, func_args: &[Val]) {
        if !self.verbose() {
            return;
        }
        let module = self.module();
        println!(
            "executing File({module:?})::{func_name}({}) ...",
            DisplaySequence::new(", ", func_args.iter().map(DisplayValue::from))
        );
    }

    /// Prints the remaining fuel so far if fuel metering was enabled.
    fn print_remaining_fuel(&self, ctx: &Context) {
        if let Some(given_fuel) = self.fuel() {
            let remaining = ctx
                .store()
                .get_fuel()
                .unwrap_or_else(|error| panic!("could not get the remaining fuel: {error}"));
            let consumed = given_fuel.saturating_sub(remaining);
            println!("fuel consumed: {consumed}, fuel remaining: {remaining}");
        }
    }

    /// Prints the results of the Wasm computation in a human readable form.
    fn print_pretty_results(results: &[Val]) {
        for result in results {
            println!("{}", DisplayValue::from(result))
        }
    }

    /// Returns the Wasm file path given to the CLI app.
    fn module(&self) -> &Path {
        Path::new(&self.module_and_args[0])
    }

    /// Returns the name of the invoked function if any.
    fn invoked(&self) -> Option<&str> {
        self.invoke.as_deref()
    }

    /// Returns the arguments for the invoked function.
    fn args(&self) -> &[String] {
        &self.module_and_args[1..]
    }

    /// Returns the amount of fuel given to the CLI app if any.
    fn fuel(&self) -> Option<u64> {
        self.fuel
    }

    /// Returns `true` if lazy Wasm compilation is enabled.
    fn compilation_mode(&self) -> wasmi::CompilationMode {
        self.compilation_mode.into()
    }

    /// Returns `true` if verbose messaging is enabled.
    fn verbose(&self) -> bool {
        self.verbose
    }
}

#[cfg(not(feature = "wasi"))]
impl RunCommand {
    /// Creates the [`StoreContext`] for this session.
    pub fn store_context(&self) -> Result<StoreContext, Error> {
        Ok(StoreContext)
    }
}

#[cfg(feature = "wasi")]
impl RunCommand {
    /// Creates the [`StoreContext`] for this session.
    pub fn store_context(&self) -> Result<StoreContext, Error> {
        let mut wasi_builder = WasiCtxBuilder::new();
        for key_val in &self.envs {
            wasi_builder.env(key_val.key(), key_val.value())?;
        }
        // Populate Wasi arguments.
        wasi_builder.args(self.argv())?;
        // Add pre-opened TCP sockets.
        //
        // Note that `num_fd` starts at 3 because the inherited `stdin`, `stdout` and `stderr`
        // are already mapped to `0, 1, 2` respectively.
        wasi_builder.inherit_stdio();
        for (socket, num_fd) in self.preopen_sockets()?.into_iter().zip(3..) {
            wasi_builder.preopened_socket(num_fd, socket)?;
        }
        // Add pre-opened directories.
        for (dir_name, dir) in self.preopen_dirs()? {
            wasi_builder.preopened_dir(dir, dir_name)?;
        }
        Ok(wasi_builder.build())
    }

    /// Pre-opens all directories given in `--dir` and returns them for use by the [`StoreContext`].
    ///
    /// # Errors
    ///
    /// If any of the given directions in `--dir` cannot be opened.
    fn preopen_dirs(&self) -> Result<Vec<(&Path, Dir)>> {
        self.dirs
            .iter()
            .map(|path| {
                let dir = Dir::open_ambient_dir(path, ambient_authority()).with_context(|| {
                    format!("failed to open directory '{path:?}' with ambient authority")
                })?;
                Ok((path.as_ref(), dir))
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Opens sockets given in `--tcplisten` and returns them for use by the [`StoreContext`].
    ///
    /// # Errors
    ///
    /// If any of the given socket addresses in `--tcplisten` cannot be listened to.
    fn preopen_sockets(&self) -> Result<Vec<TcpListener>> {
        self.tcplisten
            .iter()
            .map(|addr| {
                let std_tcp_listener = std::net::TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to tcp address '{addr}'"))?;
                std_tcp_listener.set_nonblocking(true)?;
                Ok(TcpListener::from_std(std_tcp_listener))
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Returns the Wasmi CLI arguments.
    fn argv(&self) -> &[String] {
        &self.module_and_args[..]
    }
}

/// A CLI flag value key-value argument.
#[derive(Debug, Clone)]
#[cfg(feature = "wasi")]
struct KeyValue {
    key_val: Box<str>,
    eq_pos: usize,
}

#[cfg(feature = "wasi")]
impl KeyValue {
    /// Returns the key of `self`.
    pub fn key(&self) -> &str {
        self.key_val.split_at(self.eq_pos).0
    }

    /// Returns the value of `self`.
    pub fn value(&self) -> &str {
        &self.key_val.split_at(self.eq_pos).1[1..]
    }
}

#[cfg(feature = "wasi")]
impl FromStr for KeyValue {
    type Err = Error;

    /// Parses a CLI flag value as [`KeyValue`] type.
    ///
    /// # Errors
    ///
    /// If the string cannot be parsed into a `KEY=VALUE` style pair.
    fn from_str(contents: &str) -> Result<Self, Self::Err> {
        let Some(eq_pos) = contents.find('=') else {
            bail!("missing '=' in KEY=VAL pair: {contents}")
        };
        let (key, eq_and_value) = contents.split_at(eq_pos);
        debug_assert!(eq_and_value.starts_with('='));
        let value = &eq_and_value[1..];
        if key.is_empty() {
            bail!("missing KEY in --env KEY=VAL: {contents}")
        }
        if value.is_empty() {
            bail!("missing VAL in --env KEY=VAL: {contents}")
        }
        Ok(Self {
            key_val: contents.into(),
            eq_pos,
        })
    }
}

/// The chosen Wasmi compilation mode.
#[derive(Debug, Default, Copy, Clone, ValueEnum)]
enum CompilationMode {
    Eager,
    #[default]
    LazyTranslation,
    Lazy,
}

impl From<CompilationMode> for wasmi::CompilationMode {
    fn from(mode: CompilationMode) -> Self {
        match mode {
            CompilationMode::Eager => Self::Eager,
            CompilationMode::LazyTranslation => Self::LazyTranslation,
            CompilationMode::Lazy => Self::Lazy,
        }
    }
}
