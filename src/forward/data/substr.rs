use std::{ops::Range, cell::UnsafeCell, collections::HashSet};

use derive_more::Deref;
use iset::IntervalMap;
use itertools::Itertools;

use crate::{expr::Expr, forward::future::{eventbus::EventBusRc, channel::Channel, task::{currect_task_id, self}, taskrc::TaskORc}, value::Value, utils::UnsafeCellExt, debg};

use super::size::EV;

#[derive(Default)]
pub struct StrData<T: Clone + 'static> {
    pub expected: &'static str,
    found: IntervalMap<usize, Vec<T>>,
    event: IntervalMap<usize, Vec<Channel<T>>>,
}

impl<T: Clone> StrData<T> {
    pub fn new(expected: &'static str) -> Self { Self { expected, found: IntervalMap::new(), event: IntervalMap::new() } }
    #[inline]
    pub fn add_to(&mut self, s: &'static str, value: T, hashset: &mut HashSet<Channel<T>>) {
        for (i, _) in self.expected.match_indices(s) {
            let range = i..(i+s.len());
            // for (r, _) in self.found.iter(range.clone()) {
            //     if r != range && r.start <= range.start && r.end >= range.end {
            //         return;
            //     }
            // }
            match self.found.entry(range.clone()) {
                iset::Entry::Vacant(v) => { v.insert(vec![value.clone()]); }
                iset::Entry::Occupied(mut o) => { o.get_mut().push(value.clone()) }
            }

            for (r, e) in self.event.iter(range.clone()) {
                if r.start <= range.start && r.end >= range.end {
                    for c in e {
                        hashset.insert(c.clone());
                    }
                }
            }
        }
    }
    pub fn all_subrange(&self, range: Range<usize>) -> impl Iterator<Item=&T> + '_ {
        self.found.values(range).flatten()
    }
    pub fn listen_at(&mut self, range: Range<usize>, chan: Channel<T>) {
        crate::debg2!("Task#{} listening at {:?} of {}", currect_task_id(), range, self.expected);
        match self.event.entry(range) {
            iset::Entry::Vacant(v) => {
                v.insert(vec![chan]);
            }
            iset::Entry::Occupied(mut o) => {
                o.get_mut().push(chan);
            }
        }
    }
}

pub struct Data(UnsafeCell<Vec<(usize, StrData<Value>)>>);

impl Data {
    pub fn new(output: Value, indices: &[usize]) -> Self {
        if indices.len() > 0 {
            let output: &'static [&'static str] = output.try_into().unwrap();
            let a = indices.iter().flat_map(|i| if i >= &output.len() { None } else { Some((*i, StrData::<Value>::new(output[*i])))}).collect_vec();
            Data(a.into())
        } else {
            Data(Vec::new().into())
        }
    }
    fn get(&self) -> &mut [(usize, StrData<Value>)] {
        unsafe{ self.0.as_mut().as_mut_slice() }
    }
    #[inline(always)]
    pub fn update(&self, value: Value) -> Result<(), ()> {
        if let Value::Str(s) = value {
            let mut hashset = HashSet::new();
            for (i, d) in self.get() {
                if s[*i].len() > 1 || s[*i].len() > 0 && s.windows(2).all(|w| w[0] == w[1]) {
                    d.add_to(s[*i], value, &mut hashset);
                }
            }
            if hashset.len() > 0 {
                for c in hashset {
                    c.get().send(value)?;
                }
            }
        }
        Ok(())
    }
    #[inline]
    pub fn listen_at(&self, v: Value) -> Channel<Value> {
        let v: &[&str] = v.try_into().unwrap();
        let chan = Channel::new();
        for (i, data) in self.get().iter_mut() {
            let start = if v[*i] == data.expected { 0 } else { data.expected.match_indices(v[*i]).next().unwrap().0 };
            if v[*i].len() > 0 {
                data.listen_at(start..(start + v[*i].len()), chan.clone());
            }
        }
        chan
    }
    #[inline]
    pub fn lookup_existing(&self, v: Value) -> impl Iterator<Item = &Value> + '_ {
        let v: &[&str] = v.try_into().unwrap();
        self.get().iter().flat_map(|(i, data)| {
            if v[*i].len() == 0 { return None; }
            let (start, _) = data.expected.match_indices(v[*i]).next().unwrap();
            Some(data.all_subrange(start..(start + v[*i].len())))
        }).flatten()
    }
    #[inline]
    pub fn generate_task(&'static self, v: Value, mut f: impl FnMut(Value) -> Option<&'static Expr> + 'static) -> TaskORc<&'static Expr> {
        let t = task::spawn(async move {
            for a in self.lookup_existing(v) {
                if let Some(a) = f(*a) { return a; }
            }
            
            let chan = self.listen_at(v);
            loop {
                let a = chan.await;
                if let Some(a) = f(a) { return a; }
            }
        });
        t.tasko()
    }
    #[inline]
    pub async fn try_at<T>(&'static self, v: Value, mut f: impl FnMut(Value) -> Option<T> + 'static) -> T {
        for a in self.lookup_existing(v) {
            if let Some(a) = f(*a) { return a; }
        }
            
        let chan = self.listen_at(v);
        loop {
            let a = chan.await;
            if let Some(a) = f(a) { return a; }
        }
    }
    
}



