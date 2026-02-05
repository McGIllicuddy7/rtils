use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use super::item::{DataItem, DataType};
use super::list::ArrayList;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Query {
    Less,
    LessOrEqual,
    Equal,
    NotEqual,
    GreaterOrEqual,
    Greator,
    QueriedContains,
    QuerierContains,
    ListContains,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Col {
    Bool(ArrayList<bool>),
    Int(ArrayList<i64>),
    UInt(ArrayList<u64>),
    Float(ArrayList<f64>),
    String(ArrayList<String>),
    List(ArrayList<Vec<DataItem>>),
    Struct(String, ArrayList<BTreeMap<String, DataItem>>),
}
impl Col {
    pub fn get_type(&self) -> DataType {
        match self {
            Self::Bool(_) => DataType::Bool,
            Self::Int(_) => DataType::Int,
            Self::UInt(_) => DataType::UInt,
            Self::Float(_) => DataType::Float,
            Self::String(_) => DataType::String,
            Self::List(_) => DataType::List,
            Self::Struct(_, _) => DataType::Struct,
        }
    }

    pub fn get_int(&self) -> Option<&ArrayList<i64>> {
        match self {
            Self::Int(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_uint(&self) -> Option<&ArrayList<u64>> {
        match self {
            Self::UInt(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_float(&self) -> Option<&ArrayList<f64>> {
        match self {
            Self::Float(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&ArrayList<String>> {
        match self {
            Self::String(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<&ArrayList<Vec<DataItem>>> {
        match self {
            Self::List(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_struct(&self) -> Option<&ArrayList<BTreeMap<String, DataItem>>> {
        match self {
            Self::Struct(_, x) => Some(x),
            _ => None,
        }
    }

    pub fn get_int_mut(&mut self) -> Option<&mut ArrayList<i64>> {
        match self {
            Self::Int(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_uint_mut(&mut self) -> Option<&mut ArrayList<u64>> {
        match self {
            Self::UInt(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_float_mut(&mut self) -> Option<&mut ArrayList<f64>> {
        match self {
            Self::Float(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_string_mut(&mut self) -> Option<&mut ArrayList<String>> {
        match self {
            Self::String(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_list_mut(&mut self) -> Option<&mut ArrayList<Vec<DataItem>>> {
        match self {
            Self::List(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_struct_mut(&mut self) -> Option<&mut ArrayList<BTreeMap<String, DataItem>>> {
        match self {
            Self::Struct(_, x) => Some(x),
            _ => None,
        }
    }

    pub fn remove_at(&mut self, index: usize) -> Option<()> {
        match self {
            Col::Bool(items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
            Col::Int(items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
            Col::UInt(items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
            Col::Float(items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
            Col::String(items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
            Col::List(items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
            Col::Struct(_, items) => {
                if items.len() <= index {
                    return None;
                }
                items.remove(index);
            }
        }
        Some(())
    }

    pub fn get(&self, index: usize) -> Option<DataItem> {
        match self {
            Col::Bool(items) => {
                return Some(items.get(index)?.clone().into());
            }
            Col::Int(items) => {
                return Some(items.get(index)?.clone().into());
            }
            Col::UInt(items) => {
                return Some(items.get(index)?.clone().into());
            }
            Col::Float(items) => {
                return Some(items.get(index)?.clone().into());
            }
            Col::String(items) => {
                return Some(items.get(index)?.clone().into());
            }
            Col::List(items) => {
                return Some(items.get(index)?.as_slice().into());
            }
            Col::Struct(_, items) => {
                return Some(items.get(index)?.clone().into());
            }
        }
    }

    pub fn replace_at(&mut self, item: DataItem, index: usize) -> Option<()> {
        match self {
            Col::Bool(items) => {
                *items.get_mut(index)? = item.get_bool()?;
            }
            Col::Int(items) => {
                *items.get_mut(index)? = item.get_int()?;
            }
            Col::UInt(items) => {
                *items.get_mut(index)? = item.get_uint()?;
            }
            Col::Float(items) => {
                *items.get_mut(index)? = item.get_float()?;
            }
            Col::String(items) => {
                *items.get_mut(index)? = item.get_string()?;
            }
            Col::List(items) => {
                *items.get_mut(index)? = item.get_list()?;
            }
            Col::Struct(_, items) => {
                *items.get_mut(index)? = item.get_struct()?;
            }
        }
        Some(())
    }
    pub fn insert_at(&mut self, item: DataItem, index: usize) -> Option<()> {
        match self {
            Col::Bool(items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_bool()?);
            }
            Col::Int(items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_int()?);
            }
            Col::UInt(items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_uint()?);
            }
            Col::Float(items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_float()?);
            }
            Col::String(items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_string()?);
            }
            Col::List(items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_list()?);
            }
            Col::Struct(_, items) => {
                if items.len() < index {
                    return None;
                }
                items.insert(index, item.get_struct()?);
            }
        }
        Some(())
    }

    pub fn add(&mut self, item: DataItem) -> Option<()> {
        match self {
            Col::Bool(items) => {
                items.push(item.get_bool()?);
            }
            Col::Int(items) => {
                items.push(item.get_int()?);
            }
            Col::UInt(items) => {
                items.push(item.get_uint()?);
            }
            Col::Float(items) => {
                items.push(item.get_float()?);
            }
            Col::String(items) => {
                items.push(item.get_string()?);
            }
            Col::List(items) => {
                items.push(item.get_list()?);
            }
            Col::Struct(_, items) => {
                items.push(item.get_struct()?);
            }
        }
        Some(())
    }

    pub fn clear(&mut self) {
        match self {
            Col::Bool(items) => {
                items.clear();
            }
            Col::Int(items) => {
                items.clear();
            }
            Col::UInt(items) => {
                items.clear();
            }
            Col::Float(items) => {
                items.clear();
            }
            Col::String(items) => {
                items.clear();
            }
            Col::List(items) => {
                items.clear();
            }
            Col::Struct(_, items) => {
                items.clear();
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Col::Bool(items) => items.len(),
            Col::Int(items) => items.len(),
            Col::UInt(items) => items.len(),
            Col::Float(items) => items.len(),
            Col::String(items) => items.len(),
            Col::List(items) => items.len(),
            Col::Struct(_, items) => items.len(),
        }
    }

    pub fn add_sorted(&mut self, item: DataItem) -> Option<usize> {
        match self {
            Col::Bool(items) => {
                let x = item.get_bool()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
            Col::Int(items) => {
                let x = item.get_int()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
            Col::UInt(items) => {
                let x = item.get_uint()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
            Col::Float(items) => {
                let x = item.get_float()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
            Col::String(items) => {
                let x = item.get_string()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
            Col::List(items) => {
                let x = item.get_list()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
            Col::Struct(_, items) => {
                let x = item.get_struct()?;
                for i in 0..items.len() {
                    if items[i] < x {
                        items.insert(i + 1, x);
                        return Some(i + 1);
                    }
                }
                items.push(x);
                return Some(items.len() - 1);
            }
        }
    }

    pub fn sort(&mut self) {
        match self {
            Col::Bool(items) => {
                items.sort_unstable();
            }
            Col::Int(items) => {
                items.sort_unstable();
            }
            Col::UInt(items) => {
                items.sort_unstable();
            }
            Col::Float(items) => {
                items.sort_unstable_by(|x, y| {
                    if x > y {
                        std::cmp::Ordering::Greater
                    } else if x < y {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            Col::String(items) => {
                items.sort_unstable();
            }
            Col::List(items) => {
                items.sort_unstable_by(|x, y| {
                    if x > y {
                        std::cmp::Ordering::Greater
                    } else if x < y {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            Col::Struct(_, items) => {
                items.sort_unstable_by(|x, y| {
                    if x > y {
                        std::cmp::Ordering::Greater
                    } else if x < y {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
        }
    }

    pub fn find_matching(&self, item: DataItem, query: Query) -> Option<Vec<(usize, DataItem)>> {
        let mut out = Vec::new();
        match self {
            Col::Bool(items) => {
                let it = item.get_bool()?;
                match query {
                    Query::Less => {
                        for (i, j) in items.iter().enumerate() {
                            if *j < it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::LessOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j <= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Equal => {
                        for (i, j) in items.iter().enumerate() {
                            if *j == it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::NotEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j != it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::GreaterOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j >= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Greator => {
                        for (i, j) in items.iter().enumerate() {
                            if *j > it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::Int(items) => {
                let it = item.get_int()?;
                match query {
                    Query::Less => {
                        for (i, j) in items.iter().enumerate() {
                            if *j < it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::LessOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j <= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Equal => {
                        for (i, j) in items.iter().enumerate() {
                            if *j == it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::NotEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j != it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::GreaterOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j >= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Greator => {
                        for (i, j) in items.iter().enumerate() {
                            if *j > it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::UInt(items) => {
                let it = item.get_uint()?;
                match query {
                    Query::Less => {
                        for (i, j) in items.iter().enumerate() {
                            if *j < it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::LessOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j <= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Equal => {
                        for (i, j) in items.iter().enumerate() {
                            if *j == it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::NotEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j != it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::GreaterOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j >= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Greator => {
                        for (i, j) in items.iter().enumerate() {
                            if *j > it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::Float(items) => {
                let it = item.get_float()?;
                match query {
                    Query::Less => {
                        for (i, j) in items.iter().enumerate() {
                            if *j < it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::LessOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j <= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Equal => {
                        for (i, j) in items.iter().enumerate() {
                            if *j == it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::NotEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j != it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::GreaterOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j >= it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::Greator => {
                        for (i, j) in items.iter().enumerate() {
                            if *j > it {
                                out.push((i, (*j).into()))
                            }
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::String(items) => {
                let it = item.get_string()?;
                match query {
                    Query::Less => {
                        for (i, j) in items.iter().enumerate() {
                            if *j < it {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::LessOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j <= it {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::Equal => {
                        for (i, j) in items.iter().enumerate() {
                            if *j == it {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::NotEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j != it {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::GreaterOrEqual => {
                        for (i, j) in items.iter().enumerate() {
                            if *j >= it {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::Greator => {
                        for (i, j) in items.iter().enumerate() {
                            if *j > it {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::QueriedContains => {
                        for (i, j) in items.iter().enumerate() {
                            if j.contains(&it) {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::QuerierContains => {
                        for (i, j) in items.iter().enumerate() {
                            if it.contains(j) {
                                out.push((i, (j.clone()).into()))
                            }
                        }
                    }
                    Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::List(items) => match query {
                Query::Less => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j < it {
                            out.push((i, j.as_slice().into()));
                        }
                    }
                }
                Query::LessOrEqual => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j <= it {
                            out.push((i, j.as_slice().into()));
                        }
                    }
                }
                Query::Equal => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j == it {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
                Query::NotEqual => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j != it {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
                Query::GreaterOrEqual => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j >= it {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
                Query::Greator => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j > it {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
                Query::QueriedContains => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        let mut contains = false;
                        let v = DataItem::List(it.clone());
                        for i in j {
                            if *i == v {
                                contains = true;
                                break;
                            }
                        }
                        if contains {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
                Query::QuerierContains => {
                    let it = item.get_list()?;
                    for (i, j) in items.iter().enumerate() {
                        let mut contains = false;
                        let v = j.as_slice().into();
                        for i in &it {
                            if *i == v {
                                contains = true;
                                break;
                            }
                        }
                        if contains {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
                Query::ListContains => {
                    for (i, j) in items.iter().enumerate() {
                        let mut contains = false;
                        for i in j {
                            if *i == item {
                                contains = true;
                                break;
                            }
                        }
                        if contains {
                            out.push((i, j.as_slice().into()))
                        }
                    }
                }
            },
            Col::Struct(_, items) => match query {
                Query::Less => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j < it {
                            out.push((i, j.clone().into()).into());
                        }
                    }
                }
                Query::LessOrEqual => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j <= it {
                            out.push((i, j.clone().into()).into());
                        }
                    }
                }
                Query::Equal => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j == it {
                            out.push((i, j.clone().into()).into())
                        }
                    }
                }
                Query::NotEqual => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j != it {
                            out.push((i, j.clone().into()))
                        }
                    }
                }
                Query::GreaterOrEqual => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j >= it {
                            out.push((i, j.clone().into()))
                        }
                    }
                }
                Query::Greator => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        if *j > it {
                            out.push((i, j.clone().into()).into())
                        }
                    }
                }

                Query::QueriedContains => {
                    let it = item.get_struct()?;
                    for (i, j) in items.iter().enumerate() {
                        for (_, item) in j {
                            if *item == it.clone().into() {
                                out.push((i, j.clone().into()))
                            }
                        }
                    }
                }
                Query::QuerierContains => {
                    if let Some(list) = item.get_list() {
                        for (i, j) in items.iter().enumerate() {
                            let tmp = j.clone().into();
                            if list.contains(&tmp) {
                                out.push((i, tmp));
                            }
                        }
                    } else {
                        return None;
                    }
                }
                Query::ListContains => {
                    return None;
                }
            },
        }
        Some(out)
    }

    pub fn get_if_matches(&self, index: usize, item: DataItem, query: Query) -> Option<DataItem> {
        match self {
            Col::Bool(items) => {
                let it = item.get_bool()?;
                match query {
                    Query::Less => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::LessOrEqual => {
                        let j = items.get(index)?;
                        if *j <= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Equal => {
                        let j = items.get(index)?;
                        if *j == it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::NotEqual => {
                        let j = items.get(index)?;
                        if *j != it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::GreaterOrEqual => {
                        let j = items.get(index)?;
                        if *j >= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Greator => {
                        let j = items.get(index)?;
                        if *j > it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::Int(items) => {
                let it = item.get_int()?;
                match query {
                    Query::Less => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::LessOrEqual => {
                        let j = items.get(index)?;
                        if *j <= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Equal => {
                        let j = items.get(index)?;
                        if *j == it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::NotEqual => {
                        let j = items.get(index)?;
                        if *j != it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::GreaterOrEqual => {
                        let j = items.get(index)?;
                        if *j >= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Greator => {
                        let j = items.get(index)?;
                        if *j > it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::UInt(items) => {
                let it = item.get_uint()?;
                match query {
                    Query::Less => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::LessOrEqual => {
                        let j = items.get(index)?;
                        if *j <= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Equal => {
                        let j = items.get(index)?;
                        if *j == it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::NotEqual => {
                        let j = items.get(index)?;
                        if *j != it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::GreaterOrEqual => {
                        let j = items.get(index)?;
                        if *j >= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Greator => {
                        let j = items.get(index)?;
                        if *j > it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::Float(items) => {
                let it = item.get_float()?;
                match query {
                    Query::Less => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::LessOrEqual => {
                        let j = items.get(index)?;
                        if *j <= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Equal => {
                        let j = items.get(index)?;
                        if *j == it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::NotEqual => {
                        let j = items.get(index)?;
                        if *j != it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::GreaterOrEqual => {
                        let j = items.get(index)?;
                        if *j >= it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Greator => {
                        let j = items.get(index)?;
                        if *j > it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::QueriedContains | Query::QuerierContains | Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::String(items) => {
                let it = item.get_string()?;
                match query {
                    Query::Less => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::LessOrEqual => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Equal => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::NotEqual => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::GreaterOrEqual => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::Greator => {
                        let j = items.get(index)?;
                        if *j < it {
                            return Some(j.clone().into());
                        }
                    }
                    Query::QueriedContains => {
                        let j = items.get(index)?;
                        if j.contains(&it) {
                            return Some(j.clone().into());
                        }
                    }
                    Query::QuerierContains => {
                        let j = items.get(index)?;
                        if it.contains(j) {
                            return Some(j.clone().into());
                        }
                    }
                    Query::ListContains => {
                        return None;
                    }
                }
            }
            Col::List(items) => match query {
                Query::Less => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if *j < it {
                        return Some(j.as_slice().into());
                    }
                }
                Query::LessOrEqual => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if *j <= it {
                        return Some(j.as_slice().into());
                    }
                }
                Query::Equal => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if *j == it {
                        return Some(j.as_slice().into());
                    }
                }
                Query::NotEqual => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if *j != it {
                        return Some(j.as_slice().into());
                    }
                }
                Query::GreaterOrEqual => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if *j >= it {
                        return Some(j.as_slice().into());
                    }
                }
                Query::Greator => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if *j > it {
                        return Some(j.as_slice().into());
                    }
                }
                Query::QueriedContains => {
                    let j = items.get(index)?;
                    if j.contains(&item) {
                        return Some(j.as_slice().into());
                    }
                }
                Query::QuerierContains => {
                    let it = item.get_list()?;
                    let j = items.get(index)?;
                    if it.contains(&j.as_slice().into()) {
                        return Some(j.as_slice().into());
                    }
                }
                Query::ListContains => {
                    let j = items.get(index)?;
                    let mut contains = false;
                    for i in j {
                        if *i == item {
                            contains = true;
                            break;
                        }
                    }
                    if contains {
                        return Some(j.as_slice().into());
                    }
                }
            },
            Col::Struct(_, items) => match query {
                Query::Less => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    if *j < it {
                        return Some(j.clone().into());
                    }
                }
                Query::LessOrEqual => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    if *j <= it {
                        return Some(j.clone().into());
                    }
                }
                Query::Equal => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    if *j == it {
                        return Some(j.clone().into());
                    }
                }
                Query::NotEqual => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    if *j != it {
                        return Some(j.clone().into());
                    }
                }
                Query::GreaterOrEqual => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    if *j >= it {
                        return Some(j.clone().into());
                    }
                }
                Query::Greator => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    if *j > it {
                        return Some(j.clone().into());
                    }
                }

                Query::QueriedContains => {
                    let it = item.get_struct()?;
                    let j = items.get(index)?;
                    for (_, item) in j {
                        if *item == it.clone().into() {
                            return Some(j.clone().into());
                        }
                    }
                }
                Query::QuerierContains => {
                    if let Some(list) = item.get_list() {
                        let j = items.get(index)?;
                        let tmp = j.clone().into();
                        if list.contains(&tmp) {
                            return Some(j.clone().into());
                        }
                    } else {
                        return None;
                    }
                }
                Query::ListContains => {
                    return None;
                }
            },
        }
        None
    }
}
