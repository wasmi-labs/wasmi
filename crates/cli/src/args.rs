use crate::context::StoreContext;
#[cfg(feature = "wasi")]
use anyhow::Context;
use anyhow::{Error, Result, bail};
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};
#[cfg(feature = "wasi")]
use std::{net::SocketAddr, str::FromStr};
#[cfg(feature = "wasi")]
use wasmi_wasi::{Dir, TcpListener, WasiCtxBuilder, ambient_authority};

/// The Wasmi CLI application arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
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

    /// The file containing the WebAssembly module to execute.
    #[clap(
        value_name = "MODULE",
        value_hint = clap::ValueHint::FilePath,
    )]
    module: PathBuf,

    /// Arguments given to Wasi or the invoked function.
    ///
    /// If the `--invoke` CLI argument has been passed these arguments
    /// will be provided to the invoked function.
    ///
    /// Otherwise these arguments will be passed as WASI CLI arguments.
    #[clap(value_name = "ARGS", trailing_var_arg = true)]
    args: Vec<String>,
}

impl Args {
    /// Returns the Wasm file path given to the CLI app.
    pub fn module(&self) -> &Path {
        &self.module
    }

    /// Returns the name of the invoked function if any.
    pub fn invoked(&self) -> Option<&str> {
        self.invoke.as_deref()
    }

    /// Returns the function arguments given to the CLI app.
    pub fn args(&self) -> &[String] {
        &self.args[..]
    }

    /// Returns the amount of fuel given to the CLI app if any.
    pub fn fuel(&self) -> Option<u64> {
        self.fuel
    }

    /// Returns `true` if lazy Wasm compilation is enabled.
    pub fn compilation_mode(&self) -> wasmi::CompilationMode {
        self.compilation_mode.into()
    }

    /// Returns `true` if verbose messaging is enabled.
    pub fn verbose(&self) -> bool {
        self.verbose
    }
}

#[cfg(not(feature = "wasi"))]
impl Args {
    /// Creates the [`StoreContext`] for this session.
    pub fn store_context(&self) -> Result<StoreContext, Error> {
        Ok(StoreContext)
    }
}

#[cfg(feature = "wasi")]
impl Args {
    /// Creates the [`StoreContext`] for this session.
    pub fn store_context(&self) -> Result<StoreContext, Error> {
        let mut wasi_builder = WasiCtxBuilder::new();
        for key_val in &self.envs {
            wasi_builder.env(key_val.key(), key_val.value())?;
        }
        // Populate Wasi arguments.
        let argv0 = self.module().as_os_str().to_str().unwrap_or("");
        wasi_builder.arg(argv0)?;
        wasi_builder.args(self.args())?;
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
}

/// A CLI flag value key-value argument.
#[derive(Debug, Clone)]
#[cfg(feature = "wasi")]
struct KeyValue {
    key_val: Box<str>,
    eq_pos: usize,
}

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
