use std::{collections::{hash_map::Entry, HashMap, VecDeque}, time::{self, Duration, Instant}};

use futures::StreamExt;
use tokio::{select, task::JoinHandle};

use itertools::Itertools;
use mapped_futures::mapped_futures::MappedFutures;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::{backward::Problem, debg, expr::{cfg::Cfg, context::Context, Expr, Expression}, forward::executor::Executor, galloc::{self, AllocForAny}, info, never, tree_learning::{bits::BoxSliceExt, tree_learning, Bits}};




/// A global static mutex-protected container for optionally holding condition tracking data. 
/// 
/// 
/// This item provides synchronized access to a condition tracker by encapsulating an optional tracker value within a spin-lock-based mutex, ensuring safe concurrent modification and retrieval across threads. 
/// Initially empty, it is intended to be populated at runtime with tracking data as needed.
pub static CONDITIONS: spin::Mutex<Option<ConditionTracker>> = spin::Mutex::new(None);

/// A structure for tracking condition evaluations within a given context. 
/// 
/// 
/// It maintains an internal context used for condition evaluation, a mapping from bit representations to expression references for deduplication, and a public vector storing pairs of expression references and their corresponding bit information for ordered access or iteration.
pub struct ConditionTracker {
    ctx: Context,
    hashmap: HashMap<Bits, &'static Expr>,
    pub vec: Vec<(&'static Expr, Bits)>
}

impl ConditionTracker {
    /// Creates a new condition tracker instance with an initialized context, hashmap, and vector. 
    /// This function takes a context and returns an instance where internal collections are set to their empty defaults, allowing the tracker to accumulate conditions as they are inserted later.
    pub fn new(ctx: Context) -> Self {
        Self { ctx, hashmap: HashMap::new(), vec: Vec::new() }
    }
    /// Inserts a condition expression into the tracker using its evaluated bit representation. 
    /// This method calculates the bit signature of the provided expression and, if this signature is not already present in the internal storage, allocates the expression and registers it along with its corresponding bits.
    pub fn insert(&mut self, expr: &Expr) {
        let bits = expr.eval(&self.ctx).to_bits();
        if let Entry::Vacant(e) = self.hashmap.entry(bits.clone()) {
            let expr = expr.clone().galloc();
            e.insert(expr);
            self.vec.push((expr, bits));
        }
    }
    /// Returns the number of conditions currently stored in the tracker. 
    /// 
    /// 
    /// Calculates and yields the length of the internal vector that maintains a record of condition-expression pairs, providing a quick way to assess how many conditions have been tracked.
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

/// Calculate the binomial coefficient for the given parameters.
/// 
/// This function computes the result of choosing p items from a set of n by deriving the numerator and denominator through iterative multiplication and then performing a ceiling division on these computed values to produce the final coefficient.
pub fn bicoeff(n: usize, p: usize) -> usize {
    let a: usize = (0..p).map(|i| n - i).product();
    let b: usize = (1..=p).product();
    a.div_ceil(b)
}
/// Returns a boolean indicating whether one or more "tree holes" fully satisfy the provided set of indices. 
/// The function iterates over each element in the given collection and checks if every index in the provided slice corresponds to a value of 1 in that element, thereby determining if the set of indices is entirely contained within any of the elements.
pub fn test_tree_hole_contains(tree_hole: &[Box<[u128]>], bits: &[usize]) -> bool {
    for hole in tree_hole.iter() {
        if bits.iter().all(|i| hole[*i] == 1) {
            return true;
        }
    }
    false
}

/// A structure encapsulating the state and configuration for managing synthesis solutions along with multi-threaded search execution. 
/// 
/// 
/// It integrates various components such as a configuration context, a collection of candidate solutions paired with evaluation bits, and management of concurrent solution search threads. 
/// Additionally, it tracks the synthesis start time, last update timestamp, an adaptive limit parameter, and a filtering structure (tree hole) used during example set generation and thread interruption.
pub struct Solutions {
    cfg: Cfg,
    ctx: Context,
    solutions: Vec<(&'static Expr, Bits)>,
    solved_examples: Bits,
    pub threads: MappedFutures<Vec<usize>, JoinHandle<Expression>>,
    start_time: Instant,
    last_update: Instant,
    ite_limit: usize,
    tree_hole: Vec<Box<[u128]>>,
}

impl Solutions {
    /// Creates a new instance with the provided configuration and context. 
    /// This function initializes the internal condition tracker based on the context, ensuring that no previous tracker is present, and then sets up all the initial fields required for solution management and concurrent search execution, including a default tree hole, empty solution set, and mapped futures for thread management.
    pub fn new(cfg: Cfg, ctx: Context) -> Self {
        {
            let mut lock = CONDITIONS.lock();
            assert!(lock.is_none());
            *lock = Some(ConditionTracker::new(ctx.clone()));
        }
        let solutions = Vec::new();
        let solved_examples = Bits::zeros(ctx.len);
        Self { 
            tree_hole: vec![Bits::ones(ctx.len)],
            cfg, ctx, solutions, solved_examples, threads: MappedFutures::new(), start_time: time::Instant::now(), last_update: time::Instant::now(), ite_limit: 1}
    }
    /// Counts the number of stored synthesis solutions.
    /// 
    /// Returns the total count of solution entries currently maintained within the internal collection.
    pub fn count(&self) -> usize {
        self.solutions.len()
    }

