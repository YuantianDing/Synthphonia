

use std::{cell::UnsafeCell, collections::{hash_map, HashSet}, iter, ops::Range, sync::Arc};

use derive_more::{Deref, DerefMut};
use futures::{SinkExt, StreamExt};
use iset::IntervalMap;
use itertools::{Either, Itertools};
use spin::Mutex;

use crate::{closure, expr::Expr, forward::executor::Enumerator, never, utils::{nested::{IntervalTreeN, NestedIntervalTree}, UnsafeCellExt}, value::Value};
use async_broadcast::{broadcast, Sender, Receiver};

use super::size::EV;
use ahash::AHashMap as HashMap;

pub struct Data {
    expected: &'static [&'static str],
    found: IntervalTreeN,
    event: IntervalTreeN,
    senders: HashMap<Value, (Sender<Value>, Receiver<Value>)>,
    size_limit: usize,
    exceeded_size_limit: bool,
}

impl Data {
    pub fn new(expected: Value, size_limit: usize) -> Option<Mutex<Self>> {
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
    
    pub fn update(&mut self, value: Value, exec: Arc<Enumerator>) {
        if exec.size() > self.size_limit  { return; }
        if self.expected_contains(value) {
            self.found.insert(value.to_str());

            let mut senders = Vec::new();
            for v in self.event.superstrings(value.to_str()) {
                if let Some(sd) = self.senders.get(&v.into()) {
                    senders.push(sd.0.clone());
                }
            }
            for sd in senders {
                sd.try_broadcast(value);
            }
        }
        
    }

    pub fn lookup_existing(&self, value: Value) -> impl Iterator<Item=Value> + '_ {
        self.found.substrings(value.to_str()).map(|x| x.into())
    }
    
    pub fn listen(&mut self, value: Value) -> Option<Receiver<Value>> {
        if !self.expected_contains(value) { return None }
        match self.senders.entry(value) {
            hash_map::Entry::Occupied(o) => {Some(o.get().1.clone())}
            hash_map::Entry::Vacant(v) => {
                let sd = v.insert(async_broadcast::broadcast(5));
                self.event.insert_first_occur(value.to_str());
                Some(sd.1.clone())
            }
        }

    }

    #[inline(always)]
    pub async fn listen_for_each<T>(this: &Mutex<Self>, value: Value, mut f: impl FnMut(Value) -> Option<T>) -> T {
        let rv: Option<Receiver<Value>> = { this.lock().listen(value) } ;
        if let Some(mut rv) = rv {
            {
                for v in this.lock().lookup_existing(value) {
                    if let Some(t) = f(v) { return t; }
                }
            }
            loop {
                if let Some(t) = f(rv.next().await.unwrap()) { return t; }
            }
        } else { never!() }
    }
}


