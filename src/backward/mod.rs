use std::{cmp::{max, min}, future::Future};

use crate::{debg, expr::{cfg::{Cfg, NonTerminal, ProdRule}, context::Context, Expr}, forward::executor::Executor, info, parser::problem, utils::select_all, value::Value};


use futures::{future::Either, select, FutureExt};
use itertools::Itertools;

use self::{liststr::ListDeducer, simple::SimpleDeducer, str::StrDeducer};
use derive_more::DebugCustom;
/// Deduction for string
pub mod str;

/// Basic Deduction
pub mod simple;

/// Deduction for list of strings
pub mod liststr;

use derive_more::Constructor;
#[derive(Constructor, Clone, Debug, Copy)]
/// A struct represents a synthesis problem within the backward deduction process of the string synthesis algorithm. 
/// This structure contains three primary fields: `nt`, which holds the non-terminal symbol represented as an index, `value`, which associates a `Value` to the synthesis task, and `used_cost`, which tracks the cost incurred during the deduction process for this particular problem instance.
pub struct Problem {
    pub nt: usize,
    pub value: Value,
    pub used_cost: usize
}

impl Problem {
    /// Updates the associated value in a synthesis subproblem and returns the modified instance. 
    /// This method consumes the current problem instance, replacing its stored value with the provided one, and returns the updated problem for further synthesis processing.
    pub fn with_value(mut self, v: Value) -> Problem {
        self.value = v;
        self
    }
    /// Updates an existing synthesis subproblem instance with a new non-terminal index and associated value. 
    /// This method takes ownership of the current instance, sets its non-terminal field to the supplied index and updates its associated value, and then returns the modified instance for chaining or further use.
    pub fn with_nt(mut self, nt: usize, v: Value) -> Problem {
        self.nt = nt;
        self.value = v;
        self
    }
    /// Creates a new synthesis subproblem with a specified non-terminal index and associated value, initializing the accumulated cost to zero.
    /// 
    /// Initializes a Problem instance intended to represent the root of a synthesis task in the backward deduction process, ensuring that the deduction cost starts at zero.
    pub fn root(nt: usize, value: Value) -> Problem {
        Problem { nt, value, used_cost: 0 }
    }
    /// Increments the accrued cost associated with a synthesis subproblem and returns the updated instance. 
    /// 
    /// 
    /// This method updates the internal cost metric by adding one to it, allowing the overall synthesis process to keep track of the computational expense incurred while performing backward deduction. 
    /// The function consumes the current instance, modifies its cost field, and returns the modified version for further processing.
    pub fn inccost(mut self) -> Problem {
        self.used_cost += 1;
        self
    }
}

/// Provides an asynchronous interface to synthesize an expression by resolving a synthesis subproblem using an executor context. 
/// 
/// This method takes a static reference to an executor and a synthesis problem as inputs and returns a static reference to the synthesized expression. 
/// The interface is designed for asynchronous deduction, allowing varied deduction strategies to be implemented and executed concurrently while ensuring that each subproblem is processed efficiently.
/// 
pub trait Deducer {
    async fn deduce(&'static self, exec: &'static Executor, value: Problem) -> &'static Expr;
}

#[derive(DebugCustom)]
/// Represents different deduction strategy implementations for string synthesis problems. 
/// 
/// 
/// Encapsulates three variants to handle distinct deduction approaches: one variant specializes in string-specific deduction, another provides a baseline for basic deduction, and the third addresses deduction tasks for lists of strings. 
/// Each variant is annotated to facilitate formatted debugging output.
pub enum DeducerEnum {
    #[debug(fmt = "{:?}", _0)]
    Str(StrDeducer),
    #[debug(fmt = "{:?}", _0)]
    Simple(SimpleDeducer),
    #[debug(fmt = "{:?}", _0)]
    List(ListDeducer),
}

