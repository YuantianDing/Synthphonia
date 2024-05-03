

use std::{cell::UnsafeCell, collections::HashSet, iter, ops::Range};

use derive_more::{Deref, DerefMut};
use futures::SinkExt;
use iset::IntervalMap;
use itertools::{Either, Itertools};
use rc_async::sync::broadcast;

use crate::{closure, expr::Expr, forward::executor::Executor, utils::UnsafeCellExt, value::Value};

use super::size::EV;
pub fn subrange(r1: &[Range<usize>], r2: &[Range<usize>]) -> bool {
    assert_eq!(r1.len(), r2.len());
    r1.iter().zip(r2.iter()).all(|(a,b)| b.start <= a.start && a.end <= b.end)
}


pub enum NestedIntervalTree<T>{
    Node(IntervalMap<usize, Box<NestedIntervalTree<T>>>),
    Leaf(T)
}

impl<T: Clone> NestedIntervalTree<T> {
    pub fn build(mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>>> + Clone, value: T) -> Self {
        if let Some(head) = ranges.next() {
            let mut maps = IntervalMap::new();
            for range in head {
                let inner = Self::build(ranges.clone(), value.clone());
                maps.insert(range, inner.into());
            }
            Self::Node(maps)
        } else {
            Self::Leaf(value)
        }
    }
    pub fn insert_using_iter<'a: 'b, 'b>(&'a mut self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b, update: &impl Fn(&mut T) -> (), default: T) {
        let head = ranges.next();
        match (self, head) {
            (NestedIntervalTree::Node(maps), Some(head)) => {
                for range in head {
                    if let Some(r) = maps.get_mut(range.clone()) {
                        r.insert_using_iter(ranges.clone(), update, default.clone());
                    } else {
                        maps.insert(range, Self::build(ranges.clone(), default.clone()).into());
                    }
                }
            }
            (NestedIntervalTree::Leaf(v), None) => {
                update(v);
            }
            _ => panic!("DeepIntervalTree have a different number of ranges indices."),
        }
    }
    pub fn insert_multiple<'a: 'b, 'b>(&'a mut self, ranges: &Vec<Vec<Range<usize>>>, value: T) {
        self.insert_using_iter(ranges.iter().map(|x| x.iter().cloned()), &|_| (), value)
    }
    pub fn insert<'a: 'b, 'b>(&'a mut self, ranges: &[Range<usize>], value: T) {
        self.insert_using_iter(ranges.iter().map(|x| iter::once(x.clone()).into_iter()), &|_| (), value)
    }
}

impl<T> NestedIntervalTree<T> {
    pub fn new() -> Self {
        Self::Node(IntervalMap::new())
    }
    pub fn get(&self, ranges: &[Range<usize>]) -> Option<&T> {
        match self {
            NestedIntervalTree::Node(maps) if ranges.len() > 0 => {
                let (head, tail) = (&ranges[0], &ranges[1..]);
                maps.get(head.clone()).and_then(|x| x.get(tail))
            }
            NestedIntervalTree::Leaf(v) if ranges.len() == 0 => Some(v),
            _ => None,
        }
    }
    pub fn superrange_using_iter<'a: 'b, 'b>(&'a self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b) -> Box<dyn Iterator<Item=&T> + 'b> {
        let head = ranges.next();
        match (self, head) {
            (NestedIntervalTree::Node(maps), Some(head)) => {
                let it = head.flat_map(move |head| {
                    maps.iter(head.clone())
                        .filter(move |(r, _)| head.start >= r.start && r.end >= head.end) 
                        .flat_map(closure![clone ranges; move |(_, t)| t.superrange_using_iter(ranges.clone())])
                });
                Box::new(it)
            }
            (NestedIntervalTree::Leaf(v), None) => Box::new(Some(v).into_iter()),
            _ => panic!("DeepIntervalTree have a different number of ranges indices."),
        }
    }
    pub fn subrange_using_iter<'a: 'b, 'b>(&'a self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b) -> Box<dyn Iterator<Item=&T> + 'b> {
        let head = ranges.next();
        match (self, head) {
            (NestedIntervalTree::Node(maps), Some(head)) => {
                let it = head.flat_map(move |head| {
                    maps.iter(head.clone())
                        .filter(move |(r, _)| r.start >= head.start && head.end >= r.end) 
                        .flat_map(closure![clone ranges; move |(_, t)| t.subrange_using_iter(ranges.clone())])
                });
                Box::new(it)
            }
            (NestedIntervalTree::Leaf(v), None) => Box::new(Some(v).into_iter()),
            _ => panic!("DeepIntervalTree have a different number of ranges indices."),
        }
    }
    pub fn superrange_multiple<'a: 'b, 'b>(&'a self, ranges: &'b Vec<Vec<Range<usize>>>) -> Box<dyn Iterator<Item=&T> + 'b> {
        self.superrange_using_iter(ranges.iter().map(|x| x.iter().cloned()))
    }
    pub fn subrange_multiple<'a: 'b, 'b>(&'a self, ranges: &'b Vec<Vec<Range<usize>>>) -> Box<dyn Iterator<Item=&T> + 'b> {
        self.subrange_using_iter(ranges.iter().map(|x| x.iter().cloned()))
    }
    pub fn superrange<'a: 'b, 'b>(&'a self, ranges: Vec<Range<usize>>) -> Box<dyn Iterator<Item=&T> + 'b> {
        self.superrange_using_iter(ranges.into_iter().map(|x| std::iter::once(x)))
    }
    pub fn subrange<'a: 'b, 'b>(&'a self, ranges: Vec<Range<usize>>) -> Box<dyn Iterator<Item=&T> + 'b> {
        self.subrange_using_iter(ranges.into_iter().map(|x| std::iter::once(x)))
    }
}


pub struct Data {
    expected: &'static [&'static str],
    found: NestedIntervalTree<Value>,
    event: NestedIntervalTree<broadcast::Sender<Value>>,
    size_limit: usize,
    exceeded_size_limit: bool,
}

impl Data {
    pub fn new(expected: Value, size_limit: usize) -> Option<UnsafeCell<Self>> {
        if let Value::Str(e) = expected {
            Some(Self {
                expected: e,
                found: NestedIntervalTree::new(),
                event: NestedIntervalTree::new(),
                size_limit,
                exceeded_size_limit: false,
            }.into())
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
    
    pub fn update(&mut self, value: Value, exec: &'static Executor) {
        if exec.size() > self.size_limit  { return; }

        if let Some(ranges) = self.to_ranges(value) {
            let _ = self.found.insert_multiple(&ranges, value);

            for sd in self.event.superrange_multiple(&ranges) {
                sd.send(value);
            }
        }
    }

    pub fn lookup_existing(&self, value: Value) -> impl Iterator<Item=Value> + '_ {
        self.to_range(value).into_iter().flat_map(|x| self.found.subrange(x).cloned())
    }
    
    pub fn listen(&mut self, value: Value) -> broadcast::Reciever<Value> {
        let ranges = self.to_range(value).unwrap();
        if let Some(a) = self.event.get(&ranges) {
            a.reciever()
        } else {
            let sd = broadcast::channel();
            let rv = sd.reciever();
            self.event.insert(&ranges, sd);
            rv
        }
    }

}


