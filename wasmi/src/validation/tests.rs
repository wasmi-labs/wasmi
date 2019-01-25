use super::{validate_module, ValidatedModule};
use isa;
use parity_wasm::builder::module;
use parity_wasm::elements::{
    deserialize_buffer, BlockType, External, GlobalEntry, GlobalType, ImportEntry, InitExpr,
    Instruction, Instructions, MemoryType, Module, TableType, ValueType,
};
use wabt;

#[test]
fn empty_is_valid() {
    let module = module().build();
    assert!(validate_module(module).is_ok());
}

#[test]
fn limits() {
    let test_cases = vec![
        // min > max
        (10, Some(9), false),
        // min = max
        (10, Some(10), true),
        // table/memory is always valid without max
        (10, None, true),
    ];

    for (min, max, is_valid) in test_cases {
        // defined table
        let m = module().table().with_min(min).with_max(max).build().build();
        assert_eq!(validate_module(m).is_ok(), is_valid);

        // imported table
        let m = module()
            .with_import(ImportEntry::new(
                "core".into(),
                "table".into(),
                External::Table(TableType::new(min, max)),
            ))
            .build();
        assert_eq!(validate_module(m).is_ok(), is_valid);

        // defined memory
        let m = module()
            .memory()
            .with_min(min)
            .with_max(max)
            .build()
            .build();
        assert_eq!(validate_module(m).is_ok(), is_valid);

        // imported table
        let m = module()
            .with_import(ImportEntry::new(
                "core".into(),
                "memory".into(),
                External::Memory(MemoryType::new(min, max)),
            ))
            .build();
        assert_eq!(validate_module(m).is_ok(), is_valid);
    }
}

#[test]
fn global_init_const() {
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::I32Const(42), Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_ok());

    // init expr type differs from declared global type
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I64, true),
            InitExpr::new(vec![Instruction::I32Const(42), Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_err());
}

#[test]
fn global_init_global() {
    let m = module()
        .with_import(ImportEntry::new(
            "env".into(),
            "ext_global".into(),
            External::Global(GlobalType::new(ValueType::I32, false)),
        ))
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::GetGlobal(0), Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_ok());

    // get_global can reference only previously defined globals
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::GetGlobal(0), Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_err());

    // get_global can reference only const globals
    let m = module()
        .with_import(ImportEntry::new(
            "env".into(),
            "ext_global".into(),
            External::Global(GlobalType::new(ValueType::I32, true)),
        ))
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::GetGlobal(0), Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_err());

    // get_global in init_expr can only refer to imported globals.
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, false),
            InitExpr::new(vec![Instruction::I32Const(0), Instruction::End]),
        ))
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::GetGlobal(0), Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_err());
}

#[test]
fn global_init_misc() {
    // without delimiting End opcode
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::I32Const(42)]),
        ))
        .build();
    assert!(validate_module(m).is_err());

    // empty init expr
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_err());

    // not an constant opcode used
    let m = module()
        .with_global(GlobalEntry::new(
            GlobalType::new(ValueType::I32, true),
            InitExpr::new(vec![Instruction::Unreachable, Instruction::End]),
        ))
        .build();
    assert!(validate_module(m).is_err());
}

#[test]
fn module_limits_validity() {
    // module cannot contain more than 1 memory atm.
    let m = module()
        .with_import(ImportEntry::new(
            "core".into(),
            "memory".into(),
            External::Memory(MemoryType::new(10, None)),
        ))
        .memory()
        .with_min(10)
        .build()
        .build();
    assert!(validate_module(m).is_err());

    // module cannot contain more than 1 table atm.
    let m = module()
        .with_import(ImportEntry::new(
            "core".into(),
            "table".into(),
            External::Table(TableType::new(10, None)),
        ))
        .table()
        .with_min(10)
        .build()
        .build();
    assert!(validate_module(m).is_err());
}

#[test]
fn funcs() {
    // recursive function calls is legal.
    let m = module()
        .function()
        .signature()
        .return_type()
        .i32()
        .build()
        .body()
        .with_instructions(Instructions::new(vec![
            Instruction::Call(1),
            Instruction::End,
        ]))
        .build()
        .build()
        .function()
        .signature()
        .return_type()
        .i32()
        .build()
        .body()
        .with_instructions(Instructions::new(vec![
            Instruction::Call(0),
            Instruction::End,
        ]))
        .build()
        .build()
        .build();
    assert!(validate_module(m).is_ok());
}