impl DeducerEnum {
    /// Creates an instance of a deduction strategy based on the grammar configuration, context, and non-terminal index. 
    /// 
    /// 
    /// Selects a deduction approach by first checking whether deduction is disabled in the configuration and then matching on the non-terminal's type. 
    /// For string types, it initializes a strategy that fine-tunes parameters such as splitting operations, conditional (ite) concatenation, and join operations based on specific production rules; it also sets a decay rate and appends formatters retrieved from the grammar. 
    /// For list-of-string types, it configures an alternative strategy that conditionally leverages a modified grammar when a list mapping operation is present. 
    /// In all other cases, it falls back to a simple deduction strategy.
    pub fn from_nt(cfg: &Cfg, ctx: &Context, nt: usize) -> Self {
        if cfg.config.no_deduction {
            return Self::Simple(SimpleDeducer{ nt });
        }
        match cfg[nt].ty {
            crate::value::Type::Str => {
                let mut result = StrDeducer::new(nt);
                if let Some(ProdRule::Op2(_, n1, n2)) = cfg[nt].get_op2("str.++") {
                    if n1 == n2 && n1 == nt {
                        result.split_once = (ctx.len() / 3, 5);
                        if let Some(ProdRule::Op3(_, n1, n2, n3)) = cfg[nt].get_op3("ite") {
                            if n3 == n2 && n2 == nt{
                                result.ite_concat = (ctx.len(), n1)
                            }
                        }
                    }
                }
                if let Some(ProdRule::Op2(_, n1, n2)) = cfg[nt].get_op2("str.join") {
                    if n2 == nt && cfg[n1].get_op1("list.map").is_some() {
                        result.join = (2, n1)
                    } 
                }
                result.decay_rate = cfg[nt].config.get_usize("str.decay_rate").unwrap_or(900);
                result.formatter.append(&mut cfg[nt].get_all_formatter());
                info!("Deduction: {result:?}");
                Self::Str(result)
            }
            crate::value::Type::ListStr => {
                let mut result = ListDeducer { nt, map: None};
                if cfg[nt].get_op1("list.map").is_some() {
                    let mut cfg2 = cfg.clone();
                    for nt in cfg2.iter_mut() {
                        nt.rules.retain(|x| !matches!(x, ProdRule::Var(a) if *a > 0))
                    }
                    info!("Map Cfg {:?}", cfg2);
                    result.map = Some(cfg2);
                }
                Self::List(result)
            }
            _ => Self::Simple(SimpleDeducer{ nt }),
        }
    }
}

impl Deducer for DeducerEnum {
    /// Asynchronously deduces an expression for a given synthesis subproblem using the appropriate deduction strategy based on the problem type. 
    /// 
    /// 
    /// This method first checks if the solution for the subproblem is pending in the executor's cache. 
    /// If it is, the method awaits and returns the pending result; otherwise, it delegates the deduction task to the underlying strategy implementation corresponding to the subproblem's type. 
    /// After obtaining the result, it logs the solved subproblem and records the expression back into the executor's cache for future reuse.
    async fn deduce(&'static self, exec: &'static Executor, problem: Problem) -> &'static Expr {
        let is_pending = exec.data[problem.nt].all_eq.is_pending(problem.value);
        if is_pending { return exec.data[problem.nt].all_eq.acquire(problem.value).await; }

        let result = match self {
            DeducerEnum::Str(a) => a.deduce(exec, problem).await,
            DeducerEnum::Simple(a) => a.deduce(exec, problem).await,
            DeducerEnum::List(a) => a.deduce(exec, problem).await,
        };
        debg!("Subproblem {:?} solved", problem.value);
        exec.data[problem.nt].add_ev(result, problem.value);
        result
    }
}


// #[derive(Clone, PartialEq, Eq, Hash)]
// pub struct Problem {
//     pub value: Value,
//     pub nt: usize,
// }

// impl Problem {

//     pub async fn deduce(self, exec: &'static Executor) -> &'static Expr {
//         let (is_first, task) = exec.data[self.nt].all_eq.acquire_is_first(self.value);
//         if !is_first {
//             return task.await;
//         }
//         debg!("TASK#{} Deducing subproblem: {:?}", currect_task_id(), self.value);
//         let subtasks = exec.dcfg.deduce(self.nt, self.clone(), exec);
//         let e = select! {
//             e = task.fuse() => e,
//             e = select_all(subtasks).fuse() => {
//                 let _ = exec.data[self.nt].all_eq.set(self.value, e.clone());
//                 e
//             }
//         };
//         debg!("TASK#{} Subproblem Solved: {:?} {:?}", currect_task_id(), self.value, e);
//         e
//     }
// }

