
use std::{collections::{HashMap, HashSet}, cell::UnsafeCell};

use itertools::Itertools;
use kv_trie_rs::{Trie, TrieBuilder};
use derive_more::From;

pub type ScannerRef = &'static dyn Scanner;
#[derive(From)]
pub struct TextObjData{
    trie: Vec<(ScannerRef, Trie<u8, i64>)>,
    future_exprs: UnsafeCell<HashMap<(usize, usize), Vec<(Expr, Value)>>>
}

impl TextObjData {
    pub fn future_exprs(&self) -> &mut HashMap<(usize, usize), Vec<(Expr, Value)>> {
        unsafe { self.future_exprs.as_mut() }
    }
    fn build_trie(scan: &dyn Scanner, ctx: &Context) -> Trie<u8, i64> {
        let mut triebuilder = TrieBuilder::new();
        for (k,v) in scan.parse_all(ctx) {
            debg!("Found TextObj {} -> {} {}", k, scan.name(), v);
            triebuilder.push(k.as_bytes(), v);
        }
        triebuilder.build()
    }
    pub fn new(cfg: & Cfg, ctx: & Context) -> Self {
        let mut trie: Vec<(ScannerRef, Trie<u8, i64>)> = Vec::new();
        for (i, nt) in cfg.iter().enumerate() {
            if let Some(ProdRule::Op1(_, nt1, _)) = nt.get_op1(ParseDate.into()) {
                let fmt = matches!(cfg[nt1].get_op1(FormatDate.into()), Some(ProdRule::Op1(_, i, _)));
                let scan: ScannerRef = DateScanner{ str_nt: nt1, date_nt: i, parser: true, fmt }.galloc();
                trie.push((scan, Self::build_trie(scan, ctx)))
            }
            if let Some(ProdRule::Op1(_, nt1, _)) = nt.get_op1(ParseTime.into()) {
                let fmt = matches!(cfg[nt1].get_op1(FormatTime.into()), Some(ProdRule::Op1(_, i, _)));
                let scan: ScannerRef = TimeScanner{ str_nt: nt1, date_nt: i, parser: true, fmt }.galloc();
                trie.push((scan, Self::build_trie(scan, ctx)))
            }
            if let Some(ProdRule::Op1(_, nt1, _)) = nt.get_op1(FormatMonth.into()) {
                let scan: ScannerRef = MonthScanner{ str_nt: nt1, month_nt: i, parser: false, fmt: true }.galloc();
                trie.push((scan, Self::build_trie(scan, ctx)))
            }
            if let Some(ProdRule::Op1(_, nt1, _)) = nt.get_op1(FormatWeekday.into()) {
                let scan: ScannerRef = WeekdayScanner{ str_nt: nt1, month_nt: i, parser: false, fmt: true }.galloc();
                trie.push((scan, Self::build_trie(scan, ctx)))
            }

        }
        Self {
            trie,
            future_exprs: HashMap::new().into(),
        }
    }
    pub fn update(&self, nt: usize, size: usize, e: &'static Expr, v: Value) {
        if let Value::Str(inner) = v {
            for (scan, value) in self.read_to(inner) {
                if let Some((nt, size, e,v)) = scan.generate_result(nt, size, e, value) {
                    match self.future_exprs().entry((nt, size)) {
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            entry.get_mut().push((e,v));
                        }
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(vec![(e,v)]);
                        }
                    }
                }
            }
        }
    }
    pub fn read_to(&self, input: &'static [&'static str]) -> impl Iterator<Item= (ScannerRef, Vec<i64>)> + '_ {
        self.trie.iter().flat_map(|(scan, trie)| {
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
                    return Some((*scan, value));
                }
            }
            None
        })
    }
    pub fn read_to_remainder(&self, input: &'static [&'static str]) -> impl Iterator<Item=(ScannerRef, Vec<i64>, Vec<&'static str>)> + '_ {
        self.trie.iter().flat_map(|(scan, trie)| {
            if let Some((k, v)) = trie.common_prefix_search_with_values(input[0].as_bytes()).iter().max_by_key(|(k, _)| k.len()) {
                let mut value = vec![*v];
                let mut rest = vec![&input[0][k.len()..]];
                
                let r = input.iter().enumerate().skip(1).find_map(|(i, inp)| {
                    let values = trie.common_prefix_search_with_values(inp.as_bytes());
                    if let Some((k, v)) = values.iter().max_by_key(|(k, _)| k.len()) {
                        let r = &input[i][k.len()..];
                        value.push(*v);
                        rest.push(r);
                        None
                    } else { Some(()) }
                });
                if r.is_none() {
                    return Some((*scan, value, rest));
                }
            }
            None
        })
    }
}


pub trait Scanner {
    fn name(&self) -> &'static str;
    fn parse_all(&self, ctx: &Context) -> Vec<(&'static str, i64)> {
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
    fn parse_into(&self, input: &'static str) -> Vec<(&'static str, i64)>;
    fn generate_result(&self, nt: usize, size: usize, e: &'static  Expr, input: Vec<i64>) -> Option<(usize, usize, Expr, Value)>;
    fn format_op(&self) -> Option<Op1Enum>;
}

mod date;
use date::*;
mod month;
mod weekday;
mod time;
mod int;

use crate::{forward::data::size::EV, expr::{Expr, cfg::{Cfg, ProdRule}, context::Context, Op1Enum, FormatDate, ParseDate, FormatMonth, FormatWeekday, FormatTime, ParseTime}, value::Value, arena::AllocForAny, utils::UnsafeCellExt, debg};

use self::{month::MonthScanner, weekday::WeekdayScanner, time::TimeScanner};


#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        println!("{}", -10i64 as u64);
    }
}




