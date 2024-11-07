use std::{array, cell::{RefCell, UnsafeCell}, cmp::{max, min}, collections::{HashMap, HashSet}, pin::{pin, Pin}, rc::Rc, sync::Arc, task::{Poll, Waker}};

use bumpalo::collections::CollectIn;
use figment::util::diff_paths;
use futures::{future::select, FutureExt};
use futures_core::Future;
use itertools::Itertools;
use smol::Task;
use spin::Mutex;
// use text::formatting::Op1EnumToFormattingOp
use crate::{async_closure, closure, debg, expr::{ context::Context, ops::Op1Enum, Expr}, forward::{data::{len, prefix, substr}, executor::Enumerator}, info, text::formatting::Op1EnumToFormattingOp, DEBUG};
use crate::{galloc::{self, AllocForAny, AllocForExactSizeIter, AllocForIter}, never, utils::{fut::{pending_if, select_all, select_ret, select_ret3, select_ret4, select_ret5}, UnsafeCellExt}, value::Value};

use crate::expr;
use super::{Deducer, Problem};

pub struct HandleRcVec<T: Unpin>(Arc<Mutex<Vec<Task<T>>>>);

impl<T: Unpin> Drop for HandleRcVec<T> {
    fn drop(&mut self) {
        self.cancel()
    }
}

impl<T: Unpin> Clone for HandleRcVec<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Unpin> Future for HandleRcVec<T> {
    type Output=T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut result = Poll::Pending;
        for v in self.0.lock().iter_mut() {
            if let Poll::Ready(a) = v.poll_unpin(cx) {
                result = Poll::Ready(a);
                break;
            }
        }
        result
    }
}

impl<T: Unpin> HandleRcVec<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }
    pub fn extend_iter(&self, v: impl Iterator<Item=Task<T>>) {
        for f in v {
            self.0.lock().push(f);
        }
    }
    pub fn len(&self) -> usize {
        self.0.lock().len()
    }
    pub fn cancel(&mut self) -> () {
        let v = std::mem::replace(&mut *self.0.lock(), Vec::new());
        for p in v {
            drop(p);
        }
    }
}

#[derive(Debug)]
pub struct StrDeducer {
    pub nt: usize,
    pub split_once: (usize, usize),
    pub join: (usize, usize),
    pub ite_concat: (usize, usize),
    pub formatter: Vec<(Op1Enum, usize)>,
    pub decay_rate: usize,
}

impl StrDeducer {
    pub fn new(nt: usize) -> Self {
        Self { nt, split_once: (usize::MAX, 0), join: (usize::MAX, 0), ite_concat: (usize::MAX, usize::MAX), formatter: Vec::new(), decay_rate: usize::MAX }
    }
}

