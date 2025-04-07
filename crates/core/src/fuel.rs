use crate::UntypedVal;
use alloc::boxed::Box;
use core::{fmt, fmt::Debug, mem, num::NonZeroU64};

/// Fuel costs for Wasmi IR instructions.
pub trait FuelCosts {
    /// Returns the base fuel costs for all Wasmi IR instructions.
    fn base(&self) -> u64;

    /// Returns the base fuel costs for all Wasmi IR `load` instructions.
    fn load(&self) -> u64 {
        self.base()
    }

    /// Returns the base fuel costs for all Wasmi IR `instance` instructions.
    ///
    /// # Note
    ///
    /// Entity-based instructions access or modify instance related data,
    /// such as globals, memories, tables or functions.
    fn instance(&self) -> u64 {
        self.base()
    }

    /// Returns the base fuel costs for all Wasmi IR `store` instructions.
    fn store(&self) -> u64 {
        self.base()
    }

    /// Returns the base fuel costs for all Wasmi IR `call` instructions.
    fn call(&self) -> u64 {
        self.base()
    }

    /// Returns the base fuel costs for all Wasmi IR `simd` instructions.
    fn simd(&self) -> u64 {
        self.base()
    }

    /// Returns the amount of bytes that can be copied for a single unit of fuel.
    fn bytes_per_fuel(&self) -> NonZeroU64;
}

/// Implementation of default [`FuelCosts`].
struct DefaultFuelCosts;

impl FuelCosts for DefaultFuelCosts {
    fn base(&self) -> u64 {
        1
    }

    fn bytes_per_fuel(&self) -> NonZeroU64 {
        NonZeroU64::new(64).unwrap()
    }
}

/// Type storing all kinds of fuel costs of instructions.
pub struct FuelCostsProvider {
    /// Optional custom fuel costs.
    custom: Option<Box<dyn FuelCosts>>,
}

impl Default for FuelCostsProvider {
    fn default() -> Self {
        Self { custom: None }
    }
}

impl Debug for FuelCostsProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = self.base();
        let instance = self.instance();
        let load = self.load();
        let store = self.store();
        let call = self.call();
        let simd = self.simd();
        let bytes_per_fuel = self.bytes_per_fuel();
        f.debug_struct("FuelCostsProvider")
            .field("base", &base)
            .field("instance", &instance)
            .field("load", &load)
            .field("store", &store)
            .field("call", &call)
            .field("simd", &simd)
            .field("bytes_per_fuel", &bytes_per_fuel)
            .finish()
    }
}

impl FuelCostsProvider {
    /// Applies `f` to either `self.custom` or [`DefaultFuelCosts`] if `self.custom` is `None`.
    fn apply(&self, f: impl FnOnce(&dyn FuelCosts) -> u64) -> u64 {
        match self.custom.as_deref() {
            Some(costs) => f(costs),
            None => f(&DefaultFuelCosts),
        }
    }

    /// Returns the base fuel costs for all Wasmi IR instructions.
    pub fn base(&self) -> u64 {
        self.apply(|c| c.base())
    }

    /// Returns the fuel costs for all Wasmi IR `instance` related instructions.
    pub fn instance(&self) -> u64 {
        self.apply(|c: &dyn FuelCosts| c.instance())
    }

    /// Returns the fuel costs for all Wasmi IR `load` instructions.
    pub fn load(&self) -> u64 {
        self.apply(|c: &dyn FuelCosts| c.load())
    }

    /// Returns the fuel costs for all Wasmi IR `store` instructions.
    pub fn store(&self) -> u64 {
        self.apply(|c: &dyn FuelCosts| c.store())
    }

    /// Returns the fuel costs for all Wasmi IR `call` instructions.
    pub fn call(&self) -> u64 {
        self.apply(|c: &dyn FuelCosts| c.call())
    }

    /// Returns the fuel costs for all Wasmi IR `simd` instructions.
    pub fn simd(&self) -> u64 {
        self.apply(|c: &dyn FuelCosts| c.simd())
    }

    /// Returns the number of bytes that can be copied per unit of fuel.
    fn bytes_per_fuel(&self) -> NonZeroU64 {
        match self.custom.as_deref() {
            Some(costs) => costs.bytes_per_fuel(),
            None => DefaultFuelCosts.bytes_per_fuel(),
        }
    }

    /// Returns the fuel costs for `len_copies` register copies in Wasmi IR.
    ///
    /// # Note
    ///
    /// - On overflow this returns [`u64::MAX`].
    /// - The following Wasmi IR instructions may make use of this:
    ///     - `memory.grow`
    ///     - `memory.copy`
    ///     - `memory.fill`
    ///     - `memory.init`
    pub fn fuel_for_copying_bytes(&self, len_bytes: u64) -> u64 {
        len_bytes / self.bytes_per_fuel()
    }

    /// Returns the fuel costs for copying `len_copies` [`UntypedVal`] items.
    ///
    /// # Note
    ///
    /// - On overflow this returns [`u64::MAX`].
    /// - [`UntypedVal`] might be 64-bit or 128-bit depending on the crate's config.
    /// - The following Wasmi IR instructions may make use of this:
    ///     - calls (parameter passing)
    ///     - `copy_span`
    ///     - `copy_many`
    ///     - `return_span`
    ///     - `return_many`
    ///     - `table.grow` (+ variants)
    ///     - `table.copy` (+ variants)
    ///     - `table.fill` (+ variants)
    ///     - `table.init` (+ variants)
    pub fn fuel_for_copying_values(&self, len_copies: u64) -> u64 {
        let Ok(size_of_val) = u64::try_from(mem::size_of::<UntypedVal>()) else {
            return u64::MAX;
        };
        self.fuel_for_copying_bytes(len_copies)
            .saturating_mul(size_of_val)
    }
}
