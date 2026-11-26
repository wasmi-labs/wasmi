use crate::{Error, FuncType, GlobalType, MemoryType, TableType};

/// A section of a Wasm module that carries an entry count.
///
/// Passed to [`ModuleInspector::section`] when the section is reached.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SectionKind {
    /// The type section.
    Type,
    /// The import section.
    Import,
    /// The function section.
    Function,
    /// The table section.
    Table,
    /// The memory section.
    Memory,
    /// The global section.
    Global,
    /// The export section.
    Export,
    /// The element section.
    Element,
    /// The data count section.
    ///
    /// Its count is the declared number of data segments.
    DataCount,
    /// The code section.
    Code,
    /// The data section.
    Data,
}

/// What an import brings into the module, as reported to [`ModuleInspector::import`].
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum ImportKind {
    /// An imported function of the type at `type_index` in the type section.
    Func {
        /// Index into the type section.
        type_index: u32,
    },
    /// An imported table.
    Table(TableType),
    /// An imported linear memory.
    Memory(MemoryType),
    /// An imported global variable.
    Global(GlobalType),
}

/// What kind of item an export names, as reported to [`ModuleInspector::export`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExternKind {
    /// A function.
    Func,
    /// A table.
    Table,
    /// A linear memory.
    Memory,
    /// A global variable.
    Global,
}

/// How an element or data segment is applied, as reported to
/// [`ModuleInspector::element_segment`] and [`ModuleInspector::data_segment`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SegmentMode {
    /// Copied into the table or memory at `index` when the module is instantiated.
    Active {
        /// Index of the table (element segment) or memory (data segment).
        index: u32,
    },
    /// Available to `table.init` or `memory.init`, but not applied at instantiation.
    Passive,
    /// Declared for `ref.func`, and never applied. Element segments only.
    Declared,
}

/// Custom checks run while [`Module::new_with_inspector`] parses a Wasm module.
///
/// An event-driven ("SAX-style") view of the parse Wasmi already performs: one
/// callback when each section is reached and one for each entry in it, in the order
/// they appear in the Wasm binary. An embedder can enforce its own policy on a module
/// (caps on sizes, counts or names, restrictions on imports, exports or types)
/// without walking the Wasm bytes a second time, and without Wasmi needing a
/// configuration option for every such policy.
///
/// Every method has a default implementation that accepts, so an inspector only
/// overrides the callbacks it needs. Returning an error from any callback stops
/// parsing; [`Module::new_with_inspector`] then returns that error unchanged. Use
/// [`Error::host`] to return a custom error type that the caller can recover via
/// [`Error::downcast_ref`].
///
/// # Indices, not resolved items
///
/// As in other event-driven parsers, a reference to another item is reported as its
/// index: an imported or defined function as the index of its type, an export by the
/// index of what it names. Indices follow the Wasm index spaces, in which imported
/// items come before defined ones. An inspector that needs the item an index refers
/// to records it from the earlier callback that reported it — the type section, and
/// so every [`ModuleInspector::func_type`] call, precedes every use of a type index.
///
/// # When each callback runs
///
/// - [`ModuleInspector::section`] runs when a section is reached and **before** it
///   is validated, alongside the checks of [`EnforcedLimits`]. A count is read from
///   the section header, so refusing here costs nothing proportional to the
///   section's contents.
/// - Every per-entry callback runs **after** its section has been validated, while
///   Wasmi reads the section's entries into the module, so it sees validated values
///   and adds no extra pass over the bytes.
/// - The exceptions are function bodies, reported by
///   [`ModuleInspector::function_body`] before the body is validated (see there),
///   and custom sections, which Wasm does not validate.
///
/// [`Module::new_with_inspector`]: crate::Module::new_with_inspector
/// [`EnforcedLimits`]: crate::EnforcedLimits
pub trait ModuleInspector {
    /// Called when a section with `count` entries is reached, before it is validated.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the section.
    fn section(&mut self, _kind: SectionKind, _count: u32) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each function type in the type section.
    ///
    /// The `n`-th call is the type at index `n`.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the type.
    fn func_type(&mut self, _ty: &FuncType) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each import.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the import.
    fn import(&mut self, _module: &str, _name: &str, _kind: &ImportKind) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each function the module defines, with the index of its type.
    ///
    /// Imported functions are reported through [`ModuleInspector::import`] instead.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the function.
    fn function(&mut self, _type_index: u32) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each table the module defines.
    ///
    /// Imported tables are reported through [`ModuleInspector::import`] instead.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the table.
    fn table(&mut self, _ty: &TableType) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each linear memory the module defines.
    ///
    /// Imported memories are reported through [`ModuleInspector::import`] instead.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the memory.
    fn memory(&mut self, _ty: &MemoryType) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each global variable the module defines.
    ///
    /// Imported globals are reported through [`ModuleInspector::import`] instead.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the global.
    fn global(&mut self, _ty: &GlobalType) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each export: its name, and the kind and index of what it names.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the export.
    fn export(&mut self, _name: &str, _kind: ExternKind, _index: u32) -> Result<(), Error> {
        Ok(())
    }

    /// Called for the start section, with the index of the start function.
    ///
    /// Runs after the start section is validated and before
    /// [`Config::allow_start_fn`] is applied, so an inspector sees the start function
    /// even of a module the configuration will refuse.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the start function.
    ///
    /// [`Config::allow_start_fn`]: crate::Config::allow_start_fn
    fn start(&mut self, _func_index: u32) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each element segment: how it is applied, and how many items it holds.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the segment.
    fn element_segment(&mut self, _mode: SegmentMode, _items: u32) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each function body in the code section: the index of the function
    /// it defines, and the size of the body in bytes.
    ///
    /// Runs before the body is validated or translated. Depending on the
    /// [`CompilationMode`], validating a body may be deferred until the function is
    /// first called, so this is the one point during [`Module::new_with_inspector`]
    /// at which every body is seen.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the body.
    ///
    /// [`CompilationMode`]: crate::CompilationMode
    /// [`Module::new_with_inspector`]: crate::Module::new_with_inspector
    fn function_body(&mut self, _func_index: u32, _bytes: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each data segment: how it is applied, and its size in bytes.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the segment.
    fn data_segment(&mut self, _mode: SegmentMode, _bytes: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Called for each custom section: its name, and the size of its contents in bytes.
    ///
    /// Runs even when [`Config::ignore_custom_sections`] is set, since what is being
    /// inspected is the input, not what the module keeps of it.
    ///
    /// # Errors
    ///
    /// If the inspector refuses the section.
    ///
    /// [`Config::ignore_custom_sections`]: crate::Config::ignore_custom_sections
    fn custom_section(&mut self, _name: &str, _bytes: usize) -> Result<(), Error> {
        Ok(())
    }
}
