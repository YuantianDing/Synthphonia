use std::collections::{HashMap, BTreeMap};

use itertools::Itertools;
use pest::iterators::Pair;

use crate::{galloc::AllocForCharIter, parser::problem::new_custom_error_span, value::{Value, ConstValue}};

use super::problem::{Rule, Error};
use derive_more::{From, DebugCustom};

#[derive(From, Clone, PartialEq, Eq, Hash, Default)]
pub struct Config(BTreeMap<String, ConstValue>);

impl Config {
    pub fn new() -> Self {
        Config(BTreeMap::new())
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        assert!(pair.as_rule() == Rule::config);
        let span = pair.as_span();
        let hash: Result<BTreeMap<String, ConstValue>, Error> = pair.into_inner().map(|x| {
            let [sym, v] : [Pair<'_, Rule>; 2] = x.into_inner().collect_vec().try_into().map_err(|_| new_custom_error_span("Expecting [(key value),*]".into(), span))?;
            match v.as_rule() {
                Rule::value => Ok((sym.as_str().into(), ConstValue::parse(v)?)),
                Rule::symbol => Ok((sym.as_str().into(), ConstValue::Str(v.as_str().chars().galloc_collect_str()))),
                _ => panic!(),
            }
        }).collect();
        Ok(hash?.into())
    }
    pub fn get_str(&self, name: &str) -> Option<&'static str> {
        self.0.get(name).and_then(|x| x.as_str())
    }
    pub fn get_i64(&self, name: &str) -> Option<i64> {
        self.0.get(name).and_then(|x| x.as_i64())
    }
    pub fn get_usize(&self, name: &str) -> Option<usize> {
        self.0.get(name).and_then(|x| x.as_usize())
    }
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.0.get(name).and_then(|x| x.as_bool())
    }
    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.0.iter() {
            write!(f, " #{}:{}", k, v)?
        }
        Ok(())
    }
}




