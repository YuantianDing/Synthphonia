use derive_more::Display;
use itertools::Itertools;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

pub use pest::Position;
pub use pest::Span;

use crate::{
    galloc::{self},
    value::Type,
};

use super::{ioexamples::IOExamples, prod::ProdRule, config::{Config, self}};
use derive_more::DebugCustom;

pub type Error = pest::error::Error<Rule>;

pub fn new_custom_error_span<'i>(msg: String, span: Span<'i>) -> Error { Error::new_from_span(pest::error::ErrorVariant::CustomError { message: msg }, span) }
pub fn new_costom_error_pos<'i>(msg: String, pos: Position<'i>) -> Error { Error::new_from_pos(pest::error::ErrorVariant::CustomError { message: msg }, pos) }

#[derive(DebugCustom, PartialEq, Eq, Hash, Clone)]
#[debug(fmt = "{} : {:?} -> {:?}", _0, _1, _2)]
pub struct NonTerminal(pub String, pub Type, pub Vec<ProdRule>, pub Config);

impl NonTerminal {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<NonTerminal, Error> {
        let mut vec = pair.into_inner().collect_vec();
        let config = vec.last().unwrap().clone();
        let config = if config.as_rule() == Rule::config {
            vec.pop();
            Config::parse(config.clone())?
        } else {
            Config::new()
        };
        let [symbol, typ, prods]: [_; 3] = vec.try_into().unwrap();
        let prods: Vec<_> = prods.into_inner().map(|x| ProdRule::parse(x)).try_collect()?;
        Ok(NonTerminal(symbol.as_str().into(), Type::parse(typ)?, prods, config))
    }
}

#[derive(DebugCustom, Clone)]
#[debug(fmt = "{:?} [{:?}]", "self.inner", "self.config")]
pub struct Cfg {
    pub start: String,
    pub inner: Vec<NonTerminal>, 
    pub config: Config
}

impl Cfg {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let mut cfgvec = pair.into_inner().into_iter().collect_vec();
        let config = cfgvec.last().unwrap().clone();
        let config = if config.as_rule() == Rule::config {
            cfgvec.pop();
            Config::parse(config.clone())?
        } else {
            Config::new()
        };
        let mut cfgiter = cfgvec.into_iter().peekable();
        let start = NonTerminal::parse(cfgiter.peek().unwrap().clone())?;
        let start = if let [ProdRule::Var(s, _)] =  start.2.as_slice() { cfgiter.next(); s } else { &start.0 };
        let start = start.clone();
        let mut inner: Vec<_> = cfgiter.map(|x| NonTerminal::parse(x)).try_collect()?;
        let mut cfg = Cfg{start, inner, config};
        cfg.reset_start();
        Ok(cfg)
    }
    pub fn reset_start(&mut self) {
        let start_index = self.inner.iter().position(|x| x.0 == self.start).unwrap();
        let start_nt = self.inner.remove(start_index);
        self.inner.insert(0, start_nt);
    }
    pub fn get_nt_by_type(&self, ty: &Type) -> String {
        self.inner.iter().find_map(|x| (x.1 == *ty).then_some(x.0.clone())).unwrap()
    }
    // pub fn sort(&mut self) {
    //     let mut sort = topological_sort::TopologicalSort::<NonTerminal>::new();
    //     for nt in self.inner.iter() {
    //         for rule in nt.2.iter() {
    //             if let ProdRule::Var(name, _) = rule {
    //                 if let Some(r) = self.inner.iter().find(|a| &a.0 == name) {
    //                     sort.add_dependency(*r, *nt);
    //                 }
    //             }
    //         }
    //     }
    //     let mut v = Vec::new();
    //     loop {
    //         let mut a = sort.pop_all();
    //         if a.is_empty() { break; }
    //         v.append(&mut a);
    //     }
    //     self.inner = v;
    // }
}

#[derive(Debug, Display, Clone)]
#[display(fmt = "{} ({}) {:?}", "self.name", r#"self.args.iter().map(|(s, t)| format!("({} {:?})", s, t)).collect_vec().join(" ")"#, "self.rettype")]
pub struct FunSig {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub rettype: Type,
}

