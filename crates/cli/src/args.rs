use crate::context::StoreContext;
#[cfg(feature = "wasi")]
use anyhow::Context;
use anyhow::{Error, Result};
use argh::{FromArgValue, FromArgs};
use std::path::{Path, PathBuf};
#[cfg(feature = "wasi")]
use std::{ffi::OsStr, net::SocketAddr, str::FromStr};
#[cfg(feature = "wasi")]
use wasmi_wasi::{Dir, TcpListener, WasiCtxBuilder, ambient_authority};

/// A CLI flag value key-value argument.
#[derive(Debug, Clone)]
#[cfg(feature = "wasi")]
struct KeyValue {
    key: String,
    value: String,
}

#[cfg(feature = "wasi")]
impl FromStr for KeyValue {
    type Err = Error;

    /// Parses a CLI flag value as [`KeyValue`] type.
    ///
    /// # Errors
    ///
    /// If the string cannot be parsed into a `KEY=VALUE` style pair.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let eq_pos = s
            .find('=')
            .ok_or_else(|| anyhow::anyhow!("invalid KEY=value: no `=` found in `{}`", s))?;
        let (key, eq_value) = s.split_at(eq_pos);
        assert!(s.starts_with('='));
        let value = &eq_value[1..];
        let key = key.to_string();
        let value = value.to_string();
        Ok(KeyValue { key, value })
    }
}

#[cfg(feature = "wasi")]
fn parse_key_value(value: &str) -> Result<KeyValue, String> {
    KeyValue::from_str(value).map_err(|e| e.to_string())
}

/// The Wasmi CLI application.
#[derive(FromArgs, Debug)]
#[argh(description = "The Wasmi CLI application.")]
pub struct Args {
    /// the host directory to pre-open for the `guest` to use
    #[cfg(feature = "wasi")]
    #[argh(option, long = "dir")]
    dirs: Vec<PathBuf>,

    /// the socket address provided to the module
    #[cfg(feature = "wasi")]
    #[argh(option, long = "tcplisten")]
    tcplisten: Vec<SocketAddr>,

    /// the environment variable pair made available for the program
    #[cfg(feature = "wasi")]
    #[argh(option, long = "env", from_str_fn(parse_key_value))]
    envs: Vec<KeyValue>,

    /// the file containing the WebAssembly module to execute
    #[argh(positional)]
    wasm_file: PathBuf,

    /// the name of the exported function to invoke
    #[argh(option, long = "invoke")]
    invoke: Option<String>,

    /// select Wasmi's mode of compilation (default = lazy-translation)
    #[argh(option, long = "compilation-mode")]
    compilation_mode: Option<CompilationMode>,

    /// enable execution fuel metering with N units of fuel
    #[argh(option, long = "fuel")]
    fuel: Option<u64>,

    /// enable informational messages beyond warnings or errors
    #[argh(switch, long = "verbose")]
    verbose: bool,

    /// arguments given to the Wasm module or the invoked function
    #[argh(positional)]
    func_args: Vec<String>,
}

/// The chosen Wasmi compilation mode.
#[derive(Debug, Default, Copy, Clone, FromArgValue)]
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

impl Args {
    /// Returns the Wasm file path given to the CLI app.
    pub fn wasm_file(&self) -> &Path {
        &self.wasm_file
    }

    /// Returns the name of the invoked function if any.
    pub fn invoked(&self) -> Option<&str> {
        self.invoke.as_deref()
    }

    /// Returns the function arguments given to the CLI app.
    pub fn func_args(&self) -> &[String] {
        &self.func_args[..]
    }

    /// Returns the amount of fuel given to the CLI app if any.
    pub fn fuel(&self) -> Option<u64> {
        self.fuel
    }

    /// Returns `true` if lazy Wasm compilation is enabled.
    pub fn compilation_mode(&self) -> wasmi::CompilationMode {
        self.compilation_mode.unwrap_or_default().into()
    }

    /// Returns `true` if verbose messaging is enabled.
    pub fn verbose(&self) -> bool {
        self.verbose
    }

    /// Pre-opens all directories given in `--dir` and returns them for use by the [`StoreContext`].
    ///
    /// # Errors
    ///
    /// If any of the given directions in `--dir` cannot be opened.
    #[cfg(feature = "wasi")]
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

    /// Creates the [`StoreContext`] for this session.
    #[cfg(not(feature = "wasi"))]
    pub fn store_context(&self) -> Result<StoreContext, Error> {
        Ok(StoreContext)
    }

    /// Creates the [`StoreContext`] for this session.
    #[cfg(feature = "wasi")]
    pub fn store_context(&self) -> Result<StoreContext, Error> {
        let mut wasi_builder = WasiCtxBuilder::new();
        for KeyValue { key, value } in &self.envs {
            wasi_builder.env(key, value)?;
        }
        wasi_builder.args(&self.argv())?;
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

    /// Opens sockets given in `--tcplisten` and returns them for use by the [`StoreContext`].
    ///
    /// # Errors
    ///
    /// If any of the given socket addresses in `--tcplisten` cannot be listened to.
    #[cfg(feature = "wasi")]
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

    /// Returns the arguments that the WASI invocation expects to receive.
    ///
    /// The first argument is always the module file name itself followed
    /// by the arguments to the invoked function if any.
    ///
    /// This is similar to how `UNIX` systems work, and is part of the `WASI` spec.
    #[cfg(feature = "wasi")]
    fn argv(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(self.func_args.len() + 1);
        // The WebAssembly filename is expected to be the first argument to WASI.
        // Note that the module name still has it's `.wasm` file extension.
        let module_name = self
            .wasm_file
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .into();
        args.push(module_name);
        args.extend_from_slice(&self.func_args);
        args
    }
}
