use std::{
    cell::{Cell, RefCell, UnsafeCell}, collections::{hash_map::Entry, HashMap, HashSet}, default, f64::consts::E, fs, future::Future, pin::pin, task::Poll
};

use derive_more::{Constructor, Deref, From, Into};
use futures::StreamExt;
use itertools::Itertools;
use rc_async::{sync::broadcast, task::{self, JoinHandle}};

use crate::{
    backward::{ Deducer, DeducerEnum, Problem}, debg, debg2, expr::{
         cfg::{Cfg, ProdRule}, context::Context, Expr
    }, forward::{data::{size, substr}, enumeration::ProdRuleEnumerate, executor}, galloc::AllocForAny, info, log, parser::problem::PBEProblem, solutions::CONDITIONS, text::parsing::{ParseInt, TextObjData}, utils::UnsafeCellExt, value::{ConstValue, Value}, warn
};
use crate::expr;
use super::data::{self, all_eq, size::EV, Data};

pub trait EnumFn = FnMut(Expr, Value) -> Result<(), ()>;

pub struct TaskWaitingCost {
    sender: broadcast::Sender<()>
}

impl TaskWaitingCost {
    pub fn new() -> Self {
        TaskWaitingCost { sender: broadcast::channel() }
    }
    
    pub async fn inc_cost(&mut self, problem: &mut Problem, amount: usize) -> () {
        let mut rv = self.sender.reciever();
        for _ in 0..amount {
            let _ = rv.next().await;
        }
    }

    pub fn release_cost_limit(&mut self, count: usize) -> () {
        for _ in 0..count {
            self.sender.send(());
        }
    }
}

pub struct OtherData {
    pub all_str_const: HashSet<&'static str>,
    // pub problems: UnsafeCell<HashMap<(usize, Value), TaskORc<&'static Expr>>>,
}

pub struct Executor {
    pub counter: Cell<usize>,
    pub subproblem_count: Cell<usize>,
    pub cur_size: Cell<usize>,
    pub cur_nt: Cell<usize>,
    pub ctx: Context,
    pub cfg: Cfg,
    pub deducers: Vec<DeducerEnum>,
    pub data: Vec<Data>,
    pub other: OtherData,
    pub waiting_tasks: UnsafeCell<TaskWaitingCost>,
    pub top_task: UnsafeCell<JoinHandle<&'static Expr>>,
    expr_collector: UnsafeCell<Vec<EV>>,
}

