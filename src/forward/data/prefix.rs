

use std::{cell::UnsafeCell, collections::{hash_map, HashMap, HashSet}, iter, ops::Range};

use derive_more::{Deref, DerefMut};
use futures::{SinkExt, StreamExt};
use iset::IntervalMap;
use itertools::{Either, Itertools};
use radix_trie::Trie;
use spin::Mutex;

use crate::{closure, debg2, expr::Expr, forward::executor::Enumerator, utils::{nested::RadixTrieN, UnsafeCellExt}, value::{self, Value}};
use async_broadcast::{broadcast, Sender, Receiver};

use super::size::EV;
pub type Indices = Vec<usize>;


pub struct Data {
    expected: &'static [&'static str],
    found: RadixTrieN,
    event: RadixTrieN,
    senders: HashMap<Value, (Sender<Value>, Receiver<Value>)>,
    size_limit: usize,
}

impl Data {
    pub fn new(expected: Value, size_limit: usize) -> Option<Mutex<Self>> {
        if let Value::Str(e) = expected {
            Some(Self {
                expected: e,
                found: RadixTrieN::new(e.len()),
                event: RadixTrieN::new(e.len()),
                senders: HashMap::new(),
                size_limit,
            }.into())
        } else { None }
    }
    
    pub fn to_ranges(&self, value: Value) -> Option<Vec<Vec<Range<usize>>>> {
        if let Ok(v) = TryInto::<&[&str]>::try_into(value) {
            assert!(v.len() == self.expected.len());
            let mut result = Vec::with_capacity(v.len());
            for (&e, &x) in self.expected.iter().zip(v.iter()) {
                if x.len() == 0 {  result.push(vec![0..0]); continue; }
                let r = e.match_indices(x).map(|(i, _)| i..(i+x.len())).collect_vec();
                if r.len() == 0 { return None; }
                result.push(r)
            };
            Some(result)
        } else { None }
    }
    pub fn to_range(&self, value: Value) -> Option<Vec<Range<usize>>> {
        if let Ok(v) = TryInto::<&[&str]>::try_into(value) {
            assert!(v.len() == self.expected.len());
            let mut result = Vec::with_capacity(v.len());
            for (&e, &x) in self.expected.iter().zip(v.iter()) {
                if x.len() == 0 { result.push(0..0); continue; }
                if let Some((i, _)) = e.match_indices(x).next() {
                    result.push(i..(i+x.len()))
                } else { return None; }
            };
            Some(result)
        } else { None }
    }
    pub fn expected_contains(&self, value: Value) -> bool {
        if let Ok(v) = TryInto::<&[&str]>::try_into(value) {
            v.iter().cloned().zip(self.expected.iter().cloned()).all(|(a, b)| b.contains(a))
        } else { false }
    }
    pub fn update(&mut self, value: Value, exec: &'static Enumerator) {
        if exec.size() > self.size_limit { return; }
        
        if self.expected_contains(value) {
            self.found.insert(value.to_str());
            let mut senders = Vec::new();
            for v in self.event.superfixes(value.to_str()) {
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
        self.found.prefixes(value.to_str()).map(|x| x.into())
    }
    
    pub fn listen(&mut self, value: Value) -> async_broadcast::Receiver<Value> {
        match self.senders.entry(value) {
            hash_map::Entry::Occupied(o) => {o.get().1.clone()}
            hash_map::Entry::Vacant(v) => {
                let sd = v.insert(async_broadcast::broadcast(5));
                self.event.insert(value.to_str());
                sd.1.clone()
            }
        }
    }
    
    #[inline(always)]
    pub async fn listen_for_each<T>(this: & Mutex<Self>, value: Value, mut f: impl FnMut(Value) -> Option<T>) -> T {
        for v in this.lock().lookup_existing(value) {
            if let Some(t) = f(v) { return t; }
        }
        let mut rv = this.lock().listen(value);
        loop {
            if let Some(t) = f(rv.next().await.unwrap()) { return t; }
        }
    }
}


