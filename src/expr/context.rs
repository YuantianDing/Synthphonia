
use derive_more::{DebugCustom, Constructor};
use itertools::Itertools;
use crate::{parser::{ioexamples::IOExamples, problem::PBEProblem}, tree_learning::Bits, value::Value};

use super::Expr;

#[derive(DebugCustom, Constructor, Clone)]
#[debug(fmt = "(n: {:?}, p: {:?})", n, p)]
/// A struct that encapsulates the contextual information used during a string synthesis evaluation. 
pub struct Context{
    pub len: usize, 
    /// Store inputs
    pub p: Vec<Value>,
    /// No longer used
    pub n: Vec<Value>,
    pub output: Value,
}

impl Context {
    /// Returns the length of the context of the values.
    pub fn len(&self) -> usize { self.len }
    
    /// Retrieve a input value from the context based on the provided index.
    pub fn get(&self, index: i64) -> Option<&Value> {
        if index >= 0 { self.p.get(index as usize) }
        else { self.n.get((!index) as usize) }
    }
    /// Provides an iterator over the inputs
    pub fn iter(&self) -> impl Iterator<Item=Value> + '_ {
        [self.output].into_iter().chain(self.p.iter().cloned()).chain(self.n.iter().cloned())
    }
    /// Provides an iterator over all input values contained within a given context. 
    pub fn inputs(&self) -> impl Iterator<Item=Value> + '_ {
        self.p.iter().cloned().chain(self.n.iter().cloned())
    }
    /// Returns an iterator over the output values within the context. 
    pub fn outputs(&self) -> impl Iterator<Item=Value> + '_ {
        [self.output].into_iter()
    }
    /// Evaluates an expression within the given context and determines its equivalence to the context's output. 
    pub fn evaluate(&self, e: &'static Expr) -> Option<Bits> {
        let v = e.eval(self);
        self.output.eq_bits(&v)
    }
    /// Creates a new instance by filtering the existing values with provided indices. 
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

    /// Provides a method for retrieving elements from a context using an index. 
    fn index(&self, index: i64) -> &Self::Output {
        self.get(index).expect("out of range")
    }
}


impl Context {
    /// Creates a `Context` instance from a reference to `IOExamples`. 
    pub fn from_examples(examples: &IOExamples) -> Self {
        Self {
            len: examples.output.len(),
            p: examples.inputs.clone(),
            n: Vec::new(),
            output: examples.output
        }
    }
}

