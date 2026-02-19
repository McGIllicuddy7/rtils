use std::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::{Add, Deref, DerefMut, Index, IndexMut, Range},
    sync::{Mutex, atomic::AtomicUsize},
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::global;
//yeah yeah this is unrealistically large deal with it..
pub struct Heap {
    pub lock: Mutex<()>,
    pub backing_smol: [UnsafeCell<[u64; 4]>; 1024],
    pub backing_medium: [UnsafeCell<[u64; 64]>; 256],
    pub backing_large: [UnsafeCell<[u64; 256]>; 64],
    pub backing_pages: [UnsafeCell<[u64; 1024]>; 8],
    pub smol_marks: Mutex<[u8; 1024]>,
    pub medium_marks: Mutex<[u8; 256]>,
    pub large_marks: Mutex<[u8; 64]>,
    pub page_marks: Mutex<[u8; 8]>,
}

unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}
global!(HEAP:Heap = Heap::new());
impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    pub fn new() -> Self {
        Self {
            lock: Mutex::new(()),
            backing_smol: [const { UnsafeCell::new([0; _]) }; _],
            backing_medium: [const { UnsafeCell::new([0; _]) }; _],
            backing_large: [const { UnsafeCell::new([0; _]) }; _],
            backing_pages: [const { UnsafeCell::new([0; _]) }; _],
            smol_marks: Mutex::new([0; _]),
            medium_marks: Mutex::new([0; _]),
            large_marks: Mutex::new([0; _]),
            page_marks: Mutex::new([0; _]),
        }
    }
    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, count: usize) -> Option<&mut [u64]> {
        let _lock = self.lock.lock().unwrap();
        let mut count = count;
        loop {
            if count > 1024 * 8 {
                return None;
            } else if count > 1024 {
                let mut lm = self.page_marks.lock().unwrap();
                for i in 0..lm.len() {
                    unsafe {
                        if lm[i] == 0 {
                            lm[i] = 1;
                            return self.backing_pages[i]
                                .get()
                                .as_mut()
                                .map(|i| &mut i[0..1024]);
                        }
                    }
                }
                count = 1024 * 8 + 1;
            } else if count > 256 {
                let mut lm = self.large_marks.lock().unwrap();
                for i in 0..lm.len() {
                    unsafe {
                        if lm[i] == 0 {
                            lm[i] = 1;
                            return self.backing_large[i].get().as_mut().map(|i| &mut i[0..256]);
                        }
                    }
                }
                count = 1025;
            } else if count > 32 {
                let mut lm = self.medium_marks.lock().unwrap();
                for i in 0..lm.len() {
                    unsafe {
                        if lm[i] == 0 {
                            lm[i] = 1;
                            return self.backing_medium[i].get().as_mut().map(|i| &mut i[0..64]);
                        }
                    }
                }
                count = 257;
            } else {
                let mut lm = self.smol_marks.lock().unwrap();
                for i in 0..lm.len() {
                    unsafe {
                        if lm[i] == 0 {
                            lm[i] = 1;
                            return self.backing_smol[i].get().as_mut().map(|i| &mut i[0..4]);
                        }
                    }
                }
                count = 33;
            }
        }
    }

    pub fn free(&self, ptr: *mut u8) {
        let _lock = self.lock.lock().unwrap();
        for (idx, ptr2) in self.backing_pages.iter().enumerate() {
            if std::ptr::eq(ptr2, ptr as *const u8 as *const _) {
                let mut locks = self.page_marks.lock().unwrap();
                if locks[idx] == 0 {
                    todo!()
                } else {
                    locks[idx] = 0;
                }
            }
        }
        for (idx, ptr2) in self.backing_large.iter().enumerate() {
            if std::ptr::eq(ptr2, ptr as *const u8 as *const _) {
                let mut locks = self.large_marks.lock().unwrap();
                if locks[idx] == 0 {
                    todo!()
                } else {
                    locks[idx] = 0;
                }
            }
        }
        for (idx, ptr2) in self.backing_medium.iter().enumerate() {
            if std::ptr::eq(ptr2, ptr as *const u8 as *const _) {
                let mut locks = self.medium_marks.lock().unwrap();
                if locks[idx] == 0 {
                    todo!()
                } else {
                    locks[idx] = 0;
                }
            }
        }
        for (idx, ptr2) in self.backing_smol.iter().enumerate() {
            if std::ptr::eq(ptr2, ptr as *const u8 as *const _) {
                let mut locks = self.smol_marks.lock().unwrap();
                if locks[idx] == 0 {
                    todo!()
                } else {
                    locks[idx] = 0;
                }
            }
        }
    }
}

