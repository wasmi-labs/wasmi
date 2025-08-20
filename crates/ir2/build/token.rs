use core::fmt::{self, Display};

/// Camel-case tokens, e.g. `HelloWorld`.
pub struct CamelCase<T>(pub T);

/// Snake-case tokens, e.g. `hello_world`.
pub struct SnakeCase<T>(pub T);

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
    Sub: sub,
    Mul: mul,
    Div: div,
    Rem: rem,
    Min: min,
    Max: max,
    Copysign: copysign,

    Shl: shl,
    Shr: shr,
    Rotl: rotl,
    Rotr: rotr,

    Eq: eq,
    And: and,
    Or: or,
    NotEq: ne,
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
    Select: select,
    Store8: store8,
    Store16: store16,
    Store32: store32,
    Store64: store64,
    Load8: load8,
    Load16: load16,
    Load32: load32,
    Load64: load64,

    Copy: copy,
    Fill: fill,
    Init: init,
    Grow: grow,
    Get: get,
    Set: set,

    Table: table,
    Memory: memory,
    Func: func,
    FuncType: func_type,
    Global: global,
    Elem: elem,
    Data: data,
    Trap: trap,

    Call: call,
    CallIndirect: call_indirect,
    ReturnCall: return_call,
    ReturnCallIndirect: return_call_indirect,

    U8: r#u8,
    U16: r#u16,
    U32: r#u32,
    U64: r#u64,
    I8: r#i8,
    I16: r#i16,
    I32: r#i32,
    I64: r#i64,
    S8: s8,
    S16: s16,
    S32: s32,
    S64: s64,
    F32: r#f32,
    F64: r#f64,
    Ref: r#ref,

    Not: not,
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
);
