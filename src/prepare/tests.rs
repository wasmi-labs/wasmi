// Test-only code importing std for no-std testing
extern crate std;

use alloc::vec::Vec;
use std::println;

use super::{compile_module, CompiledModule};
use crate::isa;
use parity_wasm::{deserialize_buffer, elements::Module};

fn validate(wat: &str) -> CompiledModule {
    let wasm = wat::parse_str(wat).unwrap();
    let module = deserialize_buffer::<Module>(&wasm).unwrap();
    compile_module(module).unwrap()
}

fn compile(module: &CompiledModule) -> (Vec<isa::Instruction>, Vec<u32>) {
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
		crate::isa::BrTargets::from_internal(
			&[$($target,)*]
				.iter()
				.map(|&target| crate::isa::InstructionInternal::BrTableTarget(target))
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
fn get_local_2() {
    let module = validate(
        r#"
		(module
			(func (export "call") (param i32) (param i32) (result i32)
				get_local 0
                get_local 1
                drop
			)
		)
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            isa::Instruction::GetLocal(2),
            isa::Instruction::GetLocal(2),
            isa::Instruction::Drop,
            isa::Instruction::Return(isa::DropKeep {
                drop: 2,
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
fn spec_as_br_if_value_cond() {
    use self::isa::Instruction::*;

    let module = validate(
        r#"
		  (func (export "as-br_if-value-cond") (result i32)
            (block (result i32)
              (drop
                (br_if 0
                  (i32.const 6)
                  (br_table 0 0
                    (i32.const 9)
                    (i32.const 0)
                  )
                )
              )
            (i32.const 7)
          )
        )
	"#,
    );
    let (code, _) = compile(&module);
    assert_eq!(
        code,
        vec![
            I32Const(6),
            I32Const(9),
            I32Const(0),
            isa::Instruction::BrTable(targets![
                isa::Target {
                    dst_pc: 9,
                    drop_keep: isa::DropKeep {
                        drop: 1,
                        keep: isa::Keep::Single
                    }
                },
                isa::Target {
                    dst_pc: 9,
                    drop_keep: isa::DropKeep {
                        drop: 1,
                        keep: isa::Keep::Single
                    }
                }
            ]),
            BrIfNez(isa::Target {
                dst_pc: 9,
                drop_keep: isa::DropKeep {
                    drop: 0,
                    keep: isa::Keep::Single
                }
            }),
            Drop,
            I32Const(7),
            Return(isa::DropKeep {
                drop: 0,
                keep: isa::Keep::Single
            })
        ]
    );
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
