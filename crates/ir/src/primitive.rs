use core::fmt::Debug;

/// A span of contiguous registers.
///
/// # Note
///
/// This type is unaware of its length and must be
/// provided with its associated length externally.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RegSpan {
    /// The first register of the register span.
    pub(crate) head: Reg,
}

impl RegSpan {
    /// Returns the head [`Reg`] for the [`RegSpan`].
    pub fn head(&self) -> Reg {
        self.head
    }
}

/// A `branch.table` target value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BranchTableTarget {
    /// The underlying value that signals to either return of branch with an offset.
    ///
    /// - A value of 0 signals an uninitialized branch offset during compilation.
    /// - A value of `i32::MIN` signals a returning branch.
    /// - Any other value signals a branch with the offset of `value`.
    pub(crate) value: i32,
}

impl BranchTableTarget {
    /// Create a new [`BranchTableTarget`] that signals to return the control flow.
    pub fn r#return() -> Self {
        Self { value: i32::MIN }
    }

    /// Creates a new uninitialized [`BranchTableTarget`].
    ///
    /// The returned [`BranchTableTarget`] is required to be initialized later via [`BranchTableTarget::init`].
    pub fn uninit() -> Self {
        Self { value: 0 }
    }

    /// Returns `true` if `self` is still uninitialized.
    pub fn is_uninit(&self) -> bool {
        self.value == 0
    }

    /// Initializes `self` with the given `offset`.
    ///
    /// # Note
    ///
    /// Here we allow an `offset` of 0 since the 0 value only signals an uninitialized state
    /// during Wasmi bytecode compilation but this method is supposed to be used right after
    /// compilation has ended.
    ///
    /// # Pancis
    ///
    /// - If `self` has already been initialized.
    /// - If `offset` is `i32::MIN` and thus signals a returning [`BranchTableTarget`].
    pub fn init(&mut self, offset: i32) {
        assert!(
            self.is_uninit(),
            "tried to initialize an already initialized `BranchTableTarget`"
        );
        assert_ne!(
            offset,
            i32::MIN,
            " a branch offset of i32::MIN is reserved for returns"
        );
        self.value = offset;
    }

    /// Creates a new [`BranchTableTarget`] with a concrete branch `offset`.
    ///
    /// # Pancis
    ///
    /// - If `offset` is 0 and thus signals an uninitialized [`BranchTableTarget`].
    /// - If `offset` is `i32::MIN` and thus signals a returning [`BranchTableTarget`].
    pub fn branch(offset: i32) -> Self {
        assert_ne!(
            offset, 0,
            "a branch offset of 0 is reserved for uninitialized state"
        );
        assert_ne!(
            offset,
            i32::MIN,
            " a branch offset of i32::MIN is reserved for returns"
        );
        Self { value: offset }
    }

    /// Returns `true` if this [`BranchTableTarget`] represents a return.
    pub fn is_return(&self) -> bool {
        self.value == i32::MIN
    }

    /// Returns the branch offset or `None` if this [`BranchTableTarget`] represents a return.
    ///
    /// # Note
    ///
    /// After compilation a branch offset of 0 signals a branch with an offset of 0 and is thus valid.
    pub fn get_branch(&self) -> Option<BranchOffset> {
        if self.is_return() {
            return None;
        }
        Some(BranchOffset(self.value))
    }
}

macro_rules! for_each_newtype {
    ($mac:ident) => {
        $mac! {
            /// A register that stores a single value.
            ///
            /// # Note
            ///
            /// - Positive values refer to registers on the function frame.
            /// - Negative values refer to function local constant values.
            struct Reg(pub i16);

            /// A 32-bit immediate value of unspecified type.
            struct Imm32(pub(crate) u32);

            /// A 64-bit immediate value of unspecified type.
            struct Imm64(pub(crate) u64);

            /// The index of an internal function.
            struct InternalFunc(pub u32);

            /// The index of a Wasm or host function.
            struct Func(pub u32);

            /// The index of a function type.
            struct FuncType(pub u32);

            /// The index of a Wasm table.
            struct Table(pub u32);

            /// The index of a Wasm element segment.
            struct ElementSegment(pub u32);

            /// The index of a Wasm data segment.
            struct DataSegment(pub u32);

            /// The index of a Wasm global variable.
            struct Global(pub u32);

            /// A relative offset in bytes for `load` and `store` operations.
            struct ByteOffset(pub u32);

            /// An address in bytes within a linear memory for `load` and `store` operations.
            struct Address(pub u32);

            /// An offset for branch instructions.
            ///
            /// This defines how much the instruction pointer is offset
            /// upon taking the respective branch.
            struct BranchOffset(pub i32);

            /// An amount of fuel consumed for executing a control flow basic block.
            struct BlockFuel(pub(crate) u32);

            /// A 32-bit immediate floating point value.
            struct Ieee32(pub(crate) u32);

            /// A 64-bit immediate floating point value.
            struct Ieee64(pub(crate) u64);
        }
    };
}
pub(crate) use for_each_newtype;

