
use std::{
    cell::UnsafeCell, collections::hash_map, hash::Hash, ops::Index, sync::Arc, task::Poll
};

use derive_more::{Constructor, Deref, From, Into, TryInto};
use futures::StreamExt;
use itertools::Itertools;
use spin::Mutex;

use crate::{
    debg, expr::Expr, forward::executor::Enumerator, galloc::AllocForAny, info, log, utils::UnsafeCellExt, value::Value
};
use async_broadcast::{broadcast, Sender, Receiver};

pub struct Data{
    found: HashMap<Vec<usize>, Vec<Value>>,
    event: HashMap<Vec<usize>, (Sender<Value>, Receiver<Value>)>,
    len_limit: usize,
}

use ahash::AHashMap as HashMap;


impl Data {
    pub fn new() -> Self { Data{ found: HashMap::new().into(), event: HashMap::new(), len_limit: 3 } }
    #[inline]
    pub fn update(&mut self, value: Value, exec: Arc<Enumerator>) {
        if exec.size() > self.len_limit { return; }
        if !matches!(value, Value::ListStr(_)) { return; }
        let s: &[&[&str]] = value.try_into().unwrap();
        let a = value.length_inside().unwrap();
        if let Some(chan) =  self.event.get(&a) {
            chan.0.try_broadcast(value);
        }
        match self.found.entry(a.clone()) {
            hash_map::Entry::Occupied(mut o) => { o.get_mut().push(value); }
            hash_map::Entry::Vacant(v) => { v.insert(vec![value]); }
        }
    }
    pub fn listen_at(&mut self, v: Vec<usize>) -> async_broadcast::Receiver<Value> {
        match self.event.entry(v) {
            hash_map::Entry::Occupied(o) => o.get().1.clone(),
            hash_map::Entry::Vacant(v) => v.insert(async_broadcast::broadcast(10)).1.clone(),
        }
    }
    #[inline(always)]
    pub async fn listen_for_each<T>(this: &Mutex<Self>, value: Value, mut f: impl FnMut(Value) -> Option<T>) -> T {
        let v = value.length_inside().unwrap();
        if let Some(value) = some_fun(this, &v, &mut f) {
            return value;
        }

        let mut rv = {
            this.lock().listen_at(v)
        };
        
        loop {
            if let Some(t) = f(rv.next().await.unwrap()) { return t; }
        }
    }
    #[inline(always)]
    pub async fn listen_once(this: &Mutex<Self>, value: Value) -> Value {
        let v = value.length_inside().unwrap();
        {
            if let Some(vec) = this.lock().found.get(&v) {
                if let Some(v) = vec.first() {
                    return *v;
                }
            }
        }

        let mut rv = {
            this.lock().listen_at(v)
        };
        
        rv.next().await.unwrap()
    }
}

fn some_fun<T>(this: &spin::mutex::Mutex<Data>, v: &Vec<usize>, f: &mut impl FnMut(Value) -> Option<T>) -> Option<T> {
    if let Some(vec) = this.lock().found.get(v) {
        for v in vec {
            if let Some(t) = f(*v) { return Some(t); }
        }
    }
    None
}
