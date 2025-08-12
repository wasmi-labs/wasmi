mod engine;
mod stack;

#[cfg(all(test, feature = "parser"))]
mod tests;

pub use self::{
    engine::{EnforcedLimits, EnforcedLimitsError},
    stack::StackConfig,
};
