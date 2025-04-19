use crate::debg;

use super::{Deducer, Problem};



#[derive(Debug)]
/// A structure using a basic deduction strategy for every type `int`, `bool`, etc. 
pub struct SimpleDeducer {
    pub nt: usize
}

impl Deducer for SimpleDeducer {
    /// Deduces a given `Problem` asynchronously using the `Executor`. 
    async fn deduce(&'static self, exec: &'static crate::forward::executor::Executor, problem: Problem) -> &'static crate::expr::Expr {
        debg!("Deducing subproblem: {} {:?}", exec.cfg[self.nt].name, problem.value);
        let task = exec.data[self.nt].all_eq.acquire(problem.value);
        task.await
    }
}