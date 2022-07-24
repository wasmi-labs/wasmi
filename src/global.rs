use crate::{pwasm::PwasmCompat, Error, RuntimeValue, ValueType};
use alloc::rc::Rc;
use core::cell::Cell;
use parity_wasm::elements::ValueType as EValueType;

/// Reference to a global variable (See [`GlobalInstance`] for details).
///
/// This reference has a reference-counting semantics.
///
/// [`GlobalInstance`]: struct.GlobalInstance.html
#[derive(Clone, Debug)]
pub struct GlobalRef(Rc<GlobalInstance>);

impl ::core::ops::Deref for GlobalRef {
    type Target = GlobalInstance;
    fn deref(&self) -> &GlobalInstance {
        &self.0
    }
}

/// Runtime representation of a global variable (or `global` for short).
///
/// Global contains a value of a specified type and flag which specifies whether this
/// global are mutable or immutable. Neither type of the value nor immutability can't be changed
/// after creation.
///
/// Attempt to change value of immutable global or to change type of
/// the value (e.g. assign [`I32`] value to a global that was created with [`I64`] type) will lead to an error.
///
/// [`I32`]: enum.Value.html#variant.I32
/// [`I64`]: enum.Value.html#variant.I64
#[derive(Debug)]
pub struct GlobalInstance {
    val: Cell<RuntimeValue>,
    mutable: bool,
}

impl GlobalInstance {
    /// Allocate a global variable instance.
    ///
    /// Since it is possible to export only immutable globals,
    /// users likely want to set `mutable` to `false`.
    pub fn alloc(val: RuntimeValue, mutable: bool) -> GlobalRef {
        GlobalRef(Rc::new(GlobalInstance {
            val: Cell::new(val),
            mutable,
        }))
    }

    /// Change the value of this global variable.
    ///
    /// # Errors
    ///
    /// Returns `Err` if this global isn't mutable or if
    /// type of `val` doesn't match global's type.
    pub fn set(&self, val: RuntimeValue) -> Result<(), Error> {
        if !self.mutable {
            return Err(Error::Global(
                "Attempt to change an immutable variable".into(),
            ));
        }
        if self.value_type() != val.value_type() {
            return Err(Error::Global("Attempt to change variable type".into()));
        }
        self.val.set(val);
        Ok(())
    }

    /// Get the value of this global variable.
    pub fn get(&self) -> RuntimeValue {
        self.val.get()
    }

    /// Returns if this global variable is mutable.
    ///
    /// Note: Imported and/or exported globals are always immutable.
    pub fn is_mutable(&self) -> bool {
        self.mutable
    }

    /// Returns value type of this global variable.
    pub fn value_type(&self) -> ValueType {
        self.val.get().value_type()
    }

    pub(crate) fn elements_value_type(&self) -> EValueType {
        self.value_type().into_elements()
    }
}
