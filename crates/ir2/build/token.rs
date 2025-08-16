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
    Sdiv: sdiv,
    Udiv: udiv,
    Srem: srem,
    Urem: urem,
    Min: min,
    Max: max,
    Copysign: copysign,

    Shl: shl,
    Sshr: sshr,
    Ushr: ushr,
    Rotl: rotl,
    Rotr: rotr,

    Eq: eq,
    Ne: ne,
    Slt: slt,
    Ult: ult,
    Sle: sle,
    Ule: ule,
    Lt: lt,
    Le: le,
    NotLt: not_lt,
    NotLe: not_le,

    And: and,
    Or: or,
    NotAnd: not_and,
    NotOr: not_or,

    BitAnd: bit_and,
    BitOr: bit_or,
    BitXor: bit_xor,

    Popcnt: popcnt,
    Clz: clz,
    Ctz: ctz,

    Not: not,
    Abs: abs,
    Neg: neg,
    Ceil: ceil,
    Floor: floor,
    Trunc: trunc,
    Nearest: nearest,
    Sqrt: sqrt,

    Return: r#return,
    Branch: branch,
    Select: select,
    Store: store,
    Load: load,

    Copy: copy,
    Fill: fill,
    Init: init,
    Grow: grow,
    Get: get,
    Set: set,

    Table: table,
    Memory: memory,
    Func: func,
    Global: global,
    Elem: elem,
    Data: data,
    Trap: trap,

    Call: call,
    CallIndirect: call_indirect,
    ReturnCall: return_call,
    ReturnCallIndirect: return_call_indirect,

    I32: r#i32,
    I64: r#i64,
    F32: r#f32,
    F64: r#f64,
    Ref: r#ref,
);
