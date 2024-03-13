use std::{cell::{RefCell, UnsafeCell}, cmp::{max, min}, collections::{HashMap, HashSet}, pin::{pin, Pin}, rc::Rc, task::{Context, Poll, Waker}};

use bumpalo::collections::CollectIn;
use futures::{future::select, FutureExt};
use futures_core::Future;
use itertools::Itertools;

use crate::{async_closure, closure, debg, expr::{ ops::Op1Enum, Expr}, forward::{executor::Executor, future::{task::{self, currect_task_id, number_of_task}, taskrc::TaskORc}}, DEBUG};
use crate::{galloc::{self, AllocForAny, AllocForExactSizeIter, AllocForIter}, never, text::formatting::Op1EnumToFormattingOp, utils::{pending_if, select_all, select_ret, select_ret3, select_ret4, UnsafeCellExt}, value::Value};

use crate::expr;
use super::{Deducer, Problem};

struct FutureRcVec<F: Future<Output=T>, T>(Rc<UnsafeCell<Vec<F>>>);

impl<F: Future<Output=T>, T> Clone for FutureRcVec<F, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<F: Future<Output=T> + Unpin, T> Future for FutureRcVec<F, T> {
    type Output=T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        for v in unsafe{ self.0.as_mut()}.iter_mut() {
            if let Poll::Ready(a) = v.poll_unpin(cx) {
                return Poll::Ready(a);
            }
        }
        Poll::Pending
    }
}

impl<F: Future<Output=T>, T> FutureRcVec<F, T> {
    pub fn new() -> Self {
        Self(Rc::new(UnsafeCell::new(Vec::new())))
    }
    pub fn extend(&self, mut v: Vec<F>) {
        let _ = unsafe{self.0.as_mut()}.append(&mut v);
    }
    pub fn len(&self) -> usize {
        unsafe { self.0.as_mut().len() }
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
        Self { nt: 0, split_once: (usize::MAX, 0), join: (usize::MAX, 0), ite_concat: (usize::MAX, usize::MAX), formatter: Vec::new(), decay_rate: usize::MAX }
    }
}

impl Deducer for StrDeducer {
    async fn deduce(&'static self, exec: &'static Executor, mut problem: Problem) -> &'static crate::expr::Expr {
        let this = self;
        let (is_first, task) = exec.data[self.nt].all_eq.acquire_is_first(problem.value);
        debg!("TASK#{} Deducing subproblem: {} {:?} {}", currect_task_id(), exec.cfg[self.nt].name, problem.value, is_first);
        if !is_first {
            return task.await;
        }
        let problem2 = problem.clone();

        let mut delimiterset = HashSet::<Vec<&'static str>>::new();
        let futures = FutureRcVec::new();
        let inner_func = closure! { clone futures; move |delimiter: Value| {
            let vec = delimiter.to_str().iter().zip(problem.value.to_str().iter()).map(|(a, b)| if b.contains(a) { *a } else { &"" }).collect_vec();
            if delimiterset.insert(vec) {
                futures.extend(this.on_delim(exec, problem, delimiter));
            }
            None::<&'static Expr>
        }};
        let event = async {
            exec.waiting_tasks().inc_cost(&mut problem, 1).await;
            exec.data[self.nt].substr.try_at(problem.value,inner_func).await
        };
        let iter = self.formatter.iter().map(|x| self.fmt(problem2, x, exec));
        
        let result = select_ret4(task, pin!(event), futures, pin!(select_all(iter))).await;
        if DEBUG.get() {
            assert_eq!(result.eval(&exec.ctx), problem.value, "Expression: {:?}", result);
        }
        let _ = exec.data[self.nt].all_eq.set_ref(problem.value, result);
        result
    }
}



impl StrDeducer {
    #[inline]
    pub fn on_delim(&'static self, exec: &'static Executor, problem: Problem, delimiter: Value) -> Vec<TaskORc<&'static Expr>> {
        let v: &[&str] = problem.value.try_into().unwrap();
        let delimiter: &[&str] = delimiter.try_into().unwrap();
        let contain_count: usize = v.iter().zip(delimiter.iter()).filter(|(x, y)| if **y != "" { x.contains(*y) } else { false }).count();
        let start_count: usize = v.iter().zip(delimiter.iter()).map(|(x, y)| if x.starts_with(*y) { y.len() } else {0}).sum();
        let eq_count: usize = v.iter().zip(delimiter.iter()).map(|(x, y)| if x == y { y.len() } else {0}).sum();


