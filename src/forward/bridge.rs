
use std::{cell::UnsafeCell, task::{Poll, Waker}};

use futures::FutureExt;
use itertools::Itertools;
use rc_async::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{expr::{Expr, Expression}, info, utils::UnsafeCellExt};



pub struct Bridge(UnsafeCell<Vec<(JoinHandle<Expression>, oneshot::Sender<Expression>)>>);

impl Bridge {
    pub fn new() -> Self {
        Self(Vec::new().into())
    }
    fn inner(&self) -> &mut Vec<(JoinHandle<Expression>, oneshot::Sender<Expression>)> {
        unsafe { self.0.as_mut() }
    }
    pub fn wait(&self, handle: JoinHandle<Expression>) -> oneshot::Reciever<Expression> {
        let rv = oneshot::channel();
        self.inner().push((handle, rv.sender()));
        rv
    }
    pub fn abort_all(&self) {
        for (h, p) in self.inner() {
            h.abort();
        }
        *self.inner() = Vec::new();
    }
    pub fn check(&self) {
        let vec = std::mem::replace(self.inner(), Vec::new());
        let mut v = vec.into_iter().flat_map(|(mut h, s)| {
            let mut cx = std::task::Context::from_waker(Waker::noop());
            if let Poll::Ready(r) = h.poll_unpin(&mut cx) { 
                info!("Thread {} ended", h.id());
                if let Ok(r) = r { let _ = s.send(r); }
                None
            } else { Some((h, s)) }
        }).collect_vec();
        self.inner().append(&mut v);
    }
}

