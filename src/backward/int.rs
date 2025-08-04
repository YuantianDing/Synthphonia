use std::pin::pin;

use itertools::Itertools;
use simple_rc_async::task;

use crate::utils::select_ret;
use crate::{debg, expr::Expr, galloc::AllocForAny, never};
use crate::expr;

use super::{Deducer, Problem};



#[derive(Debug)]
/// A structure using a basic deduction strategy for every type `int`, `bool`, etc. 
pub struct IntDeducer {
    pub nt: usize,
    pub len: usize,
}

impl Deducer for IntDeducer {
    /// Deduces a given `Problem` asynchronously using the `Executor`. 
    async fn deduce(&'static self, exec: &'static crate::forward::executor::Executor, problem: Problem) -> &'static crate::expr::Expr {
        debg!("Deducing subproblem: {} {:?}", exec.cfg[self.nt].name, problem.value);
        let task = pin!(exec.data[self.nt].all_eq.acquire(problem.value));
        let v = problem.value.to_int();
        if self.len == usize::MAX || v.iter().any(|x| *x < 0) || exec.data[self.len].len().is_none() {
            return task.await;
        }
        let len_task = task::spawn(async move {
            let a = exec.data[self.len].len().unwrap();
            let v = a.listen_for_each(v.iter().map(|a| *a as usize).collect(), |result| Some(result)).await;
            debg!("IntDeducer: len task result: {:?}", v);
            let result = exec.data[self.len].all_eq.get(v);
            expr!(Len {result}).galloc()
        });
            
        select_ret(task, len_task).await
    }
}