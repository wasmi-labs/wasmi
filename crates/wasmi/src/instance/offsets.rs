#[cfg(doc)]
use crate::instance::{DataSegment, ElementSegment, Func, Global, InstanceEntity, Memory, Table};
use crate::limits::LimitsError;

/// Offsets within [`InstanceEntity::handles`] buffer for various handle types.
#[derive(Debug, Copy, Clone)]
pub struct InstanceLayout {
    /// The start offset within `InstanceEntity::handles` for [`Global`] handles.
    globals: u32,
    /// The start offset within `InstanceEntity::handles` for [`Memory`] handles.
    memories: u32,
    /// The start offset within `InstanceEntity::handles` for [`Table`] handles.
    tables: u32,
    /// The start offset within `InstanceEntity::handles` for [`DataSegment`] handles.
    datas: u32,
    /// The start offset within `InstanceEntity::handles` for [`ElementSegment`] handles.
    elems: u32,
    /// The start offset within `InstanceEntity::handles` for [`Func`] handles.
    funcs: u32,
    /// The total number of handles in the instance.
    len_handles: u32,
}

macro_rules! define_addr_types {
    (
        $( pub struct $name:ident(u32) = $ty:ty );* $(;)?
    ) => {
        $(
            #[doc = concat!("The 32-bit address of a [`", stringify!($ty), "`] within the [`InstanceEntity::handles`] buffer.")]
            #[derive(Debug, Copy, Clone)]
            pub struct $name(u32);

            impl From<u32> for $name {
                #[inline]
                fn from(value: u32) -> Self {
                    Self(value)
                }
            }

            impl From<$name> for u32 {
                #[inline]
                fn from(addr: $name) -> Self {
                    addr.0
                }
            }
        )*
    };
}
define_addr_types! {
    pub struct GlobalAddr(u32) = Global;
    pub struct MemoryAddr(u32) = Memory;
    pub struct TableAddr(u32) = Table;
    pub struct FuncAddr(u32) = Func;
    pub struct DataAddr(u32) = Data;
    pub struct ElemAddr(u32) = Elem;
}

impl InstanceLayout {
    /// Creates a new [`InstanceLayoutBuilder`].
    pub(crate) fn build() -> InstanceLayoutBuilder {
        InstanceLayoutBuilder::default()
    }

    /// Creates an uninitialized [`InstanceLayout`].
    pub(crate) fn uninit() -> Self {
        Self {
            globals: 0,
            memories: 0,
            tables: 0,
            datas: 0,
            elems: 0,
            funcs: 0,
            len_handles: 0,
        }
    }

    /// Returns the number of global variables in the associated instance.
    fn len_globals(&self) -> u32 {
        // Note: globals are placed directly before memories.
        self.memories - self.globals
    }

    /// Returns the number of linear memories in the associated instance.
    fn len_memories(&self) -> u32 {
        // Note: memories are placed directly before tables.
        self.tables - self.memories
    }

    /// Returns the number of tables in the associated instance.
    fn len_tables(&self) -> u32 {
        // Note: tables are placed directly before data segments.
        self.datas - self.tables
    }

    /// Returns the number of data segments in the associated instance.
    fn len_datas(&self) -> u32 {
        // Note: data segments are placed directly before element segments.
        self.elems - self.datas
    }

    /// Returns the number of element segments in the associated instance.
    fn len_elems(&self) -> u32 {
        // Note: element segments are placed directly before functions.
        self.funcs - self.elems
    }

    /// Returns the number of functions in the associated instance.
    fn len_funcs(&self) -> u32 {
        // Note: functions are placed last in the handles buffer.
        self.len_handles - self.funcs
    }

    /// Returns the [`GlobalAddr`] for a [`Global`] Wasm index.
    #[inline]
    pub fn global_addr(&self, index: u32) -> Option<GlobalAddr> {
        if index >= self.len_globals() {
            return None;
        }
        Some(GlobalAddr(self.globals + index))
    }

    /// Returns the [`MemoryAddr`] for a [`Memory`] Wasm index.
    #[inline]
    pub fn memory_addr(&self, index: u32) -> Option<MemoryAddr> {
        if index >= self.len_memories() {
            return None;
        }
        Some(MemoryAddr(self.memories + index))
    }

    /// Returns the [`TableAddr`] for a [`Table`] Wasm index.
    #[inline]
    pub fn table_addr(&self, index: u32) -> Option<TableAddr> {
        if index >= self.len_tables() {
            return None;
        }
        Some(TableAddr(self.tables + index))
    }