    /// Adds a new candidate solution by evaluating an expression and updating the internal solution set accordingly. 
    /// 
    /// The method first attempts to derive an evaluation result from the provided expression and then checks if this new result is subsumed by any existing solution; if so, it immediately returns without modification. 
    /// Otherwise, it filters out any previously stored solutions that are redundant relative to the new one, updates the union of solved examples, and adds the new solution.
    /// 
    /// Continues by assessing whether the inclusive solved example set now covers all required cases, returning the expression if complete. 
    /// In parallel, it iterates over the currently scheduled threads, aborting any whose example sets are fully encompassed by the new evaluation and triggering the launch of new threads. 
    /// Finally, it leverages auxiliary mechanisms to generate a final solution if possible, or returns None if the candidate fails to yield a valid update.
    pub fn add_new_solution(&mut self, expr: &'static Expr) -> Option<&'static Expr> {
        if let Some(b) = self.ctx.evaluate(expr) {
            // Updating solutions
            for (_, bits) in self.solutions.iter() {
                if b.subset(bits) {
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
    /// Determines and generates a synthesized result based on solved examples. 
    /// 
    /// 
    /// Checks whether the complete set of examples has been addressed; if so, it invokes a tree-learning procedure using a configurable operator limit depending on the provided flag and returns the synthesized solution expression. 
    /// Otherwise, it returns none.
    pub fn generate_result(&self, limit: bool) -> Option<&'static Expr> {
        if self.solved_examples.count_ones() == self.ctx.len as u32 {
            self.learn_tree(if limit { self.cfg.config.ite_limit_rate } else { 1 })
        } else { None }
    }
    /// Learns a decision tree that synthesizes an expression using the current set of solutions and conditions, dynamically adjusting the iteration limit based on elapsed time and a provided rate parameter.
    /// 
    /// Computes an adaptive limit derived from the runtime duration and toggles a global condition tracker before invoking a tree learning procedure. 
    /// Returns an expression reference if the tree learning process determines that a complete solution has been found, otherwise yields None.
    pub fn learn_tree(&self, ite_limit_rate: usize) -> Option<&'static Expr> {
        let duration = time::Instant::now() - self.start_time;
        let ite_limit = if duration.as_secs() as usize >= self.cfg.config.ite_limit_giveup {
            self.ite_limit + (duration.as_millis() as usize - self.cfg.config.ite_limit_giveup * 1000) * 5 / ite_limit_rate + 1
        } else { self.ite_limit };
        
        let mut lock = CONDITIONS.lock();
        let conditions = lock.as_mut().unwrap();
        if conditions.len() == 0 {
            return None;
        }
        debg!("Tree Learning Conditions: {}, Limit: {}", conditions.len(), ite_limit);
        let bump = bumpalo::Bump::new();
        let result = tree_learning(self.solutions.clone(), &conditions.vec[..], self.ctx.len, &bump, ite_limit);
        if result.solved {
            Some(result.expr())
        } else {
            None
        }
    }
    /// Checks whether any stored solution in the current context fully covers the specified example set. 
    /// 
    /// This function iterates over all solutions and verifies if all indices in the provided example set are included in the corresponding coverage bitmask of any solution. 
    /// It returns true as soon as a matching solution is found, and false otherwise.
    /// 
    pub fn check_cover(&self, example_set: &[usize]) -> bool {
        for (_, bits) in self.solutions.iter() {
            if example_set.iter().all(|i| bits.get(*i)) {
                return true;
            }
        }
        false
    }
    /// Generates a new set of example indices for initiating a synthesis thread. 
    /// This method iterates over potential subset sizes, calculating binomial coefficients to limit enumeration, and constructs candidate subsets based on configured conditions—either filtering through a predefined mask or generating all possible combinations.
    /// 
    /// It randomizes the order of these candidate subsets and validates each by ensuring that the example set is neither already covered by existing solutions nor in use by running threads. 
    /// When a valid subset is found, it returns the set; otherwise, it yields None if no appropriate example set can be generated.
    pub fn generate_example_set(&mut self) -> Option<Vec<usize>> {
        let mut rng = rand::thread_rng();
        for k in 1..=self.ctx.len {
            if bicoeff(self.ctx.len, k) > 4000000 { break; }

            let mut vec = Vec::new();
            if self.cfg.config.tree_hole {
                for hole in self.tree_hole.iter() {
                    vec.extend((0..self.ctx.len).filter(|i| hole.get(*i)).combinations(k));
                } 
            } else {
                vec.extend((0..self.ctx.len).combinations(k).collect_vec());
            }
            
            vec.shuffle(&mut rng);
            for v in vec {
                if !self.check_cover(&v) && !self.threads.contains(&v) { return Some(v); }
            }
        }
        None
    }
    /// Updates the tree hole configuration for the current synthesis process while ensuring that threads no longer covered by the new configuration are aborted and replaced. 
    /// This method assigns the new tree hole, iterates through the active thread example sets, verifies each against the updated tree hole using a helper function, and for any that fail the condition, it aborts the corresponding thread and promptly creates a replacement thread to preserve continuous progress in the synthesis search.
    pub fn update_tree_hole(&mut self, tree_hole: Vec<Box<[u128]>>) {
        self.tree_hole = tree_hole;
        let keys = self.threads.keys().cloned().collect_vec();
        for k in keys {
            if !test_tree_hole_contains(&self.tree_hole, &k) {
                if let Some(a) = self.threads.remove(&k) {
                    a.abort();
                    info!("Interupting Thread of {k:?}");
                    self.create_new_thread();
                }
            }
        }
    }
    /// Creates a new asynchronous thread to perform synthesis search using a generated example set. 
    /// This function attempts to generate a candidate example set and, if successful, constructs a new context augmented with these examples to spawn an additional thread executing the synthesis process; otherwise, it logs that no example set is available.
    pub fn create_new_thread(&mut self) {
        if let Some(exs) = self.generate_example_set() { 
            info!("Creating new thread with examples {:?}", exs);
            let ctx2 = self.ctx.with_examples(&exs);
            self.threads.insert(exs, new_thread(self.cfg.clone(), ctx2));
        } else {
            info!("No available example set");
        }
    }
    /// Creates and registers an asynchronous thread that performs exhaustive search over all examples from the current context. 
    /// 
    /// This function gathers every example index by iterating from 0 to the context's length, clones the current configuration and context, and then spawns a new search thread using those values. 
    /// The resulting thread is inserted into the solutions' thread registry, initiating a comprehensive condition search for viable synthesis solutions.
    /// 
    pub fn create_all_search_thread(&mut self) {
        // info!("Creating condition search thread.");
        // cfg.config.cond_search = true;
        self.threads.insert((0..self.ctx.len).collect_vec(), new_thread(self.cfg.clone(), self.ctx.clone()));
    }
    /// Continuously polls and adapts the synthesis process until a valid expression covering all examples is discovered. 
    /// 
    /// This asynchronous loop concurrently listens for solutions generated by worker threads and performs periodic adaptive adjustments. 
    /// It evaluates incoming candidate expressions, updates and manages the set of current solutions, and dynamically modifies search parameters using time-based adjustments. 
    /// When a complete solution is identified, it aborts remaining threads and returns the synthesized expression.
    /// 
    pub async fn solve_loop(&mut self) -> &'static Expr {
        loop {
            select! {
                result = self.threads.next() => {
                    let (k,v) = result.unwrap();
                    let v = v.expect("Thread Execution Error").alloc_local();
                    info!("Found a solution {:?} with examples {:?}.", v, k);
                    self.last_update = time::Instant::now();
                    if let Some(e) = self.add_new_solution(v) {
                        for v in self.threads.iter() { v.abort(); }
                        return e;
                    }
                    self.create_new_thread();
                }
                _ = tokio::time::sleep(Duration::from_millis(std::cmp::min(self.cfg.config.ite_limit_rate as u64, 2000))) => {
                    if time::Instant::now() - self.last_update > Duration::from_millis(self.cfg.config.ite_limit_rate as u64 - 10) {
                        info!("Adaptive Adjustment of ITE Limit: {}", self.ite_limit);
                        self.ite_limit += 1;
                        self.last_update = time::Instant::now();
                    }
                    if let Some(e) = self.generate_result(!self.threads.is_empty()) {
                        for v in self.threads.iter() { v.abort(); }
                        return e;
                    }
                }
            }
        }
    }
}

/// Creates a new asynchronous task that executes a synthesis search using the provided configuration and evaluation context.
/// 
/// Spawns a task that initializes a solver executor with the given parameters, logs the deduction configuration, performs a top-blocked search for an expression, and then converts and returns it as the asynchronous task's result.
pub fn new_thread(cfg: Cfg, ctx: Context) -> JoinHandle<Expression> {
    tokio::spawn(async move {
        let exec = Executor::new(ctx, cfg);
        info!("Deduction Configuration: {:?}", exec.deducers);
        
        exec.solve_top_blocked().to_expression()
    })
}

/// Enables a condition search thread by modifying the configuration and initiating a new asynchronous synthesis search. 
/// This function activates condition search mode by setting the corresponding flag in the configuration, then delegates thread creation to a helper that starts the synthesis process, ultimately returning a join handle for the resulting expression.
pub fn cond_search_thread(mut cfg: Cfg, ctx: Context) -> JoinHandle<Expression> {
    cfg.config.cond_search = true;
    new_thread(cfg, ctx)
}

/// Spawns an asynchronous task that executes a limited search procedure and returns its corresponding expression.
/// 
/// Initiates an executor using the provided configuration and context, then attempts to solve the top-level problem with a limit. 
/// If the search produces a solution, the resulting expression is returned; otherwise, the process is aborted. 
/// The asynchronous execution is managed through the Tokio runtime and the result is encapsulated within a join handle.
pub fn new_thread_with_limit(cfg: Cfg, ctx: Context) -> JoinHandle<Expression> {
    tokio::spawn(async move {
        if let Some(p) = {
            
            Executor::new(ctx, cfg).solve_top_with_limit().map(|e| e.to_expression())
        } {
            p
        } else { never!() }
    })
}