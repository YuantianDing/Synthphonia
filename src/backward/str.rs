use std::{cell::{RefCell, UnsafeCell}, cmp::{max, min}, collections::{HashMap, HashSet}, pin::{pin, Pin}, rc::Rc, sync::Arc, task::{Poll, Waker}};

use bumpalo::collections::CollectIn;
use figment::util::diff_paths;
use futures::{future::select, FutureExt};
use futures_core::Future;
use itertools::Itertools;
use simple_rc_async::task::{self, JoinHandle};

use crate::{async_closure, closure, debg, expr::{ context::Context, ops::Op1Enum, Expr}, forward::executor::Executor, info, utils::select_ret5, value::Type, DEBUG};
use crate::{galloc::{self, AllocForAny, AllocForExactSizeIter, AllocForIter}, never, utils::{pending_if, select_all, select_ret, select_ret3, select_ret4, UnsafeCellExt}, value::Value};

use crate::expr;
use super::{Deducer, Problem};

/// A container for managing a collection of asynchronous task join handles. 
/// 
/// 
/// This structure encapsulates a reference-counted, mutable vector of asynchronous task handles, enabling concurrent polling of multiple tasks. 
/// Its design allows shared ownership and in-place mutation of the task collection without requiring external synchronization, supporting operations that extend or poll the set of join handles as part of asynchronous execution workflows.
pub struct HandleRcVec<T: Unpin>(Arc<UnsafeCell<Vec<JoinHandle<T>>>>);

impl<T: Unpin> Clone for HandleRcVec<T> {
    /// Clones the join handle collection by duplicating its internal shared pointer.
    /// 
    /// This method produces a new instance that shares the same underlying join handle storage, enabling multiple owners of the asynchronous task collection without copying the actual handles.
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Unpin> Future for HandleRcVec<T> {
    type Output=T;

    /// Polls a collection of asynchronous tasks and returns the output of the first task that is complete. 
    /// 
    /// This method iterates over the stored join handles and evaluates each one to determine if its associated task has finished, returning the completed result if available. 
    /// If none of the tasks are ready, it returns a pending status.
    /// 
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        for v in unsafe{ self.0.as_mut()}.iter_mut() {
            if let Poll::Ready(a) = v.poll_unpin(cx) {
                return Poll::Ready(a);
            }
        }
        Poll::Pending
    }
}

impl<T: Unpin> Default for HandleRcVec<T> {
    /// Returns a new default instance by invoking the primary constructor.
    /// 
    /// This method serves as a convenience alias for creating a default instance, allowing integration with traits or patterns that rely on default initialization. 
    /// It encapsulates the functionality of the constructor without exposing implementation details.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Unpin> HandleRcVec<T> {
    /// Creates and returns a new instance of the container for asynchronous task handles.
    /// 
    /// Constructs an empty container by encapsulating an empty vector within an Arc and UnsafeCell, thereby enabling shared mutable access suitable for concurrent asynchronous operations.
    pub fn new() -> Self {
        Self(Arc::new(UnsafeCell::new(Vec::new())))
    }
    /// Extends the internal collection with join handles obtained from the provided iterator. 
    /// This method iterates over each join handle from the iterator and appends it into the underlying storage, making the join handles available for future asynchronous polling.
    pub fn extend_iter(&self, v: impl Iterator<Item=JoinHandle<T>>) {
        for f in v {
            unsafe{self.0.as_mut()}.push(f);
        }
    }
    /// Returns the number of join handles stored in the underlying collection. 
    /// This method accesses the internal vector unsafely to determine its length, providing a simple way to query how many tasks have been added.
    pub fn len(&self) -> usize {
        unsafe { self.0.as_mut().len() }
    }
}

#[derive(Debug)]
/// A struct hold configuration for string deduction tasks.
pub struct StrDeducer {
    /// The non-terminal identifier of this deducer.
    pub nt: usize,
    /// (No longer used, non-terminal to split to)
    pub split_once: (usize, usize),
    /// (No longer used, non-terminal to split to)
    pub join: (usize, usize),
    /// (No longer used, non-terminal to split to)
    pub ite_concat: (usize, usize),
    pub index: (usize, usize),
    /// Formatting operations to be applied during deduction, (operator, non-terminal to format to).
    pub formatter: Vec<(Op1Enum, usize)>,
    /// No longer used
    pub decay_rate: usize,
}

impl StrDeducer {
    /// Creates a new instance of the associated type with a specified non-terminal identifier, using the default setting. 
    pub fn new(nt: usize) -> Self {
        Self { nt, split_once: (usize::MAX, 0), join: (usize::MAX, 0), ite_concat: (usize::MAX, usize::MAX), index: (usize::MAX, usize::MAX), formatter: Vec::new(), decay_rate: usize::MAX }
    }
}

