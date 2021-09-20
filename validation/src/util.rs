use crate::Error;
use alloc::string::String;
use parity_wasm::elements::{Local, ValueType};

/// Locals are the concatenation of a slice of function parameters
/// with function declared local variables.
///
/// Local variables are given in the form of groups represented by pairs
/// of a value_type and a count.
#[derive(Debug)]
pub struct Locals<'a> {
    params: &'a [ValueType],
    local_groups: &'a [Local],
    count: u32,
}

impl<'a> Locals<'a> {
    /// Create a new wrapper around declared variables and parameters.
    pub fn new(params: &'a [ValueType], local_groups: &'a [Local]) -> Result<Locals<'a>, Error> {
        let mut acc = params.len() as u32;
        for locals_group in local_groups {
            acc = acc
                .checked_add(locals_group.count())
                .ok_or_else(|| Error(String::from("Locals range not in 32-bit range")))?;
        }

        Ok(Locals {
            params,
            local_groups,
            count: acc,
        })
    }

    /// Returns parameter count.
    pub fn param_count(&self) -> u32 {
        self.params.len() as u32
    }

    /// Returns total count of all declared locals and paramaterers.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Returns the type of a local variable (either a declared local or a param).
    ///
    /// Returns `Err` in the case of overflow or when idx falls out of range.
    pub fn type_of_local(&self, idx: u32) -> Result<ValueType, Error> {
        if let Some(param) = self.params.get(idx as usize) {
            return Ok(*param);
        }

        // If an index doesn't point to a param, then we have to look into local declarations.
        let mut start_idx = self.param_count();
        for locals_group in self.local_groups {
            let end_idx = start_idx
                .checked_add(locals_group.count())
                .ok_or_else(|| Error(String::from("Locals range not in 32-bit range")))?;

            if idx >= start_idx && idx < end_idx {
                return Ok(locals_group.value_type());
            }

            start_idx = end_idx;
        }

        // We didn't find anything, that's an error.
        // At this moment `start_idx` should hold the count of all locals
        // (since it's either set to the `end_idx` or equal to `params.len()`)
        let total_count = start_idx;

        Err(Error(format!(
            "Trying to access local with index {} when there are only {} locals",
            idx, total_count
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn locals_it_works() {
        let params = vec![ValueType::I32, ValueType::I64];
        let local_groups = vec![Local::new(2, ValueType::F32), Local::new(2, ValueType::F64)];
        let locals = Locals::new(&params, &local_groups).unwrap();

        assert_matches!(locals.type_of_local(0), Ok(ValueType::I32));
        assert_matches!(locals.type_of_local(1), Ok(ValueType::I64));
        assert_matches!(locals.type_of_local(2), Ok(ValueType::F32));
        assert_matches!(locals.type_of_local(3), Ok(ValueType::F32));
        assert_matches!(locals.type_of_local(4), Ok(ValueType::F64));
        assert_matches!(locals.type_of_local(5), Ok(ValueType::F64));
        assert_matches!(locals.type_of_local(6), Err(_));
    }

    #[test]
    fn locals_no_declared_locals() {
        let params = vec![ValueType::I32];
        let locals = Locals::new(&params, &[]).unwrap();

        assert_matches!(locals.type_of_local(0), Ok(ValueType::I32));
        assert_matches!(locals.type_of_local(1), Err(_));
    }

    #[test]
    fn locals_no_params() {
        let local_groups = vec![Local::new(2, ValueType::I32), Local::new(3, ValueType::I64)];
        let locals = Locals::new(&[], &local_groups).unwrap();

        assert_matches!(locals.type_of_local(0), Ok(ValueType::I32));
        assert_matches!(locals.type_of_local(1), Ok(ValueType::I32));
        assert_matches!(locals.type_of_local(2), Ok(ValueType::I64));
        assert_matches!(locals.type_of_local(3), Ok(ValueType::I64));
        assert_matches!(locals.type_of_local(4), Ok(ValueType::I64));
        assert_matches!(locals.type_of_local(5), Err(_));
    }

    #[test]
    fn locals_u32_overflow() {
        let local_groups = vec![
            Local::new(u32::max_value(), ValueType::I32),
            Local::new(1, ValueType::I64),
        ];
        assert_matches!(Locals::new(&[], &local_groups), Err(_));
    }
}
