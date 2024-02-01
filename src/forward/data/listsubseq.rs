use std::{collections::HashMap, cell::UnsafeCell};

use itertools::Itertools;

use crate::{value::Value, forward::future::channel::Channel, utils::UnsafeCellExt};





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
    table: HashMap<String, Vec<(ListStr, Channel<Value>)>>
}

impl ListData {
    pub fn new() -> Self { ListData { table: HashMap::new() } }
    #[inline]
    pub fn update(&mut self, v: ListStr, value: Value) -> Result<(), ()> {
        for (i, s) in v.iter().enumerate() {
            if let Some(vec) = self.table.get(*s) {
                for (ls,chan) in vec {
                    if listsubseq(ls, &v[i..]) {
                        chan.get().send(value)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn listen_at(&mut self, l: ListStr, chan: Channel<Value>) {
        if let Some(v) = self.table.get_mut(*l.first().unwrap()) {
            v.push((l, chan));
        } else {
            self.table.insert(l.first().unwrap().to_string(), vec![(l, chan)]);
        }
    }
}

pub struct Data(UnsafeCell<Vec<(usize, ListData)>>);

impl Data {
    pub fn new(output: Value, indices: &[usize]) -> Self {
        let output: &'static [&'static str] = output.try_into().unwrap();
        let a = indices.iter().flat_map(|i| if i >= &output.len() { None } else { Some((*i, ListData::new()))}).collect_vec();
        Data(a.into())
    }
    pub fn len(&self) -> usize { self.get().len() }
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
    pub fn listen_at(&self, v: Value) -> Channel<Value> {
        let v: &'static [ListStr] = v.try_into().unwrap();
        let chan = Channel::new();
        for (i, data) in self.get().iter_mut() {
            if v[*i].len() > 0 {
                data.listen_at(v[*i], chan);
            }
        }
        chan
    }
}



