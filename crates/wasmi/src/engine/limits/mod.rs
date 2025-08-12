mod engine;
mod stack;

#[cfg(test)]
mod tests;

#[expect(deprecated)]
pub use self::stack::StackLimits;
pub use self::{
    engine::{EnforcedLimits, EnforcedLimitsError},
    stack::StackConfig,
};
