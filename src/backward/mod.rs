use std::{cmp::{max, min}, future::Future};

use crate::{debg, expr::{cfg::{Cfg, NonTerminal, ProdRule}, context::Context, Expr}, forward::{executor::Executor, future::taskrc::{TaskRc, TaskTRc, TaskORc}, future::task::{self, currect_task_id}}, info, parser::problem, utils::select_all, value::Value};


use futures::{future::Either, select, FutureExt};
use itertools::Itertools;

use self::{simple::SimpleDeducer, str::StrDeducer};

pub mod str;
pub mod simple;

pub trait Deducer {
    async fn deduce(&'static self, exec: &'static Executor, value: Value) -> &'static Expr;
}


pub enum DeducerEnum {
    Str(StrDeducer),
    Simple(SimpleDeducer)
}

impl DeducerEnum {
    pub fn from_nt(cfg: &Cfg, ctx: &Context, nt: usize) -> Self {
        match cfg[nt].ty {
            crate::value::Type::Str => {
                let mut result = StrDeducer::new(nt);
                if let Some(ProdRule::Op2(_, n1, n2)) = cfg[nt].get_op2("str.++") {
                    if n1 == n2 && n1 == nt {
                        result.split_once = min(ctx.len(), max(5, ctx.len() / 10));
                        if let Some(ProdRule::Op3(_, n1, n2, n3)) = cfg[nt].get_op3("ite") {
                            if n3 == n2 && n2 == nt {
                                result.ite_concat = (max(1, ctx.len() / 30), n1)
                            }
                        }
                    }
                }
                if let Some(ProdRule::Op2(_, n1, n2)) = cfg[nt].get_op2("list.join") {
                    if n2 == nt {
                        result.join = (min(ctx.len(), max(5, ctx.len() / 10)), n1)
                    }
                }
                info!("Deduction: {result:?}");
                Self::Str(result)
            }
            crate::value::Type::ListStr => Self::Simple(SimpleDeducer{ nt }),
            _ => Self::Simple(SimpleDeducer{ nt }),
        }
    }
}

impl Deducer for DeducerEnum {
    async fn deduce(&'static self, exec: &'static Executor, value: Value) -> &'static Expr {
        let result = match self {
            DeducerEnum::Str(a) => a.deduce(exec, value).await,
            DeducerEnum::Simple(a) => a.deduce(exec, value).await,
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

