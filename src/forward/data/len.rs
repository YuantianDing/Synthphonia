
use std::{
    cell::UnsafeCell,
    collections::{hash_map, HashMap},
    task::Poll, ops::Index,
};

use derive_more::{Constructor, Deref, From, Into, TryInto};

use crate::{
    galloc::AllocForAny,
    expr::Expr,
    forward::future::{task::currect_task, eventbus::{EventBus, EventBusRc}, channel::Channel},
    utils::UnsafeCellExt,
    value::Value, log, info, debg,
};


pub struct Data(UnsafeCell<HashMap<Vec<usize>, Channel<Value>>>);


impl Data {
    pub fn new() -> Self { Data(HashMap::new().into()) }
    fn get(&self) -> &mut HashMap<Vec<usize>, Channel<Value>> {
        unsafe{ self.0.as_mut() }
    }
    #[inline]
    pub fn update(&self, value: Value) -> Result<(), ()> {
        let s: &[&[&str]] = value.try_into().unwrap();
        if let Some(chan) =  self.get().get(&value.length_inside().unwrap()) {
            chan.get().send(value)?;
        }
        Ok(())
    }
    pub fn listen_at(&self, v: Vec<usize>) -> Channel<Value> {
        match self.get().entry(v) {
            hash_map::Entry::Occupied(o) => *o.get(),
            hash_map::Entry::Vacant(v) => {
                let c = Channel::new();
                v.insert(c);
                c
            }
        }
    }
}