#[test]
fn globals() {
    // import immutable global is legal.
    let m = module()
        .with_import(ImportEntry::new(
            "env".into(),
            "ext_global".into(),
            External::Global(GlobalType::new(ValueType::I32, false)),
        ))
        .build();
    assert!(validate_module(m).is_ok());

    // import mutable global is invalid.
    let m = module()
        .with_import(ImportEntry::new(
            "env".into(),
            "ext_global".into(),
            External::Global(GlobalType::new(ValueType::I32, true)),
        ))
        .build();
    assert!(validate_module(m).is_err());
}

#[test]
fn if_else_with_return_type_validation() {
    let m = module()
        .function()
        .signature()
        .build()
        .body()
        .with_instructions(Instructions::new(vec![
            Instruction::I32Const(1),
            Instruction::If(BlockType::NoResult),
            Instruction::I32Const(1),
            Instruction::If(BlockType::Value(ValueType::I32)),
            Instruction::I32Const(1),
            Instruction::Else,
            Instruction::I32Const(2),
            Instruction::End,
            Instruction::Drop,
            Instruction::End,
            Instruction::End,
        ]))
        .build()
        .build()
        .build();
    validate_module(m).unwrap();
}

fn validate(wat: &str) -> ValidatedModule {
    let wasm = wabt::wat2wasm(wat).unwrap();
    let module = deserialize_buffer::<Module>(&wasm).unwrap();
    let validated_module = validate_module(module).unwrap();
    validated_module
}

fn compile(module: &ValidatedModule) -> (Vec<isa::Instruction>, Vec<u32>) {
    let code = &module.code_map[0];
    let mut instructions = Vec::new();
    let mut pcs = Vec::new();
    let mut iter = code.iterate_from(0);
    loop {
        let pc = iter.position();
        if let Some(instruction) = iter.next() {
            instructions.push(instruction.clone());
            pcs.push(pc);
        } else {
            break;
        }
    }

    (instructions, pcs)
}

macro_rules! targets {
	($($target:expr),*) => {
		::isa::BrTargets::from_internal(
			&[$($target,)*]
				.iter()
				.map(|&target| ::isa::InstructionInternal::BrTableTarget(target))
				.collect::<Vec<_>>()[..]
		)
	};
}

#[test]
fn implicit_return_no_value() {
    let module = validate(
        r#"
		(module
			(func (export "call")
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![isa::Instruction::Return(isa::DropKeep {
            drop: 0,
            keep: isa::Keep::None,
        })]
    )
}

#[test]
fn implicit_return_with_value() {
    let module = validate(
        r#"
		(module
			(func (export "call") (result i32)
				i32.const 0
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(0),
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::Single,
            }),
        ]
    )
}

#[test]
fn implicit_return_param() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32)
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![isa::Instruction::Return(isa::DropKeep {
            drop: 1,
            keep: isa::Keep::None,
        }),]
    )
}

#[test]
fn get_local() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				get_local 0
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::GetLocal(1),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::Single,
            }),
        ]
    )
}

#[test]
fn explicit_return() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				get_local 0
				return
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::GetLocal(1),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::Single,
            }),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::Single,
            }),
        ]
    )
}

#[test]
fn add_params() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32) (param i32) (result i32)
				get_local 0
				get_local 1
				i32.add
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            // This is tricky. Locals are now loaded from the stack. The load
            // happens from address relative of the current stack pointer. The first load
            // takes the value below the previous one (i.e the second argument) and then, it increments
            // the stack pointer. And then the same thing hapens with the value below the previous one
            // (which happens to be the value loaded by the first get_local).
            isa::Instruction::GetLocal(2),
            isa::Instruction::GetLocal(2),
            isa::Instruction::I32Add,
            isa::Instruction::Return(isa::DropKeep {
                drop: 2,
                keep: isa::Keep::Single,
            }),
        ]
    )
}

#[test]
fn drop_locals() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32)
				(local i32)
				get_local 0
				set_local 1
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::GetLocal(2),
            isa::Instruction::SetLocal(1),
            isa::Instruction::Return(isa::DropKeep {
                drop: 2,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn if_without_else() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				i32.const 1
				if
					i32.const 2
					return
				end
				i32.const 3
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfEqz(isa::Target {
                dst_pc: pcs[4],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(2),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,                 // 1 param
                keep: isa::Keep::Single, // 1 result
            }),
            isa::Instruction::I32Const(3),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::Single,
            }),
        ]
    )
}

