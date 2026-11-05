use core::fmt;
use wasmi::{
    Engine,
    Error,
    ExternKind,
    FuncType,
    GlobalType,
    ImportKind,
    MemoryType,
    Module,
    ModuleInspector,
    SectionKind,
    SegmentMode,
    TableType,
    errors::HostError,
};

/// An inspector that records every callback, and refuses one kind of section.
#[derive(Default)]
struct Recorder {
    seen: Vec<String>,
    refuse: Option<SectionKind>,
}

/// The error a [`Recorder`] refuses with.
#[derive(Debug)]
struct Refused(SectionKind);

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "refused the {:?} section", self.0)
    }
}

impl HostError for Refused {}

impl ModuleInspector for Recorder {
    fn section(&mut self, kind: SectionKind, count: u32) -> Result<(), Error> {
        if self.refuse == Some(kind) {
            return Err(Error::host(Refused(kind)));
        }
        self.seen.push(format!("{kind:?} section: {count}"));
        Ok(())
    }

    fn func_type(&mut self, ty: &FuncType) -> Result<(), Error> {
        self.seen
            .push(format!("type {:?} -> {:?}", ty.params(), ty.results()));
        Ok(())
    }

    fn import(&mut self, module: &str, name: &str, kind: &ImportKind) -> Result<(), Error> {
        let kind = match kind {
            ImportKind::Func { type_index } => format!("func of type {type_index}"),
            ImportKind::Global(ty) => format!("global {:?}", ty.content()),
            other => format!("{other:?}"),
        };
        self.seen.push(format!("import {module}::{name}: {kind}"));
        Ok(())
    }

    fn function(&mut self, type_index: u32) -> Result<(), Error> {
        self.seen.push(format!("function of type {type_index}"));
        Ok(())
    }

    fn table(&mut self, ty: &TableType) -> Result<(), Error> {
        self.seen
            .push(format!("table {}..{:?}", ty.minimum(), ty.maximum()));
        Ok(())
    }

    fn memory(&mut self, ty: &MemoryType) -> Result<(), Error> {
        self.seen
            .push(format!("memory {}..{:?}", ty.minimum(), ty.maximum()));
        Ok(())
    }

    fn global(&mut self, ty: &GlobalType) -> Result<(), Error> {
        self.seen
            .push(format!("global {:?} {:?}", ty.mutability(), ty.content()));
        Ok(())
    }

    fn export(&mut self, name: &str, kind: ExternKind, index: u32) -> Result<(), Error> {
        self.seen.push(format!("export {name}: {kind:?} {index}"));
        Ok(())
    }

    fn start(&mut self, func_index: u32) -> Result<(), Error> {
        self.seen.push(format!("start {func_index}"));
        Ok(())
    }

    fn element_segment(&mut self, mode: SegmentMode, items: u32) -> Result<(), Error> {
        self.seen.push(format!("element {mode:?}: {items} items"));
        Ok(())
    }

    fn function_body(&mut self, func_index: u32, bytes: usize) -> Result<(), Error> {
        self.seen.push(format!("body of {func_index}: {bytes} bytes"));
        Ok(())
    }

    fn data_segment(&mut self, mode: SegmentMode, bytes: usize) -> Result<(), Error> {
        self.seen.push(format!("data {mode:?}: {bytes} bytes"));
        Ok(())
    }

    fn custom_section(&mut self, name: &str, bytes: usize) -> Result<(), Error> {
        self.seen.push(format!("custom {name}: {bytes} bytes"));
        Ok(())
    }
}

/// A module with at least one entry in every section but the data count section.
///
/// Indices rather than `$names`, so the assembler emits no `name` custom section.
/// Functions are 0 (`env::f`), 1 and 2.
const WAT: &str = r#"
    (module
        (type (func (param i32)))
        (type (func))
        (import "env" "f" (func (type 0)))
        (import "env" "g" (global i32))
        (table 3 10 funcref)
        (memory 2)
        (global (mut i64) (i64.const 0))
        (func (type 1) (call 0 (i32.const 0)))
        (func (type 1))
        (export "run" (func 1))
        (export "memory" (memory 0))
        (start 2)
        (elem (i32.const 0) 1 2)
        (data (i32.const 0) "abc")
        (data "xy")
    )
"#;

/// [`WAT`] assembled, with a custom section named `note` appended.
fn wasm_with_custom_section() -> Vec<u8> {
    let mut wasm = wat::parse_str(WAT).unwrap();
    let name = b"note";
    let data = b"hello";
    let len = u8::try_from(1 + name.len() + data.len()).unwrap();
    wasm.extend([0x00, len, u8::try_from(name.len()).unwrap()]);
    wasm.extend(name);
    wasm.extend(data);
    wasm
}

