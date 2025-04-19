use std::collections::{HashMap, BTreeMap};

use itertools::Itertools;
use pest::iterators::Pair;

use crate::{expr::Expr, galloc::AllocForCharIter, parser::problem::new_custom_error_span, value::{ConstValue, Value}};

use super::problem::{Error, FunSig, Rule};
use derive_more::{From, DebugCustom};

#[derive(From, Clone, PartialEq, Eq, Hash, Default)]
/// A configuration fields of extended SyGuS-IF. Holding a collection of key-value pairs.
pub struct Config(BTreeMap<String, ConstValue>);

impl Config {
    /// Creates a new instance of the type containing an empty BTreeMap. 
    pub fn new() -> Self {
        Config(BTreeMap::new())
    }
    /// Parses a `Pair` of a configuration from a parsed syntax tree and returns a `Config` object. 
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        assert!(pair.as_rule() == Rule::config);
        let span = pair.as_span();
        let hash: Result<BTreeMap<String, ConstValue>, Error> = pair.into_inner().map(|x| {
            let [sym, v] : [Pair<'_, Rule>; 2] = x.into_inner().collect_vec().try_into().map_err(|_| new_custom_error_span("Expecting [(key value),*]".into(), span))?;
            match v.as_rule() {
                Rule::value => Ok((sym.as_str().into(), ConstValue::parse(v)?)),
                Rule::symbol => Ok((sym.as_str().into(), ConstValue::Str(v.as_str().chars().galloc_collect_str()))),
                Rule::expr => Ok((sym.as_str().into(), ConstValue::Expr(Expr::parse(v, None).unwrap()))),
                _ => panic!(),
            }
        }).collect();
        Ok(hash?.into())
    }
    /// Provides a method to retrieve a string reference from the configuration for a given key. 
    pub fn get_str(&self, name: &str) -> Option<&'static str> {
        self.0.get(name).and_then(|x| x.as_str())
    }
    /// Retrieves an `i64` value from the configuration map based on the provided string name key. 
    /// This function attempts to access the key within the internal map and converts the associated `ConstValue` to an `i64` if possible, returning `Some(i64)` when successful or `None` if the key does not exist or cannot be converted.
    pub fn get_i64(&self, name: &str) -> Option<i64> {
        self.0.get(name).and_then(|x| x.as_i64())
    }
    /// Provides a method to retrieve a `usize` value from the configuration. 
    pub fn get_usize(&self, name: &str) -> Option<usize> {
        self.0.get(name).and_then(|x| x.as_usize())
    }
    /// Retrieves an optional boolean value associated with a given key from the configuration. 
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.0.get(name).and_then(|x| x.as_bool())
    }
    /// Provides functionality to retrieve a static reference to an `Expr` associated with a given name.  
    pub fn get_expr(&self, name: &str) -> Option<&'static Expr> {
        self.0.get(name).and_then(|x| x.as_expr())
    }
    /// Merges another instance into the current one by extending its internal map with the entries from the provided instance. 

    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl std::fmt::Debug for Config {
    /// Formats the contents of the `Config` into a user-friendly string representation. 

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.0.iter() {
            write!(f, " #{}:{}", k, v)?
        }
        Ok(())
    }
}