impl Deducer for StrDeducer {
    /// Deducing string synthesis problems. 
    async fn deduce(&'static self, exec: &'static Executor, prob: Problem) -> &'static crate::expr::Expr {
        assert!(self.nt == prob.nt);
        assert!(prob.value.ty() == Type::Str, "Expected a string value, got: {:?}", prob.value);
        let this = self;
        let mut eq = pin!(exec.data[self.nt].all_eq.acquire(prob.value));
        debg!("Deducing subproblem: {} {:?}", self.nt, prob.value);
        if let Poll::Ready(r) = futures::poll!(&mut eq) { return r; }

        // let mut delimiterset = HashSet::<Vec<&'static str>>::new();
        let futures = HandleRcVec::new();

        let substr_event = closure! { clone futures, clone prob; async move {
            if exec.data[self.nt].substr().is_some() {
                exec.data[self.nt].substr().unwrap().listen_for_each(prob.value, closure! { clone futures, clone prob; move |delimiter: Value| {
                    futures.extend_iter(this.split1(exec, prob, delimiter).into_iter());
                    futures.extend_iter(this.join(exec, prob, delimiter).into_iter());
                    None::<&'static Expr>
                }}).await
            } else { never!(&'static Expr) }
        }};
        
        let prefix_event = closure! { clone futures, clone prob; async move {
            if exec.data[self.nt].prefix().is_some() {
                exec.data[self.nt].prefix().unwrap().listen_for_each(prob.value, move |prefix: Value| {
                    futures.extend_iter(this.ite_concat(exec, prob, prefix).into_iter());
                    None::<&'static Expr>
                }).await
            } else { never!(&'static Expr) }
        }};

        let index_event = closure! { clone futures, clone prob; async move {
            if self.index.0 != usize::MAX && prob.used_cost < 3 && exec.data[self.index.0].contains.is_some() {
                exec.data[self.index.0].contains.as_ref().unwrap().listen_for_each(prob.value, move |list: Value| {
                    futures.extend_iter(this.index(exec, prob, list).into_iter());
                    None::<&'static Expr>
                }).await
            } else { never!(&'static Expr) }
        }};

        let join_empty_str_cond = self.join.0 < usize::MAX && prob.used_cost <= 8 &&
            prob.value.to_str().iter().all(|x| x.chars().all(|c| c.is_alphanumeric())) &&
            prob.value.to_str().iter().any(|x| x.len() > 2);
            
        let map_event = pin!(closure! {clone futures; async move {
            if join_empty_str_cond {
                let v = exec.data[self.join.1].len().unwrap().listen_once(prob.value).await;
                futures.extend_iter(this.join_empty_str(exec, prob).into_iter());
            } 
            never!(&'static Expr)
        }});
        let iter = self.formatter.iter().map(|x| self.fmt(prob, x, exec));

        let substr_event = pin!(substr_event);
        let prefix_event = pin!(prefix_event);
        let index_event = pin!(index_event);
        let events = select_ret4(prefix_event, substr_event, map_event, index_event);

        let result = select_ret4(eq, events, futures, pin!(select_all(iter))).await;
        result
    }
}



impl StrDeducer {
    
    #[inline]
    /// Deduce a string splitting by a specified delimiter. 
     fn split1(&'static self, exec: &'static Executor, mut prob: Problem, delimiter: Value) -> Option<JoinHandle<&'static Expr>> {
        let delimiter = delimiter.to_str();
        let v = prob.value.to_str();
        let contain_count: usize = v.iter().zip(delimiter.iter()).filter(|(x, y)| if !y.is_empty() { x.contains(*y) } else { false }).count();
        // if !(contain_count >= self.split_once_count(exec) && prob.used_cost < 15) { return None; }

        
        Some(task::spawn(async move {
            let (a, b, cases) = split_once(v, delimiter);
            if !cases.is_all_true() || self.ite_concat.1 == usize::MAX { return never!() }
            exec.waiting_tasks().inc_cost(&mut prob, 1).await;

            debg!("StrDeducer::split1 {v:?} {delimiter:?}");

            let left = exec.solve_task(prob.with_value(a)).await;
            let right = exec.solve_task(prob.with_value(b)).await;
            
            let mut result = exec.data[prob.nt].all_eq.get(delimiter.into());
            if self.ite_concat.1 != usize::MAX {
                result = self.generate_condition(exec, prob.with_nt(self.ite_concat.1, cases), result).await;
            }
            if !a.is_all_empty() {
                result = expr!(Concat {left} {result}).galloc();
            }
            if !b.is_all_empty() {
                return expr!(Concat {result} {right}).galloc();
            }
            result
        }))
    }
    #[inline]
    /// Generates a conditional expression 
    pub async fn generate_condition(&'static self, exec: &'static Executor, prob: Problem, result: &'static Expr) -> &'static Expr {
        if prob.value.is_all_true() { return result; }
        let left = pin!(exec.solve_task(prob));
        let right = pin!(exec.solve_task(prob.with_value(prob.value.bool_not())));
        let cond = futures::future::select(left, right).await;
        match cond {
            futures::future::Either::Left((c, _)) => 
                expr!(Ite {c} {result} "").galloc(),
            futures::future::Either::Right((c, _)) => 
                expr!(Ite {c} "" {result}).galloc(),
        }
    }
    #[inline]
    /// Deduce conditional concatenation deduction for string synthesis problems. 
    pub fn ite_concat(&'static self, exec: &'static Executor, mut prob: Problem, prefix: Value) -> Option<JoinHandle<&'static Expr>> {
        let v: &[&str] = prob.value.to_str();
        let prefix: &[&str] = prefix.to_str();
        let start_count: usize = v.iter().zip(prefix.iter()).map(|(x, y)| if x.starts_with(*y) { y.len() } else { 0 }).sum();
        let eq_count: usize = v.iter().zip(prefix.iter()).map(|(x, y)| if x == y { y.len() } else { 0 }).sum();

        // if !(start_count >= self.ite_concat_count(exec) || eq_count >= self.ite_concat_eq_count(exec)) { return None; }

        
        Some(task::spawn(async move {
            debg!("StrDeducer::ite_concat {} {:?} {:?} {start_count} {eq_count}", prob.nt, v, prefix);
            let (a, b) = ite_concat_split(v, prefix);
            
            exec.waiting_tasks().inc_cost(&mut prob, 1).await;

            let right = exec.solve_task(prob.with_value(b)).await;
            
            let mut result = exec.data[prob.nt].all_eq.get(prefix.into());
            result = self.generate_condition(exec, prob.with_nt(self.ite_concat.1, a), result).await;
            if !b.is_all_empty() {
                result = expr!(Concat {result} {right}).galloc();
            }
            result
        }))
    }

    pub fn index(&'static self, exec: &'static Executor, mut prob: Problem, list: Value) -> Option<JoinHandle<&'static Expr>> {
        let v: &[&str] = prob.value.to_str();
        let list : &[&[&str]] = list.to_liststr();

        let indices = v.iter().zip(list.iter()).map(|(x, y)| {
            y.iter().position(|&z| z == *x).unwrap_or(y.len()) as i64
        }).galloc_scollect();
        if self.index.0 == usize::MAX { return None; }
        Some(task::spawn(async move {
            debg!("StrDeducer::index {} {:?} {:?} {:?} {} ", prob.nt, v, list, indices, self.index.1);
            // exec.waiting_tasks().inc_cost(&mut prob, 1).await;

            let indices = exec.data[self.index.1].all_eq.acquire(indices.into()).await;
            let mut result = exec.data[self.index.0].all_eq.get(list.into());
            expr!(At {result} {indices}).galloc()
        }))
    }

    #[inline]
    /// Deduce a string joining operation based on a specified delimiter. 
    fn join(&'static self, exec: &'static Executor, mut prob: Problem, delimiter: Value) -> Option<JoinHandle<&'static Expr>> {
        let delimiter = delimiter.to_str();
        let v = prob.value.to_str();
        if prob.used_cost >= 5 { return None; }
        
        let contain_count: usize = v.iter().zip(delimiter.iter()).map(|(x, y)| x.matches(y).count() + 1).max().unwrap_or(10000);
        if contain_count < self.join.0 { return None; }
        
        
        Some(task::spawn(async move {
            debg!("StrDeducer::join {v:?} {delimiter:?} {} {}", prob.used_cost, contain_count);
            // exec.waiting_tasks().inc_cost(&mut prob, 1).await;

            let a = value_split(v, delimiter);

            let list = exec.solve_task(prob.with_nt(self.join.1, a)).await;
            
            let mut delim = exec.data[prob.nt].all_eq.get(delimiter.into());
            expr!(Join {list} {delim}).galloc()
        }))
    }
    #[inline]
    /// Deduce to list of strings using join
    fn join_empty_str(&'static self, exec: &'static Executor, mut prob: Problem) -> Option<JoinHandle<&'static Expr>> {
        debg!("StrDeducer::join_empty_str {:?}", prob.value);

        Some(task::spawn(async move {
            exec.waiting_tasks().inc_cost(&mut prob, 1).await;
            let v = prob.value.to_str();
            let li = v.iter().map(|x| (0..x.len()).map(|i| &x[i..i+1]).galloc_scollect() ).galloc_scollect();
            let list = exec.solve_task(prob.with_nt(self.join.1, li.into())).await;
            expr!(Join {list} "").galloc()
        }))
    }
    // #[inline]
    // async fn join(&self, problem: SubProblem, delimiter: &'static [&'static str], exec: &'static Executor) -> &'static Expr {
    //     let v = problem.value.to_str();
    //     debg!("TASK#{} StrDeducer::join {v:?} {delimiter:?}", currect_task_id());
    //     let a = value_split(v, delimiter);
    //     let list = exec.solve_task(problem.with_nt(self.join.1, a)).await;
    //     let delim = exec.data[self.nt].all_eq.get(delimiter.into());
    //     expr!(Join {list} {delim}).galloc()
    // }
    #[inline]
    /// Deduce formats of strings using the provided formatter
    async fn fmt(&self, mut problem: Problem, formatter: &(Op1Enum, usize), exec: &'static Executor) -> &'static Expr {
        let v = problem.value.to_str();
        if let Some((op, a, b, cond)) = formatter.0.format_all(v) {
            debg!("StrDeducer::fmt {v:?} {formatter:?}");
            if !cond.is_all_true() { exec.waiting_tasks().inc_cost(&mut problem, 1).await; }
            else { exec.waiting_tasks().inc_cost(&mut problem, 1).await; }

            let inner = exec.solve_task(problem.with_nt(formatter.1, a)).await;
            let rest = exec.solve_task(problem.with_nt(self.nt, b)).await;
            
            let mut result = Expr::Op1(op.clone().galloc(), inner).galloc();
            if self.ite_concat.1 != usize::MAX {
                result = exec.generate_condition(problem.with_nt(self.ite_concat.1, cond), result).await;
            }
            result = expr!(Concat {result} {rest}).galloc();
            if DEBUG.get() {
                assert_eq!(result.eval(&exec.ctx), Value::Str(v), "Expression: {:?} {:?}", result, a);
            }
            result
        } else { never!() }
    }
}

/// Deduce splits for each string in the input slice once over the corresponding delimiter, resulting in two separate string parts and a boolean indicating successful splits. 
pub fn split_once(s: &'static [&'static str], delimiter: &'static [&'static str]) -> (Value, Value, Value) {
    assert!(s.len() == delimiter.len());
    let mut a = galloc::new_bvec(s.len());
    let mut b = galloc::new_bvec(s.len());
    let mut cases = galloc::new_bvec(s.len());
    for (x, y) in s.iter().zip(delimiter.iter()) {
        if y.is_empty() {
            a.push("");
            b.push(*x);
            cases.push(true)
        } else if let Some((l, r)) = x.split_once(*y) {
            a.push(l);
            b.push(r);
            cases.push(true)
        } else {
            a.push(x);
            b.push("");
            cases.push(false)
        }
    }
    (Value::Str(a.into_bump_slice()), Value::Str(b.into_bump_slice()), Value::Bool(cases.into_bump_slice()))
}

/// Performs conditional splitting of string slices based on whether each string starts with its corresponding delimiter. 
/// The function returns a tuple containing a boolean value and a string value.
/// 
/// Iterates over paired elements from the input slices, asserting equal lengths, and checks if each string begins with the associated delimiter. 
/// It collects a boolean flag indicating the match and, when a match is present, stores the string segment after the delimiter; otherwise, it retains the original string. 
/// The resulting vectors are converted into a boolean value and a string value, respectively.
pub fn ite_concat_split(s: &'static [&'static str], delimiter: &'static [&'static str]) -> (Value, Value) {
    assert!(s.len() == delimiter.len());
    let mut a = galloc::new_bvec(s.len());
    let mut b = galloc::new_bvec(s.len());
    for (x, y) in s.iter().zip(delimiter.iter()) {
        let v = x.starts_with(y);
        a.push(v);
        if v {
            b.push(&x[y.len()..])
        } else {
            b.push(x)
        }
    }
    (Value::Bool(a.into_bump_slice()), Value::Str(b.into_bump_slice()))
}

/// Splits each string in the provided slice using corresponding delimiters and returns the collection of split results as a list value wrapped in Value. 
/// 
/// 
/// Processes two slices of static string slices by pairing each input string with its associated delimiter, performing a split operation on the string, and then collecting each resultant iterator of substrings into a nested list structure conforming to the Value type.
pub fn value_split(s: &'static [&'static str], delimiter: &'static [&'static str]) -> Value {
    Value::ListStr(s.iter().zip(delimiter.iter()).map(|(x, y)| x.split(y).galloc_collect()).galloc_collect())
}