use std::{collections::HashMap, cmp::min};

use crate::{
    expr::ops::{Op1Enum, Op2Enum, Op3Enum}, galloc::AllocForAny, parser::{
        self,
        problem::{self, Error, PBEProblem, SynthFun},
        prod, config::Config,
    }, value::{ConstValue, Type}
};
use derive_more::{DebugCustom, Deref, DerefMut, From, Into, Index, IndexMut};
use itertools::Itertools;
use joinery::prelude::*;

// use super::{Expr, context::Context, Op1, Op3, Op2};

#[derive(DebugCustom, Clone)]
/// An enum representing production rules for expressions in the synthesis problem framework. 
/// 
/// This variant can encompass constants, variables, and non-terminal symbols along with unary, binary, and ternary operations. 
/// Each variant includes a formatting directive, used for debugging purposes, to provide a human-readable description of its content. 
/// The `Const` variant holds a constant value, `Var` represents a variable identified by an integer, and `Nt` refers to a non-terminal symbol using an index. 
/// The `Op1`, `Op2`, and `Op3` variants represent unary, binary, and ternary operations, respectively, each associated with operator enumerations and indices to expressions they relate to. 
/// This structure facilitates both the expression construction and the debugging process in the synthesis tasks.
/// 
pub enum ProdRule {
    #[debug(fmt = "{:?}", _0)]
    Const(ConstValue),
    #[debug(fmt = "v{:?}", _0)]
    Var(i64),
    #[debug(fmt = "nt{:?}", _0)]
    Nt(usize),
    #[debug(fmt = "({} {:?})", _0, _1)]
    Op1(&'static Op1Enum, usize),
    #[debug(fmt = "({} {:?} {:?})", _0, _1, _2)]
    Op2(&'static Op2Enum, usize, usize),
    #[debug(fmt = "({} {:?} {:?} {:?})", _0, _1, _2, _3)]
    Op3(&'static Op3Enum, usize, usize, usize),
}

impl ProdRule {
    /// Creates a new instance from a raw production rule and a synthesis function problem context. 
    /// 
    /// It matches various kinds of production rules such as variables, constants, and operations, transforming them into corresponding variants. 
    /// For variables, it checks if the variable corresponds to an argument in the synthesis function, returning either a variable or a nonterminal rule. 
    /// The constant variant simply maps to its equivalent, maintaining its value. 
    /// For operations, it maps the operation names to their respective enums with memory allocation and resolves nonterminals using the synthesis function's context, ensuring that each element corresponds to a valid component in the synthesis task. 
    /// Any unrecognized variables or nonterminals lead to a panic.
    /// 
    pub fn new(raw: &prod::ProdRule, problem: &SynthFun) -> Self {
        match raw {
            prod::ProdRule::Var(s, config) => {
                if let Some(a) = problem.lookup_arg(s.as_str()) {
                    Self::Var(a as i64)
                } else if let Some((a, _)) = problem.cfg.inner.iter().enumerate().find(|(a,b)| &b.0 == s) {
                    Self::Nt(a)
                } else { panic!("Unrecongized Variable / Nonterminal") }
            },
            prod::ProdRule::Const(v, config) => Self::Const(*v),
            prod::ProdRule::Op1(a, b, config) => Self::Op1(Op1Enum::from_name(a.as_str(), config).galloc(), problem.lookup_nt(b).expect("Unknow non terminal")),
            prod::ProdRule::Op2(a, b, c, config) => Self::Op2(
                Op2Enum::from_name(a.as_str(), config).galloc(),
                problem.lookup_nt(b).expect("Unknow non terminal"),
                problem.lookup_nt(c).expect("Unknow non terminal"),
            ),
            prod::ProdRule::Op3(a, b, c, d, config) => Self::Op3(
                Op3Enum::from_name(a.as_str(), config).galloc(),
                problem.lookup_nt(b).expect("Unknow non terminal"),
                problem.lookup_nt(c).expect("Unknow non terminal"),
                problem.lookup_nt(d).expect("Unknow non terminal"),
            ),
        }
    }
    
}

#[derive(DebugCustom, Clone)]
#[debug(fmt = "({}: {:?}) -> {:?}", name, ty, rules)]
/// A struct representing a grammar non-terminal. 
/// 
/// This construct includes several fields essential for defining a non-terminal within a string synthesis problem. 
/// The `name` field holds the identifier for the non-terminal, while `ty` specifies the associated type. 
/// The `rules` field is a collection of production rules that describe how this non-terminal can be expanded. 
/// The `start` field is a boolean flag indicating whether this non-terminal serves as the starting point in the grammar. 
/// Lastly, `config` encompasses additional settings or parameters that may influence the synthesis process for this non-terminal.
/// 
pub struct NonTerminal {
    pub name: String,
    pub ty: Type,
    pub rules: Vec<ProdRule>,
    pub config: Config,
}

impl NonTerminal {
    /// Retrieves a unary operation production rule by name. 
    /// 
    /// This function iterates through the list of production rules associated with a non-terminal to find a unary operation rule (`Op1`) matching the specified operation name. 
    /// If a matching `Op1` operation is found, it returns the corresponding production rule; otherwise, it returns `None`. 
    /// This functionality enables specific lookups for unary operations within a non-terminal's rule set.
    /// 
    pub fn get_op1(&self, op1: &str) -> Option<ProdRule>{
        for rule in self.rules.iter() {
            if let ProdRule::Op1(r, _) = rule {
                if r.name() == op1 {
                    return Some(rule.clone());
                }
            }
        }
        None
    }
    /// Retrieves an `Op2` production rule matching a given operation name. 
    /// 
    /// This method iterates over the set of production rules associated with a non-terminal. 
    /// For each rule, if the rule is an `Op2` operation and its name matches the specified `op2` string, the method returns a cloned instance of that rule. 
    /// If no matching rule is found, it returns `None`. 
    /// This allows for extracting specific binary operations from the rule set, aiding in identifying or constructing expressions based on these operations.
    /// 
    pub fn get_op2(&self, op2: &str) -> Option<ProdRule>{
        for rule in self.rules.iter() {
            if let ProdRule::Op2(r, _, _) = rule {
                if r.name() == op2 {
                    return Some(rule.clone());
                }
            }
        }
        None
    }
    /// Retrieves a `ProdRule` of type `Op3` from the `NonTerminal` if it matches a specific operation name. 
    /// 
    /// This method iterates over the `rules` vector within a `NonTerminal` instance and checks if each rule is an `Op3` operation. 
    /// If an `Op3` operation matches the given `op3` string name, it returns a clone of that `ProdRule`. 
    /// If no matching operation is found, the method returns `None`.
    /// 
    pub fn get_op3(&self, op3: &str) -> Option<ProdRule>{
        for rule in self.rules.iter() {
            if let ProdRule::Op3(r, _, _, _) = rule {
                if r.name() == op3 {
                    return Some(rule.clone());
                }
            }
        }
        None
    }
    
    /// Retrieves a vector of all one-operand operations that are formatting operations from the production rules associated with the non-terminal. 
    /// 
    /// This method iterates through the list of production rules, checking each rule to see if it is a unary operation (`Op1`). 
    /// If the operation is identified as a formatting operation via `is_formatting_op`, it adds it to the results along with its associated non-terminal index. 
    /// The method compiles these into a vector of tuples containing the operation enum and the index, which is then returned.
    /// 
    pub fn get_all_formatter(&self) -> Vec<(Op1Enum, usize)> {
        let mut result = Vec::new();
        for rule in self.rules.iter() {
            if let ProdRule::Op1(r, nt) = rule {
                if r.is_formatting_op() {
                    result.push(((*r).clone(), *nt));
                }
            }
        }
        result
    }
    pub fn map_nt_number(&mut self, mut f: impl FnMut(usize) -> usize) {
        for prod in self.rules.iter_mut() {
            match prod {
                ProdRule::Nt(n) => {
                    *n = f(*n);
                }
                ProdRule::Op1(_, n) => {
                    *n = f(*n);
                }
                ProdRule::Op2(_, n1, n2) => {
                    *n1 = f(*n1);
                    *n2 = f(*n2);
                }
                ProdRule::Op3(_, n1, n2, n3) => {
                    *n1 = f(*n1);
                    *n2 = f(*n2);
                    *n3 = f(*n3);
                }
                _ => {}
            }
        }
    }
}
#[derive(Clone)]
/// A configuration structure for controlling various parameters related to the synthesis problem-solving process. 
/// 
/// This structure contains fields that define limits and options for resource consumption and operational modes during synthesis. 
/// These fields include limits on size, time, and substring occurrences, as well as sample sizes for list subsequences. 
/// It also includes options for controlling conditional searching, deduction skipping, iteration limit rates, and special tree hole conditions. 
/// This configuration enables users to tailor the synthesis process by adjusting these parameters to suit different problem constraints and computational scenarios.
/// 
pub struct CfgConfig {
    pub size_limit: usize,
    pub time_limit: usize,
    pub substr_limit: usize,
    pub listsubseq_samples: usize,
    pub increase_cost_limit: usize,
    pub cond_search: bool,
    pub no_deduction: bool,
    pub ite_limit_rate: usize,
    pub ite_limit_giveup: usize,
    pub tree_hole: bool,
}

impl From<Config> for CfgConfig {
    /// Creates a new instance by converting from a `Config` object. 
    /// 
    /// This method initializes each field of the struct with corresponding values fetched from the `Config` object, using specified keys. 
    /// If a key does not exist in the `Config`, a default value is assigned. 
    /// For `size_limit` and `time_limit`, the size defaults to `usize::MAX`. 
    /// The `substr_limit` defaults to `4`, `listsubseq_samples` to `0`, `increase_cost_limit` to `2000`, `ite_limit_rate` to `1000`, and `ite_limit_giveup` to `40`. 
    /// The boolean fields `cond_search`, `no_deduction`, and `tree_hole` are initialized as `false`. 
    /// This method is essential for transforming configuration data into a structured format used for synthesis constraints.
    /// 
    fn from(value: Config) -> Self {
        Self {
            size_limit: value.get_usize("size_limit").unwrap_or(usize::MAX),
            time_limit: value.get_usize("time_limit").unwrap_or(usize::MAX),
            substr_limit: value.get_i64("data.substr.limit").unwrap_or(4) as usize,
            listsubseq_samples: value.get_i64("data.listsubseq.sample").unwrap_or(0) as usize,
            increase_cost_limit: value.get_i64("increase_cost_limit").unwrap_or(2000) as usize,
            cond_search: false,
            no_deduction: false,
            ite_limit_rate: value.get_i64("ite_limit_rate").unwrap_or(1000) as usize,
            ite_limit_giveup: value.get_i64("ite_limit_giveup").unwrap_or(40) as usize,
            tree_hole: false,
        }
    }
}

#[derive(Deref, DerefMut, Into, Index, IndexMut, Clone)]
/// Context-free grammar representation
/// 
/// This data structure embeds a vector of `NonTerminal` elements, facilitating operations that require dereferencing and indexing, both mutable and immutable. 
/// These operations are streamlined through attributes such as `#[deref]`, `#[deref_mut]`, `#[index]`, and `#[index_mut]`, enabling direct access and manipulation of the `inner` vector's elements. 
/// Additionally, it includes a public configuration field, `config`, of type `CfgConfig`, which likely governs specific settings or parameters associated with the non-terminal sequence within the larger framework of the synthesis process.
/// 
pub struct Cfg{
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    inner: Vec<NonTerminal>,
    pub config: CfgConfig
}

impl std::fmt::Debug for Cfg {
    /// Formats the configuration and writes it into the provided formatter. 
    /// 
    /// This method iterates over the collection of non-terminals within the internal vector, indexes them, and prints each accompanied by its index to the given formatter. 
    /// The output format aligns with Rust's debug formatting conventions, ensuring the representation is both informative and concise, and returning a result indicates whether formatting was successful or encountered an error.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, nt) in self.inner.iter().enumerate() {
            writeln!(f, "{}: {:?}", i, nt)?;
        }
        Ok(())
    }
}

impl Cfg {
    /// Constructs an instance from a provided `SynthFun` problem. 
    /// 
    /// This function extracts the configuration for constructing a context-free grammar (CFG) as represented by the `SynthFun` and populates the `Cfg` structure. 
    /// It iterates over the non-terminal definitions within the given problem, mapping them to `NonTerminal` structures with relevant details such as name, type, production rules, and configuration. 
    /// The first non-terminal is designated as the starting point. 
    /// The overall configuration for the CFG is cloned and assigned, ensuring the new `Cfg` instance accurately embodies the grammar and constraints defined in the `SynthFun` problem.
    /// 
    pub fn from_synthfun(problem: &SynthFun) -> Self {
        Self {
            inner: problem.cfg.inner.iter().enumerate().map(|(i, nt)| NonTerminal {
                name: nt.0.clone(),
                ty: nt.1,
                rules: nt.2.iter().map(|p| ProdRule::new(p, problem)).collect(), 
                config: nt.3.clone(),
            }).collect_vec(),
            config: problem.cfg.config.clone().into(),
        }
    }
    /// Find and return the index of the first `NonTerminal` in the collection with a specified type. 
    /// 
    /// The method iterates over the internal `Vec<NonTerminal>`, checking each element's type against the given `ty`. 
    /// If a matching type is found, it returns the index as an `Option<usize>`, otherwise, it returns `None`. 
    /// This function facilitates the retrieval of non-terminal symbols of a certain type, enhancing the ability to programmatically manipulate and access parts of the grammar defined within the `Cfg`.
    /// 
    pub fn find_by_type(&self, ty: Type) -> Option<usize> {
        self.iter().enumerate().find(|x| x.1.ty == ty).map(|(i, _)| i)
    }
    pub fn change_start(&self, nstart: usize) -> Self {
        let mut new = self.clone();
        for nt in new.iter_mut() {
            nt.map_nt_number(|i| if i == 0 { nstart } else if i == nstart { 0 } else { i });
        }
        new.swap(0, nstart);
        new
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{parser::problem::PBEProblem, log};

    use super::Cfg;

    #[test]
    fn test_cfg() {
        log::set_log_level(5);
        let s = fs::read_to_string("test/test.sl").unwrap();
        let problem = PBEProblem::parse(s.as_str()).unwrap();
        let cfg = Cfg::from_synthfun(problem.synthfun());
        println!("{:?}", cfg);
    }
}