use crate::{
    ir::{Op, Slot},
    Error,
};

/// Extension trait for [`Op`] to conditionally relink result [`Slot`]s.
pub trait RelinkResult {
    /// Relinks the result [`Slot`] of `self` to `new_result` if its current `result` [`Slot`] equals `old_result`.
    ///
    /// # Note (Return Value)
    ///
    /// - `Ok(true)`: the result has been relinked
    /// - `Ok(false)`: the result has _not_ been relinked
    /// - `Err(_)`: translation error
    fn relink_result(&mut self, new_result: Slot, old_result: Slot) -> Result<bool, Error>;
}

impl RelinkResult for Op {
    fn relink_result(&mut self, new_result: Slot, old_result: Slot) -> Result<bool, Error> {
        let Some(result_mut) = self.result_mut() else {
            return Ok(false);
        };
        if *result_mut != old_result {
            return Ok(false);
        }
        *result_mut = new_result;
        Ok(true)
    }
}
