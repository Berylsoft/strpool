use std::{borrow::Cow, ops::Deref, cell::UnsafeCell};
use indexmap::IndexSet;

pub struct StrPool {
    pool: IndexSet<Cow<'static, str>>,
}

impl Default for StrPool {
    fn default() -> Self {
        Self { pool: Default::default() }
    }
}

thread_local! {
    static GLOBAL_POOL: UnsafeCell<StrPool> = Default::default();
}

#[derive(Clone)]
pub struct StrRef {
    ptr: usize,
}

impl StrPool {
    pub fn put_static(&mut self, str: &'static str) -> StrRef {
        let (ptr, _) = self.pool.insert_full(Cow::Borrowed(str));
        // println!("put_static: '{}' -> {} new={}", str, ptr, new);
        StrRef { ptr }
    }

    pub fn put_heap(&mut self, str: String) -> StrRef {
        // print!("put_heap: '{}'", str);
        let (ptr, _) = self.pool.insert_full(Cow::Owned(str));
        // println!(" -> {} new={}", ptr, new);
        StrRef { ptr }
    }

    pub fn get(&self, r: StrRef) -> Option<&str> {
        let s = self.pool.get_index(r.ptr).map(AsRef::as_ref);
        // println!("get: {} -> {:?}", r.ptr, s);
        s
    }
}

impl Deref for StrRef {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { global() }.get(self.clone()).expect("null string ref")
    }
}

unsafe fn global<'a>() -> &'a mut StrPool {
    &mut *GLOBAL_POOL.with(|r| r.get())
}

pub fn put_static(str: &'static str) -> StrRef {
    unsafe { global() }.put_static(str)
}

pub fn put_heap(str: String) -> StrRef {
    unsafe { global() }.put_heap(str)
}

impl Default for StrRef {
    fn default() -> Self {
        put_static("")
    }
}

impl PartialEq<Self> for StrRef {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Eq for StrRef {}

impl PartialOrd<Self> for StrRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl Ord for StrRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl PartialEq<str> for StrRef {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

// impl<T: AsRef<str>> PartialEq<T> for StrRef {
//     fn eq(&self, other: &T) -> bool {
//         self == other
//     }
// }

impl AsRef<StrRef> for StrRef {
    fn as_ref(&self) -> &StrRef {
        self
    }
}

impl AsRef<[u8]> for StrRef {
    fn as_ref(&self) -> &[u8] {
        self.deref().as_bytes()
    }
}

impl AsRef<str> for StrRef {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl core::hash::Hash for StrRef {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl core::borrow::Borrow<str> for StrRef {
    fn borrow(&self) -> &str {
        self
    }
}

impl From<String> for StrRef {
    #[inline]
    fn from(value: String) -> Self {
        put_heap(value)
    }
}

impl From<&str> for StrRef {
    #[inline]
    fn from(value: &str) -> Self {
        put_heap(value.to_owned())
    }
}

impl From<Box<str>> for StrRef {
    #[inline]
    fn from(value: Box<str>) -> Self {
        put_heap(String::from(value))
    }
}

impl From<StrRef> for String {
    #[inline]
    fn from(value: StrRef) -> Self {
        value.deref().to_owned()
    }
}

impl TryFrom<&[u8]> for StrRef {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(put_heap(core::str::from_utf8(value)?.to_owned()))
    }
}

impl TryFrom<Vec<u8>> for StrRef {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let buf = String::from_utf8(value).map_err(|err| err.utf8_error())?;
        Ok(put_heap(buf))
    }
}

impl<const N: usize> TryFrom<[u8; N]> for StrRef {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(value: [u8; N]) -> Result<Self, Self::Error> {
        StrRef::try_from(&value[..])
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for StrRef {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(value: &[u8; N]) -> Result<Self, Self::Error> {
        StrRef::try_from(&value[..])
    }
}

impl core::fmt::Debug for StrRef {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(fmt)
    }
}

impl core::fmt::Display for StrRef {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(fmt)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use serde::{
        de::{Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    use super::StrRef;

    impl Serialize for StrRef {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.as_ref())
        }
    }

    impl<'de> Deserialize<'de> for StrRef {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer).map(StrRef::from)
        }
    }
}
