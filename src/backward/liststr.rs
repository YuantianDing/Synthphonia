

use std::{pin::pin, sync::Arc};


use smol::Task;

use crate::{backward::str::HandleRcVec, closure, debg, expr::{cfg::Cfg, context::Context, ops::{self, Op1Enum}, Expr}, forward::{data::len, executor::Enumerator}, galloc::{self, AllocForAny}, never, solutions::new_thread_with_limit, utils::fut::{select_ret, select_ret3}, value::Value};

use super::{Deducer, Problem};



#[derive(Debug)]
pub struct ListDeducer {
    pub nt: usize,
    pub map: Option<Cfg>,
}

impl Deducer for ListDeducer {
    async fn deduce(&'static self, exec: Arc<Enumerator>, prob: Problem) -> &'static crate::expr::Expr {
        debg!("{exec:?} Deducing subproblem: {} {:?}", exec.cfg[self.nt].name, prob.value);
        let exec2 = exec.clone();
        let task = exec2.data[self.nt].all_eq.acquire(prob.value);

        let futures = HandleRcVec::new();
        let map_event = len::Data::listen_for_each(exec2.data[self.nt].len.as_ref().unwrap(), prob.value, closure! { clone futures, clone prob; move |delimiter: Value| {
                if self.map.is_some() {
                    futures.extend_iter(self.map(exec.clone(), prob, delimiter).into_iter());
                }
                None::<&'static Expr>
        }});

        let result = select_ret3(pin!(map_event), pin!(task), pin!(futures)).await;
        result
    }
}

impl ListDeducer {
    #[inline]
    pub fn map(&'static self, exec: Arc<Enumerator>, mut prob: Problem, list: Value) -> Option<Task<&'static Expr>> {
        if prob.used_cost >= 4 { return None; }
        let p = prob.value.to_liststr();
        if p.iter().all(|x| x.len() <= 2) {  return None; }
        let l = list.to_liststr();
        assert!(p.iter().zip(l.iter()).all(|(a, b)| a.len() == b.len()));
        

        let p = prob.value.flatten_leak();
        let l = list.flatten_leak();
        Some(smol::spawn(async move {

            let mut cfg = self.map.as_ref().unwrap().clone();
            let ctx = Context::new(p.len(), vec![l.into()], vec![], p.into());
            cfg.config.size_limit = 8;
            cfg.config.time_limit = 500;
            let handle = new_thread_with_limit(cfg, ctx);
            debg!("{exec:?} ListDeducer::map {:?} {:?} new thread {:?}", prob.value, list, handle);
            let inner = handle.await;
            let mut result = exec.data[prob.nt].all_eq.get(list.into());
            Expr::Op1(Op1Enum::Map(ops::Map(Some(inner.alloc_local()))).galloc(), result).galloc()
        }))
    }
}