use std::{
    cell::{Cell, RefCell, UnsafeCell}, collections::{hash_map::Entry, HashMap, HashSet}, default, f64::consts::E, fs, future::Future, pin::pin, sync::{atomic::AtomicUsize, Arc}, task::Poll, time::{self, Duration, Instant}
};

use derive_more::{Constructor, Deref, From, Into, DebugCustom};
use futures::{FutureExt, StreamExt};
use itertools::Itertools;
use smol::Task;
use spin::Mutex;

use crate::{
    backward::{ Deducer, DeducerEnum, Problem}, debg, debg2, expr::{
         cfg::{Cfg, ProdRule}, context::Context, Expr
    }, forward::{data::{size, substr}, enumeration::ProdRuleEnumerate, executor}, galloc::AllocForAny, info, log, parser::problem::PBEProblem, solutions::CONDITIONS, utils::UnsafeCellExt, value::{ConstValue, Value}, warn
};
use crate::expr;
use super::{bridge::Bridge, data::{self, all_eq, size::EV, Data}};

pub trait EnumFn = FnMut(Expr, Value) -> Result<(), ()>;

// pub struct TaskWaitingCost {
//     sender: broadcastque::Sender<()>,
//     cur_cost: usize,
// }

// impl TaskWaitingCost {
//     pub fn new() -> Self {
//         TaskWaitingCost { sender: broadcastque::channel(), cur_cost: 0  }
//     }
    
//     pub async fn inc_cost(&mut self, problem: &mut Problem, amount: usize) -> () {
//         // let mut rv: broadcastque::Reciever<()> = self.sender.reciever();
//         // problem.used_cost += amount;
//         // let amount = problem.used_cost as isize - self.cur_cost as isize;
//         // if amount > 0 {
//         //     for _ in 0..amount {
//         //         let _ = rv.next().await;
//         //     }
//         // }
//         ()
//     }
    
//     pub fn release_cost_limit(&mut self, count: usize) -> () {
//         self.sender.send((), count);
//     }
// }

pub struct OtherData {
    pub all_str_const: HashSet<&'static str>,
    // pub problems: UnsafeCell<HashMap<(usize, Value), TaskORc<&'static Expr>>>,
}

pub struct Counter {
    pub counter: usize,
    pub subproblem_count: usize,
    pub cur_size: usize,
    pub cur_nt: usize,
}

impl Counter {
    pub fn new() -> Self {
        Counter { counter: 0, subproblem_count: 0, cur_size: 0, cur_nt: 0 }
    }
}
#[derive(DebugCustom)]
#[debug(fmt="{:x?}", id)]
pub struct Enumerator {
    pub id: u32,
    pub counter: Mutex<Counter>,
    pub ctx: Context,
    pub cfg: Cfg,
    pub deducers: Vec<&'static DeducerEnum>,
    pub data: Vec<Data>,
    pub other: OtherData,
    pub top_task: Mutex<Task<&'static Expr>>,
    expr_collector: Mutex<Vec<EV>>,
    pub bridge: Bridge,
    pub start_time: time::Instant,
}

impl Drop for Enumerator {
    fn drop(&mut self) {
        info!("{self:?} Enumerator Finished: {:?}", self.ctx.output);
    }
}

