use std::{cell::{RefCell, UnsafeCell}, collections::{HashMap, HashSet}, pin::{pin, Pin}, rc::Rc, task::Poll};

use futures::{future::select, FutureExt};
use futures_core::Future;

use crate::{async_closure, closure, debg, expr::{ Expr}, forward::{executor::Executor, future::task::currect_task_id}, galloc::{self, AllocForAny, AllocForIter}, never, utils::{pending_if, select_all, select_ret, select_ret3, UnsafeCellExt}, value::Value};

use crate::expr;
use super::Deducer;

struct FutureHashMap<F: Future<Output=T>, T>(Rc<UnsafeCell<HashMap<Value, F>>>);

impl<F: Future<Output=T>, T> Clone for FutureHashMap<F, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<F: Future<Output=T> + Unpin, T> Future for FutureHashMap<F, T> {
    type Output=T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        for v in unsafe{ self.0.as_mut()}.values_mut() {
            if let Poll::Ready(a) = v.poll_unpin(cx) {
                return Poll::Ready(a);
            }
        }
        Poll::Pending
    }
}

impl<F: Future<Output=T>, T> FutureHashMap<F, T> {
    pub fn new() -> Self {
        Self(Rc::new(UnsafeCell::new(HashMap::new())))
    }
    pub fn try_insert(&self, k: Value, v: F) {
        let _ = unsafe{self.0.as_mut()}.try_insert(k, v);
    }
}



#[derive(Debug)]
pub struct StrDeducer {
    pub nt: usize,
    pub split_once: usize,
    pub join: (usize, usize),
    pub ite_concat: (usize, usize),
}

impl StrDeducer {
    pub fn new(nt: usize) -> Self {
        Self { nt: 0, split_once: usize::MAX, join: (usize::MAX, 0), ite_concat: (usize::MAX, 0) }
    }
}

impl Deducer for StrDeducer {
    async fn deduce(&'static self, exec: &'static Executor, value: Value) -> &'static crate::expr::Expr {
        debg!("TASK#{} Deducing subproblem: {} {:?}", currect_task_id(), exec.cfg[self.nt].name, value);
        let this = self;
        let (is_first, task) = exec.data[self.nt].all_eq.acquire_is_first(value);
        if !is_first {
            return task.await;
        }

        let futures = FutureHashMap::new();
        let event = exec.data[self.nt].substr.try_at(value, 
            closure! { clone futures; move |delimiter| {
                let _ = futures.try_insert(delimiter, Box::pin(this.on_delim(exec, value, delimiter)));
                None::<&'static Expr>
            }}
        );
        select_ret3(task, pin!(event), futures).await
    }
}

impl StrDeducer {
    pub async fn on_delim(&self, exec: &'static Executor, value: Value, delimiter: Value) -> &'static Expr {
        let v: &[&str] = value.try_into().unwrap();
        let delimiter: &[&str] = delimiter.try_into().unwrap();
        let contain_count = v.iter().zip(delimiter.iter()).filter(|(x, y)| x.contains(**y)).count();
        let start_count = v.iter().zip(delimiter.iter()).filter(|(x, y)| x.starts_with(**y)).count();
        // let eq_count = v.iter().zip(delimiter.iter()).filter(|(x, y)| x == y).count();
        let split1 = pending_if(contain_count >= self.split_once, self.split1(v, delimiter, exec));
        let ite_concat = pending_if(start_count >= self.ite_concat.0, self.ite_concat(v, delimiter, exec));
        
        select_ret(pin!(split1), pin!(ite_concat)).await
    }
    #[inline]
    async fn split1(&self, v: &'static [&'static str], delimiter: &'static [&'static str], exec: &'static Executor) -> &'static Expr {
        debg!("TASK#{} StrDeducer::split1 {v:?} {delimiter:?}", currect_task_id());
        let (a,b) = split_once(v, delimiter);
        let left = exec.spawn_task(self.nt, a).await;
        let right = exec.spawn_task(self.nt, b).await;
        let delim = exec.data[self.nt].all_eq.get(delimiter.into());
        if a.to_str().iter().all(|x| x.len() == 0) {
            return expr!(Concat {delim} {right}).galloc();
        }
        if b.to_str().iter().all(|x| x.len() == 0) {
            return expr!(Concat {left} {delim}).galloc();
        }
        expr!(Concat (Concat {left} {delim}) {right}).galloc()
    }
    #[inline]
    async fn ite_concat(&self, v: &'static [&'static str], delimiter: &'static [&'static str], exec: &'static Executor) -> &'static Expr {
        debg!("TASK#{} StrDeducer::ite_concat {v:?} {delimiter:?}", currect_task_id());
        let (a,b) = ite_concat_split(v, delimiter);
        let cond = exec.spawn_task(self.ite_concat.1, a).await;
        let right = exec.spawn_task(self.nt, b).await;
        let left = exec.data[self.nt].all_eq.get(delimiter.into());
        expr!(Concat (Ite {cond} {left} "") {right}).galloc()
    }
    #[inline]
    async fn join(&self, v: &'static [&'static str], delimiter: &'static [&'static str], exec: &'static Executor) -> &'static Expr {
        debg!("TASK#{} StrDeducer::join {v:?} {delimiter:?}", currect_task_id());
        let a = value_split(v, delimiter);
        let list = exec.spawn_task(self.join.1, a).await;
        let delim = exec.data[self.nt].all_eq.get(delimiter.into());
        expr!(Join {list} {delim}).galloc()
    }
}

pub fn split_once(s: &'static [&'static str], delimiter: &'static [&'static str]) -> (Value, Value) {
    assert!(s.len() == delimiter.len());
    let mut a = galloc::new_bvec();
    let mut b = galloc::new_bvec();
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
    let mut a = galloc::new_bvec();
    let mut b = galloc::new_bvec();
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