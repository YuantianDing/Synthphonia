use std::{collections::HashMap, cell::UnsafeCell};

use crate::{forward::executor::Executor, backward::Problem, expr::{cfg::{Cfg, ProdRule}, context::Context, Expr}, value::Value, arena::AllocForAny, parser::config::Config, log, utils::UnsafeCellExt};


thread_local! {
    static COND_PROMBLEMS: UnsafeCell<HashMap<RunningConfig, Option<&'static Expr>>> = HashMap::new().into();
}


#[derive(Hash, PartialEq, Eq, Clone)]
/// This structure encapsulates configuration information used during the synthesis process. 
/// It stores input and target values along with their associated sizes, a general size parameter, and a flag controlling conditional search behavior.
/// 
/// It aggregates key runtime parameters required for configuring synthesis operations. 
/// The tuple fields bind a value to a specific size while the separate size field and boolean flag provide additional control over the synthesis process, allowing fine-tuning of search constraints.
pub struct RunningConfig {
    pub input: (Value, usize),
    pub target: (Value, usize),
    pub size: usize,
    pub cond_search: bool,
}

impl RunningConfig {
    /// Searches for a synthesized expression that satisfies the problem constraints defined by the running configuration.
    /// 
    /// Configures the synthesis process by first checking a global cache for a precomputed result. 
    /// If absent, it updates the provided configuration and synthesis context using the input and target values, including appending a variable production rule to represent the new input. 
    /// The method then temporarily suppresses logging, instantiates an asynchronous executor, and initiates the deduction process on the target expression. 
    /// Once the deduction completes, it resets the logging level, caches the result, and returns the synthesized expression if one exists.
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








