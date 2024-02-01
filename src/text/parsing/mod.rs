
use std::{collections::{HashMap, HashSet}, cell::UnsafeCell};

use itertools::Itertools;
use kv_trie_rs::{Trie, TrieBuilder};
use derive_more::From;

use crate::{debg, expr::{cfg::ProdRule, context::Context, ops::{Op1, Op1Enum}, Expr}, forward::executor::Executor, utils::UnsafeCellExt, value::{consts_to_value, ConstValue, Value}};

pub struct TextObjData {
    trie: UnsafeCell<Vec<(&'static Op1Enum, usize, Trie<u8, ConstValue>)>>,
    future_exprs: UnsafeCell<Vec<Vec<(Expr, Value)>>>,
}

impl TextObjData {
    pub fn trie(&self) -> &mut Vec<(&'static Op1Enum, usize, Trie<u8, ConstValue>)> {
        unsafe { self.trie.as_mut() }
    }
    pub fn future_exprs(&self) -> &mut Vec<Vec<(Expr, Value)>> {
        unsafe { self.future_exprs.as_mut() }
    }
    pub fn enumerate(&self, exec: &'static Executor) -> Result<(), ()> {
        if exec.size() >= self.future_exprs().len() { return Ok(()); }
        for (e, v) in self.future_exprs()[exec.size()].drain(0..) {
            exec.enum_expr(e, v)?;
        }
        Ok(())
    }
    pub fn build_trie(exec: &Executor) {
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
                    exec.data[*from_nt].to.trie().push((op1, nt, triebuilder.build()));
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
    pub fn update(&self, exec: &'static Executor, e: &'static Expr, v: Value) {
        if let Value::Str(inner) = v {
            for (scan, nt,  v) in self.read_to(inner) {
                let expr = Expr::Op1(scan, e);
                let value = consts_to_value(v);
                let target = exec.data[nt].to.future_exprs();
                let size = exec.size() + scan.cost();
                while target.len() <= size {
                    target.push(Vec::new());
                }
                target[size].push((expr, value));
            }
        }
    }
    pub fn read_to(&self, input: &'static [&'static str]) -> impl Iterator<Item= (&'static Op1Enum, usize, Vec<ConstValue>)> + '_ {
        self.trie().iter().flat_map(|(scan, nt, trie)| {
            for (k, v) in trie.common_prefix_search_with_values(input[0].as_bytes()) {
                let mut value = vec![v];
                if k.len() != input[0].len() { return None; }
                
                let r = input[1..].iter().find_map(|inp| {
                    let values = trie.common_prefix_search_with_values(inp.as_bytes());
                    let r = values.iter().find_map(|(k, v)| {
                        if inp.len() == k.len() { Some(v) } else { None }
                    });
                    if let Some(v) = r {
                        value.push(*v);
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



