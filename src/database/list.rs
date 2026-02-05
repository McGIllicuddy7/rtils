use serde::de::Visitor;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Index, IndexMut, Range};
pub struct StaticList<T, const COUNT: usize = 16> {
    values: MaybeUninit<[T; COUNT]>,
    len: usize,
}
impl<T: PartialEq, const COUNT: usize> PartialEq for StaticList<T, COUNT> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T: PartialOrd, const COUNT: usize> PartialOrd for StaticList<T, COUNT> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_slice().partial_cmp(&other.as_slice())
    }
}
impl<T, const COUNT: usize> StaticList<T, COUNT> {
    pub fn new() -> Self {
        Self {
            values: MaybeUninit::zeroed(),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if self.len() <= index {
            None
        } else {
            Some(unsafe { &self.values.assume_init_ref()[index] })
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.len() <= index {
            None
        } else {
            Some(unsafe { &mut self.values.assume_init_mut()[index] })
        }
    }

    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len() >= COUNT {
            Err(value)
        } else {
            unsafe {
                std::ptr::write(&mut self.values.assume_init_mut()[self.len], value);
                self.len += 1;
            }
            Ok(())
        }
    }

    pub fn try_pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            let t = unsafe { std::ptr::read(&self.values.assume_init_mut()[self.len - 1]) };
            self.len -= 1;
            Some(t)
        }
    }

    pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), T> {
        if index > self.len() {
            return Err(value);
        }
        if index == self.len() {
            return self.try_push(value);
        }
        if self.len() >= COUNT {
            return Err(value);
        } else {
            unsafe {
                std::ptr::copy(
                    &self.values.assume_init_ref()[index],
                    &mut self.values.assume_init_mut()[index + 1],
                    self.len() - index,
                );
                std::ptr::write(&mut self.values.assume_init_mut()[index], value);
            }
            self.len += 1;
        }
        Ok(())
    }

    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if index > self.len() {
            return None;
        }
        if index == self.len() {
            return None;
        }
        if self.len() == 0 {
            return None;
        } else {
            unsafe {
                let out = std::ptr::read(&self.values.assume_init_ref()[index]);
                std::ptr::copy(
                    &self.values.assume_init_ref()[index + 1],
                    &mut self.values.assume_init_mut()[index],
                    self.len() - index - 1,
                );
                self.len -= 1;
                Some(out)
            }
        }
    }

    pub fn as_slice<'a>(&'a self) -> &'a [T] {
        unsafe { &self.values.assume_init_ref()[0..self.len()] }
    }

    pub fn as_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        let l = self.len();
        unsafe { &mut self.values.assume_init_mut()[0..l] }
    }
}

impl<T, const COUNT: usize> IntoIterator for StaticList<T, COUNT> {
    type IntoIter = StaticListIter<T, COUNT>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        StaticListIter {
            list: ManuallyDrop::new(self),
            index: 0,
        }
    }
}

impl<'a, T, const COUNT: usize> IntoIterator for &'a StaticList<T, COUNT> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'a, T, const COUNT: usize> IntoIterator for &'a mut StaticList<T, COUNT> {
    type IntoIter = std::slice::IterMut<'a, T>;
    type Item = &'a mut T;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<T, const COUNT: usize> Drop for StaticList<T, COUNT> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                std::ptr::drop_in_place(&mut self.values.assume_init_mut()[i]);
            }
        }
    }
}

pub struct StaticListIter<T, const COUNT: usize> {
    list: ManuallyDrop<StaticList<T, COUNT>>,
    index: usize,
}

impl<T, const COUNT: usize> Iterator for StaticListIter<T, COUNT> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let value = std::ptr::read(self.list.get_mut(self.index)?);
            self.index += 1;
            Some(value)
        }
    }
}
impl<T, const COUNT: usize> Drop for StaticListIter<T, COUNT> {
    fn drop(&mut self) {
        for i in self.index..self.list.len {
            unsafe { std::ptr::drop_in_place(self.list.get_mut(i).unwrap()) };
        }
    }
}

