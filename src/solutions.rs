use std::{collections::VecDeque, time::{self, Duration, Instant}};

use futures::StreamExt;
use tokio::{select, task::JoinHandle};

use itertools::Itertools;
use mapped_futures::mapped_futures::MappedFutures;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::{backward::Problem, debg, expr::{cfg::Cfg, context::Context, Expr}, forward::executor::Executor, galloc::AllocForAny, info, never, tree_learning::{bits::BoxSliceExt, tree_learning, Bits}};



pub static CONDITIONS: spin::Mutex<Vec<(&'static Expr, Bits)>> = spin::Mutex::new(Vec::new());


pub struct Solutions {
    cfg: Cfg,
    ctx: Context,
    solutions: Vec<(&'static Expr, Bits)>,
    solved_examples: Bits,
    pub threads: MappedFutures<Vec<usize>, JoinHandle<&'static Expr>>,
    start_time: Instant,
}

impl Solutions {
    pub fn new(cfg: Cfg, ctx: Context) -> Self {
        let solutions = Vec::new();
        let solved_examples = Bits::zeros(ctx.len);
        Self { cfg, ctx, solutions, solved_examples, threads: MappedFutures::new(), start_time: time::Instant::now() }
    }

    pub fn add_new_solution(&mut self, expr: &'static Expr) -> Option<&'static Expr> {
        if let Some(b) = self.ctx.evaluate(expr) {
            // Updating solutions
            for (e, bits) in self.solutions.iter() {
                if b.subset(&bits) {
                    return None;
                }
            }
            self.solutions.retain(|(e, bits)| !bits.subset(&b));
            self.solved_examples.union_assign(&b);
            self.solutions.push((expr, b.clone()));
            debg!("Solutions [{}/{}]: {:?}", self.solved_examples.count_ones(), self.ctx.len, self.solutions);

            // Updating threads
            let keys = self.threads.keys().cloned().collect_vec();
            for k in keys {
                if k.iter().all(|i| b.get(*i)) {
                    if let Some(a) = self.threads.remove(&k) {
                        a.abort();
                        info!("Interupting Thread of {k:?}");
                        self.create_new_thread();
                    }
                }
            }

            // Generating Solution
            if b.count_ones() == self.ctx.len as u32 {
                Some(expr)
            } else { self.generate_result(true) }
        } else { None }
    }
    pub fn generate_result(&self, limit: bool) -> Option<&'static Expr> {
        if self.solved_examples.count_ones() == self.ctx.len as u32 {
            self.learn_tree(if limit { self.cfg.config.ite_limit_rate } else { 1 })
        } else { None }
    }
    pub fn learn_tree(&self, ite_limit_rate: usize) -> Option<&'static Expr> {
        let duration = time::Instant::now() - self.start_time;
        let ite_limit = duration.as_millis() as usize / ite_limit_rate + 1;
        let conditions = CONDITIONS.lock();
        debg!("Conditions: {}", conditions.len());
        if conditions.len() == 0 { return None; }
        let bump = bumpalo::Bump::new();
        let result = tree_learning(self.solutions.clone(), &conditions[..], self.ctx.len, &bump, ite_limit);
        if result.solved {
            Some(result.expr())
        } else {
            None
        }
    }
    pub fn check_cover(&self, example_set: &[usize]) -> bool {
        for (e, bits) in self.solutions.iter() {
            if example_set.iter().all(|i| bits.get(*i)) {
                return true;
            }
        }
        false
    }
    pub fn generate_example_set(&mut self) -> Option<Vec<usize>> {
        let mut rng = rand::thread_rng();
        for k in 1..std::cmp::min(5, self.ctx.len) {
            let mut vec = (0..self.ctx.len).combinations(k).collect_vec();
            vec.shuffle(&mut rng);
            for v in vec {
                if !self.check_cover(&v) && !self.threads.contains(&v) { return Some(v); }
            }
        }
        None
    }
    pub fn create_new_thread(&mut self) {
        if let Some(exs) = self.generate_example_set() {
            info!("Creating new thread with examples {:?}", exs);
            let ctx2 = self.ctx.with_examples(&exs);
            self.threads.insert(exs, new_thread(self.cfg.clone(), ctx2));
        }
    }
    pub fn create_cond_search_thread(&mut self) {
        info!("Creating condition search thread.");
        let mut cfg = self.cfg.clone();
        cfg.config.cond_search = true;
        self.threads.insert((0..self.ctx.len).collect_vec(), new_thread(cfg, self.ctx.clone()));
    }
    pub async fn solve_loop(&mut self) -> &'static Expr {
        loop {
            select! {
                result = self.threads.next() => {
                    let (k,v) = result.unwrap();
                    let v = v.expect("Thread Execution Error");
                    info!("Found a solution {:?} with examples {:?}.", v, k);
                    if let Some(e) = self.add_new_solution(v) {
                        return e;
                    }
                    self.create_new_thread();
                }
                _ = tokio::time::sleep(Duration::from_millis(2000)) => {
                    if let Some(e) = self.generate_result(self.threads.len() != 0) { return e; }
                }
            }
        }
    }
}

pub fn new_thread(cfg: Cfg, ctx: Context) -> JoinHandle<&'static Expr> {
    tokio::spawn(async move {
        let exec = Executor::new(ctx, cfg).galloc();
        info!("Deduction Configuration: {:?}", exec.deducers);
        exec.block_on(exec.spawn_task(Problem::root(0, exec.ctx.output))).expect("Failure")
    })
}

pub fn cond_search_thread(mut cfg: Cfg, ctx: Context) -> JoinHandle<&'static Expr> {
    cfg.config.cond_search = true;
    tokio::spawn(async move {
        let exec = Executor::new(ctx, cfg).galloc();
        info!("Deduction Configuration: {:?}", exec.deducers);
        exec.block_on(exec.spawn_task(Problem::root(0, exec.ctx.output))).expect("Failure")
    })
}