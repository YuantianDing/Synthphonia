use std::{collections::HashMap, cell::UnsafeCell};

use itertools::Itertools;
use simple_rc_async::sync::broadcast::{self, Sender};

use crate::{value::Value, utils::UnsafeCellExt};


pub type ListStr = &'static [&'static str];
/// Determines whether all elements of the first sequence appear in order within the second sequence. 
/// 
/// The function iterates over the second sequence while maintaining an iterator over the first sequence. 
/// For each element encountered in the second sequence, if it matches the next expected element from the first sequence, the iterator advances. 
/// The process terminates early and returns true if all elements of the first sequence have been matched; otherwise, it returns false.
/// 
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
pub(crate) struct ListData {
    table: HashMap<String, Vec<(ListStr, broadcast::Sender<Value>)>>
}

impl ListData {
    /// Creates and returns a new instance containing an empty hash table. 
    /// This function initializes an internal state with a hash map that can later be employed to store associations of string lists and related broadcast sender values for synthesis term dispatching.
    pub(crate) fn new() -> Self { ListData { table: HashMap::new() } }
    #[inline]
    /// Updates term dispatch channels by evaluating each element in the provided list and propagating associated values where applicable.
    pub(crate) fn update(&mut self, v: ListStr, value: Value) -> Result<(), ()> {
        todo!()
    }
    /// Adds a new tuple consisting of a list-based key and its associated communication channel to a managed table. 
    pub(crate) fn listen_at(&mut self, l: ListStr, chan: Sender<Value>) {
        todo!()
    }
}
/// Term dispatcher for contains
pub struct Data(UnsafeCell<Vec<(usize, ListData)>>);

impl Data {

    pub fn new(output: Value, indices: &[usize]) -> Self {
        todo!();
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
}



