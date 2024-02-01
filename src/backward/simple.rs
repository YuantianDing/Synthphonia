use crate::{debg, forward::future::task::currect_task_id};

use super::Deducer;



pub struct SimpleDeducer {
    pub nt: usize
}

impl Deducer for SimpleDeducer {
    async fn deduce(&'static self, exec: &'static crate::forward::executor::Executor, value: crate::value::Value) -> &'static crate::expr::Expr {
        debg!("TASK#{} Deducing subproblem: {} {:?}", currect_task_id(), exec.cfg[self.nt].name, value);
        let task = exec.data[self.nt].all_eq.acquire(value);
        task.await
    }
}