
use std::{cell::UnsafeCell, task::{Poll, Waker}};

use futures::FutureExt;
use itertools::Itertools;
use smol::Task;
use spin::Mutex;

use crate::{expr::{Expr, Expression}, info, utils::UnsafeCellExt};
use async_oneshot::{oneshot, Sender, Receiver};


pub struct Bridge(Mutex<Vec<(Task<Expression>, Sender<Expression>)>>);

impl Bridge {
    pub fn new() -> Self {
        Self(Vec::new().into())
    }
    pub fn wait(&self, handle: Task<Expression>) -> Receiver<Expression> {
        let (sd, rv) = oneshot();
        self.0.lock().push((handle, sd));
        rv
    }
    pub fn abort_all(&self) {
        let vec = std::mem::replace(&mut *self.0.lock(), Vec::new());
        for (h, p) in vec {
            drop(h);
        }
    }
    pub fn check(&self) {
        let vec = std::mem::replace(&mut *self.0.lock(), Vec::new());

        let mut v = vec.into_iter().flat_map(|(mut h, mut s)| {
            let mut cx = std::task::Context::from_waker(Waker::noop());
            if let Poll::Ready(r) = h.poll_unpin(&mut cx) { 
                info!("Thread {:?} ended", h);
                let _ = s.send(r);
                None
            } else { Some((h, s)) }
        }).collect_vec();
        self.0.lock().append(&mut v);
    }
}