impl Deducer for StrDeducer {
    async fn deduce(&'static self, exec: Arc<Enumerator>, prob: Problem) -> &'static crate::expr::Expr {
        assert!(self.nt == prob.nt);
        let this = self;
        let mut eq = pin!(exec.data[self.nt].all_eq.acquire(prob.value));
        debg!("{exec:?} Deducing subproblem: {} {:?}", self.nt, prob.value);
        let [exec2, exec3, exec4, exec5] = array::from_fn(|_| exec.clone());
        if let Poll::Ready(r) = futures::poll!(&mut eq) { return r; }

        // let mut delimiterset = HashSet::<Vec<&'static str>>::new();
        let futures = HandleRcVec::new();

        let substr_event = substr::Data::listen_for_each(exec2.data[self.nt].substr.as_ref().unwrap(), prob.value, closure! { clone futures, clone prob; move |delimiter: Value| {
            // let vec = delimiter.to_str().iter().zip(prob.value.to_str().iter()).map(|(a, b)| if b.contains(a) { *a } else { &"" }).collect_vec();
            // if delimiterset.insert(vec) {
                futures.extend_iter(this.split1(exec4.clone(), prob, delimiter).into_iter());
                futures.extend_iter(this.join(exec4.clone(), prob, delimiter).into_iter());
            // }
            None::<&'static Expr>
        }});
        
        // let mut prefixset = HashSet::<Vec<&'static str>>::new();
        let prefix_event = prefix::Data::listen_for_each(exec2.data[self.nt].prefix.as_ref().unwrap(), prob.value, closure! { clone futures, clone prob; move |prefix: Value| {
            // let vec = prefix.to_str().iter().zip(prob.value.to_str().iter()).map(|(a, b)| if b.starts_with(a) { *a } else { &"" }).collect_vec();
            // if prefixset.insert(vec) {
                futures.extend_iter(this.ite_concat(exec5.clone(), prob, prefix).into_iter());
            // }
            None::<&'static Expr>
        }});
        let join_empty_str_cond = self.join.0 < usize::MAX && prob.used_cost <= 8 &&
            prob.value.to_str().iter().all(|x| x.chars().all(|c| c.is_alphanumeric())) &&
            prob.value.to_str().iter().any(|x| x.len() > 2);
            
        let map_event = pin!(closure! {clone futures; async move {
            if join_empty_str_cond {
                let _v = len::Data::listen_once(exec3.clone().data[self.join.1].len.as_ref().unwrap(), prob.value).await;
                futures.extend_iter(this.join_empty_str(exec3, prob).into_iter());
            } 
            never!(&'static Expr)
        }});
        let iter = self.formatter.iter().map(|x| self.fmt(prob.clone(), x.clone(), exec.clone()));

        let substr_event = pin!(substr_event);
        let prefix_event = pin!(prefix_event);
        let events = select_ret3(prefix_event, substr_event, map_event);

        let result = select_ret4(eq, events, futures, pin!(select_all(iter))).await;
        result
    }
}



impl StrDeducer {
    // #[inline]
    // pub fn split1(&'static self, id: usize, prob: Problem, delimiter: &'static Expr, dvalue: Value) -> Option<JoinHandle<&'static Expr>> {
    //     let v: &[&str] = prob.value.try_into().unwrap();
    //     let dvalue: &[&str] = dvalue.try_into().unwrap();
    //     let contain_count: usize = v.iter().zip(dvalue.iter()).filter(|(x, y)| if **y != "" { x.contains(*y) } else { false }).count();


    //     if contain_count >= self.split_once.0 {
    //         Some(self.split1_task(id, subprob, delimiter, dvalue, contain_count))
    //     } else { None }
    // }
    // #[inline]
    // fn decay(&self, i: usize) -> usize {
    //     let rate = self.decay_rate;
    //     let task = rate + min(100 * rate, task::number_of_tasks());
    //     if i != usize::MAX { (i * task) / rate } else { i }
    // }
    // #[inline]
    // fn decay2(&self, i: usize) -> usize {
    //     let rate = self.decay_rate;
    //     let task = rate + min(100 * rate, 20 * task::number_of_tasks());
    //     if i != usize::MAX { (i * task) / rate } else { i }
    // }
    // #[inline]
    // fn split_once_count(&self, exec: &'static Executor) -> usize {
    //     min(exec.ctx.len(), self.decay(self.split_once.0))
    // }
    // #[inline]
    // fn ite_concat_count(&self, exec: &'static Executor) -> usize {
    //     self.decay(self.ite_concat.0)
    // }
    // #[inline]
    // fn ite_concat_eq_count(&self, exec: &'static Executor) -> usize {
    //     self.decay2(3)
    // }
    #[inline]
    fn split1(&'static self, exec: Arc<Enumerator>, mut prob: Problem, delimiter: Value) -> Option<Task<&'static Expr>> {
        let delimiter = delimiter.to_str();
        let v = prob.value.to_str();
        // let contain_count: usize = v.iter().zip(delimiter.iter()).filter(|(x, y)| if **y != "" { x.contains(*y) } else { false }).count();
        // if !(contain_count >= self.split_once_count(exec) && prob.used_cost < 15) { return None; }

        
        Some(smol::spawn(async move {
            let (a, b, cases) = split_once(v, delimiter);
            if !cases.is_all_true() && self.ite_concat.1 == usize::MAX { return never!() }
            exec.waiting_tasks.inc_cost(&mut prob, 1).await;

            debg!("{exec:?} StrDeducer::split1 {v:?} {delimiter:?}");

            let left = exec.clone().solve_task(prob.with_value(a)).await;
            let right = exec.clone().solve_task(prob.with_value(b)).await;
            
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
    pub async fn generate_condition(&'static self, exec: Arc<Enumerator>, prob: Problem, result: &'static Expr) -> &'static Expr {
        if prob.value.is_all_true() { return result; }
        let left = pin!(exec.clone().solve_task(prob.clone()));
        let right = pin!(exec.clone().solve_task(prob.clone().with_value(prob.value.bool_not())));
        let cond = futures::future::select(left, right).await;
        match cond {
            futures::future::Either::Left((c, _)) => 
                expr!(Ite {c} {result} "").galloc(),
            futures::future::Either::Right((c, _)) => 
                expr!(Ite {c} "" {result}).galloc(),
        }
    }
    #[inline]
    pub fn ite_concat(&'static self, exec: Arc<Enumerator>, mut prob: Problem, prefix: Value) -> Option<Task<&'static Expr>> {
        let v: &[&str] = prob.value.to_str();
        let prefix: &[&str] = prefix.to_str();
        let start_count: usize = v.iter().zip(prefix.iter()).map(|(x, y)| if x.starts_with(*y) { y.len() } else { 0 }).sum();
        let eq_count: usize = v.iter().zip(prefix.iter()).map(|(x, y)| if x == y { y.len() } else { 0 }).sum();

        // if !(start_count >= self.ite_concat_count(exec) || eq_count >= self.ite_concat_eq_count(exec)) { return None; }

        
        Some(smol::spawn(async move {
            debg!("{exec:?} StrDeducer::ite_concat {} {:?} {:?} {start_count} {eq_count}", prob.nt, v, prefix);
            let (a, b) = ite_concat_split(v, prefix);
            
            exec.waiting_tasks.inc_cost(&mut prob, 1).await;

            let right = exec.clone().solve_task(prob.with_value(b)).await;
            
            let mut result = exec.clone().data[prob.nt].all_eq.get(prefix.into());
            result = self.generate_condition(exec.clone(), prob.with_nt(self.ite_concat.1, a), result).await;
            if !b.is_all_empty() {
                result = expr!(Concat {result} {right}).galloc();
            }
            result
        }))
    }

    #[inline]
    fn join(&'static self, exec: Arc<Enumerator>, mut prob: Problem, delimiter: Value) -> Option<Task<&'static Expr>> {
        if prob.used_cost >= 6 { return None; }

        let delimiter = delimiter.to_str();
        let v = prob.value.to_str();
        let contain_count: usize = v.iter().zip(delimiter.iter()).map(|(x, y)| x.matches(y).count() + 1).max().unwrap_or(10000);
        if contain_count < self.join.0 { return None; }

        
        Some(smol::spawn(async move {
            exec.waiting_tasks.inc_cost(&mut prob, 1).await;

            let a = value_split(v, delimiter);
            debg!("{exec:?} StrDeducer::join {v:?} {delimiter:?}");

            let list = exec.clone().solve_task(prob.with_nt(self.join.1, a)).await;
            
            let mut delim = exec.data[prob.nt].all_eq.get(delimiter.into());
            expr!(Join {list} {delim}).galloc()
        }))
    }
    #[inline]
    fn join_empty_str(&'static self, exec: Arc<Enumerator>, mut prob: Problem) -> Option<Task<&'static Expr>> {
        debg!("{exec:?} StrDeducer::join_empty_str {:?}", prob.value);

        Some(smol::spawn(async move {
            exec.waiting_tasks.inc_cost(&mut prob, 1).await;
            let v = prob.value.to_str();
            let li = v.into_iter().map(|x| (0..x.len()).map(|i| &x[i..i+1]).galloc_scollect() ).galloc_scollect();
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
    fn fmt(&'static self, mut problem: Problem, formatter: (Op1Enum, usize), exec: Arc<Enumerator>) -> Task<&'static Expr> {
        let v = problem.value.to_str();
        // debg!("StrDeducer::fmt1 {v:?} {formatter:?}");
        smol::spawn(async move {
            if let Some((op, a, b, cond)) = formatter.0.format_all(v) {
                debg!("StrDeducer::fmt {v:?} {formatter:?} {a:?}");
                // if !cond.is_all_true() { exec.waiting_tasks.inc_cost(&mut problem, 1).await; }
                // else { exec.waiting_tasks.inc_cost(&mut problem, 1).await; }
                
                let inner = exec.clone().solve_task(problem.with_nt(formatter.1, a)).await;
                
                debg!("StrDeducer::fmt1 {v:?} {formatter:?} {b:?}");
                let rest = exec.clone().solve_task(problem.with_nt(self.nt, b)).await;
                
                debg!("StrDeducer::fmt2 {v:?} {formatter:?}");

                let mut result = Expr::Op1(op.clone().galloc(), inner).galloc();
                if self.ite_concat.1 != usize::MAX {
                    result = exec.clone().generate_condition(problem.with_nt(self.ite_concat.1, cond), result).await;
                }
                result = expr!(Concat {result} {rest}).galloc();
                if DEBUG.get() {
                    assert_eq!(result.eval(&exec.ctx), Value::Str(v), "Expression: {:?} {:?}", result, a);
                }
                result
            } else { never!() }
        })
    }
}

pub fn split_once(s: &'static [&'static str], delimiter: &'static [&'static str]) -> (Value, Value, Value) {
    assert!(s.len() == delimiter.len());
    let mut a = galloc::new_bvec(s.len());
    let mut b = galloc::new_bvec(s.len());
    let mut cases = galloc::new_bvec(s.len());
    for (x, y) in s.iter().zip(delimiter.iter()) {
        if y.len() == 0 {
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

pub fn value_split(s: &'static [&'static str], delimiter: &'static [&'static str]) -> Value {
    Value::ListStr(s.iter().zip(delimiter.iter()).map(|(x, y)| x.split(y).galloc_collect()).galloc_collect())
}