#[test]
fn if_else() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				(local i32)
				i32.const 1
				if
					i32.const 2
					set_local 0
				else
					i32.const 3
					set_local 0
				end
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfEqz(isa::Target {
                dst_pc: pcs[5],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(2),
            isa::Instruction::SetLocal(1),
            isa::Instruction::Br(isa::Target {
                dst_pc: pcs[7],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(3),
            isa::Instruction::SetLocal(1),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn if_else_returns_result() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				i32.const 1
				if (result i32)
					i32.const 2
				else
					i32.const 3
				end
				drop
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfEqz(isa::Target {
                dst_pc: pcs[4],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(2),
            isa::Instruction::Br(isa::Target {
                dst_pc: pcs[5],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(3),
            isa::Instruction::Drop,
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn if_else_branch_from_true_branch() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				i32.const 1
				if (result i32)
					i32.const 1
					i32.const 1
					br_if 0
					drop
					i32.const 2
				else
					i32.const 3
				end
				drop
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfEqz(isa::Target {
                dst_pc: pcs[8],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(1),
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfNez(isa::Target {
                dst_pc: pcs[9],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::Single,
                },
            }),
            isa::Instruction::Drop,
            isa::Instruction::I32Const(2),
            isa::Instruction::Br(isa::Target {
                dst_pc: pcs[9],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(3),
            isa::Instruction::Drop,
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn if_else_branch_from_false_branch() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				i32.const 1
				if (result i32)
					i32.const 1
				else
					i32.const 2
					i32.const 1
					br_if 0
					drop
					i32.const 3
				end
				drop
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfEqz(isa::Target {
                dst_pc: pcs[4],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(1),
            isa::Instruction::Br(isa::Target {
                dst_pc: pcs[9],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(2),
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfNez(isa::Target {
                dst_pc: pcs[9],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::Single,
                },
            }),
            isa::Instruction::Drop,
            isa::Instruction::I32Const(3),
            isa::Instruction::Drop,
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn loop_() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				loop (result i32)
					i32.const 1
					br_if 0
					i32.const 2
				end
				drop
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(1),
            isa::Instruction::BrIfNez(isa::Target {
                dst_pc: 0,
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(2),
            isa::Instruction::Drop,
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn loop_empty() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				loop
				end
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![isa::Instruction::Return(isa::DropKeep {
            drop: 0,
            keep: isa::Keep::None,
        }),]
    )
}

#[test]
fn brtable() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				block $1
					loop $2
						i32.const 0
						br_table $2 $1
					end
				end
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(0),
            isa::Instruction::BrTable(targets![
                isa::Target {
                    dst_pc: 0,
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                },
                isa::Target {
                    dst_pc: pcs[2],
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::None,
                    },
                }
            ]),
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn brtable_returns_result() {
    let module = validate(
        r#"
		(module
			(func (export "call")
				block $1 (result i32)
					block $2 (result i32)
						i32.const 0
						i32.const 1
						br_table $2 $1
					end
					unreachable
				end
				drop
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    println!("{:?}", (&code, &pcs));
    assert_eq!(
        code,
        vec![
            isa::Instruction::I32Const(0),
            isa::Instruction::I32Const(1),
            isa::Instruction::BrTable(targets![
                isa::Target {
                    dst_pc: pcs[3],
                    drop_keep: isa::DropKeep {
                        drop: 0,
                        keep: isa::Keep::Single,
                    },
                },
                isa::Target {
                    dst_pc: pcs[4],
                    drop_keep: isa::DropKeep {
                        keep: isa::Keep::Single,
                        drop: 0,
                    },
                }
            ]),
            isa::Instruction::Unreachable,
            isa::Instruction::Drop,
            isa::Instruction::Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::None,
            }),
        ]
    )
}

#[test]
fn wabt_example() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				block $exit
					get_local 0
					br_if $exit
					i32.const 1
					return
				end
				i32.const 2
				return
			)
		)
	"#,
    );
    let (code, pcs) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::GetLocal(1),
            isa::Instruction::BrIfNez(isa::Target {
                dst_pc: pcs[4],
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::None,
                },
            }),
            isa::Instruction::I32Const(1),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1, // 1 parameter
                keep: isa::Keep::Single,
            }),
            isa::Instruction::I32Const(2),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::Single,
            }),
            isa::Instruction::Return(isa::DropKeep {
                drop: 1,
                keep: isa::Keep::Single,
            }),
        ]
    )
}
