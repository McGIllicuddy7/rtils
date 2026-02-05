use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, LinkedList},
    sync::{Mutex, RwLock},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DataItem {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    List(Vec<DataItem>),
    Struct(BTreeMap<String, DataItem>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum DataType {
    Bool,
    Int,
    UInt,
    Float,
    String,
    List,
    Struct,
}

impl From<bool> for DataItem {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<i64> for DataItem {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<u64> for DataItem {
    fn from(value: u64) -> Self {
        Self::UInt(value)
    }
}
impl From<i32> for DataItem {
    fn from(value: i32) -> Self {
        Self::Int(value as i64)
    }
}

impl From<u32> for DataItem {
    fn from(value: u32) -> Self {
        Self::UInt(value as u64)
    }
}

impl From<f64> for DataItem {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<f32> for DataItem {
    fn from(value: f32) -> Self {
        Self::Float(value as f64)
    }
}

impl From<String> for DataItem {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<BTreeMap<String, DataItem>> for DataItem {
    fn from(value: BTreeMap<String, DataItem>) -> Self {
        Self::Struct(value)
    }
}

impl From<&str> for DataItem {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl<T: Into<DataItem> + Clone> From<&[T]> for DataItem {
    fn from(value: &[T]) -> Self {
        Self::List(value.iter().map(|i| i.clone().into()).collect())
    }
}

impl DataItem {
    pub fn new_struct<'a, T: 'a>(values: &'a [&dyn AsRef<T>], names: &[String]) -> Self
    where
        &'a T: Into<DataItem>,
    {
        let mut out = BTreeMap::new();
        for i in 0..values.len() {
            out.insert(names[i].clone(), values[i].as_ref().into());
        }
        Self::Struct(out)
    }
    pub fn get_type(&self) -> DataType {
        match self {
            DataItem::Bool(_) => DataType::Bool,
            DataItem::Int(_) => DataType::Int,
            DataItem::UInt(_) => DataType::UInt,
            DataItem::Float(_) => DataType::Float,
            DataItem::String(_) => DataType::String,
            DataItem::List(_) => DataType::List,
            DataItem::Struct(_) => DataType::Struct,
        }
    }
    pub fn get_bool(&self) -> Option<bool> {
        match self {
            DataItem::Bool(x) => Some(*x),
            _ => None,
        }
    }
    pub fn get_int(&self) -> Option<i64> {
        match self {
            DataItem::Int(x) => Some(*x),
            _ => None,
        }
    }

    pub fn get_uint(&self) -> Option<u64> {
        match self {
            DataItem::UInt(x) => Some(*x),
            _ => None,
        }
    }

    pub fn get_float(&self) -> Option<f64> {
        match self {
            DataItem::Float(x) => Some(*x),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            DataItem::String(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<Vec<DataItem>> {
        match self {
            DataItem::List(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn get_struct(&self) -> Option<BTreeMap<String, DataItem>> {
        match self {
            DataItem::Struct(x) => Some(x.clone()),
            _ => None,
        }
    }
    pub fn get_as_struct(&self) -> Option<(String, BTreeMap<String, DataItem>)> {
        match self {
            DataItem::Struct(x) => Some(("struct".to_string(), x.clone())),
            _ => None,
        }
    }

    pub fn get_int_mut(&mut self) -> Option<&mut i64> {
        match self {
            DataItem::Int(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_uint_mut(&mut self) -> Option<&mut u64> {
        match self {
            DataItem::UInt(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_float_mut(&mut self) -> Option<&mut f64> {
        match self {
            DataItem::Float(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_string_mut(&mut self) -> Option<&mut String> {
        match self {
            DataItem::String(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_list_mut(&mut self) -> Option<&mut Vec<DataItem>> {
        match self {
            DataItem::List(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_struct_mut(&mut self) -> Option<&mut BTreeMap<String, DataItem>> {
        match self {
            DataItem::Struct(x) => Some(x),
            _ => None,
        }
    }
}

pub trait Data: Sized {
    fn as_data(&self) -> DataItem;
    fn from_data(item: DataItem) -> Option<Self>;
}

impl Data for bool {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        item.get_bool()
    }
}
impl Data for i32 {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(item.get_int()? as i32)
    }
}
impl Data for i64 {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(item.get_int()? as i64)
    }
}

impl Data for u32 {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(item.get_uint()? as u32)
    }
}
impl Data for u64 {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(item.get_uint()? as u64)
    }
}

impl Data for f32 {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(item.get_float()? as f32)
    }
}
impl Data for f64 {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        item.get_float()
    }
}

impl Data for String {
    fn as_data(&self) -> DataItem {
        self.clone().into()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        item.get_string()
    }
}

impl<T: Data> Data for BTreeMap<String, T> {
    fn as_data(&self) -> DataItem {
        let mut out = BTreeMap::new();
        for (i, j) in self {
            out.insert(i.clone(), j.as_data());
        }
        DataItem::Struct(out)
    }

    fn from_data(item: DataItem) -> Option<Self> {
        let mut out = BTreeMap::new();
        for (i, j) in item.get_struct()? {
            out.insert(i.clone(), T::from_data(j)?);
        }
        Some(out)
    }
}

impl<T: Data> Data for Vec<T> {
    fn as_data(&self) -> DataItem {
        let mut out = Vec::new();
        for i in self {
            out.push(i.as_data());
        }
        DataItem::List(out)
    }

    fn from_data(item: DataItem) -> Option<Self> {
        let it = item.get_list()?;
        let mut out = Vec::new();
        for i in it {
            out.push(T::from_data(i)?);
        }
        Some(out)
    }
}

impl<T: Data> Data for LinkedList<T> {
    fn as_data(&self) -> DataItem {
        let mut out = Vec::new();
        for i in self {
            out.push(i.as_data());
        }
        DataItem::List(out)
    }

    fn from_data(item: DataItem) -> Option<Self> {
        let it = item.get_list()?;
        let mut out = LinkedList::new();
        for i in it {
            out.push_back(T::from_data(i)?);
        }
        Some(out)
    }
}

impl<T: Data> Data for Mutex<T> {
    fn as_data(&self) -> DataItem {
        self.lock().unwrap().as_data()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(Mutex::new(T::from_data(item)?))
    }
}

impl<T: Data> Data for RwLock<T> {
    fn as_data(&self) -> DataItem {
        self.read().unwrap().as_data()
    }

    fn from_data(item: DataItem) -> Option<Self> {
        Some(RwLock::new(T::from_data(item)?))
    }
}