macro_rules! define_newtype {
    (
        $(
            $( #[$docs:meta] )*
            struct $name:ident($vis:vis $ty:ty);
        )*
    ) => {
        $(
            $( #[$docs] )*
            #[derive(
                ::core::fmt::Debug,
                ::core::marker::Copy,
                ::core::clone::Clone,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::cmp::PartialOrd,
                ::core::cmp::Ord,
            )]
            #[repr(transparent)]
            pub struct $name($vis $ty);

            impl ::core::convert::From<$ty> for $name {
                fn from(value: $ty) -> Self {
                    Self(value)
                }
            }

            impl ::core::convert::From<$name> for $ty {
                fn from(value: $name) -> Self {
                    value.0
                }
            }
        )*
    };
}
for_each_newtype!(define_newtype);

impl BlockFuel {
    /// Bump the fuel by `amount` if possible.
    ///
    /// Returns the new fuel amount of the [`BlockFuel`] after this operation.
    /// Returns `None` if the new fuel amount was out of bounds.
    pub fn bump_by(&mut self, amount: u64) -> Option<u32> {
        let new_amount: u32 = self.to_u64().checked_add(amount)?.try_into().ok()?;
        self.0 = new_amount;
        Some(new_amount)
    }

    /// Returns the index value as `u64`.
    pub fn to_u64(self) -> u64 {
        u64::from(self.0)
    }
}

impl From<i32> for Imm32 {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}

impl From<f32> for Imm32 {
    fn from(value: f32) -> Self {
        Self(value.to_bits())
    }
}

impl From<Imm32> for i32 {
    fn from(value: Imm32) -> Self {
        value.0 as _
    }
}

impl From<Imm32> for f32 {
    fn from(value: Imm32) -> Self {
        f32::from_bits(value.0)
    }
}

impl From<i64> for Imm64 {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<f64> for Imm64 {
    fn from(value: f64) -> Self {
        Self(value.to_bits())
    }
}

impl From<Imm64> for i64 {
    fn from(value: Imm64) -> Self {
        value.0 as _
    }
}

impl From<Imm64> for f64 {
    fn from(value: Imm64) -> Self {
        f64::from_bits(value.0)
    }
}

impl From<f32> for Ieee32 {
    fn from(value: f32) -> Self {
        Self(value.to_bits())
    }
}

impl From<Ieee32> for f32 {
    fn from(value: Ieee32) -> Self {
        f32::from_bits(value.0)
    }
}

impl From<f64> for Ieee64 {
    fn from(value: f64) -> Self {
        Self(value.to_bits())
    }
}

impl From<Ieee64> for f64 {
    fn from(value: Ieee64) -> Self {
        f64::from_bits(value.0)
    }
}

/// The signednesss of a value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Sign {
    /// The value is positive.
    Pos = 0,
    /// The value is negative.
    Neg = 1,
}

macro_rules! for_each_trap_code {
    ($mac:ident) => {
        $mac! {
            UnreachableCodeReached,
            MemoryOutOfBounds,
            TableOutOfBounds,
            IndirectCallToNull,
            IntegerDivisionByZero,
            IntegerOverflow,
            BadConversionToInteger,
            StackOverflow,
            BadSignature,
            OutOfFuel,
            GrowthOperationLimited,
        }
    };
}
pub(crate) use for_each_trap_code;

macro_rules! define_trap_code {
    ( $($name:ident),* $(,)? ) => {
        /// Equivalent to [`wasmi_core::TrapCode`].
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(u8)]
        pub enum TrapCode {
            $(
                #[doc = ::core::concat!("Equivalent to [`wasmi_core::TrapCode::", ::core::stringify!($name), "`].")]
                $name,
            )*
        }

        impl From<crate::core::TrapCode> for TrapCode {
            fn from(value: crate::core::TrapCode) -> Self {
                match value {
                    $(
                        crate::core::TrapCode::$name => Self::$name,
                    )*
                }
            }
        }

        impl From<TrapCode> for crate::core::TrapCode {
            fn from(value: TrapCode) -> Self {
                match value {
                    $(
                        TrapCode::$name => Self::$name,
                    )*
                }
            }
        }
    }
}
for_each_trap_code!(define_trap_code);
