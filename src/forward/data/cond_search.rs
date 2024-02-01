use std::{collections::HashMap, cell::UnsafeCell};

use crate::{forward::executor::Executor, backward::Problem, expr::{cfg::{Cfg, ProdRule}, context::Context, Expr}, value::Value, arena::AllocForAny, parser::config::Config, log, utils::UnsafeCellExt};


thread_local! {
    static COND_PROMBLEMS: UnsafeCell<HashMap<RunningConfig, Option<&'static Expr>>> = HashMap::new().into();
}


#[derive(Hash, PartialEq, Eq, Clone)]
pub struct RunningConfig {
    pub input: (Value, usize),
    pub target: (Value, usize),
    pub size: usize,
    pub cond_search: bool,
}

impl RunningConfig {
    pub fn search(self, mut cfg: Cfg, mut ctx: Context) -> Option<&'static Expr> {
        if let Some(a) = unsafe { COND_PROMBLEMS.with(|x| x.as_mut().get(&self).cloned()) } {
            return a;
        }
        cfg.condition_search = self.cond_search;
        cfg.size_limit = self.size;
        ctx.n.push(self.input.0);
        ctx.len = self.target.0.len();
        cfg[self.input.1].rules.push(ProdRule::Var(-(ctx.n.len() as i64), Config::new()));
        let log_level = log::log_level();
        log::set_log_level(0);
        // Memory Leak !!!!!!!!!!!!!!!!!!!!!!
        // let output = ctx.output;
        // ctx.output = self.target.0;
        let exec = Executor::new(ctx, cfg).galloc();
        let result = exec.block_on(Problem{ value: self.target.0, nt: self.target.1}.deduce(exec));
        log::set_log_level(log_level);
        unsafe { COND_PROMBLEMS.with(|x| x.as_mut().insert(self, result)) };
        result
    }
}