impl Executor {
    pub fn problem_count(&self) -> usize{
        self.subproblem_count.get()
    }
    pub fn new(ctx: Context, cfg: Cfg) -> &'static Self {
        let all_str_const = cfg[0].rules.iter().flat_map(|x| if let ProdRule::Const(ConstValue::Str(s)) = x { Some(*s) } else { None }).collect();
        let data = Data::new(&cfg, &ctx);
        let deducers = (0..cfg.len()).map(|i, | DeducerEnum::from_nt(&cfg, &ctx, i)).collect_vec();
        let other = OtherData { all_str_const };
        let exec = Self { counter: 0.into(), subproblem_count: 0.into(), ctx, cfg, data, other, deducers, expr_collector: Vec::new().into(),
            cur_size: 0.into(), cur_nt: 0.into(), waiting_tasks: TaskWaitingCost::new().into(), top_task: task::spawn(futures::future::pending()).into() };
        TextObjData::build_trie(&exec);
        exec.galloc()
    }
    pub fn top_task(&self) -> &mut JoinHandle<&'static Expr> {
        unsafe { self.top_task.as_mut() }
    }
    pub fn collect_expr(&self, e: &'static Expr, v: Value) {
        unsafe { self.expr_collector.as_mut().push((e, v)) }
    }
    pub fn waiting_tasks(&self) -> &mut TaskWaitingCost {
        unsafe { self.waiting_tasks.as_mut() }
    }
    pub fn extract_expr_collector(&self) -> Vec<EV> {
        self.expr_collector.replace(Vec::new())
    }
    pub fn cur_data(&self) -> &Data {
        &self.data[self.cur_nt.get()]
    }
    #[inline]
    pub async fn solve_task(&'static self, problem: Problem) -> &'static Expr {
        if let Some(e) = self.data[problem.nt].all_eq.at(problem.value) {
            return e;
        }
        self.subproblem_count.update(|x| x+1);
        task::spawn(self.deducers[problem.nt].deduce(self, problem)).await
    }
    #[inline]
    pub async fn generate_condition(&'static self, problem: Problem, result: &'static Expr) -> &'static Expr {
        if problem.value.is_all_true() { return result; }
        let left = pin!(self.solve_task(problem.clone()));
        let right = pin!(self.solve_task(problem.clone().with_value(problem.value.bool_not())));
        let cond = futures::future::select(left, right).await;
        match cond {
            futures::future::Either::Left((c, _)) => 
                expr!(Ite {c} {result} "").galloc(),
            futures::future::Either::Right((c, _)) => 
                expr!(Ite {c} "" {result}).galloc(),
        }
    }
    pub fn solve_top_blocked(&'static self, problem: Problem) -> &'static Expr {
        self.subproblem_count.update(|x| x+1);
        *self.top_task() = task::spawn(self.deducers[problem.nt].deduce(self, problem));
        let _ = self.run();
        
        if let Poll::Ready(r) = self.top_task().poll_rc_nocx() {
            r
        } else { panic!("should not reach here.") }
        // match problems.entry((nt, value)) {
        //     Entry::Occupied(o) => o.get().clone(),
        //     Entry::Vacant(e) => {
        //         let t = ;
        //         e.insert(t.clone());
        //         t
        //     }
        // }
    }

    pub fn size(&self) -> usize { self.cur_size.get() }
    pub fn nt(&self) -> usize { self.cur_nt.get() }
    pub fn count(&self) -> usize { self.counter.get() }
    
    #[inline]
    pub fn enum_expr(&'static self, e: Expr, v: Value) -> Result<(), ()> {
        if self.counter.get() % 300000 == 0 {
            info!("Searching size={} [{}] - {:?} {:?}", self.cur_size.get(), self.counter.get(), e, v);
            if self.counter.get() > 300000 {
                self.waiting_tasks().release_cost_limit(self.cfg.config.increase_cost_limit);
            }
        }
        self.counter.update(|x| x + 1);
        
        if let Some(e) = self.cur_data().update(self, e, v)? {
            self.collect_expr(e,v);
            if self.cfg.config.cond_search {
                self.collect_condition(e, v);
            }
        }
        if self.top_task().is_ready() {
            return Err(());
        }
        Ok(())
    }
    fn collect_condition(&'static self, e: &'static Expr, v: Value) {
        if let Value::Bool(p) = v {
            let tcount: usize = p.iter().map(|x| if *x { 1 } else { 0 }).sum();
            if tcount >= 1 {
                let mut conditions = CONDITIONS.lock();
                conditions.push((e, v.to_bits()));
            }
        }
    }
    fn run(&'static self) -> Result<(), ()> {
        let _ = self.extract_expr_collector();
        for size in 1 ..self.cfg.config.size_limit {
            for (nt, ntdata) in self.cfg.iter().enumerate() {
                self.cur_size.set(size);
                self.cur_nt.set(nt);
                info!("Enumerating size={} nt={} with - {}", size, ntdata.name, self.counter.get());
                self.cur_data().to.enumerate(self)?;
                for rule in &ntdata.rules {
                    rule.enumerate(self)?;
                }
                
                self.cur_data().size.add(size, self.extract_expr_collector());
            }
        }
        Ok(())
    }
    // pub fn get_problem(&'static self, p: Problem) -> TaskORc<&'static Expr> {
    //     let hash = unsafe { self.other.problems.as_mut() };
    //     match hash.entry(p.clone()) {
    //         std::collections::hash_map::Entry::Occupied(p) => p.get().clone(),
    //         std::collections::hash_map::Entry::Vacant(v) => {
    //             let t = task::spawn(p.deduce(self)).tasko();
    //             v.insert(t.clone());
    //             t
    //         }
    //     }
    // }
    // pub fn block_on<T>(&'static self, t: TaskORc<T>) -> Option<T> {
    //     task::with_top_task(t.clone().task(), || {
    //         let _ = self.run();
    //     });
    //     match t.poll_unpin() {
    //         Poll::Ready(res) => Some(res),
    //         Poll::Pending => None,
    //     }
    // }
    // pub fn run_with(&'static self, p: Problem) -> Option<&'static Expr> {
    //     let t = self.get_problem(p);
    //     task::with_top_task(t.task(), || {
    //         let _ = self.run();
    //     });
    //     match t.poll_unpin() {
    //         Poll::Ready(res) => Some(res),
    //         Poll::Pending => None,
    //     }
    // }
}

