

use std::{cell::UnsafeCell, collections::{HashMap, HashSet}, iter, ops::Range};

use derive_more::{Deref, DerefMut};
use futures::SinkExt;
use iset::IntervalMap;
use itertools::{Either, Itertools};
use rc_async::sync::broadcast;
use smallvec::SmallVec;
use tokio::{runtime::Handle, sync::mpsc};

use crate::{closure, expr::Expr, forward::executor::Executor, utils::UnsafeCellExt, value::Value};

use super::size::EV;
pub type Indices = SmallVec<[usize; 4]>;

pub struct EData {
    expected: &'static [&'static str],
    found: HashMap<Indices, Vec<(Indices, Value)>>,
    event: HashMap<Indices, Vec<(Indices, broadcast::Sender<Value>)>>,
    size_limit: usize,
    exceeded_size_limit: bool,
}

impl EData {
    pub fn new(expected: Value, size_limit: usize) -> Option<Self> {
        if let Value::Str(e) = expected {
            Some(Self {
                expected: e,
                found: HashMap::new(),
                event: HashMap::new(),
                size_limit,
                exceeded_size_limit: false,
            })
        } else { None }
    }
    
    pub fn to_ranges(&self, value: Value) -> Option<Vec<Vec<Range<usize>>>> {
        if let Ok(v) = TryInto::<&[&str]>::try_into(value) {
            assert!(v.len() == self.expected.len());
            let mut result = Vec::with_capacity(v.len());
            for (&e, &x) in self.expected.iter().zip(v.iter()) {
                if x.len() == 0 { return None; }
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
                if x.len() == 0 { return None; }
                if let Some((i, _)) = e.match_indices(x).next() {
                    result.push(i..(i+x.len()))
                } else { return None; }
            };
            Some(result)
        } else { None }
    }
    
    pub fn update(&mut self, exec: &'static Executor, value: Value, expr: &'static Expr) {
        if self.exceeded_size_limit { return; }
        if exec.size() > self.size_limit {
            self.exceeded_size_limit = true;
            return;
        }
        if let Some(ranges) = self.to_ranges(value) {
            for ranges in ranges.into_iter().map(|v| v.into_iter()).multi_cartesian_product() {
                let starts: Indices = ranges.iter().map(|x| x.start).collect();
                let ends: Indices = ranges.iter().map(|x| x.start).collect();
                for (ends2, sd) in self.event.get(&starts).iter().flat_map(|x| x.into_iter()) {
                    if ends.iter().zip(ends2.iter()).all(|(a, b)| a <= b) {
                        let _ = sd.send(value);
                    }
                }
                match self.found.entry(starts) {
                    std::collections::hash_map::Entry::Occupied(mut o) => { o.get_mut().push((ends, expr)); }
                    std::collections::hash_map::Entry::Vacant(mut v) => { v.insert(vec![(ends, expr)]); }
                }
            }
        }
    }

    pub fn lookup_existing(&self, value: Value) -> impl Iterator<Item=Value> + '_ {
        self.to_range(value).into_iter().flat_map(move |ranges| {
            let starts: Indices = ranges.iter().map(|x| x.start).collect();
            self.found.get(&starts).into_iter().flat_map(move |x| {
                let ends: Indices = ranges.iter().map(|x| x.start).collect();
                x.iter().filter(move |(ends2, _)| ends2.iter().zip(ends.iter()).all(|(a,b)| a <= b) )
                .map(|(_, e)| *e)
            })
        })
    }
    
    pub fn send_to(&mut self, value: Value, sd: mpsc::Sender<&'static Expr>) {
        if self.exceeded_size_limit { return; }
        if let Some(ranges) = self.to_range(value) {
            let starts: Indices = ranges.iter().map(|x| x.start).collect();
            let ends: Indices = ranges.iter().map(|x| x.start).collect();
            match self.event.entry(starts) {
                std::collections::hash_map::Entry::Occupied(mut o) => { o.get_mut().push((ends, sd)); }
                std::collections::hash_map::Entry::Vacant(v) => { v.insert(vec![(ends, sd)]); }
            }
        }
    }
}


