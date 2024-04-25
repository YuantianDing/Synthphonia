
use derive_more::{DebugCustom, Constructor};
use itertools::Itertools;
use crate::{parser::{ioexamples::IOExamples, problem::PBEProblem}, tree_learning::Bits, value::Value};

use super::Expr;

#[derive(DebugCustom, Constructor, Clone)]
#[debug(fmt = "(n: {:?}, p: {:?})", n, p)]
pub struct Context{
    pub len: usize, 
    pub p: Vec<Value>,
    pub n: Vec<Value>,
    pub output: Value,
}

impl Context {
    pub fn len(&self) -> usize { self.len }
    pub fn get(&self, index: i64) -> Option<&Value> {
        if index >= 0 { self.p.get(index as usize) }
        else { self.n.get((!index) as usize) }
    }
    pub fn iter(&self) -> impl Iterator<Item=Value> + '_ {
        [self.output].into_iter().chain(self.p.iter().cloned()).chain(self.n.iter().cloned())
    }
    pub fn inputs(&self) -> impl Iterator<Item=Value> + '_ {
        self.p.iter().cloned().chain(self.n.iter().cloned())
    }
    pub fn outputs(&self) -> impl Iterator<Item=Value> + '_ {
        [self.output.clone()].into_iter()
    }
    pub fn evaluate(&self, e: &'static Expr) -> Option<Bits> {
        let v = e.eval(self);
        self.output.eq_bits(&v)
    }
    pub fn with_examples(&self, exs: &[usize]) -> Context {
        Context {
            len: exs.len(),
            p: self.p.iter().map(|x| x.with_examples(exs)).collect_vec(),
            n: self.n.iter().map(|x| x.with_examples(exs)).collect_vec(),
            output: self.output.with_examples(exs),
        }
    }
}

impl std::ops::Index<i64> for Context {
    type Output = Value;

    fn index(&self, index: i64) -> &Self::Output {
        self.get(index).expect("out of range")
    }
}


impl Context {
    pub fn from_examples(examples: &IOExamples) -> Self {
        Self {
            len: examples.output.len(),
            p: examples.inputs.clone(),
            n: Vec::new(),
            output: examples.output
        }
    }
}

