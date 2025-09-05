use core::fmt::{self, Display};

#[derive(Copy, Clone)]
pub enum Case {
    Camel,
    Snake,
}

impl Case {
    pub fn wrap<T>(self, value: T) -> ChosenCase<T> {
        match self {
            Self::Camel => ChosenCase::Camel(value),
            Self::Snake => ChosenCase::Snake(value),
        }
    }
}

/// Runtime selected casing, either [`CamelCase`] or [`SnakeCase`].
#[derive(Copy, Clone)]
pub enum ChosenCase<T> {
    Camel(T),
    Snake(T),
}

impl<T> Display for ChosenCase<T>
where
    CamelCase<T>: Display,
    SnakeCase<T>: Display,
    T: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Camel(value) => write!(f, "{}", CamelCase(value.clone())),
            Self::Snake(value) => write!(f, "{}", SnakeCase(value.clone())),
        }
    }
}

/// Camel-case tokens, e.g. `HelloWorld`.
pub struct CamelCase<T>(pub T);

/// Snake-case tokens, e.g. `hello_world`.
pub struct SnakeCase<T>(pub T);

/// A word separator as required by some casings, e.g. snake case uses `_`.
#[derive(Copy, Clone)]
pub struct Sep;

impl Display for CamelCase<Sep> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for SnakeCase<Sep> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}

