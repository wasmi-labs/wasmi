use crate::{
    core::{FuelCostsProvider, Typed, TypedVal, ValType},
    ir::{Const16, Instruction, Sign},
    Error,
    ExternRef,
    FuncRef,
};

macro_rules! impl_typed_for {
    ( $( $ty:ident ),* $(,)? ) => {
        $(
            impl Typed for $ty {
                const TY: ValType = crate::core::ValType::$ty;
            }

            impl From<TypedVal> for $ty {
                fn from(typed_value: TypedVal) -> Self {
                    // # Note
                    //
                    // We only use a `debug_assert` here instead of a proper `assert`
                    // since the whole translation process assumes that Wasm validation
                    // was already performed and thus type checking does not necessarily
                    // need to happen redundantly outside of debug builds.
                    debug_assert!(matches!(typed_value.ty(), <$ty as Typed>::TY));
                    Self::from(typed_value.untyped())
                }
            }
        )*
    };
}
impl_typed_for! {
    FuncRef,
    ExternRef,
}

/// A WebAssembly integer. Either `i32` or `i64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmInteger:
    Copy + Eq + From<TypedVal> + Into<TypedVal> + TryInto<Const16<Self>>
{
    /// Returns `true` if `self` is equal to zero (0).
    fn eq_zero(self) -> bool;
}

impl WasmInteger for i32 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for u32 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for i64 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for u64 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

/// A WebAssembly float. Either `f32` or `f64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmFloat: Copy + Into<TypedVal> + From<TypedVal> {
    /// Returns `true` if `self` is any kind of NaN value.
    fn is_nan(self) -> bool;

    /// Returns the [`Sign`] of `self`.
    fn sign(self) -> Sign<Self>;
}

impl WasmFloat for f32 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn sign(self) -> Sign<Self> {
        Sign::from(self)
    }
}

impl WasmFloat for f64 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn sign(self) -> Sign<Self> {
        Sign::from(self)
    }
}

/// Implemented by integer types to wrap them to another (smaller) integer type.
pub trait Wrap<T> {
    /// Wraps `self` into a value of type `T`.
    fn wrap(self) -> T;
}

impl<T> Wrap<T> for T {
    #[inline]
    fn wrap(self) -> T {
        self
    }
}

macro_rules! impl_wrap_for {
    ( $($from_ty:ty => $to_ty:ty),* $(,)? ) => {
        $(
            impl Wrap<$to_ty> for $from_ty {
                #[inline]
                fn wrap(self) -> $to_ty { self as _ }
            }
        )*
    };
}
impl_wrap_for! {
    // signed
    i16 => i8,
    i32 => i8,
    i32 => i16,
    i64 => i8,
    i64 => i16,
    i64 => i32,
    // unsigned
    u16 => u8,
    u32 => u8,
    u32 => u16,
    u64 => u8,
    u64 => u16,
    u64 => u32,
}

/// Fuel metering information for a certain translation state.
#[derive(Debug, Clone)]
pub enum FuelInfo {
    /// Fuel metering is disabled.
    None,
    /// Fuel metering is enabled with the following information.
    Some {
        /// The [`FuelCostsProvider`] for the function translation.
        costs: FuelCostsProvider,
        /// Index to the current [`Instruction::ConsumeFuel`] of a parent Wasm control frame.
        instr: Instr,
    },
}

impl FuelInfo {
    /// Create a new [`FuelInfo`] for enabled fuel metering.
    pub fn some(costs: FuelCostsProvider, instr: Instr) -> Self {
        Self::Some { costs, instr }
    }
}

/// Extension trait to bump the consumed fuel of [`Instruction::ConsumeFuel`].
pub trait BumpFuelConsumption {
    /// Increases the fuel consumption of the [`Instruction::ConsumeFuel`] instruction by `delta`.
    ///
    /// # Error
    ///
    /// - If `self` is not a [`Instruction::ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error>;
}

impl BumpFuelConsumption for Instruction {
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error> {
        match self {
            Self::ConsumeFuel { block_fuel } => block_fuel.bump_by(delta).map_err(Error::from),
            instr => panic!("expected `Instruction::ConsumeFuel` but found: {instr:?}"),
        }
    }
}

/// Extension trait to query if an [`Instruction`] is a parameter.
pub trait IsInstructionParameter {
    /// Returns `true` if `self` is a parameter to an [`Instruction`].
    fn is_instruction_parameter(&self) -> bool;
}

impl IsInstructionParameter for Instruction {
    #[rustfmt::skip]
    fn is_instruction_parameter(&self) -> bool {
        matches!(self,
            | Self::TableIndex { .. }
            | Self::MemoryIndex { .. }
            | Self::DataIndex { .. }
            | Self::ElemIndex { .. }
            | Self::Const32 { .. }
            | Self::I64Const32 { .. }
            | Self::F64Const32 { .. }
            | Self::BranchTableTarget { .. }
            | Self::BranchTableTargetNonOverlapping { .. }
            | Self::Imm16AndImm32 { .. }
            | Self::RegisterAndImm32 { .. }
            | Self::RegisterSpan { .. }
            | Self::Register { .. }
            | Self::Register2 { .. }
            | Self::Register3 { .. }
            | Self::RegisterList { .. }
            | Self::CallIndirectParams { .. }
            | Self::CallIndirectParamsImm16 { .. }
        )
    }
}

/// A reference to an encoded [`Instruction`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instr(u32);

impl From<u32> for Instr {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl From<Instr> for u32 {
    fn from(instr: Instr) -> Self {
        instr.0
    }
}

impl Instr {
    /// Creates an [`Instr`] from the given `usize` value.
    ///
    /// # Note
    ///
    /// This intentionally is an API intended for test purposes only.
    ///
    /// # Panics
    ///
    /// If the `value` exceeds limitations for [`Instr`].
    pub fn from_usize(value: usize) -> Self {
        let Ok(index) = u32::try_from(value) else {
            panic!("out of bounds index {value} for `Instr`")
        };
        Self(index)
    }

    /// Returns an `usize` representation of the instruction index.
    pub fn into_usize(self) -> usize {
        match usize::try_from(self.0) {
            Ok(index) => index,
            Err(error) => {
                panic!("out of bound index {} for `Instr`: {error}", self.0)
            }
        }
    }

    /// Returns the absolute distance between `self` and `other`.
    ///
    /// - Returns `0` if `self == other`.
    /// - Returns `1` if `self` is adjacent to `other` in the sequence of instructions.
    /// - etc..
    pub fn distance(self, other: Self) -> u32 {
        self.0.abs_diff(other.0)
    }
}
