use super::{cache::CachedInstance, InstructionPtr, Stack};
use crate::{
    core::{wasm, ReadAs, UntypedVal, WriteAs},
    engine::{
        code_map::CodeMap,
        executor::stack::FrameSlots,
        utils::unreachable_unchecked,
        DedupFuncType,
    },
    ir::{index, Slot},
    memory::DataSegment,
    store::PrunedStore,
    table::ElementSegment,
    Error,
    Func,
    Global,
    Memory,
    Table,
};

#[cfg(doc)]
use crate::Instance;

/// Executes compiled function instructions until execution returns from the root function.
///
/// # Errors
///
/// If the execution encounters a trap.
#[inline(never)]
pub fn execute_instrs<'engine>(
    store: &mut PrunedStore,
    stack: &'engine mut Stack,
    code_map: &'engine CodeMap,
) -> Result<(), Error> {
    let instance = stack.calls.instance_expect();
    let cache = CachedInstance::new(store.inner_mut(), instance);
    let mut executor = Executor::new(stack, code_map, cache);
    if let Err(error) = executor.execute(store) {
        if error.is_out_of_fuel() {
            if let Some(frame) = executor.stack.calls.peek_mut() {
                // Note: we need to update the instruction pointer to make it possible to
                //       resume execution at the current instruction after running out of fuel.
                frame.update_instr_ptr(executor.ip);
            }
        }
        return Err(error);
    }
    Ok(())
}

/// An execution context for executing a Wasmi function frame.
#[derive(Debug)]
struct Executor<'engine> {
    /// Stores the value stack of live values on the Wasm stack.
    sp: FrameSlots,
    /// The pointer to the currently executed instruction.
    ip: InstructionPtr,
    /// The cached instance and instance related data.
    cache: CachedInstance,
    /// The value and call stacks.
    stack: &'engine mut Stack,
    /// The static resources of an [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    code_map: &'engine CodeMap,
}

impl<'engine> Executor<'engine> {
    /// Creates a new [`Executor`] for executing a Wasmi function frame.
    #[inline(always)]
    pub fn new(
        stack: &'engine mut Stack,
        code_map: &'engine CodeMap,
        cache: CachedInstance,
    ) -> Self {
        let frame = stack
            .calls
            .peek()
            .expect("must have call frame on the call stack");
        // Safety: We are using the frame's own base offset as input because it is
        //         guaranteed by the Wasm validation and translation phase to be
        //         valid for all register indices used by the associated function body.
        let sp = unsafe { stack.values.stack_ptr_at(frame.base_offset()) };
        let ip = frame.instr_ptr();
        Self {
            sp,
            ip,
            cache,
            stack,
            code_map,
        }
    }

    /// Executes the function frame until it returns or traps.
    #[inline(always)]
    fn execute(&mut self, _store: &mut PrunedStore) -> Result<(), Error> {
        todo!()
    }
}

macro_rules! get_entity {
    (
        $(
            fn $name:ident(&self, index: $index_ty:ty) -> $id_ty:ty;
        )*
    ) => {
        $(
            #[doc = ::core::concat!(
                "Returns the [`",
                ::core::stringify!($id_ty),
                "`] at `index` for the currently used [`Instance`].\n\n",
                "# Panics\n\n",
                "- If there is no [`",
                ::core::stringify!($id_ty),
                "`] at `index` for the currently used [`Instance`] in `store`."
            )]
            #[inline]
            fn $name(&self, index: $index_ty) -> $id_ty {
                unsafe { self.cache.$name(index) }
                    .unwrap_or_else(|| {
                        const ENTITY_NAME: &'static str = ::core::stringify!($id_ty);
                        // Safety: within the Wasmi executor it is assumed that store entity
                        //         indices within the Wasmi bytecode are always valid for the
                        //         store. This is an invariant of the Wasmi translation.
                        unsafe {
                            unreachable_unchecked!(
                                "missing {ENTITY_NAME} at index {index:?} for the currently used instance",
                            )
                        }
                    })
            }
        )*
    }
}

impl Executor<'_> {
    get_entity! {
        fn get_func(&self, index: index::Func) -> Func;
        fn get_func_type_dedup(&self, index: index::FuncType) -> DedupFuncType;
        fn get_memory(&self, index: index::Memory) -> Memory;
        fn get_table(&self, index: index::Table) -> Table;
        fn get_global(&self, index: index::Global) -> Global;
        fn get_data_segment(&self, index: index::Data) -> DataSegment;
        fn get_element_segment(&self, index: index::Elem) -> ElementSegment;
    }

    /// Returns the [`Slot`] value.
    fn get_stack_slot(&self, slot: Slot) -> UntypedVal {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.get(slot) }
    }

    /// Returns the [`Slot`] value as type `T`.
    fn get_stack_slot_as<T>(&self, slot: Slot) -> T
    where
        UntypedVal: ReadAs<T>,
    {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.read_as::<T>(slot) }
    }

    /// Sets the [`Slot`] value to `value`.
    fn set_stack_slot(&mut self, slot: Slot, value: impl Into<UntypedVal>) {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.set(slot, value.into()) };
    }

    /// Sets the [`Slot`] value to `value` of type `T`.
    fn set_stack_slot_as<T>(&mut self, slot: Slot, value: T)
    where
        UntypedVal: WriteAs<T>,
    {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.write_as::<T>(slot, value) };
    }
}

/// Extension method for [`UntypedVal`] required by the [`Executor`].
trait UntypedValueExt: Sized {
    /// Executes a logical `i{32,64}.and` instruction.
    fn and(x: Self, y: Self) -> bool;

    /// Executes a logical `i{32,64}.or` instruction.
    fn or(x: Self, y: Self) -> bool;

    /// Executes a fused `i{32,64}.and` + `i{32,64}.eqz` instruction.
    fn nand(x: Self, y: Self) -> bool {
        !Self::and(x, y)
    }

    /// Executes a fused `i{32,64}.or` + `i{32,64}.eqz` instruction.
    fn nor(x: Self, y: Self) -> bool {
        !Self::or(x, y)
    }
}

impl UntypedValueExt for i32 {
    fn and(x: Self, y: Self) -> bool {
        wasm::i32_bitand(x, y) != 0
    }

    fn or(x: Self, y: Self) -> bool {
        wasm::i32_bitor(x, y) != 0
    }
}

impl UntypedValueExt for i64 {
    fn and(x: Self, y: Self) -> bool {
        wasm::i64_bitand(x, y) != 0
    }

    fn or(x: Self, y: Self) -> bool {
        wasm::i64_bitor(x, y) != 0
    }
}

/// Extension method for [`UntypedVal`] required by the [`Executor`].
trait UntypedValueCmpExt: Sized {
    fn not_le(lhs: Self, rhs: Self) -> bool;
    fn not_lt(lhs: Self, rhs: Self) -> bool;
}

impl UntypedValueCmpExt for f32 {
    fn not_le(x: Self, y: Self) -> bool {
        !wasm::f32_le(x, y)
    }

    fn not_lt(x: Self, y: Self) -> bool {
        !wasm::f32_lt(x, y)
    }
}

impl UntypedValueCmpExt for f64 {
    fn not_le(x: Self, y: Self) -> bool {
        !wasm::f64_le(x, y)
    }

    fn not_lt(x: Self, y: Self) -> bool {
        !wasm::f64_lt(x, y)
    }
}
