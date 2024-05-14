

use std::pin::pin;

use rc_async::task::{self, JoinHandle};

use crate::{backward::str::HandleRcVec, closure, debg, expr::{cfg::Cfg, context::Context, ops::{self, Op1Enum}, Expr}, forward::executor::Executor, galloc::{self, AllocForAny}, never, solutions::{new_thread_with_limit}, utils::{select_ret, select_ret3}, value::Value};

use super::{Deducer, Problem};



#[derive(Debug)]
pub struct ListDeducer {
    pub nt: usize,
    pub map: Option<Cfg>,
}

impl Deducer for ListDeducer {
    async fn deduce(&'static self, exec: &'static crate::forward::executor::Executor, prob: Problem) -> &'static crate::expr::Expr {
        debg!("Deducing subproblem: {} {:?}", exec.cfg[self.nt].name, prob.value);
        let task = exec.data[self.nt].all_eq.acquire(prob.value);

        let futures = HandleRcVec::new();
        let map_event = exec.data[self.nt].len().unwrap().listen_for_each(prob.value, closure! { clone futures, clone prob; move |delimiter: Value| {
                if self.map.is_some() {
                    futures.extend_iter(self.map(exec, prob, delimiter).into_iter());
                }
                None::<&'static Expr>
        }});

        select_ret3(pin!(map_event), pin!(task), pin!(futures)).await
    }
}

impl ListDeducer {
    #[inline]
    pub fn map(&'static self, exec: &'static Executor, mut prob: Problem, list: Value) -> Option<JoinHandle<&'static Expr>> {
        if prob.used_cost >= 6 { return None; }
        let p = prob.value.to_liststr();
        if p.iter().all(|x| x.len() <= 2) {  return None; }
        let l = list.to_liststr();
        assert!(p.iter().zip(l.iter()).all(|(a, b)| a.len() == b.len()));
        

        let p = prob.value.flatten_leak();
        let l = list.flatten_leak();
        Some(task::spawn(async move {

            let mut cfg = self.map.as_ref().unwrap().clone();
            let ctx = Context::new(p.len(), vec![l.into()], vec![], p.into());
            cfg.config.size_limit = 10;
            cfg.config.time_limit = 1000;
            let handle = new_thread_with_limit(cfg, ctx);
            debg!("ListDeducer::map {:?} {:?} new thread {}", prob.value, list, handle.id());
            let inner = exec.bridge.wait(handle).await;
            let mut result = exec.data[prob.nt].all_eq.get(list.into());
            Expr::Op1(Op1Enum::Map(ops::Map(Some(inner.alloc_local()))).galloc(), result).galloc()
        }))
    }
}