pub struct Ptr<T: ?Sized> {
    value: *mut T,
}
impl<T> Ptr<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            let hp = HEAP.get();
            let addr = hp.alloc(size_of::<T>());
            if let Some(addr) = addr {
                let ptr = addr.as_mut_ptr() as *mut T;
                if !ptr.is_null() {
                    ptr.write(value);
                }
                Self { value: ptr }
            } else {
                Self {
                    value: std::ptr::null_mut(),
                }
            }
        }
    }
    pub fn null() -> Self {
        Self {
            value: std::ptr::null_mut(),
        }
    }
}
impl<T: ?Sized> Ptr<T> {
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }
}
impl<T: Clone> Ptr<T> {
    pub fn from_ref(value: &T) -> Self {
        Ptr::new(value.clone())
    }
}

impl<T: Clone> Ptr<[T]> {
    pub fn null_slice() -> Self {
        Self {
            value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut::<T>(), 0),
        }
    }
    pub fn len(&self) -> usize {
        self.value.len()
    }
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn new_slice(value: T, count: usize) -> Self {
        let hp = HEAP.get();
        let addr = hp.alloc(size_of::<T>() * count);
        unsafe {
            if let Some(addr) = addr {
                let ptr = std::ptr::slice_from_raw_parts_mut(addr.as_mut_ptr() as *mut T, count);
                for i in 0..count {
                    (&raw mut (*ptr)[i]).write(value.clone())
                }
                Self { value: ptr }
            } else {
                Self {
                    value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut(), 0),
                }
            }
        }
    }
    pub fn from_slice(value: &[T]) -> Self {
        let hp = HEAP.get();
        let count = value.len();
        let addr = hp.alloc(std::mem::size_of_val(value));
        unsafe {
            if let Some(addr) = addr {
                let ptr = std::ptr::slice_from_raw_parts_mut(addr.as_mut_ptr() as *mut T, count);
                for (i, it) in value.iter().enumerate() {
                    (&raw mut (*ptr)[i]).write(it.clone())
                }
                Self { value: ptr }
            } else {
                Self {
                    value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut(), 0),
                }
            }
        }
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { self.value.as_ref().unwrap() }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { self.value.as_mut().unwrap() }
    }
}
impl<T: Clone> Index<usize> for Ptr<[T]> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T: Clone> IndexMut<usize> for Ptr<[T]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T: Clone> Index<Range<usize>> for Ptr<[T]> {
    type Output = [T];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T: Clone> IndexMut<Range<usize>> for Ptr<[T]> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T: ?Sized> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref().unwrap() }
    }
}

impl<T: ?Sized> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.as_mut().unwrap() }
    }
}
impl<T: ?Sized> AsRef<T> for Ptr<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.value.as_ref().unwrap() }
    }
}

impl<T: ?Sized> AsMut<T> for Ptr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.value.as_mut().unwrap() }
    }
}

impl<T: ?Sized> Drop for Ptr<T> {
    fn drop(&mut self) {
        unsafe {
            self.value.drop_in_place();
            HEAP.get().free(self.value as *mut u8);
        }
    }
}
impl<T: ?Sized + PartialEq> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl<T: ?Sized + Eq> Eq for Ptr<T> {}

impl<T: Clone> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self::from_ref(self.as_ref())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.as_ref())
    }
}
struct SharedHeader<T: ?Sized> {
    count: AtomicUsize,
    value: T,
}

pub struct SharedPtr<T: ?Sized> {
    value: *mut SharedHeader<T>,
}

impl<T> SharedPtr<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            let hp = HEAP.get();
            let addr = hp.alloc(size_of::<SharedHeader<T>>());
            if let Some(addr) = addr {
                let ptr = addr.as_mut_ptr() as *mut SharedHeader<T>;
                if !ptr.is_null() {
                    ptr.write(SharedHeader {
                        count: AtomicUsize::new(1),
                        value,
                    });
                }
                Self { value: ptr }
            } else {
                Self {
                    value: std::ptr::null_mut(),
                }
            }
        }
    }
    pub fn null() -> Self {
        Self {
            value: std::ptr::null_mut(),
        }
    }
}
impl<T: ?Sized> SharedPtr<T> {
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }
}

