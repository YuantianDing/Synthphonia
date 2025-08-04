
use std::{
    cell::UnsafeCell, collections::{hash_map}, hash::Hash, ops::Index, task::Poll
};

use derive_more::{Constructor, Deref, From, Into, TryInto};
use futures::StreamExt;
use itertools::Itertools;
use simple_rc_async::sync::broadcast;

use crate::{
    debg, expr::Expr, forward::executor::Executor, galloc::AllocForAny, info, log, utils::UnsafeCellExt, value::Value
};

/// Term Dispatcher for length
pub struct Data {
    found: HashMap<Vec<usize>, Vec<Value>>,
    event: HashMap<Vec<usize>, broadcast::Sender<Value>>,
    len_limit: usize,
}

use ahash::AHashMap as HashMap;


impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl Data {
    pub fn new() -> Self { Data{ found: HashMap::new(), event: HashMap::new(), len_limit: 3 } }
    #[inline]
    pub fn update(&mut self, value: Value, exec: &'static Executor) {
        if exec.size() > self.len_limit { return; }
        if !matches!(value, Value::ListStr(_)) { return; }
        let s: &[&[&str]] = value.try_into().unwrap();
        let a = value.length_inside().unwrap();
        if let Some(chan) =  self.event.get(&a) {
            chan.send(value);
        }
        match self.found.entry(a.clone()) {
            hash_map::Entry::Occupied(mut o) => { o.get_mut().push(value); }
            hash_map::Entry::Vacant(v) => { v.insert(vec![value]); }
        }
    }
    pub fn listen_at(&mut self, v: Vec<usize>) -> broadcast::Reciever<Value> {
        match self.event.entry(v) {
            hash_map::Entry::Occupied(o) => o.get().reciever(),
            hash_map::Entry::Vacant(v) => v.insert(broadcast::channel()).reciever(),
        }
    }
    #[inline(always)]
    pub async fn listen_for_each<T>(&mut self, value: Vec<usize>, mut f: impl FnMut(Value) -> Option<T>) -> T {
        if let Some(vec) = self.found.get(&value) {
            for v in vec {
                if let Some(t) = f(*v) { return t; }
            }
        }

        let mut rv = self.listen_at(value);
        loop {
            if let Some(t) = f(rv.next().await.unwrap()) { return t; }
        }
    }
    #[inline(always)]
    pub async fn listen_once(&mut self, value: Value) -> Value {
        let v = value.length_inside().unwrap();

        if let Some(vec) = self.found.get(&v) {
            if let Some(v) = vec.first() {
                return *v;
            }
        }

        self.listen_at(v).next().await.unwrap()
    }
}