impl<T: Clone, const COUNT: usize> Clone for StaticList<T, COUNT> {
    fn clone(&self) -> Self {
        let mut out = Self::new();
        for i in self {
            _ = out.try_push(i.clone());
        }
        out
    }
}

impl<T: std::fmt::Debug, const COUNT: usize> std::fmt::Debug for StaticList<T, COUNT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.as_slice())
    }
}

impl<T: Serialize, const COUNT: usize> Serialize for StaticList<T, COUNT> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for i in self {
            seq.serialize_element(i)?;
        }
        seq.end()
    }
}
impl<'de, T: Deserialize<'de>, const COUNT: usize> Deserialize<'de> for StaticList<T, COUNT> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visit<T, const COUNT: usize> {
            inner: StaticList<T, COUNT>,
        }
        impl<'de, T: Deserialize<'de>, const COUNT: usize> Visitor<'de> for Visit<T, COUNT> {
            type Value = StaticList<T, COUNT>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "expecting a list")
            }
            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while let Some(i) = seq.next_element()? {
                    let r = self.inner.try_push(i);
                    if r.is_err() {
                        return Ok(self.inner);
                    }
                }
                Ok(self.inner)
            }
        }
        deserializer.deserialize_seq(Visit {
            inner: StaticList::new(),
        })
    }
}
impl<T, const COUNT: usize> Index<usize> for StaticList<T, COUNT> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, const COUNT: usize> IndexMut<usize> for StaticList<T, COUNT> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T, const COUNT: usize> Index<Range<usize>> for StaticList<T, COUNT> {
    type Output = [T];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, const COUNT: usize> IndexMut<Range<usize>> for StaticList<T, COUNT> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T, const COUNT: usize> AsRef<[T]> for StaticList<T, COUNT> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, const COUNT: usize> AsMut<[T]> for StaticList<T, COUNT> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}
pub struct StaticListRefIter<'a, T, const COUNT: usize> {
    list: &'a StaticList<T, COUNT>,
    index: usize,
}

impl<'a, T, const COUNT: usize> Iterator for StaticListRefIter<'a, T, COUNT> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.list.get(self.index)?;
        self.index += 1;
        Some(out)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct ArrayList<T> {
    lists: Vec<Box<StaticList<T, 32>>>,
    len: usize,
}