        let mut result = Vec::with_capacity(2);
        if contain_count >= self.split_once_count(exec) && exec.data[self.nt].all_eq.get(delimiter.into()).cost() <= self.split_once.1 && currect_task_id() <= 3000 {
            result.push(self.split1(problem, delimiter, exec, contain_count));
        }
        if start_count >= self.ite_concat_count(exec) || eq_count >= self.ite_concat_eq_count(exec) && self.ite_concat.0 != usize::MAX {
            result.push(self.ite_concat(problem, delimiter, exec, start_count, eq_count));
        }
        result
    }
    #[inline]
    fn decay(&self, exec: &'static Executor, i: usize) -> usize {
        let rate = self.decay_rate;
        let task = rate + min(100 * rate, number_of_task());
        if i != usize::MAX { (i * task) / rate } else { i }
    }
    #[inline]
    fn decay2(&self, exec: &'static Executor, i: usize) -> usize {
        let rate = self.decay_rate;
        let task = rate + min(100 * rate, 20 * number_of_task());
        if i != usize::MAX { (i * task) / rate } else { i }
    }
    #[inline]
    fn split_once_count(&self, exec: &'static Executor) -> usize {
        min(exec.ctx.len(), self.decay(exec, self.split_once.0))
    }
    #[inline]
    fn ite_concat_count(&self, exec: &'static Executor) -> usize {
        self.decay(exec, self.ite_concat.0)
    }
    #[inline]
    fn ite_concat_eq_count(&self, exec: &'static Executor) -> usize {
        self.decay2(exec, 3)
    }
    #[inline]
    fn split1(&'static self, mut problem: Problem, delimiter: &'static [&'static str], exec: &'static Executor, count: usize) -> TaskORc<&'static Expr> {
        let v = problem.value.to_str();
        debg!("TASK#{}/{} StrDeducer::split1 {count} {v:?} {delimiter:?}", currect_task_id(), exec.problem_count());
        task::spawn( async move {
            let (a,b, cases) = split_once(v, delimiter);
            if !cases.is_all_true() && self.ite_concat.1 == usize::MAX { return never!() }
            if !cases.is_all_true() { exec.waiting_tasks().inc_cost(&mut problem, 1).await; }

            let left = exec.solve_task(problem.with_value(a)).await;
            let right = exec.solve_task(problem.with_value(b)).await;
            let delim = exec.data[self.nt].all_eq.get(delimiter.into());
            
            let mut result = delim;
            if self.ite_concat.1 != usize::MAX {
                result = exec.generate_condition(problem.with_nt(self.ite_concat.1, cases), result).await;
            }
            if !a.is_all_empty() {
                result = expr!(Concat {left} {result}).galloc();
            }
            if !b.is_all_empty() {
                return expr!(Concat {result} {right}).galloc();
            }
            result
        }).tasko()
    }
    #[inline]
    fn ite_concat(&'static self, mut problem: Problem, delimiter: &'static [&'static str], exec: &'static Executor, count: usize, eq_count: usize) -> TaskORc<&'static Expr> {
        let v = problem.value.to_str();
        debg!("TASK#{}/{} StrDeducer::ite_concat {count}/{} {eq_count}/{} {v:?} {delimiter:?}", currect_task_id(), exec.problem_count(), self.ite_concat_count(exec), self.ite_concat_eq_count(exec));
        task::spawn( async move {
            let (a, b) = ite_concat_split(v, delimiter);
            if !a.is_all_true() { exec.waiting_tasks().inc_cost(&mut problem, 2).await; }

            let right = exec.solve_task(problem.with_value(b)).await;
            let left = exec.data[self.nt].all_eq.get(delimiter.into());
            let mut result = left;
            result = exec.generate_condition(problem.with_nt(self.ite_concat.1, a), result).await;
            if !b.is_all_empty() {
                result = expr!(Concat {result} {right}).galloc();
            }
            result
        }).tasko()
    }
    #[inline]
    async fn join(&self, problem: Problem, delimiter: &'static [&'static str], exec: &'static Executor) -> &'static Expr {
        let v = problem.value.to_str();
        debg!("TASK#{} StrDeducer::join {v:?} {delimiter:?}", currect_task_id());
        let a = value_split(v, delimiter);
        let list = exec.solve_task(problem.with_nt(self.join.1, a)).await;
        let delim = exec.data[self.nt].all_eq.get(delimiter.into());
        expr!(Join {list} {delim}).galloc()
    }
    #[inline]
    async fn fmt(&self, mut problem: Problem, formatter: &(Op1Enum, usize), exec: &'static Executor) -> &'static Expr {
        let v = problem.value.to_str();
        if let Some((op, a, b, cond)) = formatter.0.format_all(v) {
            debg!("TASK#{}/{} StrDeducer::fmt {v:?} {formatter:?}", currect_task_id(), exec.problem_count());
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