impl<T: Clone> SharedPtr<[T]> {
    pub fn null_slice() -> Self {
        Self {
            value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut::<T>(), 0)
                as *mut SharedHeader<[T]>,
        }
    }
    pub fn len(&self) -> usize {
        if self.value.is_null() {
            0
        } else {
            unsafe { self.value.as_ref().unwrap().value.len() }
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new_slice(value: T, count: usize) -> Self {
        let hp = HEAP.get();
        let addr = hp.alloc(size_of::<usize>() + size_of::<T>() * count);
        if let Some(addr) = addr {
            let out = std::ptr::slice_from_raw_parts_mut(addr.as_mut_ptr() as *mut T, count)
                as *mut SharedHeader<[T]>;
            unsafe {
                (&raw mut out.as_mut().unwrap().count).write(AtomicUsize::new(1));
                for i in 0..count {
                    out.as_mut().unwrap().value[i] = value.clone();
                }
            }
            Self { value: out }
        } else {
            Self {
                value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut::<T>(), 0)
                    as *mut SharedHeader<[T]>,
            }
        }
    }
    pub fn from_slice(value: &[T]) -> Self {
        if value.is_empty() {
            return Self::null_slice();
        }
        let hp = HEAP.get();
        let count = value.len();
        let addr = hp.alloc(size_of::<usize>() + size_of_val(value));
        if let Some(addr) = addr {
            let out = std::ptr::slice_from_raw_parts_mut(addr.as_mut_ptr(), count)
                as *mut SharedHeader<[T]>;

            unsafe {
                (&raw mut out.as_mut().unwrap().count).write(AtomicUsize::new(1));
                out.as_mut().unwrap().value[..count].clone_from_slice(&value[..count]);
            }
            Self { value: out }
        } else {
            Self {
                value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut::<T>(), 0)
                    as *mut SharedHeader<[T]>,
            }
        }
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { &self.value.as_ref().unwrap().value }
    }
}

impl<T: ?Sized> Clone for SharedPtr<T> {
    fn clone(&self) -> Self {
        if self.is_null() {
            Self { value: self.value }
        } else {
            unsafe {
                let lock = &self.value.as_ref().unwrap().count;
                let mut r = lock.load(std::sync::atomic::Ordering::Acquire);
                loop {
                    let old = lock.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                    if r == old {
                        break;
                    }
                    r = old;
                }
                Self { value: self.value }
            }
        }
    }
}

impl<T: Clone> Index<usize> for SharedPtr<[T]> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T: Clone> Index<Range<usize>> for SharedPtr<[T]> {
    type Output = [T];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T: ?Sized> Deref for SharedPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.value.as_ref().unwrap().value }
    }
}

impl<T: ?Sized> AsRef<T> for SharedPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &self.value.as_ref().unwrap().value }
    }
}
impl<T: ?Sized> PartialEq for SharedPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value as *const () == other.value as *const ()
    }
}
impl<T: ?Sized> Eq for SharedPtr<T> {}

impl<T: ?Sized> Drop for SharedPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let lock = &self.value.as_ref().unwrap().count;
            let mut r = lock.load(std::sync::atomic::Ordering::Acquire);
            loop {
                let old = lock.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                if r == old {
                    break;
                }
                r = old;
            }
            if r == 1 {
                (self.value).drop_in_place();
                HEAP.get().free(self.value as *mut u8);
            }
        }
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for SharedPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.as_ref())
    }
}

pub fn malloc<T>(value: T) -> Ptr<T> {
    Ptr::new(value)
}

pub fn calloc<T: Clone>(value: &T, count: usize) -> Ptr<[T]> {
    Ptr::new_slice(value.clone(), count)
}

pub fn malloc_shared<T>(value: T) -> SharedPtr<T> {
    SharedPtr::new(value)
}

pub fn calloc_shared<T: Clone>(value: &T, count: usize) -> SharedPtr<[T]> {
    SharedPtr::new_slice(value.clone(), count)
}

pub fn realloc<T: Clone>(value: &[T], count: usize) -> Ptr<[T]> {
    if value.is_empty() {
        Ptr::null_slice()
    } else {
        let mut out = Ptr::new_slice(value[0].clone(), count);
        let l = if value.len() > count {
            count
        } else {
            value.len()
        };
        for i in 0..l {
            out[i] = value[i].clone();
        }
        out
    }
}
pub fn realloc_shared<T: Clone>(value: &[T], count: usize) -> SharedPtr<[T]> {
    if value.is_empty() {
        SharedPtr::null_slice()
    } else {
        let hp = HEAP.get();
        let addr = hp.alloc(size_of::<T>() * count);
        if let Some(addr) = addr {
            let out = std::ptr::slice_from_raw_parts_mut(addr.as_mut_ptr() as *mut T, count)
                as *mut SharedHeader<[T]>;

            let _l = if value.len() > count {
                count
            } else {
                value.len()
            };
            unsafe {
                for (i, it) in value.iter().enumerate() {
                    (&raw mut (*out).value[i]).write(it.clone());
                }
            }
            SharedPtr { value: out }
        } else {
            SharedPtr::null_slice()
        }
    }
}

