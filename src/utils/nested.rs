use std::{iter, ops::Range};

use iset::IntervalMap;
use itertools::{Itertools};
use radix_trie::{Trie, TrieCommon};

use crate::closure;
use ahash::AHashMap as HashMap;
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
    pub fn insert_using_iter<'a: 'b, 'b>(&'a mut self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b, update: &impl Fn(&mut T), default: T) {
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
        self.insert_using_iter(ranges.iter().map(|x| iter::once(x.clone())), &|_| (), value)
    }
}

impl<T> Default for NestedIntervalTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> NestedIntervalTree<T> {
    pub fn new() -> Self {
        Self::Node(IntervalMap::new())
    }
    pub fn get(&self, ranges: &[Range<usize>]) -> Option<&T> {
        match self {
            NestedIntervalTree::Node(maps) if !ranges.is_empty() => {
                let (head, tail) = (&ranges[0], &ranges[1..]);
                maps.get(head.clone()).and_then(|x| x.get(tail))
            }
            NestedIntervalTree::Leaf(v) if ranges.is_empty() => Some(v),
            _ => None,
        }
    }
    pub fn superrange_using_iter<'a: 'b, 'b>(&'a self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b) -> Box<dyn Iterator<Item=&'b T> + 'b> {
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
    pub fn subrange_using_iter<'a: 'b, 'b>(&'a self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b) -> Box<dyn Iterator<Item=&'b T> + 'b> {
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
    pub fn superrange_multiple<'a: 'b, 'b>(&'a self, ranges: &'b Vec<Vec<Range<usize>>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.superrange_using_iter(ranges.iter().map(|x| x.iter().cloned()))
    }
    pub fn subrange_multiple<'a: 'b, 'b>(&'a self, ranges: &'b Vec<Vec<Range<usize>>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.subrange_using_iter(ranges.iter().map(|x| x.iter().cloned()))
    }
    pub fn superrange<'a: 'b, 'b>(&'a self, ranges: Vec<Range<usize>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.superrange_using_iter(ranges.into_iter().map(std::iter::once))
    }
    pub fn subrange<'a: 'b, 'b>(&'a self, ranges: Vec<Range<usize>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.subrange_using_iter(ranges.into_iter().map(std::iter::once))
    }
}

pub struct Encoder<K: std::hash::Hash, V> {
    values: Vec<(K, V)>,
    codes: HashMap<K, u32>
}

impl<K: std::hash::Hash + Clone + std::cmp::Eq, V> Default for Encoder<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::hash::Hash + Clone + std::cmp::Eq, V> Encoder<K, V> {
    pub fn new() -> Self {
        Encoder { values: Vec::new(), codes: HashMap::new() }
    }
    pub fn insert(&mut self, k: K, v: V) -> u32 {
        let code = self.values.len() as u32;
        self.values.push((k.clone(), v));
        self.codes.insert(k, code);
        code
    }
    pub fn encode(&self, t: &K) -> Option<u32> {
        self.codes.get(t).cloned()
    }
    pub fn decode(&self, t: u32) -> &K {
        &self.values[t as usize].0
    }
    pub fn value(&self, t: u32) -> &V {
        &self.values[t as usize].1
    }
}

pub struct RadixTrieN(Vec<Trie<&'static str, Vec<&'static [&'static str]>>>);

impl RadixTrieN {
    pub fn new(len: usize) -> Self {
        Self( (0..len).map(|_| Trie::new()).collect_vec() )
    }
    pub fn insert(&mut self, key: &'static [&'static str]) {
        for (s, v) in key.iter().cloned().zip(self.0.iter_mut()) {
            if let Some(a) = v.get_mut(s) {
                a.push(key);
            } else {
                v.insert(s, vec![key]);
            }
        }
    }
    #[inline]
    pub fn superfixes(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().max_by_key(|(i, x)| x.len()).unwrap();
        self.0[i].subtrie(key[i]).map(|x| x.values().flat_map(|v| v.iter().cloned().filter(|x| is_prefix(key, x))) ).into_iter().flatten()
    }
    #[inline]
    pub fn prefixes(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().min_by_key(|(i, x)| x.len()).unwrap();
        PrefixIter{ trie: &self.0[i], key: Some(key[i])}.flat_map(|x|  x.iter().cloned().filter(|x| is_prefix(x, key)) )
    }
}

fn is_prefix(x: &[&str], k: &[&str]) -> bool {
    x.iter().cloned().zip(k.iter().cloned()).all(|(a,b)| b.starts_with(a))
}

pub struct PrefixIter<'a, T>{
    trie: &'a Trie<&'static str, T>,
    key: Option<&'static str>,
}