/// Every callback is called, in the order its item appears in the binary.
#[test]
fn reports_every_section_and_entry_in_binary_order() {
    let engine = Engine::default();
    let mut recorder = Recorder::default();
    Module::new_with_inspector(&engine, wasm_with_custom_section(), &mut recorder).unwrap();
    assert_eq!(
        recorder.seen,
        [
            "Type section: 2",
            "type [I32] -> []",
            "type [] -> []",
            "Import section: 2",
            "import env::f: func of type 0",
            "import env::g: global I32",
            "Function section: 2",
            "function of type 1",
            "function of type 1",
            "Table section: 1",
            "table 3..Some(10)",
            "Memory section: 1",
            "memory 2..None",
            "Global section: 1",
            "global Var I64",
            "Export section: 2",
            "export run: Func 1",
            "export memory: Memory 0",
            "start 2",
            "Element section: 1",
            "element Active { index: 0 }: 2 items",
            "Code section: 2",
            // A body is its locals count, its instructions, and `end`.
            "body of 1: 6 bytes",
            "body of 2: 2 bytes",
            "Data section: 2",
            "data Active { index: 0 }: 3 bytes",
            "data Passive: 2 bytes",
            "custom note: 5 bytes",
        ]
    );
}

/// A refusal stops the parse and is returned unchanged, so the caller can recover
/// its own error type.
#[test]
fn a_refusal_is_returned_unchanged() {
    let engine = Engine::default();
    let mut recorder = Recorder {
        refuse: Some(SectionKind::Memory),
        ..Recorder::default()
    };
    let error = Module::new_with_inspector(&engine, WAT, &mut recorder).unwrap_err();
    let refused = error
        .downcast_ref::<Refused>()
        .expect("the inspector's own error");
    assert_eq!(refused.0, SectionKind::Memory);
    // Parsing stopped at the refusal: nothing after the memory section was seen.
    assert_eq!(recorder.seen.last().unwrap(), "table 3..Some(10)");
}

/// The section callback runs before the section is validated, so an inspector can
/// refuse a section on its count alone without the validator reading its entries.
///
/// The type section here claims a million entries but contains none, which the
/// validator would report as malformed. The inspector's refusal comes first.
#[test]
fn a_section_is_inspected_before_it_is_validated() {
    let wasm = [
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, 0x03, // type section, 3 bytes
        0xC0, 0x84, 0x3D, // count: 1_000_000 as LEB128, then no entries
    ];
    let engine = Engine::default();

    let unchecked = Module::new(&engine, &wasm[..]).unwrap_err();
    assert!(unchecked.downcast_ref::<Refused>().is_none());

    let mut recorder = Recorder {
        refuse: Some(SectionKind::Type),
        ..Recorder::default()
    };
    let error = Module::new_with_inspector(&engine, &wasm[..], &mut recorder).unwrap_err();
    assert!(error.downcast_ref::<Refused>().is_some(), "{error}");
}

/// A data count section, which the assembler emits only when code refers to a data
/// segment by index, reports the number of data segments.
#[test]
fn reports_the_data_count_section() {
    let wat = r#"
        (module
            (memory 1)
            (func (data.drop 0))
            (data "x")
        )
    "#;
    let engine = Engine::default();
    let mut recorder = Recorder::default();
    Module::new_with_inspector(&engine, wat, &mut recorder).unwrap();
    assert!(
        recorder.seen.iter().any(|seen| seen == "DataCount section: 1"),
        "{:?}",
        recorder.seen
    );
}

/// The start function is reported even when the configuration refuses it, since the
/// inspector sees the input before the configuration's verdict.
#[test]
fn a_start_function_is_reported_even_when_disallowed() {
    let mut config = wasmi::Config::default();
    config.allow_start_fn(false);
    let engine = Engine::new(&config);
    let mut recorder = Recorder::default();
    Module::new_with_inspector(&engine, WAT, &mut recorder).unwrap_err();
    assert_eq!(recorder.seen.last().unwrap(), "start 2");
}

/// An inspector that overrides nothing accepts whatever [`Module::new`] accepts.
#[test]
fn the_default_inspector_accepts_everything() {
    struct Nothing;
    impl ModuleInspector for Nothing {}

    let engine = Engine::default();
    Module::new_with_inspector(&engine, wasm_with_custom_section(), &mut Nothing).unwrap();
}
