use alloc::vec::Vec;
use core::cmp::Ordering;
use wasmi_core::ValueType;

/// A registry where local variables of a function are registered and resolved.
///
/// # Note
///
/// Note that in WebAssembly function parameters are also local variables.
///
/// The locals registry efficiently registers and resolves local variables.
/// The problem is that the Wasm specification allows to encode up to `u32::MAX`
/// local variables in a small and constant space via the binary encoding.
/// Therefore we need a way to efficiently cope with this worst-case scenario
/// in order to protect the `wasmi` interpreter against exploitations.
///
/// This implementation allows to access local variables in this worst-case
/// scenario with a worst time complexity of O(log n) and space requirement
/// of O(m + n) where n is the number of registered groups of local variables
/// and m is the number of actually used local variables.
///
/// Besides that local variable usages are cached to further minimize potential
/// exploitation impact.
#[derive(Debug, Default)]
pub struct LocalsRegistry {
    /// An efficient store for the registered local variable groups.
    groups: Vec<LocalGroup>,
    /// Max local index.
    max_index: u32,
}

/// A group of local values as encoded in the Wasm binary.
#[derive(Debug, Copy, Clone)]
pub struct LocalGroup {
    /// The (included) minimum local index of the local variable group.
    min_index: u32,
    /// The (excluded) maximum local index of the local variable group.
    max_index: u32,
    /// The shared [`ValueType`] of the local variables in the group.
    value_type: ValueType,
}

impl LocalGroup {
    /// Creates a new [`LocalGroup`] with the given `amount` of local of shared [`ValueType`].
    pub fn new(value_type: ValueType, min_index: u32, max_index: u32) -> Self {
        assert!(min_index < max_index);
        Self {
            value_type,
            min_index,
            max_index,
        }
    }

    /// Returns the shared [`ValueType`] of the local group.
    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    /// Returns the minimum viable local index for the local group.
    pub fn min_index(&self) -> u32 {
        self.min_index
    }

    /// Returns the maximum viable local index for the local group.
    pub fn max_index(&self) -> u32 {
        self.max_index
    }
}

impl LocalsRegistry {
    /// Returns the number of registered local variables.
    ///
    /// # Note
    ///
    /// Since in WebAssembly function parameters are also local variables
    /// this function actually returns the amount of function parameters
    /// and explicitly defined local variables.
    pub fn len_registered(&self) -> u32 {
        self.max_index
    }

    /// Registers the `amount` of locals with their shared [`ValueType`].
    ///
    /// # Panics
    ///
    /// If too many local variables have been registered.
    pub fn register_locals(&mut self, value_type: ValueType, amount: u32) {
        if amount == 0 {
            return;
        }
        let min_index = self.max_index;
        let max_index = self.max_index.checked_add(amount).unwrap_or_else(|| {
            panic!(
                "encountered local variable index overflow \
                 upon registering {} locals of type {:?}",
                amount, value_type
            )
        });
        self.groups
            .push(LocalGroup::new(value_type, min_index, max_index));
        self.max_index = max_index;
    }

    /// Resolves the local variable at the given index.
    pub fn resolve_local(&mut self, local_index: u32) -> Option<ValueType> {
        if local_index >= self.max_index {
            // Bail out early if the local index is invalid.
            return None;
        }
        // Search for the local variable type in the groups
        // array using efficient binary search, insert it into the
        // `used` cache and return it to the caller.
        match self.groups.binary_search_by(|group| {
            if local_index < group.min_index() {
                return Ordering::Greater;
            }
            if local_index >= group.max_index() {
                return Ordering::Less;
            }
            Ordering::Equal
        }) {
            Ok(found_index) => {
                let value_type = self.groups[found_index].value_type();
                Some(value_type)
            }
            Err(_) => unreachable!(
                "unexectedly could not find valid local group index \
                using `local_index` = {}",
                local_index
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn register_too_many() {
        let mut registry = LocalsRegistry::default();
        registry.register_locals(ValueType::I32, u32::MAX);
        registry.register_locals(ValueType::I32, 1);
    }

    #[test]
    fn empty_works() {
        let mut registry = LocalsRegistry::default();
        for local_index in 0..10 {
            assert!(registry.resolve_local(local_index).is_none());
        }
        assert_eq!(registry.len_registered(), 0);
    }

    #[test]
    fn single_works() {
        let mut registry = LocalsRegistry::default();
        registry.register_locals(ValueType::I32, 1);
        registry.register_locals(ValueType::I64, 1);
        registry.register_locals(ValueType::F32, 1);
        registry.register_locals(ValueType::F64, 1);
        // Duplicate value types with another set of local groups.
        registry.register_locals(ValueType::I32, 1);
        registry.register_locals(ValueType::I64, 1);
        registry.register_locals(ValueType::F32, 1);
        registry.register_locals(ValueType::F64, 1);
        fn assert_valid_accesses(registry: &mut LocalsRegistry, offset: u32) {
            assert_eq!(registry.resolve_local(offset + 0), Some(ValueType::I32));
            assert_eq!(registry.resolve_local(offset + 1), Some(ValueType::I64));
            assert_eq!(registry.resolve_local(offset + 2), Some(ValueType::F32));
            assert_eq!(registry.resolve_local(offset + 3), Some(ValueType::F64));
        }
        // Assert the value types of the first group.
        assert_valid_accesses(&mut registry, 0);
        // Assert that also the second bunch of local groups work properly.
        assert_valid_accesses(&mut registry, 4);
        // Repeat the process to also check if the cache works.
        assert_valid_accesses(&mut registry, 0);
        assert_valid_accesses(&mut registry, 4);
        // Assert that an index out of bounds yields `None`.
        assert!(registry.resolve_local(registry.len_registered()).is_none());
    }

    #[test]
    fn multiple_works() {
        let mut registry = LocalsRegistry::default();
        let amount = 10;
        registry.register_locals(ValueType::I32, amount);
        registry.register_locals(ValueType::I64, amount);
        registry.register_locals(ValueType::F32, amount);
        registry.register_locals(ValueType::F64, amount);
        // Duplicate value types with another set of local groups.
        registry.register_locals(ValueType::I32, amount);
        registry.register_locals(ValueType::I64, amount);
        registry.register_locals(ValueType::F32, amount);
        registry.register_locals(ValueType::F64, amount);
        fn assert_local_group(
            registry: &mut LocalsRegistry,
            offset: u32,
            amount: u32,
            value_type: ValueType,
        ) {
            for local_index in 0..amount {
                assert_eq!(
                    registry.resolve_local(offset + local_index),
                    Some(value_type)
                );
            }
        }
        fn assert_valid_accesses(registry: &mut LocalsRegistry, offset: u32, amount: u32) {
            let value_types = [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ];
            for i in 0..4 {
                assert_local_group(
                    registry,
                    offset + i * amount,
                    amount,
                    value_types[i as usize],
                );
            }
        }
        // Assert the value types of the first group.
        assert_valid_accesses(&mut registry, 0, amount);
        // Assert that also the second bunch of local groups work properly.
        assert_valid_accesses(&mut registry, 4 * amount, amount);
        // Repeat the process to also check if the cache works.
        assert_valid_accesses(&mut registry, 0, amount);
        assert_valid_accesses(&mut registry, 4 * amount, amount);
        // Assert that an index out of bounds yields `None`.
        assert!(registry.resolve_local(registry.len_registered()).is_none());
    }
}
