use super::{GetOrInternWithHint, InternHint, Sym};
use alloc::{
    borrow::Borrow,
    collections::{BTreeMap, btree_map::Entry},
    sync::Arc,
    vec::Vec,
};
use core::{cmp::Ordering, mem, ops::Deref};

pub type StringInternerImpl = StringInterner;

mod hint {
    /// Indicates that the calling scope is unlikely to be executed.
    #[cold]
    #[inline]
    pub fn cold() {}
}

/// A string interner.
///
/// Efficiently interns strings and distributes symbols.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StringInterner {
    string2symbol: BTreeMap<LenOrder, Sym>,
    strings: Vec<Arc<str>>,
}

impl GetOrInternWithHint for StringInterner {
    fn get_or_intern_with_hint<T>(&mut self, string: T, hint: InternHint) -> Sym
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        match hint {
            InternHint::LikelyExists => self.get_or_intern_hint_existing(string),
            InternHint::LikelyNew | InternHint::None => self.get_or_intern_hint_new(string),
        }
    }
}

impl StringInterner {
    /// Creates a new empty [`StringInterner`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of interned strings.
    #[inline]
    pub fn len(&self) -> usize {
        self.string2symbol.len()
    }

    /// Returns `true` if the [`StringInterner`] is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the symbol of the string and interns it if necessary.
    ///
    /// # Note
    ///
    /// - Optimized for `string` not to be contained in [`StringInterner`] before this operation.
    /// - Allocates `string` twice on the heap if it already existed prior to this operation.
    fn get_or_intern_hint_new(&mut self, string: &str) -> Sym {
        match self.string2symbol.entry(LenOrder(string.into())) {
            Entry::Vacant(entry) => {
                let symbol = Sym::from_usize(self.strings.len());
                self.strings.push(entry.key().clone().0);
                entry.insert(symbol);
                symbol
            }
            Entry::Occupied(entry) => {
                hint::cold();
                *entry.get()
            }
        }
    }

    /// Returns the symbol of the string and interns it if necessary.
    ///
    /// # Note
    ///
    /// - Optimized for `string` to already be contained in [`StringInterner`] before this operation.
    /// - Queries the position within `strings2symbol` twice in case `string` already existed.
    #[inline]
    fn get_or_intern_hint_existing(&mut self, string: &str) -> Sym {
        match self.string2symbol.get(<&LenOrderStr>::from(string)) {
            Some(symbol) => *symbol,
            None => self.intern(string),
        }
    }

    /// Interns the `string` into the [`StringInterner`].
    ///
    /// # Panics
    ///
    /// If the `string` already exists.
    #[cold]
    fn intern<T>(&mut self, string: T) -> Sym
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        let symbol = Sym::from_usize(self.strings.len());
        let rc_string: Arc<str> = Arc::from(string);
        let old = self
            .string2symbol
            .insert(LenOrder(rc_string.clone()), symbol);
        assert!(old.is_none());
        self.strings.push(rc_string);
        symbol
    }

    /// Returns the symbol for the string if interned.
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<Sym>
    where
        T: AsRef<str>,
    {
        self.string2symbol
            .get(<&LenOrderStr>::from(string.as_ref()))
            .copied()
    }

    /// Resolves the symbol to the underlying string.
    #[inline]
    pub fn resolve(&self, symbol: Sym) -> Option<&str> {
        self.strings.get(symbol.into_usize()).map(Deref::deref)
    }
}

/// An `Arc<str>` that defines its own (more efficient) [`Ord`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LenOrder(Arc<str>);

impl Ord for LenOrder {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for LenOrder {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl LenOrder {
    #[inline]
    pub fn as_str(&self) -> &LenOrderStr {
        (&*self.0).into()
    }
}

/// A `str` that defines its own (more efficient) [`Ord`].
#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct LenOrderStr(str);

impl<'a> From<&'a str> for &'a LenOrderStr {
    #[inline]
    fn from(value: &'a str) -> Self {
        // Safety: This operation is safe because
        //
        // - we preserve the lifetime `'a`
        // - the `LenOrderStr` type is a `str` newtype wrapper and `#[repr(transparent)`
        unsafe { mem::transmute(value) }
    }
}

impl Borrow<LenOrderStr> for LenOrder {
    #[inline]
    fn borrow(&self) -> &LenOrderStr {
        (&*self.0).into()
    }
}

impl PartialOrd for LenOrderStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LenOrderStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = self.0.as_bytes();
        let rhs = other.0.as_bytes();
        match lhs.len().cmp(&rhs.len()) {
            Ordering::Equal => {
                if lhs.len() < 8 {
                    for (l, r) in lhs.iter().zip(rhs) {
                        match l.cmp(r) {
                            Ordering::Equal => (),
                            ordering => return ordering,
                        }
                    }
                    Ordering::Equal
                } else {
                    lhs.cmp(rhs)
                }
            }
            ordering => ordering,
        }
    }
}
