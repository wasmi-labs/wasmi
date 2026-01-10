use crate::{
    wasm_exporttype_t,
    wasm_extern_t,
    wasm_externtype_t,
    wasm_frame_t,
    wasm_functype_t,
    wasm_globaltype_t,
    wasm_importtype_t,
    wasm_memorytype_t,
    wasm_tabletype_t,
    wasm_val_t,
    wasm_valtype_t,
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::{mem::MaybeUninit, ptr, slice};

/// A Wasm name string buffer.
pub type wasm_name_t = wasm_byte_vec_t;

impl wasm_name_t {
    pub(crate) fn from_name(name: String) -> wasm_name_t {
        name.into_bytes().into()
    }
}

macro_rules! declare_vecs {
    (
        $(
            struct $name:ident $(<$lt:tt>)? {
                type element = $elem_ty:ty;
                fn new: $new:ident;
                fn empty: $empty:ident;
                fn uninit: $uninit:ident;
                fn copy: $copy:ident;
                fn delete: $delete:ident;
            }
        )*
    ) => {$(
        #[doc = concat!("A Wasm compatible vector with element type [`", stringify!($elem_ty), "`].")]
        #[doc = ""]
        #[doc = "# Note"]
        #[doc = ""]
        #[doc = concat!("This is similar to `Box<[", stringify!($elem_ty), "]>`.")]
        #[repr(C)]
        pub struct $name $(<$lt>)? {
            size: usize,
            data: *mut $elem_ty,
        }

        impl$(<$lt>)? $name $(<$lt>)? {
            /// Sets the data buffer of `self` to `buffer` and leaks the current data buffer.
            pub fn set_buffer(&mut self, buffer: Box<[$elem_ty]>) {
                let slice = Box::leak(buffer);
                self.size = slice.len();
                self.data = slice.as_mut_ptr();
            }

            /// Returns the underlying data as shared slice.
            pub fn as_slice(&self) -> &[$elem_ty] {
                // Note: here we avoid creating a slice with a `null` data pointer
                //       because this is undefined behavior in Rust.
                match self.size {
                    0 => &[],
                    _ => {
                        assert!(!self.data.is_null());
                        unsafe { slice::from_raw_parts(self.data, self.size) }
                    }
                }
            }

            /// Returns the underlying data as [`MaybeUninit`] slice.
            pub fn as_uninit_slice(&mut self) -> &mut [MaybeUninit<$elem_ty>] {
                // Note: here we avoid creating a slice with a `null` data pointer
                //       because this is undefined behavior in Rust.
                match self.size {
                    0 => &mut [],
                    _ => {
                        assert!(!self.data.is_null());
                        unsafe { slice::from_raw_parts_mut(self.data as _, self.size) }
                    }
                }
            }

            /// Takes the data from `self` and returns it.
            ///
            /// # Note
            ///
            /// This leaves `self` empty after this operation.
            pub fn take(&mut self) -> Box<[$elem_ty]> {
                if self.data.is_null() {
                    return [].into();
                }
                let vec = unsafe {
                    Vec::from_raw_parts(self.data, self.size, self.size).into_boxed_slice()
                };
                self.size = 0;
                self.data = ptr::null_mut();
                return vec;
            }
        }

        impl$(<$lt>)? Clone for $name $(<$lt>)? {
            fn clone(&self) -> Self {
                let slice: Box<[$elem_ty]> = self.as_slice().into();
                Self::from(slice)
            }
        }

        impl$(<$lt>)? From<Box<[$elem_ty]>> for $name $(<$lt>)? {
            fn from(slice: Box<[$elem_ty]>) -> Self {
                let slice = Box::leak(slice);
                let result = $name {
                    size: slice.len(),
                    data: slice.as_mut_ptr(),
                };
                result
            }
        }

        impl$(<$lt>)? From<Vec<$elem_ty>> for $name $(<$lt>)? {
            fn from(vec: Vec<$elem_ty>) -> Self {
                Self::from(vec.into_boxed_slice())
            }
        }

        impl$(<$lt>)? Drop for $name $(<$lt>)? {
            fn drop(&mut self) {
                drop(self.take());
            }
        }

        #[doc = concat!("Creates an empty [`", stringify!($name),"`]")]
        #[doc = ""]
        #[doc = "# Note"]
        #[doc = ""]
        #[doc = concat!("Returns the resulting [`", stringify!($name), "`] in `out`.")]
        #[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn $empty(out: &mut $name) {
            out.size = 0;
            out.data = ptr::null_mut();
        }

        #[doc = concat!("Creates an uninitialized [`", stringify!($name),"`] with the given `size`.")]
        #[doc = ""]
        #[doc = "# Note"]
        #[doc = ""]
        #[doc = concat!("Returns the resulting [`", stringify!($name), "`] in `out`.")]
        #[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn $uninit(out: &mut $name, size: usize) {
            out.set_buffer(vec![Default::default(); size].into());
        }

        #[doc = concat!("Creates an new [`", stringify!($name),"`] with the given `size` and `ptr` data.")]
        #[doc = ""]
        #[doc = "# Note"]
        #[doc = ""]
        #[doc = "- The `ptr` must point to a buffer of length `size` or larger."]
        #[doc = concat!("- Returns the resulting [`", stringify!($name), "`] in `out`.")]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "It is the callers responsibility to provide a valid pair of `ptr` and `size`."]
        #[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub unsafe extern "C" fn $new $(<$lt>)? (
            out: &mut $name $(<$lt>)?,
            size: usize,
            ptr: *const $elem_ty,
        ) {
            let vec = (0..size).map(|i| ptr.add(i).read()).collect();
            out.set_buffer(vec);
        }

        #[doc = concat!("Copies the [`", stringify!($name),"`] in `src`.")]
        #[doc = ""]
        #[doc = "# Note"]
        #[doc = ""]
        #[doc = concat!("- Returns the resulting [`", stringify!($name), "`] in `out`.")]
        #[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn $copy $(<$lt>)? (
            out: &mut $name $(<$lt>)?,
            src: &$name $(<$lt>)?,
        ) {
            out.set_buffer(src.as_slice().into());
        }

        #[doc = concat!("Frees memory associated to the [`", stringify!($name),"`].")]
        #[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
        #[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
        pub extern "C" fn $delete $(<$lt>)? (out: &mut $name $(<$lt>)?) {
            out.take();
        }
    )*};
}

