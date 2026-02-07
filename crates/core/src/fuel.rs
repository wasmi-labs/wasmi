use alloc::sync::Arc;
use core::{
    error::Error,
    fmt::{self, Debug},
    mem,
    num::NonZeroU64,
};

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

/// Implementation of default [`FuelCostsProvider`].
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
#[derive(Default, Clone)]
pub struct FuelCostsProvider {
    /// Optional custom fuel costs.
    custom: Option<Arc<dyn FuelCosts + Send + Sync>>,
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

    /// Returns the fuel costs for `len_bytes` byte copies in Wasmi IR.
    ///
    /// # Note
    ///
    /// - On overflow this returns [`u64::MAX`].
    /// - The following Wasmi IR instructions may make use of this:
    ///     - calls (parameter passing)
    ///     - `memory.grow`
    ///     - `memory.copy`
    ///     - `memory.fill`
    ///     - `memory.init`
    ///     - `copy_span`
    ///     - `copy_many`
    ///     - `return_span`
    ///     - `return_many`
    ///     - `table.grow` (+ variants)
    ///     - `table.copy` (+ variants)
    ///     - `table.fill` (+ variants)
    ///     - `table.init` (+ variants)
    fn fuel_for_copying_bytes(&self, len_bytes: u64) -> u64 {
        len_bytes / self.bytes_per_fuel()
    }

    /// Returns the fuel costs for copying `len` items of type `T`.
    ///
    /// # Note
    ///
    /// - On overflow this returns [`u64::MAX`].
    pub fn fuel_for_copying_values<T>(&self, len_values: u64) -> u64 {
        let Ok(bytes_per_value) = u64::try_from(mem::size_of::<T>()) else {
            return u64::MAX;
        };
        let len_bytes = len_values.saturating_mul(bytes_per_value);
        self.fuel_for_copying_bytes(len_bytes)
    }
}

/// An error that may be encountered when using [`Fuel`].
#[derive(Debug, Clone)]
pub enum FuelError {
    /// Returned by some [`Fuel`] methods when fuel metering is disabled.
    FuelMeteringDisabled,
    /// Raised when trying to consume more fuel than is available.
    OutOfFuel { required_fuel: u64 },
}

impl Error for FuelError {}

impl fmt::Display for FuelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FuelMeteringDisabled => write!(f, "fuel metering is disabled"),
            Self::OutOfFuel { required_fuel } => write!(f, "ouf of fuel. required={required_fuel}"),
        }
    }
}

impl FuelError {
    /// Returns an error indicating that fuel metering has been disabled.
    ///
    /// # Note
    ///
    /// This method exists to indicate that this execution path is cold.
    #[cold]
    pub fn fuel_metering_disabled() -> Self {
        Self::FuelMeteringDisabled
    }

    /// Returns an error indicating that too much fuel has been consumed.
    ///
    /// # Note
    ///
    /// This method exists to indicate that this execution path is cold.
    #[cold]
    pub fn out_of_fuel(required_fuel: u64) -> Self {
        Self::OutOfFuel { required_fuel }
    }
}

/// The remaining and consumed fuel counters.
#[derive(Debug)]
pub struct Fuel {
    /// The remaining fuel.
    remaining: u64,
    /// This is `true` if fuel metering is enabled.
    enabled: bool,
    /// The fuel costs.
    costs: FuelCostsProvider,
}

impl Fuel {
    /// Creates a new [`Fuel`].
    pub fn new(enabled: bool, costs: FuelCostsProvider) -> Self {
        Self {
            remaining: 0,
            enabled,
            costs,
        }
    }

    /// Returns `true` if fuel metering is enabled.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns `Ok` if fuel metering is enabled.
    ///
    /// Returns descriptive [`FuelError`] otherwise.
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    fn check_fuel_metering_enabled(&self) -> Result<(), FuelError> {
        if !self.is_fuel_metering_enabled() {
            return Err(FuelError::fuel_metering_disabled());
        }
        Ok(())
    }

    /// Sets the remaining fuel to `fuel`.
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), FuelError> {
        self.check_fuel_metering_enabled()?;
        self.remaining = fuel;
        Ok(())
    }

    /// Returns the remaining fuel.
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, FuelError> {
        self.check_fuel_metering_enabled()?;
        Ok(self.remaining)
    }

    /// Synthetically consumes an amount of [`Fuel`].
    ///
    /// Returns the remaining amount of [`Fuel`] after this operation.
    ///
    /// # Note
    ///
    /// - This does _not_ check if fuel metering is enabled.
    /// - This API is intended for use cases where it is clear that fuel metering is
    ///   enabled and where a check would incur unnecessary overhead in a hot path.
    ///   An example of this is the execution of consume fuel instructions since
    ///   those only exist if fuel metering is enabled.
    ///
    /// # Errors
    ///
    /// If out of fuel.
    pub fn consume_fuel_unchecked(&mut self, delta: u64) -> Result<u64, FuelError> {
        self.remaining = self
            .remaining
            .checked_sub(delta)
            .ok_or(FuelError::out_of_fuel(delta))?;
        Ok(self.remaining)
    }

    /// Consumes an amount of [`Fuel`].
    ///
    /// Returns the remaining amount of [`Fuel`] after this operation.
    ///
    /// # Errors
    ///
    /// - If fuel metering is disabled.
    /// - If out of fuel.
    pub fn consume_fuel(
        &mut self,
        f: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<u64, FuelError> {
        self.check_fuel_metering_enabled()?;
        self.consume_fuel_unchecked(f(&self.costs))
    }

    /// Consumes an amount of [`Fuel`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// This does nothing if fuel metering is disabled.
    ///
    /// # Errors
    ///
    /// - If out of fuel.
    pub fn consume_fuel_if(
        &mut self,
        f: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), FuelError> {
        if !self.is_fuel_metering_enabled() {
            return Ok(());
        }
        self.consume_fuel_unchecked(f(&self.costs))?;
        Ok(())
    }
}