macro_rules! define_ident {
    (
        $(
            $camel_ident:ident: $snake_ident:ident
        ),* $(,)?
    ) => {
        #[derive(Copy, Clone)]
        pub enum Ident {
            $( $camel_ident ),*
        }

        impl Display for CamelCase<Ident> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let frag: &'static str = match self.0 {
                    $(
                        Ident::$camel_ident => stringify!($camel_ident),
                    )*
                };
                write!(f, "{frag}")
            }
        }

        impl Display for SnakeCase<Ident> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let frag: &'static str = match self.0 {
                    $(
                        Ident::$camel_ident => stringify!($snake_ident),
                    )*
                };
                write!(f, "{frag}")
            }
        }
    };
}
define_ident!(
    Add: add,
    AddSat: add_sat,
    Sub: sub,
    SubSat: sub_sat,
    Mul: mul,
    Div: div,
    Rem: rem,
    Min: min,
    Max: max,
    Pmin: pmin,
    Pmax: pmax,
    Copysign: copysign,
    Avgr: avgr,

    Shl: shl,
    Shr: shr,
    Rotl: rotl,
    Rotr: rotr,

    Eq: eq,
    And: and,
    AndNot: and_not,
    Or: or,
    Xor: xor,
    NotEq: not_eq,
    NotAnd: not_and,
    NotOr: not_or,
    Lt: lt,
    Le: le,
    NotLt: not_lt,
    NotLe: not_le,

    BitAnd: bit_and,
    BitOr: bit_or,
    BitXor: bit_xor,

    Branch: branch,
    BranchTable: branch_table,
    BranchTableSpan: branch_table_span,
    Select: select,
    Store8: store8,
    Store16: store16,
    Store32: store32,
    Store64: store64,
    Store128: store128,
    Load8: load8,
    Load16: load16,
    Load32: load32,
    Load64: load64,
    Load: load,
    Load8x8: load8x8,
    Load16x4: load16x4,
    Load32x2: load32x2,
    Load8Splat: load8_splat,
    Load16Splat: load16_splat,
    Load32Splat: load32_splat,
    Load64Splat: load64_splat,
    Load32Zero: load32_zero,
    Load64Zero: load64_zero,

    Copy: copy,
    Copy32: copy32,
    Copy64: copy64,
    CopySpan: copy_span,

    Table: table,
    Memory: memory,
    Func: func,
    FuncType: func_type,
    Global: global,
    Elem: elem,
    Data: data,
    Trap: trap,

    CallInternal: call_internal,
    CallImported: call_imported,
    CallIndirect: call_indirect,
    ReturnCallInternal: return_call_internal,
    ReturnCallImported: return_call_imported,
    ReturnCallIndirect: return_call_indirect,

    I32: i32,
    I64: i64,

    Clz: clz,
    Ctz: ctz,
    Popcnt: popcnt,
    Wrap: wrap,
    Sext8: sext8,
    Sext16: sext16,
    Sext32: sext32,
    Abs: abs,
    Neg: neg,
    Ceil: ceil,
    Floor: floor,
    Trunc: trunc,
    TruncSat: trunc_sat,
    Nearest: nearest,
    Sqrt: sqrt,
    Demote: demote,
    Promote: promote,
    Convert: convert,

    Offset: offset,
    TrapCode: trap_code,
    ConsumeFuel: consume_fuel,
    Fuel: fuel,
    Return: r#return,
    Return32: return32,
    Return64: return64,
    ReturnSlot: return_slot,
    ReturnSpan: return_span,
    Values: values,
    Value: value,
    Result: result,
    Results: results,
    Len: len,
    LenTargets: len_targets,
    LenValues: len_values,
    Delta: delta,
    Dst: dst,
    Src: src,
    Index: index,
    DstMemory: dst_memory,
    SrcMemory: src_memory,
    DstTable: dst_table,
    SrcTable: src_table,
    TableGet: table_get,
    TableSet: table_set,
    TableSize: table_size,
    TableGrow: table_grow,
    TableCopy: table_copy,
    TableFill: table_fill,
    TableInit: table_init,
    ElemDrop: elem_drop,
    DataDrop: data_drop,
    MemoryGrow: memory_grow,
    MemorySize: memory_size,
    MemoryCopy: memory_copy,
    MemoryFill: memory_fill,
    MemoryInit: memory_init,
    GlobalGet: global_get,
    GlobalSet: global_set,
    GlobalSet32: global_set32,
    GlobalSet64: global_set64,
    RefFunc: ref_func,
    Mem0: mem0,
    Offset16: offset16,

    I64Add128: i64_add128,
    I64Sub128: i64_sub128,
    S64MulWide: s64_mul_wide,
    U64MulWide: u64_mul_wide,

    Lhs: lhs,
    Rhs: rhs,
    LhsLo: lhs_lo,
    LhsHi: lhs_hi,
    RhsLo: rhs_lo,
    RhsHi: rhs_hi,
    Ptr: ptr,
    ValTrue: val_true,
    ValFalse: val_false,

    Copy128: copy128,
    ValueLo: value_lo,
    ValueHi: value_hi,
    Selector: selector,

    V128: v128,
    Lane: lane,
    Splat: splat,
    S8x16ExtractLane: s8x16_extract_lane,
    U8x16ExtractLane: u8x16_extract_lane,
    S16x8ExtractLane: s16x8_extract_lane,
    U16x8ExtractLane: u16x8_extract_lane,
    U32x4ExtractLane: u32x4_extract_lane,
    U64x2ExtractLane: u64x2_extract_lane,
    ReplaceLane: replace_lane,
    LoadLane: load_lane,
    Swizzle: swizzle,
    I8x16Shuffle: i8x16_shuffle,
    Q15MulrSat: q15_mulr_sat,
    NarrowI16x8: narrow_i16x8,
    NarrowI32x4: narrow_i32x4,
    ExtmulLowI8x16: extmul_low_i8x16,
    ExtmulHighI8x16: extmul_high_i8x16,
    ExtmulLowI16x8: extmul_low_i16x8,
    ExtmulHighI16x8: extmul_high_i16x8,
    ExtmulLowI32x4: extmul_low_i32x4,
    ExtmulHighI32x4: extmul_high_i32x4,
    Not: not,
    AnyTrue: any_true,
    DotI16x8: dot_i16x8,
    AllTrue: all_true,
    Bitmask: bitmask,
    ExtaddPairwise: extadd_pairwise,
    ExtendLow: extend_low,
    ExtendHigh: extend_high,
    DemoteZero: demote_zero,
    PromoteLow: promote_low,
    TruncSatZero: trunc_sat_zero,
    ConvertLow: convert_low,
    Store8Lane: store8_lane,
    Store16Lane: store16_lane,
    Store32Lane: store32_lane,
    Store64Lane: store64_lane,

    RelaxedDotI8x16I7x16: relaxed_dot_i8x16_i7x16,
    RelaxedDotI8x16I7x16Add: relaxed_dot_i8x16_i7x16_add,
    RelaxedMadd: relaxed_madd,
    RelaxedNmadd: relaxed_nmadd,
);
