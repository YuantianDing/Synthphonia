use std::{cell::{RefCell, UnsafeCell}, cmp::{max, min}, collections::{HashMap, HashSet}, pin::{pin, Pin}, rc::Rc, task::Poll};

use bumpalo::collections::CollectIn;
use futures::{future::select, FutureExt};
use futures_core::Future;
use itertools::Itertools;

use crate::{async_closure, closure, debg, expr::{ ops::Op1Enum, Expr}, forward::{executor::Executor, future::{task::{self, currect_task_id}, taskrc::TaskORc}}};
use crate::{galloc::{self, AllocForAny, AllocForExactSizeIter, AllocForIter}, never, text::formatting::Op1EnumToFormattingOp, utils::{pending_if, select_all, select_ret, select_ret3, select_ret4, UnsafeCellExt}, value::Value};

use crate::expr;
use super::Deducer;

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
    async fn deduce(&'static self, exec: &'static Executor, value: Value) -> &'static crate::expr::Expr {
        let this = self;
        let (is_first, task) = exec.data[self.nt].all_eq.acquire_is_first(value);
        debg!("TASK#{} Deducing subproblem: {} {:?} {}", currect_task_id(), exec.cfg[self.nt].name, value, is_first);
        if !is_first {
            return task.await;
        }

        let mut delimiterset = HashSet::<Vec<&'static str>>::new();
        let futures = FutureRcVec::new();
        let event = exec.data[self.nt].substr.try_at(value, 
            closure! { clone futures; move |delimiter| {
                let vec = delimiter.to_str().iter().zip(value.to_str().iter()).map(|(a, b)| if b.contains(a) { *a } else { &"" }).collect_vec();
                if delimiterset.insert(vec) {
                    futures.extend(this.on_delim(exec, value, delimiter));
                }
                None::<&'static Expr>
            }}
        );
        let iter = self.formatter.iter().map(|x| self.fmt(value.try_into().unwrap(), x, exec));
        
        let result = select_ret4(task, pin!(event), futures, pin!(select_all(iter))).await;
        let _ = exec.data[self.nt].all_eq.set_ref(value, result);
        result
    }
}



impl StrDeducer {
    #[inline]
    pub fn on_delim(&'static self, exec: &'static Executor, value: Value, delimiter: Value) -> Vec<TaskORc<&'static Expr>> {
        let v: &[&str] = value.try_into().unwrap();
        let delimiter: &[&str] = delimiter.try_into().unwrap();
        let contain_count: usize = v.iter().zip(delimiter.iter()).filter(|(x, y)| if **y != "" { x.contains(*y) } else { false }).count();
        let start_count: usize = v.iter().zip(delimiter.iter()).map(|(x, y)| if x.starts_with(*y) { y.len() } else {0}).sum();
        let eq_count: usize = v.iter().zip(delimiter.iter()).map(|(x, y)| if x == y { y.len() } else {0}).sum();

        let split_once_count = min(exec.ctx.len(), self.decay(exec, self.split_once.0));
        let ite_concat_count = self.decay(exec, self.ite_concat.0);
        let ite_concat_eq_count = self.decay(exec, 3);

        let mut result = Vec::with_capacity(2);
        if contain_count >= split_once_count && exec.data[self.nt].all_eq.get(delimiter.into()).cost() <= self.split_once.1 {
            result.push(self.split1(v, delimiter, exec, contain_count));
        }
        if start_count >= ite_concat_count || eq_count >= ite_concat_eq_count && self.ite_concat.0 != usize::MAX {
            result.push(self.ite_concat(v, delimiter, exec, start_count));
        }
        result
    }
    #[inline]
    fn decay(&self, exec: &'static Executor, i: usize) -> usize {
        let rate = self.decay_rate;
        let task = rate + min(7 * rate, exec.problem_count());
        return (i * task) / rate
    }
    #[inline]
    fn split1(&'static self, v: &'static [&'static str], delimiter: &'static [&'static str], exec: &'static Executor, count: usize) -> TaskORc<&'static Expr> {
        debg!("TASK#{}/{} StrDeducer::split1 {count} {v:?} {delimiter:?}", currect_task_id(), exec.problem_count());
        task::spawn( async move {
            let (a,b) = split_once(v, delimiter);
            let left = exec.solve_task(self.nt, a).await;
            let right = exec.solve_task(self.nt, b).await;
            let delim = exec.data[self.nt].all_eq.get(delimiter.into());
            if a.to_str().iter().all(|x| x.len() == 0) {
                return expr!(Concat {delim} {right}).galloc();
            }
            if b.to_str().iter().all(|x| x.len() == 0) {
                return expr!(Concat {left} {delim}).galloc();
            }
            expr!(Concat (Concat {left} {delim}) {right}).galloc()
        }).tasko()
    }
    #[inline]
    fn ite_concat(&'static self, v: &'static [&'static str], delimiter: &'static [&'static str], exec: &'static Executor, count: usize) -> TaskORc<&'static Expr> {
        debg!("TASK#{}/{} StrDeducer::ite_concat {count} {v:?} {delimiter:?}", currect_task_id(), exec.problem_count());
        task::spawn( async move {
            let (a,b) = ite_concat_split(v, delimiter);
            let cond = exec.solve_task(self.ite_concat.1, a).await;
            let right = exec.solve_task(self.nt, b).await;
            let left = exec.data[self.nt].all_eq.get(delimiter.into());
            expr!(Concat (Ite {cond} {left} "") {right}).galloc()
        }).tasko()
    }
    #[inline]
    async fn join(&self, v: &'static [&'static str], delimiter: &'static [&'static str], exec: &'static Executor) -> &'static Expr {
        debg!("TASK#{} StrDeducer::join {v:?} {delimiter:?}", currect_task_id());
        let a = value_split(v, delimiter);
        let list = exec.solve_task(self.join.1, a).await;
        let delim = exec.data[self.nt].all_eq.get(delimiter.into());
        expr!(Join {list} {delim}).galloc()
    }
    #[inline]
    async fn fmt(&self, v: &'static [&'static str], formatter: &(Op1Enum, usize), exec: &'static Executor) -> &'static Expr {
        if let Some((op, a, b)) = formatter.0.format_all(v) {
            debg!("TASK#{}/{} StrDeducer::fmt {v:?} {formatter:?}", currect_task_id(), exec.problem_count());
            let inner = exec.solve_task(formatter.1, a).await;
            let rest = exec.solve_task(self.nt, b).await;
            
            let expr = Expr::Op1(op.galloc(), inner).galloc();
            expr!(Concat {expr} {rest}).galloc()
        } else { never!() }
    }
}

pub fn split_once(s: &'static [&'static str], delimiter: &'static [&'static str]) -> (Value, Value) {
    assert!(s.len() == delimiter.len());
    let mut a = galloc::new_bvec(s.len());
    let mut b = galloc::new_bvec(s.len());
    for (x, y) in s.iter().zip(delimiter.iter()) {
        if y.len() == 0 {
            a.push("");
            b.push(*x);
        } else if let Some((l, r)) = x.split_once(*y) {
            a.push(l);
            b.push(r);
        } else {
            a.push(x);
            b.push("");
        }
    }
    (Value::Str(a.into_bump_slice()), Value::Str(b.into_bump_slice()))
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