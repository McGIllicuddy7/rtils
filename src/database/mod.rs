pub mod col;
pub mod item;
pub mod list;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};

use col::{Col, Query};
pub use item::{DataItem, DataType};
use list::ArrayList;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Table {
    pub names: HashMap<String, usize>,
    pub schema: Vec<DataType>,
    pub data: Vec<Col>,
    pub sorted: bool,
}
impl Table {
    pub fn new(schema: &[DataType], names: &[&str], sorted: bool) -> Self {
        assert!(schema.len() == names.len());
        let mut name_table = HashMap::new();
        let scheme = schema.to_vec();
        for (i, name) in names.iter().enumerate() {
            name_table.insert(name.to_string(), i);
        }
        let mut data = Vec::new();
        for i in schema {
            data.push(match i {
                DataType::Bool => Col::Bool(ArrayList::new()),
                DataType::Int => Col::Int(ArrayList::new()),
                DataType::UInt => Col::UInt(ArrayList::new()),
                DataType::Float => Col::Float(ArrayList::new()),
                DataType::String => Col::String(ArrayList::new()),
                DataType::List => Col::List(ArrayList::new()),
                DataType::Struct => Col::Struct(String::new(), ArrayList::new()),
            })
        }
        Self {
            names: name_table,
            schema: scheme,
            data,
            sorted,
        }
    }

    pub fn remove_entry(&mut self, row: usize) {
        for i in &mut self.data {
            i.remove_at(row);
        }
    }

    pub fn get_row_base(&mut self, row: usize) -> Option<Vec<DataItem>> {
        let mut out = Vec::new();
        for i in 0..self.data.len() {
            out.push(self.data[i].get(row)?);
        }
        Some(out)
    }

    pub fn get_row(&self, row: usize) -> Option<BTreeMap<String, DataItem>> {
        let mut names = HashMap::new();
        for (x, y) in &self.names {
            names.insert(*y, x.clone());
        }
        let mut out = BTreeMap::new();
        for i in 0..self.data.len() {
            out.insert(names[&i].clone(), self.data[i].get(row)?);
        }
        Some(out)
    }

    pub fn validate_entry(&self, entry: Vec<DataItem>) -> Result<Vec<DataItem>, Vec<DataItem>> {
        if entry.len() != self.schema.len() {
            return Err(entry);
        }
        for i in 0..self.schema.len() {
            if entry[i].get_type() != self.schema[i] {
                return Err(entry);
            }
        }
        Ok(entry)
    }

    pub fn replace_entry(&mut self, row: usize, entry: Vec<DataItem>) -> Result<(), Vec<DataItem>> {
        let entry = self.validate_entry(entry)?;
        if self.sorted {
            self.remove_entry(row);
            self.add_sorted(entry);
        } else {
            for (i, item) in entry.into_iter().enumerate() {
                self.data[i].replace_at(item, row);
            }
        }
        Ok(())
    }

    pub fn add_entry(&mut self, entry: Vec<DataItem>) -> Result<(), Vec<DataItem>> {
        let entry = self.validate_entry(entry)?;
        if self.sorted {
            self.add_sorted(entry);
        } else {
            for (i, item) in entry.into_iter().enumerate() {
                self.data[i].add(item);
            }
        }
        Ok(())
    }

    pub fn find_matching(
        &mut self,
        item: DataItem,
        query: Query,
    ) -> Option<Vec<(usize, DataItem)>> {
        let mut names = HashMap::new();
        for (x, y) in &self.names {
            names.insert(*y, x.clone());
        }
        let mut out = Vec::new();
        let bases = self.data[0].find_matching(item, query)?;
        for (i, b) in bases {
            let mut v = BTreeMap::new();
            v.insert(names.get(&i)?.clone(), b);
            for j in 1..self.data.len() {
                v.insert(names.get(&j)?.clone(), self.data[j].get(i)?);
            }
            out.push((i, v.into()));
        }
        Some(out)
    }

    pub fn remove_matching(&mut self, item: DataItem, query: Query) {
        let mut i = 0;
        while i < self.data.len() {
            if let Some(_) = self.data[i].get_if_matches(i, item.clone(), query.clone()) {
                for j in 0..self.data.len() {
                    self.data[i].remove_at(j);
                }
            } else {
                i += 1;
            }
        }
    }

    pub fn add_sorted(&mut self, entry: Vec<DataItem>) -> Option<()> {
        let idx = self.data[0].add_sorted(entry[0].clone())?;
        for i in 1..self.data.len() {
            self.data[i].insert_at(entry[i].clone(), idx);
        }
        Some(())
    }

    pub fn sort(&mut self) -> Option<()> {
        let mut out = self.clone();
        for i in &mut out.data {
            i.clear();
        }
        for i in 0..self.data[0].len() {
            let row = self.get_row_base(i)?;
            out.add_sorted(row);
        }
        *self = out;
        Some(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DataBase {
    pub tables: HashMap<String, Table>,
    pub used: VecDeque<String>,
}