    /// Returns the [`DataAddr`] for a [`DataSegment`] Wasm index.
    #[inline]
    pub fn data_addr(&self, index: u32) -> Option<DataAddr> {
        if index >= self.len_datas() {
            return None;
        }
        Some(DataAddr(self.datas + index))
    }

    /// Returns the [`ElemAddr`] for a [`ElementSegment`] Wasm index.
    #[inline]
    pub fn elem_addr(&self, index: u32) -> Option<ElemAddr> {
        if index >= self.len_elems() {
            return None;
        }
        Some(ElemAddr(self.elems + index))
    }

    /// Returns the [`FuncAddr`] for a [`Func`] Wasm index.
    #[inline]
    pub fn func_addr(&self, index: u32) -> Option<FuncAddr> {
        if index >= self.len_funcs() {
            return None;
        }
        Some(FuncAddr(self.funcs + index))
    }
}

#[derive(Debug, Default)]
pub struct InstanceLayoutBuilder {
    /// The start offset within `InstanceEntity::handles` for [`Global`] handles.
    globals: Option<u32>,
    /// The start offset within `InstanceEntity::handles` for [`Memory`] handles.
    memories: Option<u32>,
    /// The start offset within `InstanceEntity::handles` for [`Table`] handles.
    tables: Option<u32>,
    /// The start offset within `InstanceEntity::handles` for [`DataSegment`] handles.
    datas: Option<u32>,
    /// The start offset within `InstanceEntity::handles` for [`ElementSegment`] handles.
    elems: Option<u32>,
    /// The start offset within `InstanceEntity::handles` for [`Func`] handles.
    funcs: Option<u32>,
}

macro_rules! impl_builder {
    (
        $( pub fn $name:ident(&mut self, $len:ident: usize) -> Result<&mut Self, LimitsError> = ($ty:ident, $check_limit:expr));*
        $(;)?
    ) => {
        $(
            #[doc = concat!("Initializes the number of [`", stringify!($ty), "`] handles in `self`.")]
            pub fn $name(&mut self, $len: usize) -> Result<&mut Self, LimitsError> {
                assert!(self.$name.is_none());
                $check_limit($len)?;
                let $name = $len as u32;
                self.$name = Some($name);
                Ok(self)
            }
        )*
    };
}
impl InstanceLayoutBuilder {
    impl_builder! {
        pub fn globals(&mut self, len_globals: usize) -> Result<&mut Self, LimitsError> = (Global, LimitsError::max_global_count);
        pub fn memories(&mut self, len_memories: usize) -> Result<&mut Self, LimitsError> = (Memory, LimitsError::max_memory_count);
        pub fn tables(&mut self, len_tables: usize) -> Result<&mut Self, LimitsError> = (Tables, LimitsError::max_table_count);
        pub fn datas(&mut self, len_datas: usize) -> Result<&mut Self, LimitsError> = (DataSegment, LimitsError::max_data_count);
        pub fn elems(&mut self, len_elems: usize) -> Result<&mut Self, LimitsError> = (ElementSegment, LimitsError::max_elem_count);
        pub fn funcs(&mut self, len_funcs: usize) -> Result<&mut Self, LimitsError> = (Func, LimitsError::max_func_count);
    }

    pub fn finish(self) -> Result<InstanceLayout, LimitsError> {
        let err = LimitsError::TooManyInstanceHandles;
        let len_globals = self.globals.unwrap_or(0);
        let len_memories = self.memories.unwrap_or(0);
        let len_tables = self.tables.unwrap_or(0);
        let len_datas = self.datas.unwrap_or(0);
        let len_elems = self.elems.unwrap_or(0);
        let len_funcs = self.funcs.unwrap_or(0);
        let globals = 0;
        let memories = len_globals;
        let tables = memories.checked_add(len_memories).ok_or(err)?;
        let datas = tables.checked_add(len_tables).ok_or(err)?;
        let elems = datas.checked_add(len_datas).ok_or(err)?;
        let funcs = elems.checked_add(len_elems).ok_or(err)?;
        let len_handles = funcs.checked_add(len_funcs).ok_or(err)?;
        Ok(InstanceLayout {
            globals,
            memories,
            tables,
            datas,
            elems,
            funcs,
            len_handles,
        })
    }
}
