mod engine;
mod stack;

#[cfg(test)]
mod tests;

// use self::{
//     engine::{EngineLimits, EngineLimitsError, AvgBytesPerFunctionLimit},
//     stack::{StackLimits, LimitsError},
// };
pub use self::{
    engine::{EngineLimits, EngineLimitsError},
    stack::{LimitsError, StackLimits},
};
