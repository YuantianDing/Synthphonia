use std::{cmp::{max, min}, future::Future, sync::Arc};

use crate::{debg, expr::{cfg::{Cfg, NonTerminal, ProdRule}, context::Context, Expr}, forward::executor::Enumerator, info, parser::problem, utils::fut::select_all, value::Value};


use futures::{future::Either, select, FutureExt};
use itertools::Itertools;

use self::{liststr::ListDeducer, simple::SimpleDeducer, str::StrDeducer};
use derive_more::DebugCustom;
pub mod str;
pub mod simple;
pub mod liststr;

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
    fn deduce(&'static self, exec: Arc<Enumerator>, value: Problem) -> impl std::future::Future<Output = &'static Expr> + Send;
}

#[derive(DebugCustom)]
pub enum DeducerEnum {
    #[debug(fmt = "{:?}", _0)]
    Str(StrDeducer),
    #[debug(fmt = "{:?}", _0)]
    Simple(SimpleDeducer),
    #[debug(fmt = "{:?}", _0)]
    List(ListDeducer),
}

impl DeducerEnum {
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
    async fn deduce(&'static self, exec: Arc<Enumerator>, problem: Problem) -> &'static Expr {
        let is_pending = exec.data[problem.nt].all_eq.is_pending(problem.value);
        if is_pending { return exec.data[problem.nt].all_eq.acquire(problem.value).await; }

        let result = match self {
            DeducerEnum::Str(a) => a.deduce(exec.clone(), problem).await,
            DeducerEnum::Simple(a) => a.deduce(exec.clone(), problem).await,
            DeducerEnum::List(a) => a.deduce(exec.clone(), problem).await,
        };
        debg!("{exec:?} Subproblem {:?} solved", problem.value);
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