impl FunSig {
    pub fn index(&self, argname: &str) -> Option<usize> {
        self.args.iter().position(|x| x.0 == argname)
    }
}

#[derive(Debug, Clone)]
pub struct SynthFun {
    pub sig: FunSig,
    pub cfg: Cfg,
    pub subproblem: bool
}

impl SynthFun {
    pub fn parse(synthfun: Pair<'_, Rule>) -> Result<Self, Error> {
        let subproblem = synthfun.as_rule() == Rule::synthsubproblem;
        let [name, arglist, typ, cfg]: [_; 4] = synthfun.into_inner().collect_vec().try_into().unwrap();
        let args: Vec<(String, Type)> = arglist
            .into_inner()
            .map(|x| {
                let [name, typ]: [_; 2] = x.into_inner().collect_vec().try_into().unwrap();
                Ok((name.as_str().to_owned(), Type::parse(typ)?))
            })
            .try_collect()?;
        let rettype = Type::parse(typ)?;
        let cfg = Cfg::parse(cfg)?;
        Ok(Self{sig: FunSig{name: name.as_str().into(), args, rettype}, cfg, subproblem})
    }
    pub fn lookup_nt(&self, nt: &str) -> Option<usize> {
        self.cfg.inner.iter().find_position(|x| x.0.as_str() == nt).map(|x| x.0)
    }
    pub fn lookup_arg(&self, arg: &str) -> Option<usize> {
        self.sig.args.iter().find_position(|x| x.0.as_str() == arg).map(|x| x.0)
    }
}




impl Type {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let [symbol]: [_; 1] = pair.clone().into_inner().collect_vec().try_into().unwrap();
        let basic = match symbol.as_str() {
            "Int" => Self::Int,
            "String" => Self::Str,
            "Bool" => Self::Bool,
            "Float" => Self::Float,
            _ => panic!("Unknown Type {}", symbol.as_str()),
        };
        if pair.as_str().contains("List") {
            basic.to_list().ok_or(new_custom_error_span("Unsupported list type".into(), pair.as_span()))
        } else {
            Ok(basic)
        }
    }
}
#[derive(Debug)]
pub struct PBEProblem {
    pub logic: String,
    pub synthfuns: Vec<SynthFun>,
    pub problem_index: usize,
    pub examples: IOExamples,
}

impl PBEProblem {
    pub fn synthfun(&self) -> &SynthFun {
        &self.synthfuns[self.problem_index]
    } 
    
    pub fn parse<'i>(input: &'i str) -> Result<PBEProblem, Error> {
        let [file]: [_; 1] = ProblemParser::parse(Rule::file, input)?.collect_vec().try_into().unwrap();
        let [_, logic, synthproblem, examples, checksynth]: [_; 5] = file.into_inner().collect_vec().try_into().unwrap();
        let [logic]: [_; 1] = logic.into_inner().collect_vec().try_into().unwrap();
        let synthfuns: Vec<_> = synthproblem.into_inner().enumerate().map(|(i, pair)| SynthFun::parse(pair)).collect::<Result<Vec<_>, _>>()?;
        let vec = synthfuns.iter().enumerate().filter(|x| !x.1.subproblem).map(|i|i.0).collect_vec();
        let problem_index = if let [a] = vec.as_slice() {*a} else { panic!("There should be only one synth-fun."); };
        let examples = IOExamples::parse(examples, &synthfuns[problem_index].sig, true)?;

        Ok(PBEProblem {
            logic: logic.as_str().to_owned(),
            synthfuns,
            problem_index,
            examples,
        })
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "src/parser/problem.pest"]
pub struct ProblemParser;

#[cfg(test)]
mod tests {
    use std::fs;

    use super::PBEProblem;

    #[test]
    fn parse_test() {
        let s = fs::read_to_string("test/test.sl").unwrap();
        let result = PBEProblem::parse(s.as_str());
        println!("{:?}", result.map(|x| x.synthfun().cfg.clone()));
    }
}
