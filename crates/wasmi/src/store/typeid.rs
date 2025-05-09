use core::{any::TypeId, marker::PhantomData, mem};

/// Returns the [`TypeId`] of `T`.
///
/// # Note
///
/// - `T` does _not_ need to be `'static` for this to work.
/// - This uses a trick copied from the [`typeid` crate](https://docs.rs/typeid) by [dtolnay](https://github.com/dtolnay).
#[must_use]
#[inline(always)]
pub fn of<T>() -> TypeId
where
    T: ?Sized,
{
    trait NonStaticAny {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static;
    }
    impl<T: ?Sized> NonStaticAny for PhantomData<T> {
        #[inline(always)]
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<T>()
        }
    }
    let phantom_data = PhantomData::<T>;
    NonStaticAny::get_type_id(unsafe {
        mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom_data)
    })
}
