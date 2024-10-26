use std::{collections::{hash_map::Entry, HashMap, VecDeque}, time::{self, Duration, Instant}};

use futures::StreamExt;
use tokio::{select, task::JoinHandle};

use itertools::Itertools;
use mapped_futures::mapped_futures::MappedFutures;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::{backward::Problem, debg, expr::{cfg::Cfg, context::Context, Expr, Expression}, forward::executor::Executor, galloc::{self, AllocForAny}, info, never, tree_learning::{bits::BoxSliceExt, tree_learning, Bits}};




pub static CONDITIONS: spin::Mutex<Option<ConditionTracker>> = spin::Mutex::new(None);

pub struct ConditionTracker {
    ctx: Context,
    hashmap: HashMap<Bits, &'static Expr>,
    pub vec: Vec<(&'static Expr, Bits)>
}

impl ConditionTracker {
    pub fn new(ctx: Context) -> Self {
        Self { ctx, hashmap: HashMap::new(), vec: Vec::new() }
    }
    pub fn insert(&mut self, expr: &Expr) {
        let bits = expr.eval(&self.ctx).to_bits();
        if let Entry::Vacant(e) = self.hashmap.entry(bits.clone()) {
            let expr = expr.clone().galloc();
            e.insert(expr);
            self.vec.push((expr, bits));
        }
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

pub fn bicoeff(n: usize, p: usize) -> usize {
    let a: usize = (0..p).map(|i| n - i).product();
    let b: usize = (1..=p).product();
    a.div_ceil(b)
}

pub struct Solutions {
    cfg: Cfg,
    ctx: Context,
    solutions: Vec<(&'static Expr, Bits)>,
    solved_examples: Bits,
    pub threads: MappedFutures<Vec<usize>, JoinHandle<Expression>>,
    start_time: Instant,
}

impl Solutions {
    pub fn new(cfg: Cfg, ctx: Context) -> Self {
        {
            let mut lock = CONDITIONS.lock();
            assert!(lock.is_none());
            *lock = Some(ConditionTracker::new(ctx.clone()));
        }
        let solutions = Vec::new();
        let solved_examples = Bits::zeros(ctx.len);
        Self { cfg, ctx, solutions, solved_examples, threads: MappedFutures::new(), start_time: time::Instant::now() }
    }

    pub fn add_new_solution(&mut self, expr: &'static Expr) -> Option<&'static Expr> {
        if let Some(b) = self.ctx.evaluate(expr) {
            // Updating solutions
            for (_, bits) in self.solutions.iter() {
                if b.subset(&bits) {
                    return None;
                }
            }
            self.solutions.retain(|(e, bits)| !bits.subset(&b));
            self.solved_examples.union_assign(&b);
            self.solutions.push((expr, b.clone()));
            debg!("Solutions [{}/{} {}]: {:?}", self.solved_examples.count_ones(), self.ctx.len, self.threads.len(), self.solutions);

            if b.count_ones() == self.ctx.len as u32 {
                return Some(expr);
            }
            
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
            self.generate_result(true)
        } else { None }
    }
    pub fn generate_result(&self, limit: bool) -> Option<&'static Expr> {
        if self.solved_examples.count_ones() == self.ctx.len as u32 {
            self.learn_tree(if limit { self.cfg.config.ite_limit_rate } else { 1 })
        } else { None }
    }
    pub fn learn_tree(&self, ite_limit_rate: usize) -> Option<&'static Expr> {
        let duration = time::Instant::now() - self.start_time;
        let ite_limit = if duration.as_secs() as usize >= self.cfg.config.ite_limit_giveup {
            self.cfg.config.ite_limit_giveup * 1000 / ite_limit_rate + (duration.as_millis() as usize - self.cfg.config.ite_limit_giveup * 1000) * 5 / ite_limit_rate + 1
        } else { duration.as_millis() as usize / ite_limit_rate + 1};
        let mut lock = CONDITIONS.lock();
        let conditions = lock.as_mut().unwrap();
        debg!("Conditions: {}", conditions.len());
        let bump = bumpalo::Bump::new();
        let result = tree_learning(self.solutions.clone(), &conditions.vec[..], self.ctx.len, &bump, ite_limit);
        if result.solved {
            Some(result.expr())
        } else {
            
            None
        }
    }
    pub fn check_cover(&self, example_set: &[usize]) -> bool {
        for (_, bits) in self.solutions.iter() {
            if example_set.iter().all(|i| bits.get(*i)) {
                return true;
            }
        }
        false
    }
    pub fn generate_example_set(&mut self) -> Option<Vec<usize>> {
        let mut rng = rand::thread_rng();
        for k in 1..=self.ctx.len {
            if bicoeff(self.ctx.len, k) > 4000000 { break; }
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
        } else {
            info!("No available example set");
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
                    let v = v.expect("Thread Execution Error").alloc_local();
                    info!("Found a solution {:?} with examples {:?}.", v, k);
                    if let Some(e) = self.add_new_solution(v) {
                        for v in self.threads.iter() { v.abort(); }
                        return e;
                    }
                    self.create_new_thread();
                }
                _ = tokio::time::sleep(Duration::from_millis(2000)) => {
                    if let Some(e) = self.generate_result(self.threads.len() != 0) {
                        for v in self.threads.iter() { v.abort(); }
                        return e;
                    }
                }
            }
        }
    }
}

pub fn new_thread(cfg: Cfg, ctx: Context) -> JoinHandle<Expression> {
    tokio::spawn(async move {
        let exec = Executor::new(ctx, cfg);
        info!("Deduction Configuration: {:?}", exec.deducers);
        let result = exec.solve_top_blocked().to_expression();
        result
    })
}

pub fn cond_search_thread(mut cfg: Cfg, ctx: Context) -> JoinHandle<Expression> {
    cfg.config.cond_search = true;
    new_thread(cfg, ctx)
}

pub fn new_thread_with_limit(cfg: Cfg, ctx: Context) -> JoinHandle<Expression> {
    tokio::spawn(async move {
        if let Some(p) = (move || {
            let result = Executor::new(ctx, cfg).solve_top_with_limit().map(|e| e.to_expression());
            result
        })() {
            p
        } else { never!() }
    })
}