declare_vecs! {
    struct wasm_byte_vec_t {
        type element = u8;
        fn new: wasm_byte_vec_new;
        fn empty: wasm_byte_vec_new_empty;
        fn uninit: wasm_byte_vec_new_uninitialized;
        fn copy: wasm_byte_vec_copy;
        fn delete: wasm_byte_vec_delete;
    }
    struct wasm_valtype_vec_t {
        type element = Option<Box<wasm_valtype_t>>;
        fn new: wasm_valtype_vec_new;
        fn empty: wasm_valtype_vec_new_empty;
        fn uninit: wasm_valtype_vec_new_uninitialized;
        fn copy: wasm_valtype_vec_copy;
        fn delete: wasm_valtype_vec_delete;
    }
    struct wasm_functype_vec_t {
        type element = Option<Box<wasm_functype_t>>;
        fn new: wasm_functype_vec_new;
        fn empty: wasm_functype_vec_new_empty;
        fn uninit: wasm_functype_vec_new_uninitialized;
        fn copy: wasm_functype_vec_copy;
        fn delete: wasm_functype_vec_delete;
    }
    struct wasm_globaltype_vec_t {
        type element = Option<Box<wasm_globaltype_t>>;
        fn new: wasm_globaltype_vec_new;
        fn empty: wasm_globaltype_vec_new_empty;
        fn uninit: wasm_globaltype_vec_new_uninitialized;
        fn copy: wasm_globaltype_vec_copy;
        fn delete: wasm_globaltype_vec_delete;
    }
    struct wasm_tabletype_vec_t {
        type element = Option<Box<wasm_tabletype_t>>;
        fn new: wasm_tabletype_vec_new;
        fn empty: wasm_tabletype_vec_new_empty;
        fn uninit: wasm_tabletype_vec_new_uninitialized;
        fn copy: wasm_tabletype_vec_copy;
        fn delete: wasm_tabletype_vec_delete;
    }
    struct wasm_memorytype_vec_t {
        type element = Option<Box<wasm_memorytype_t>>;
        fn new: wasm_memorytype_vec_new;
        fn empty: wasm_memorytype_vec_new_empty;
        fn uninit: wasm_memorytype_vec_new_uninitialized;
        fn copy: wasm_memorytype_vec_copy;
        fn delete: wasm_memorytype_vec_delete;
    }
    struct wasm_externtype_vec_t {
        type element = Option<Box<wasm_externtype_t>>;
        fn new: wasm_externtype_vec_new;
        fn empty: wasm_externtype_vec_new_empty;
        fn uninit: wasm_externtype_vec_new_uninitialized;
        fn copy: wasm_externtype_vec_copy;
        fn delete: wasm_externtype_vec_delete;
    }
    struct wasm_importtype_vec_t {
        type element = Option<Box<wasm_importtype_t>>;
        fn new: wasm_importtype_vec_new;
        fn empty: wasm_importtype_vec_new_empty;
        fn uninit: wasm_importtype_vec_new_uninitialized;
        fn copy: wasm_importtype_vec_copy;
        fn delete: wasm_importtype_vec_delete;
    }
    struct wasm_exporttype_vec_t {
        type element = Option<Box<wasm_exporttype_t>>;
        fn new: wasm_exporttype_vec_new;
        fn empty: wasm_exporttype_vec_new_empty;
        fn uninit: wasm_exporttype_vec_new_uninitialized;
        fn copy: wasm_exporttype_vec_copy;
        fn delete: wasm_exporttype_vec_delete;
    }
    struct wasm_val_vec_t {
        type element = wasm_val_t;
        fn new: wasm_val_vec_new;
        fn empty: wasm_val_vec_new_empty;
        fn uninit: wasm_val_vec_new_uninitialized;
        fn copy: wasm_val_vec_copy;
        fn delete: wasm_val_vec_delete;
    }
    struct wasm_frame_vec_t<'a> {
        type element = Option<Box<wasm_frame_t<'a>>>;
        fn new: wasm_frame_vec_new;
        fn empty: wasm_frame_vec_new_empty;
        fn uninit: wasm_frame_vec_new_uninitialized;
        fn copy: wasm_frame_vec_copy;
        fn delete: wasm_frame_vec_delete;
    }
    struct wasm_extern_vec_t {
        type element = Option<Box<wasm_extern_t>>;
        fn new: wasm_extern_vec_new;
        fn empty: wasm_extern_vec_new_empty;
        fn uninit: wasm_extern_vec_new_uninitialized;
        fn copy: wasm_extern_vec_copy;
        fn delete: wasm_extern_vec_delete;
    }
}