impl<T> ArrayList<T> {
    pub fn new() -> Self {
        Self {
            lists: Vec::new(),
            len: 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let mut base = 0;
        for i in &self.lists {
            let bprime = base + i.len();
            if base <= index && index < bprime {
                return i.get(index - base);
            } else {
                base = bprime;
            }
        }
        None
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let mut base = 0;
        for i in &mut self.lists {
            let bprime = base + i.len();
            if base <= index && index < bprime {
                return i.get_mut(index - base);
            } else {
                base = bprime;
            }
        }
        None
    }

    pub fn push(&mut self, mut value: T) {
        if !self.lists.is_empty() {
            let idx = self.lists.len() - 1;
            if let Err(e) = self.lists[idx].try_push(value) {
                value = e;
            } else {
                self.len += 1;
                return;
            }
        }
        let mut list = StaticList::new();
        let _ = list.try_push(value);
        self.lists.push(Box::new(list));
        self.len += 1;
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let mut base = 0;
        for i in 0..self.lists.len() {
            let bprime = base + self.lists[i].len();
            if base <= index && index < bprime {
                if self.lists[i].len() < 32 {
                    _ = self.lists[i].try_insert(index - base, value);
                    self.len += 1;
                    return;
                } else {
                    let Some(valuep) = self.lists[i].try_pop() else {
                        return;
                    };
                    let x = self.lists[i].try_insert(index - base, value);
                    assert!(!x.is_err());
                    if i < self.lists.len() - 1 {
                        if self.lists[i + 1].len() < 32 {
                            _ = self.lists[i + 1].try_insert(0, valuep);
                            self.len += 1;
                            return;
                        }
                    }
                    let mut list = StaticList::new();
                    let _ = list.try_push(valuep);
                    if i == self.lists.len() - 1 {
                        self.lists.push(Box::new(list));
                    } else {
                        self.lists.insert(i + 1, Box::new(list));
                    }
                    self.len += 1;
                    return;
                }
            } else {
                base = bprime;
            }
        }
        self.len += 1;
        let mut list = StaticList::new();
        let _ = list.try_push(value);
        self.lists.push(Box::new(list));
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let mut base = 0;
        for i in 0..self.lists.len() {
            let bprime = base + self.lists[i].len();
            if base <= index && index < bprime {
                let out = self.lists[i].try_remove(index - base);
                if self.lists[i].is_empty() {
                    self.lists.remove(i);
                }
                self.len -= 1;
                return out;
            } else {
                base = bprime;
            }
        }
        None
    }

    pub fn collect(&mut self) {
        'outer: loop {
            for i in 0..self.lists.len() {
                if self.lists[i].is_empty() {
                    self.lists.remove(i);
                    continue 'outer;
                }
            }
            break;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.lists.clear();
        self.len = 0;
    }
    pub fn iter(&self) -> ArrayListIterRef<'_, T> {
        self.into_iter()
    }
    pub fn sort_unstable_by(&mut self, f: impl Fn(&T, &T) -> Ordering) {
        let mut l = Vec::new();
        std::mem::swap(&mut self.lists, &mut l);
        let mut list: Vec<T> = l.into_iter().map(|i| i.into_iter()).flatten().collect();
        list.sort_unstable_by(f);
        for i in list {
            self.push(i);
        }
    }
}
impl<T: PartialOrd> ArrayList<T> {
    pub fn sort_default(&mut self) {
        let mut l = Vec::new();
        std::mem::swap(&mut self.lists, &mut l);
        let mut list: Vec<T> = l.into_iter().map(|i| i.into_iter()).flatten().collect();
        list.sort_unstable_by(|i, j| {
            if i > j {
                std::cmp::Ordering::Greater
            } else if i < j {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        for i in list {
            self.push(i);
        }
    }
}
impl<T: Ord> ArrayList<T> {
    pub fn sort_unstable(&mut self) {
        let mut l = Vec::new();
        std::mem::swap(&mut self.lists, &mut l);
        let mut list: Vec<T> = l.into_iter().map(|i| i.into_iter()).flatten().collect();
        list.sort_unstable();
        for i in list {
            self.push(i);
        }
    }
}
impl<'a, T> IntoIterator for &'a ArrayList<T> {
    type Item = &'a T;

    type IntoIter = ArrayListIterRef<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArrayListIterRef {
            list: self,
            indx: 0,
        }
    }
}
impl<'a, T> IntoIterator for &'a mut ArrayList<T> {
    type Item = &'a mut T;

    type IntoIter = ArrayListIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArrayListIterMut {
            list: self,
            indx: 0,
        }
    }
}
impl<T> IntoIterator for ArrayList<T> {
    type Item = T;

    type IntoIter = ArrayListIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        ArrayListIter { list: self }
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for ArrayList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for i in self {
            list.entry(i);
        }
        list.finish()
    }
}
pub struct ArrayListIterRef<'a, T> {
    list: &'a ArrayList<T>,
    indx: usize,
}

impl<'a, T> Iterator for ArrayListIterRef<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let out = self.list.get(self.indx)?;
        self.indx += 1;
        Some(out)
    }
}

pub struct ArrayListIterMut<'a, T> {
    list: &'a mut ArrayList<T>,
    indx: usize,
}

impl<'a, T> Iterator for ArrayListIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        let out = self.list.get_mut(self.indx)?;
        self.indx += 1;
        //safety, will never alias
        Some(unsafe { std::mem::transmute_copy(&out) })
    }
}
pub struct ArrayListIter<T> {
    list: ArrayList<T>,
}

impl<T> Iterator for ArrayListIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.list.remove(0)
    }
}

impl<T> Index<usize> for ArrayList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<usize> for ArrayList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
