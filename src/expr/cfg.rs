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
use crate::text::formatting::Op1EnumToFormattingOp;

// use super::{Expr, context::Context, Op1, Op3, Op2};

#[derive(DebugCustom, Clone)]
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
pub struct NonTerminal {
    pub name: String,
    pub ty: Type,
    pub rules: Vec<ProdRule>,
    pub start: bool,
    pub config: Config,
}

impl NonTerminal {
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
}
#[derive(Clone)]
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
pub struct Cfg{
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    inner: Vec<NonTerminal>,
    pub config: CfgConfig
}

impl std::fmt::Debug for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, nt) in self.inner.iter().enumerate() {
            writeln!(f, "{}: {:?}", i, nt)?;
        }
        Ok(())
    }
}

impl Cfg {
    pub fn from_synthfun(problem: &SynthFun) -> Self {
        Self {
            inner: problem.cfg.inner.iter().enumerate().map(|(i, nt)| NonTerminal {
                name: nt.0.clone(),
                ty: nt.1,
                rules: nt.2.iter().map(|p| ProdRule::new(p, problem)).collect(), 
                start: i == 0,
                config: nt.3.clone(),
            }).collect_vec(),
            config: problem.cfg.config.clone().into(),
        }
    }
    pub fn find_by_type(&self, ty: Type) -> Option<usize> {
        self.iter().enumerate().find(|x| x.1.ty == ty).map(|(i, _)| i)
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