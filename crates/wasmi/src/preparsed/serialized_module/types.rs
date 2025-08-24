use alloc::vec::Vec;
use wasmi_core::{Mutability, ReadAs, ValType};

use crate::module::init_expr::ConstExpr;
use crate::{FuncType, GlobalType, MemoryType, TableType};

#[cfg(feature = "serialization")]
use alloc::borrow::ToOwned;

#[cfg(feature = "serialization")]
use crate::{ExternType, ImportType};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedFuncType {
    pub(crate) params: Vec<SerializedValType>,
    pub(crate) results: Vec<SerializedValType>,
}

impl From<&FuncType> for SerializedFuncType {
    fn from(func_type: &FuncType) -> Self {
        Self {
            params: func_type
                .params()
                .iter()
                .map(SerializedValType::from)
                .collect(),
            results: func_type
                .results()
                .iter()
                .map(SerializedValType::from)
                .collect(),
        }
    }
}

impl From<&SerializedFuncType> for FuncType {
    fn from(func_type: &SerializedFuncType) -> Self {
        FuncType::new(
            func_type.params.iter().map(ValType::from),
            func_type.results.iter().map(ValType::from),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) enum SerializedValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl From<&ValType> for SerializedValType {
    fn from(value: &ValType) -> Self {
        match value {
            ValType::I32 => SerializedValType::I32,
            ValType::I64 => SerializedValType::I64,
            ValType::F32 => SerializedValType::F32,
            ValType::F64 => SerializedValType::F64,
            ValType::V128 => SerializedValType::V128,
            ValType::FuncRef => SerializedValType::FuncRef,
            ValType::ExternRef => SerializedValType::ExternRef,
        }
    }
}

impl From<&SerializedValType> for ValType {
    fn from(val: &SerializedValType) -> Self {
        match val {
            SerializedValType::I32 => crate::ValType::I32,
            SerializedValType::I64 => crate::ValType::I64,
            SerializedValType::F32 => crate::ValType::F32,
            SerializedValType::F64 => crate::ValType::F64,
            SerializedValType::V128 => crate::ValType::V128,
            SerializedValType::FuncRef => crate::ValType::FuncRef,
            SerializedValType::ExternRef => crate::ValType::ExternRef,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) enum SerializedConstExpr {
    I32Const(i32),
    I64Const(i64),
    F32Const(u32), // Bit pattern
    F64Const(u64), // Bit pattern
    V128Const([u8; 16]),
    GlobalGet(u32), // Index into globals
    #[cfg(feature = "deserialization")]
    RefNull(SerializedValType),
    RefFunc(u32), // Index into functions
}

impl From<&ConstExpr> for SerializedConstExpr {
    fn from(expr: &ConstExpr) -> Self {
        match &expr.op {
            crate::module::init_expr::Op::Const(const_op) => {
                // Extract the value from ConstOp and determine its type
                let value = const_op.value;
                // We need to determine the type from the value itself
                // For now, let's handle the common cases by trying different types
                let i32_val = ReadAs::<i32>::read_as(&value);
                let i64_val = ReadAs::<i64>::read_as(&value);
                let f32_val = ReadAs::<f32>::read_as(&value);
                let f64_val = ReadAs::<f64>::read_as(&value);

                // Check if it's a V128 value (when SIMD is enabled)
                #[cfg(feature = "simd")]
                {
                    let v128_val = ReadAs::<crate::V128>::read_as(&value);
                    // If the value is different when read as V128, it's likely a V128
                    if v128_val.as_u128() != i64_val as u128 {
                        return SerializedConstExpr::V128Const(v128_val.as_u128().to_le_bytes());
                    }
                }

                // Try to determine the most appropriate type
                if i32_val == i64_val as i32 && i32_val >= 0 {
                    SerializedConstExpr::I32Const(i32_val)
                } else if i64_val != i64::from(i32_val) {
                    SerializedConstExpr::I64Const(i64_val)
                } else if f32_val != i32_val as f32 {
                    SerializedConstExpr::F32Const(f32_val.to_bits())
                } else if f64_val != f64::from(i32_val) {
                    SerializedConstExpr::F64Const(f64_val.to_bits())
                } else {
                    // Default to i32
                    SerializedConstExpr::I32Const(i32_val)
                }
            }
            crate::module::init_expr::Op::Global(global_op) => {
                SerializedConstExpr::GlobalGet(global_op.global_index)
            }
            crate::module::init_expr::Op::FuncRef(func_ref_op) => {
                SerializedConstExpr::RefFunc(func_ref_op.function_index)
            }
            crate::module::init_expr::Op::Expr(_) => {
                // For now, just serialize as i32 0 as a fallback
                SerializedConstExpr::I32Const(0)
            }
        }
    }
}

impl From<&SerializedConstExpr> for ConstExpr {
    fn from(expr: &SerializedConstExpr) -> Self {
        use crate::module::init_expr::{ConstOp, FuncRefOp, GlobalOp, Op};
        let op = match expr {
            SerializedConstExpr::I32Const(value) => Op::Const(ConstOp {
                value: (*value).into(),
            }),
            SerializedConstExpr::I64Const(value) => Op::Const(ConstOp {
                value: (*value).into(),
            }),
            SerializedConstExpr::F32Const(bits) => Op::Const(ConstOp {
                value: f32::from_bits(*bits).into(),
            }),
            SerializedConstExpr::F64Const(bits) => Op::Const(ConstOp {
                value: f64::from_bits(*bits).into(),
            }),
            SerializedConstExpr::V128Const(bytes) => {
                // Convert bytes to u128 and then to V128
                let mut u128_bytes = [0u8; 16];
                u128_bytes.copy_from_slice(bytes);
                #[cfg(feature = "simd")]
                {
                    let u128_val = u128::from_le_bytes(u128_bytes);
                    Op::Const(ConstOp {
                        value: crate::V128::from(u128_val).into(),
                    })
                }
                #[cfg(not(feature = "simd"))]
                {
                    // Fallback for when SIMD is not enabled
                    Op::Const(ConstOp { value: 0i32.into() })
                }
            }
            SerializedConstExpr::GlobalGet(global_idx) => Op::Global(GlobalOp {
                global_index: *global_idx,
            }),
            #[cfg(feature = "deserialization")]
            SerializedConstExpr::RefNull(_val_type) => {
                // Actually, not sure what to do here, so let's panic
                unimplemented!("not expecting null references during deserialization")
            }
            SerializedConstExpr::RefFunc(func_idx) => Op::FuncRef(FuncRefOp {
                function_index: *func_idx,
            }),
        };
        ConstExpr { op }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedGlobal {
    pub(crate) ty: SerializedGlobalType,
    pub(crate) init: SerializedConstExpr,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) enum SerializedExternType {
    Func(u32), // Function index for exports
    Table(SerializedTableType),
    Memory(SerializedMemoryType),
    Global(SerializedGlobalType), // Global type for imports
    GlobalIdx(u32),               // Global index for exports
}

impl From<&TableType> for SerializedExternType {
    fn from(table: &TableType) -> Self {
        Self::Table(SerializedTableType::from(table))
    }
}

impl From<&MemoryType> for SerializedExternType {
    fn from(value: &MemoryType) -> Self {
        Self::Memory(SerializedMemoryType::from(value))
    }
}

impl From<&GlobalType> for SerializedExternType {
    fn from(value: &GlobalType) -> Self {
        Self::Global(SerializedGlobalType::from(value))
    }
}

#[cfg(feature = "serialization")]
impl SerializedExternType {
    pub(crate) fn from_func_type(
        func_ty: &FuncType,
        ser_func_types: &[SerializedFuncType],
    ) -> Self {
        let ser_func_type = SerializedFuncType::from(func_ty);
        let func_idx = ser_func_types
            .iter()
            .position(|ser_fn_ty| ser_func_type == *ser_fn_ty)
            .expect("function type not found");
        SerializedExternType::Func(func_idx as u32)
    }

    pub(crate) fn from_func_idx(func_idx: u32) -> Self {
        SerializedExternType::Func(func_idx)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedTableType {
    pub(crate) element: SerializedValType,
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

impl From<&TableType> for SerializedTableType {
    fn from(table: &TableType) -> Self {
        SerializedTableType {
            element: SerializedValType::from(&table.element()),
            min: table.minimum() as u32,
            max: table.maximum().map(|m| m as u32),
        }
    }
}

impl From<&SerializedTableType> for TableType {
    fn from(table: &SerializedTableType) -> Self {
        TableType::new((&table.element).into(), table.min, table.max)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedMemoryType {
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

impl From<&MemoryType> for SerializedMemoryType {
    fn from(mem: &MemoryType) -> Self {
        let min = mem.minimum();
        let max = mem.maximum();
        SerializedMemoryType {
            min: min as u32,
            max: max.map(|m| m as u32),
        }
    }
}

impl From<&SerializedMemoryType> for MemoryType {
    fn from(mem: &SerializedMemoryType) -> Self {
        MemoryType::new(mem.min, mem.max)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedGlobalType {
    pub(crate) val_type: SerializedValType,
    pub(crate) mutable: bool,
}

impl From<&GlobalType> for SerializedGlobalType {
    fn from(value: &GlobalType) -> Self {
        Self {
            val_type: SerializedValType::from(&value.content()),
            mutable: value.mutability().is_mut(),
        }
    }
}

impl From<&SerializedGlobalType> for GlobalType {
    fn from(value: &SerializedGlobalType) -> Self {
        GlobalType::new(
            (&value.val_type).into(),
            if value.mutable {
                Mutability::Var
            } else {
                Mutability::Const
            },
        )
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedImport {
    pub(crate) module: alloc::string::String,
    pub(crate) name: alloc::string::String,
    pub(crate) ty: SerializedExternType,
}

#[cfg(feature = "serialization")]
impl SerializedImport {
    pub(crate) fn from_import(import: &ImportType, ser_func_types: &[SerializedFuncType]) -> Self {
        let module_name = import.module().to_owned();
        let name = import.name().to_owned();
        let ty = match import.ty() {
            ExternType::Table(table_ty) => SerializedExternType::from(table_ty),
            ExternType::Memory(mem_ty) => SerializedExternType::from(mem_ty),
            ExternType::Global(global_ty) => SerializedExternType::from(global_ty),
            ExternType::Func(func_ty) => {
                SerializedExternType::from_func_type(func_ty, ser_func_types)
            }
        };
        Self {
            module: module_name,
            name,
            ty,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedExport {
    pub(crate) name: alloc::string::String,
    pub(crate) ty: SerializedExternType,
}

#[cfg(feature = "serialization")]
impl SerializedExport {
    pub(crate) fn from_export_with_func_idx(name: &str, func_idx: u32) -> Self {
        SerializedExport {
            name: name.to_owned(),
            ty: SerializedExternType::from_func_idx(func_idx),
        }
    }

    pub(crate) fn from_export_with_memory_type(name: &str, memory_type: &MemoryType) -> Self {
        SerializedExport {
            name: name.to_owned(),
            ty: SerializedExternType::from(memory_type),
        }
    }

    pub(crate) fn from_export_with_global_idx(name: &str, global_idx: u32) -> Self {
        SerializedExport {
            name: name.to_owned(),
            ty: SerializedExternType::GlobalIdx(global_idx),
        }
    }
}

/// Serializable representation of an active data segment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedActiveDataSegment {
    pub(crate) memory_index: u32,
    pub(crate) offset: i32,
    pub(crate) bytes: Vec<u8>,
}

/// Serializable representation of a passive data segment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedPassiveDataSegment {
    pub(crate) bytes: Vec<u8>,
}

/// Serializable representation of either an active or passive data segment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) enum SerializedDataSegment {
    Active(SerializedActiveDataSegment),
    Passive(SerializedPassiveDataSegment),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub struct SerializedInternalFunc {
    pub(crate) type_idx: u32,
    pub(crate) len_registers: u16,
    pub(crate) consts: Vec<crate::core::UntypedVal>,
    pub(crate) instructions: Vec<crate::ir::Instruction>,
}

/// Serializable representation of an element segment.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialization", derive(serde::Deserialize))]
pub(crate) struct SerializedElementSegment {
    /// The table index this element segment applies to.
    pub(crate) table_index: u32,
    /// The offset in the table where elements should be placed.
    pub(crate) offset: SerializedConstExpr,
    /// The function indices to place in the table.
    pub(crate) function_indices: Vec<u32>,
}
