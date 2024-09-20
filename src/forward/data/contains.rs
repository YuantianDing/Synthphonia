use std::{collections::HashMap, cell::UnsafeCell};

use itertools::Itertools;
use rc_async::sync::broadcast;

use crate::{value::Value, utils::UnsafeCellExt};


pub type ListStr = &'static [&'static str];
pub fn listsubseq(that: ListStr, this: ListStr) -> bool {
    let mut iter = that.iter().peekable();
    for item in this {
        match iter.peek() {
            Some(i)  if i == &item => { iter.next(); }
            Some(_) => (),
            None => return true,
        }
    }
    return iter.peek() == None;
}
pub struct ListData {
    table: HashMap<String, Vec<(ListStr, broadcast::Sender<Value>)>>
}

impl ListData {
    pub fn new() -> Self { ListData { table: HashMap::new() } }
    pub fn count(&self) -> usize { self.table.len() }
    #[inline]
    pub fn update(&mut self, v: ListStr, value: Value) -> Result<(), ()> {
        for (i, s) in v.iter().enumerate() {
            if let Some(vec) = self.table.get(*s) {
                for (ls,chan) in vec {
                    // if listsubseq(ls, &v[i..]) {
                    //     chan.get().send(value)?;
                    // }
                }
            }
        }
        Ok(())
    }
    pub fn listen_at(&mut self, l: ListStr) {
        self.table.insert(l.first().unwrap().to_string(), vec![(l, broadcast::channel())]);
    }
}

pub struct Data(UnsafeCell<Vec<(usize, ListData)>>);

impl Data {
    pub fn new(output: Value, indices: &[usize]) -> UnsafeCell<Self> {
        if indices.len() > 0 {
            let output: &'static [&'static str] = output.try_into().unwrap();
            let a = indices.iter().flat_map(|i| if i >= &output.len() { None } else { Some((*i, ListData::new()))}).collect_vec();
            Data(a.into()).into()
        } else {
            Data(Vec::new().into()).into()
        }
    }
    pub fn len(&self) -> usize { self.get().len() }
    pub fn count(&self) -> usize { self.get().iter().map(|(_, d)| d.count()).sum() }
    fn get(&self) -> &mut [(usize, ListData)] {
        unsafe{ self.0.as_mut().as_mut_slice() }
    }
    pub fn update(&self, value: Value) -> Result<(), ()> {
        if let Value::ListStr(ls) = value {
            for (i, d) in self.get() {
                d.update(ls[*i], value)?;
            }
        }
        Ok(())
    }
    pub fn listen_at(&self, v: Value) -> () {
        let v: &'static [ListStr] = v.try_into().unwrap();
        for (i, data) in self.get().iter_mut() {
            if v[*i].len() > 0 {
                data.listen_at(v[*i]);
            }
        }
    }
}