impl<T: Serialize + Clone> Serialize for Ptr<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_null() {
            Option::<T>::serialize(&None, serializer)
        } else {
            Option::<T>::serialize(&Some(self.as_ref().clone()), serializer)
        }
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Ptr<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tmp = Option::<T>::deserialize(deserializer)?;
        if let Some(tmp) = tmp {
            Ok(malloc(tmp))
        } else {
            Ok(Ptr::null())
        }
    }
}
impl<T: Serialize> Serialize for Ptr<[T]> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_null() {
            <Option<&[T]>>::serialize(&None, serializer)
        } else {
            <Option<&[T]>>::serialize(&Some(self.as_ref()), serializer)
        }
    }
}
impl<'a, 'de, T: Clone + 'a> Deserialize<'de> for Ptr<[T]>
where
    &'a [T]: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let out: Option<&[T]> = <Option<&[T]>>::deserialize(deserializer)?;
        if let Some(out) = out {
            Ok(Ptr::from_slice(out))
        } else {
            Ok(Ptr::null_slice())
        }
    }
}

impl<T: Serialize + Clone> Serialize for SharedPtr<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_null() {
            Option::<T>::serialize(&None, serializer)
        } else {
            Option::<T>::serialize(&Some(self.as_ref().clone()), serializer)
        }
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for SharedPtr<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tmp = Option::<T>::deserialize(deserializer)?;
        if let Some(tmp) = tmp {
            Ok(malloc_shared(tmp))
        } else {
            Ok(SharedPtr::null())
        }
    }
}
impl<T: Serialize> Serialize for SharedPtr<[T]> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_null() {
            <Option<&[T]>>::serialize(&None, serializer)
        } else {
            <Option<&[T]>>::serialize(&Some(self.as_ref()), serializer)
        }
    }
}
impl<'a, 'de, T: Clone + 'a> Deserialize<'de> for SharedPtr<[T]>
where
    &'a [T]: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let out: Option<&[T]> = <Option<&[T]>>::deserialize(deserializer)?;
        if let Some(out) = out {
            Ok(SharedPtr::from_slice(out))
        } else {
            Ok(SharedPtr::null_slice())
        }
    }
}
#[derive(Clone)]
pub struct CharPtr {
    ptr: SharedPtr<str>,
}
impl Serialize for CharPtr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_null() {
            None::<String>.serialize(serializer)
        } else {
            Some(self.as_ref().to_string()).serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for CharPtr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x = Option::<String>::deserialize(deserializer)?;
        if let Some(x) = x {
            Ok(Self::new(&x))
        } else {
            Ok(Self::null())
        }
    }
}
impl CharPtr {
    pub fn new(st: &str) -> Self {
        let bytes = st.as_bytes();
        let s = ManuallyDrop::new(SharedPtr::from_slice(bytes));
        let out = s.value;
        let st = out as *mut SharedHeader<str>;
        Self {
            ptr: SharedPtr { value: st },
        }
    }
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
    pub fn null() -> Self {
        Self {
            ptr: SharedPtr {
                value: std::ptr::slice_from_raw_parts_mut(std::ptr::null_mut::<u8>(), 0)
                    as *mut SharedHeader<str>,
            },
        }
    }
}
impl AsRef<str> for CharPtr {
    fn as_ref(&self) -> &str {
        &self.ptr
    }
}
impl Deref for CharPtr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl Add for CharPtr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let x = self.as_ref().to_string() + rhs.as_ref();
        Self::new(&x)
    }
}

impl Add<&str> for CharPtr {
    type Output = Self;
    fn add(self, rhs: &str) -> Self::Output {
        let x = self.as_ref().to_string() + rhs;
        Self::new(&x)
    }
}

unsafe impl<T: Send + ?Sized> Send for Ptr<T> {}
unsafe impl<T: Sync + ?Sized> Sync for Ptr<T> {}
unsafe impl<T: Send + ?Sized> Send for SharedPtr<T> {}
unsafe impl<T: Sync + ?Sized> Sync for SharedPtr<T> {}