impl<'a, T> Iterator for PrefixIter<'a, T> {
    type Item=&'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.key {
            self.trie.get_ancestor(key).and_then(|x| {
                x.key().map(|k| {
                    if k.is_empty() {
                        self.key = None;    
                    } else {
                        self.key = Some(&k[0..k.len()-1]);
                    }
                    x.value().unwrap()
                })
            })
        } else { None }
    }
}

pub struct IntervalTreeN {
    expected: &'static [&'static str],
    maps: Vec<IntervalMap<usize, Vec<&'static [&'static str]>>>
}

impl IntervalTreeN {
    pub fn new(expected: &'static [&'static str]) -> Self {
        Self { expected, maps: (0..expected.len()).map(|_| IntervalMap::new()).collect_vec()}
    }
    pub fn insert(&mut self, key: &'static [&'static str]) {
        for (k, (e, v)) in key.iter().cloned().zip(self.expected.iter().cloned().zip(self.maps.iter_mut())) {
            for (i, _) in e.match_indices(k) {
                let range = i..(i + k.len());
                if let Some(l) = v.get_mut(range.clone()) {
                    l.push(key);
                } else {
                    v.insert(range, vec![key]);
                }
            }
        }
    }
    pub fn insert_first_occur(&mut self, key: &'static [&'static str]) {
        for (k, (e, v)) in key.iter().cloned().zip(self.expected.iter().cloned().zip(self.maps.iter_mut())) {
            if let Some((i, _)) = e.match_indices(k).next() {
                let range = i..(i + k.len());
                if let Some(l) = v.get_mut(range.clone()) {
                    l.push(key);
                } else {
                    v.insert(range, vec![key]);
                }
            }
        }
    }
    #[inline]
    pub fn superstrings(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().max_by_key(|(_, x)| x.len()).unwrap();
        self.expected[i].match_indices(key[i]).map(move |(k, _)| k..(k+key[i].len())).flat_map(move |range| {
            self.maps[i].iter(range).flat_map(move |(_, v)| {
                v.iter().cloned().filter(|x| is_substring(key, x))
            })
        })
    }
    #[inline]
    pub fn substrings(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().min_by_key(|(_, x)| x.len()).unwrap();
        self.expected[i].match_indices(key[i]).map(move |(k, _)| k..(k+key[i].len())).flat_map(move |range| {
            self.maps[i].iter(range).flat_map(move |(_, v)| {
                v.iter().cloned().filter(|x| is_substring(x, key))
            })
        })
    }
}

fn is_substring(x: &[&str], k: &[&str]) -> bool {
    x.iter().cloned().zip(k.iter().cloned()).all(|(a,b)| b.contains(a))
}


#[cfg(test)]
mod test {
    use itertools::Itertools;
    use radix_trie::Trie;

    use crate::galloc::AllocForAny;

    use super::RadixTrieN;

    #[test]
    fn test() {
        let mut trie = RadixTrieN::new(2);
        trie.insert(["", "a"].galloc());
        trie.insert(["al", ""].galloc());
        trie.insert(["alpha", "alp"].galloc());
        trie.insert(["", "alp"].galloc());
        trie.insert(["", "alpha"].galloc());
        trie.insert(["", "alphb"].galloc());
        trie.insert(["alpha", "alpha"].galloc());
        
        for k in trie.prefixes(["alpha", "alpha"].galloc()) {
            println!("{:?}", k);
        }

    }
}
