

use std::{cell::UnsafeCell, collections::{hash_map, HashMap, HashSet}, iter, ops::Range};

use derive_more::{Deref, DerefMut};
use futures::{SinkExt, StreamExt};
use iset::IntervalMap;
use itertools::{Either, Itertools};
use rc_async::sync::broadcast;

use crate::{closure, expr::Expr, forward::executor::Executor, never, utils::{nested::{IntervalTreeN, NestedIntervalTree}, UnsafeCellExt}, value::Value};

use super::size::EV;

pub struct Data {
    expected: &'static [&'static str],
    found: IntervalTreeN,
    event: IntervalTreeN,
    senders: HashMap<Value, broadcast::Sender<Value>>,
    size_limit: usize,
    exceeded_size_limit: bool,
}

impl Data {
    pub fn new(expected: Value, size_limit: usize) -> Option<UnsafeCell<Self>> {
        if let Value::Str(e) = expected {
            Some(Self {
                expected: e,
                found: IntervalTreeN::new(e),
                event: IntervalTreeN::new(e),
                senders: HashMap::new(),
                size_limit,
                exceeded_size_limit: false,
            }.into())
        } else { None }
    }
    pub fn expected_contains(&self, value: Value) -> bool {
        if let Ok(v) = TryInto::<&[&str]>::try_into(value) {
            v.iter().cloned().zip(self.expected.iter().cloned()).all(|(a, b)| b.contains(a) && a.len() > 0)
        } else { false }
    }
    
    pub fn update(&mut self, value: Value, exec: &'static Executor) {
        if exec.size() > self.size_limit  { return; }
        if self.expected_contains(value) {
            self.found.insert(value.to_str());

            let mut senders = Vec::new();
            for v in self.event.superstrings(value.to_str()) {
                if let Some(sd) = self.senders.get(&v.into()) {
                    senders.push(sd.clone());
                }
            }
            for sd in senders {
                sd.send(value);
            }
        }
        
    }

    pub fn lookup_existing(&self, value: Value) -> impl Iterator<Item=Value> + '_ {
        self.found.substrings(value.to_str()).map(|x| x.into())
    }
    
    pub fn listen(&mut self, value: Value) -> Option<broadcast::Reciever<Value>> {
        if !self.expected_contains(value) { return None }
        match self.senders.entry(value) {
            hash_map::Entry::Occupied(o) => {Some(o.get().reciever())}
            hash_map::Entry::Vacant(v) => {
                let sd = v.insert(broadcast::channel());
                self.event.insert_first_occur(value.to_str());
                Some(sd.reciever())
            }
        }

    }

    #[inline(always)]
    pub async fn listen_for_each<T>(&mut self, value: Value, mut f: impl FnMut(Value) -> Option<T>) -> T {
        if let Some(mut rv) = self.listen(value) {
            for v in self.lookup_existing(value) {
                if let Some(t) = f(v) { return t; }
            }
            loop {
                if let Some(t) = f(rv.next().await.unwrap()) { return t; }
            }
        } else { never!() }
    }
}