impl Enumerator {
    pub fn new(ctx: Context, cfg: Cfg) -> Arc<Self> {
        let all_str_const = cfg[0].rules.iter().flat_map(|x| if let ProdRule::Const(ConstValue::Str(s)) = x { Some(*s) } else { None }).collect();
        let data = Data::new(&cfg, &ctx);
        let deducers = (0..cfg.len()).map(|i, | DeducerEnum::from_nt(&cfg, &ctx, i).galloc()).collect_vec();
        let other = OtherData { all_str_const };
        let exec = Self { id: rand::random(),
            counter: Counter::new().into(), ctx, cfg, data, other, deducers, expr_collector: Vec::new().into(),
            top_task: smol::spawn(futures::future::pending()).into(), bridge: Bridge::new(),
            start_time: Instant::now() };
        // TextObjData::build_trie(&exec);
        Arc::new(exec)
    }
    pub fn collect_expr(&self, e: &'static Expr, v: Value) {
        self.expr_collector.lock().push((e, v))
    }
    pub fn extract_expr_collector(&self) -> Vec<EV> {
        let mut lock = self.expr_collector.lock();
        std::mem::replace(&mut *lock, Vec::new())
    }
    pub fn cur_data(&self) -> &Data {
        &self.data[self.counter.lock().cur_nt]
    }
    #[inline]
    pub async fn solve_task(self: Arc<Self>, problem: Problem) -> &'static Expr {
        if let Some(e) = self.data[problem.nt].all_eq.at(problem.value) {
            return e;
        }
        self.counter.lock().subproblem_count += 1;
        smol::spawn(self.deducers[problem.nt].deduce(self, problem)).await
    }
    #[inline]
    pub async fn generate_condition(self: Arc<Self>, problem: Problem, result: &'static Expr) -> &'static Expr {
        if problem.value.is_all_true() { return result; }
        let left = pin!(self.clone().solve_task(problem.clone()));
        let right = pin!(self.solve_task(problem.clone().with_value(problem.value.bool_not())));
        let cond = futures::future::select(left, right).await;
        match cond {
            futures::future::Either::Left((c, _)) => 
                expr!(Ite {c} {result} "").galloc(),
            futures::future::Either::Right((c, _)) => 
                expr!(Ite {c} "" {result}).galloc(),
        }
    }
    pub fn solve_top_blocked(self: Arc<Self>) -> &'static Expr {
        let problem = Problem::root(0, self.ctx.output);
        self.counter.lock().subproblem_count += 1;
        *self.top_task.lock() = smol::spawn(self.clone().deducers[problem.nt].deduce(self.clone(), problem));
        let _ = self.clone().run();
        self.bridge.abort_all();
        let task = std::mem::replace(&mut *self.top_task.lock(), smol::spawn(futures::future::pending()));
        if let Some(r) = smol::block_on(task.cancel()) {
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

    pub fn solve_top_with_limit(self: Arc<Self>) -> Option<&'static Expr> {
        let problem = Problem::root(0, self.ctx.output);
        self.counter.lock().subproblem_count += 1;
        *self.top_task.lock() = smol::spawn(self.clone().deducers[problem.nt].deduce(self.clone(), problem));
        let _ = self.clone().run();
        self.bridge.abort_all();
        let task = std::mem::replace(&mut *self.top_task.lock(), smol::spawn(futures::future::pending()));
        smol::block_on(task.cancel())
    }

    pub fn size(&self) -> usize { self.counter.lock().cur_size }
    pub fn nt(&self) -> usize { self.counter.lock().cur_nt }
    pub fn count(&self) -> usize { self.counter.lock().counter }
    pub fn subproblem_count(&self) -> usize { self.counter.lock().subproblem_count }
    
    #[inline]
    pub fn enum_expr(self: Arc<Self>, e: Expr, v: Value) -> Result<(), ()> {
        if self.count() % 10000 == 1 {
            if self.count() % 300000 == 1 {
                info!("{self:?} Searching size={} [{}] - {:?} {:?} {}", self.size(), self.count(), e, v, self.subproblem_count());
            }
            // self.waiting_tasks().release_cost_limit(self.cfg.config.increase_cost_limit);
            self.bridge.check();
        }
        self.counter.lock().counter += 1;
        if let Some(e) = self.cur_data().update(self.clone(), e, v)? {
            self.collect_expr(e,v);
            if self.cfg.config.cond_search {
                self.clone().collect_condition(e, v);
            }
        }
        if self.top_task.lock().is_finished() || (Instant::now() - self.start_time).as_millis() >= self.cfg.config.time_limit as u128 {
            return Err(());
        }
        Ok(())
    }
    fn collect_condition(self: Arc<Self>, e: &'static Expr, v: Value) {
        if let Value::Bool(p) = v {
            let tcount: usize = p.iter().map(|x| if *x { 1 } else { 0 }).sum();
            if tcount >= 1 {
                let mut conditions = CONDITIONS.lock();
                conditions.push((e, v.to_bits()));
            }
        }
    }
    fn run(self: Arc<Self>) -> Result<(), ()> {
        let _ = self.extract_expr_collector();
        for size in 1 ..self.cfg.config.size_limit {
            let self1 = self.clone();
            for (nt, ntdata) in self1.cfg.iter().enumerate() {
                self.counter.lock().cur_size = size;
                self.counter.lock().cur_nt = nt;
                info!("{self:?} Enumerating size={} nt={} with - {}", size, ntdata.name, self.count());
                // self.cur_data().to.enumerate(self)?;
                for rule in &ntdata.rules {
                    rule.enumerate(self.clone())?;
                }
                
                self.cur_data().size.add(size, self.extract_expr_collector());
            }
        }
        Ok(())
    }
    // pub fn get_problem(self: Arc<Self>, p: Problem) -> TaskORc<&'static Expr> {
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
    // pub fn block_on<T>(self: Arc<Self>, t: TaskORc<T>) -> Option<T> {
    //     task::with_top_task(t.clone().task(), || {
    //         let _ = self.run();
    //     });
    //     match t.poll_unpin() {
    //         Poll::Ready(res) => Some(res),
    //         Poll::Pending => None,
    //     }
    // }
    // pub fn run_with(self: Arc<Self>, p: Problem) -> Option<&'static Expr> {
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
