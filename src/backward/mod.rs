use std::{cmp::{max, min}, future::Future};

use crate::{debg, expr::{cfg::{Cfg, NonTerminal, ProdRule}, context::Context, Expr}, forward::{executor::Executor, future::taskrc::{TaskRc, TaskTRc, TaskORc}, future::task::{self, currect_task_id}}, info, parser::problem, utils::select_all, value::Value};


use futures::{future::Either, select, FutureExt};
use itertools::Itertools;

use self::{simple::SimpleDeducer, str::{StrDeducer}};
use derive_more::DebugCustom;
pub mod str;
pub mod simple;

use derive_more::Constructor;
#[derive(Constructor, Clone, Debug, Copy)]
pub struct Problem {
    pub nt: usize,
    pub value: Value,
    pub used_cost: usize
}

impl Problem {
    pub fn with_value(mut self, v: Value) -> Problem {
        self.value = v;
        self
    }
    pub fn with_nt(mut self, nt: usize, v: Value) -> Problem {
        self.nt = nt;
        self.value = v;
        self
    }
    pub fn root(nt: usize, value: Value) -> Problem {
        Problem { nt, value, used_cost: 0 }
    }
    pub fn inccost(mut self) -> Problem {
        self.used_cost += 1;
        self
    }

}

pub trait Deducer {
    async fn deduce(&'static self, exec: &'static Executor, value: Problem) -> &'static Expr;
}

#[derive(DebugCustom)]
pub enum DeducerEnum {
    #[debug(fmt = "{:?}", _0)]
    Str(StrDeducer),
    #[debug(fmt = "{:?}", _0)]
    Simple(SimpleDeducer),
}

impl DeducerEnum {
    pub fn from_nt(cfg: &Cfg, ctx: &Context, nt: usize) -> Self {
        match cfg[nt].ty {
            crate::value::Type::Str => {
                let mut result = StrDeducer::new(nt);
                if let Some(ProdRule::Op2(_, n1, n2)) = cfg[nt].get_op2("str.++") {
                    if n1 == n2 && n1 == nt {
                        result.split_once = (ctx.len() / 3, 5);
                        if let Some(ProdRule::Op3(_, n1, n2, n3)) = cfg[nt].get_op3("ite") {
                            if n3 == n2 && n2 == nt {
                                result.ite_concat = (ctx.len(), n1)
                            }
                        }
                    }
                }
                if let Some(ProdRule::Op2(_, n1, n2)) = cfg[nt].get_op2("list.join") {
                    if n2 == nt {
                        result.join = (min(ctx.len(), max(5, ctx.len() / 10)), n1)
                    }
                }
                result.decay_rate = cfg[nt].config.get_usize("str.decay_rate").unwrap_or(900);
                result.formatter.append(&mut cfg[nt].get_all_formatter());
                info!("Deduction: {result:?}");
                Self::Str(result)
            }
            crate::value::Type::ListStr => Self::Simple(SimpleDeducer{ nt }),
            _ => Self::Simple(SimpleDeducer{ nt }),
        }
    }
}

impl Deducer for DeducerEnum {
    async fn deduce(&'static self, exec: &'static Executor, problem: Problem) -> &'static Expr {
        let result = match self {
            DeducerEnum::Str(a) => a.deduce(exec, problem).await,
            DeducerEnum::Simple(a) => a.deduce(exec, problem).await,
        };
        debg!("TASK#{} finished", currect_task_id());
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

