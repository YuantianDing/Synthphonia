
use std::{collections::{HashMap, HashSet}, cell::UnsafeCell};

use itertools::Itertools;
use kv_trie_rs::{Trie, TrieBuilder};
use derive_more::From;
use spin::Mutex;

use crate::{debg, expr::{cfg::ProdRule, context::Context, ops::{Op1, Op1Enum}, Expr}, forward::executor::Enumerator, utils::UnsafeCellExt, value::{consts_to_value, ConstValue, Value}};

pub struct TextObjData {
    trie: Vec<(&'static Op1Enum, usize, Trie<u8, ConstValue>)>>,
    future_exprs: Vec<Vec<(Expr, Value)>>,
}

impl TextObjData {
    pub fn enumerate(&mut self, exec: Arc<Enumerator>) -> Result<(), ()> {
        if exec.size() >= self.future_exprs.len() { return Ok(()); }
        for (e, v) in self.future_exprs[exec.size()].drain(0..) {
            exec.enum_expr(e, v)?;
        }
        Ok(())
    }
    pub fn build_trie(exec: &Enumerator) {
        for (nt, ntdata) in exec.cfg.iter().enumerate() {
            for rule in &ntdata.rules {
                if let ProdRule::Op1(op1, from_nt) = rule {
                    let vec = op1.parse_all(&exec.ctx);
                    if vec.len() == 0 { continue; }
                    let mut triebuilder = TrieBuilder::new();
                    for (k,v) in vec {
                        debg!("Found TextObj {} -> {} {}", k, op1.name(), v);
                        triebuilder.push(k.as_bytes(), v);
                    }
                    let mut trie = triebuilder.build();
                    exec.data[*from_nt].to.lock().trie.push((op1, nt, trie));
                }
            }
        }

    }
    pub fn new() -> Self {
        Self {
            trie: Vec::new().into(),
            future_exprs: Vec::new().into(),
        }
    }
    pub fn update(&self, exec: Arc<Enumerator>, e: &'static Expr, v: Value) {
        if let Value::Str(inner) = v {
            for (scan, nt,  v) in self.read_to(inner) {
                let expr = Expr::Op1(scan, e);
                let value = consts_to_value(v);
                let target = &mut exec.data[nt].to.lock();
                let size = exec.size() + scan.cost();
                while target.future_exprs.len() <= size {
                    target.future_exprs.push(Vec::new());
                }
                target.future_exprs[size].push((expr, value));
            }
        }
    }
    pub fn read_to(&self, input: &'static [&'static str]) -> impl Iterator<Item= (&'static Op1Enum, usize, Vec<ConstValue>)> + '_ {
        self.trie.iter().flat_map(|(scan, nt, trie)| {
            if trie.exact_match(input[0].as_bytes()) {
                let mut value = vec![trie.get(input[0].as_bytes()).unwrap().clone()];
                
                let r = input[1..].iter().find_map(|inp| {
                    if trie.exact_match(inp.as_bytes()) {
                        let v = trie.get(inp.as_bytes()).unwrap();
                        value.push(v.clone());
                        None
                    } else { Some(()) }
                });
                if r.is_none() {
                    return Some((*scan, *nt, value));
                }
            }
            None
        })
    }
}


pub trait ParsingOp {
    fn parse_all(&self, ctx: &Context) -> Vec<(&'static str, ConstValue)> {
        let mut result = Vec::new();
        for v in ctx.iter() {
            if let Value::Str(a) = v {
                for input in a {
                    let mut res = self.parse_into(input);
                    res.sort_by_key(|(a,b)| -(a.len() as isize));
                    let mut a = HashSet::new();
                    result.append(&mut res.into_iter().filter(|(s, _)| {
                        if a.contains(&s.as_ptr()) { false } else { a.insert(s.as_ptr()); true}
                    }).collect_vec());
                }
            }
        }
        result
    }
    fn parse_into(&self, input: &'static str) -> Vec<(&'static str, ConstValue)>;
}

pub mod date;
pub use date::*;
pub mod int;
pub use int::*;
mod month;
pub use month::*;
mod weekday;
pub use weekday::*;
mod time;
pub use time::*;
mod float;
pub use float::*;

impl ParsingOp for Op1Enum {
    fn parse_into(&self, input: &'static str) -> Vec<(&'static str, ConstValue)> {
        match self {
            Op1Enum::ParseTime(p) => p.parse_into(input),
            Op1Enum::ParseDate(p) => p.parse_into(input),
            Op1Enum::ParseMonth(p) => p.parse_into(input),
            Op1Enum::ParseInt(p) => p.parse_into(input),
            Op1Enum::ParseWeekday(p) => p.parse_into(input),
            _ => Vec::new(),
        }
    }
}



