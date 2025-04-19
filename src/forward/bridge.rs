
use std::{cell::UnsafeCell, task::{Poll, Waker}};

use futures::FutureExt;
use itertools::Itertools;
use simple_rc_async::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{expr::{Expr, Expression}, info, utils::UnsafeCellExt};



/// a bridge for interthread communication.
pub struct Bridge(UnsafeCell<Vec<(JoinHandle<Expression>, oneshot::Sender<Expression>)>>);

impl Default for Bridge {
    /// A default constructor for the type. 
    fn default() -> Self {
        Self::new()
    }
}

impl Bridge {
    /// Creates a new instance of Bridge by initializing an empty vector within an `UnsafeCell`. 
    pub fn new() -> Self {
        Self(Vec::new().into())
    }
    /// Provides a mutable reference to the inner vector of tuples, each containing a `JoinHandle` and a `oneshot::Sender`, which are used for asynchronous computation and message passing, respectively. 
    fn inner(&self) -> &mut Vec<(JoinHandle<Expression>, oneshot::Sender<Expression>)> {
        unsafe { self.0.as_mut() }
    }
    /// Waits for the completion of a synthesis task and returns a receiver for results. 
    /// 
    /// This method takes a `JoinHandle`, which represents a spawned asynchronous task that will output an `Expression`. 
    /// It creates a oneshot channel, which is used for sending an expression once the task completes. 
    /// The sender part of the channel is paired with the `JoinHandle` and added to the vector inside the `Bridge`. 
    /// The method returns the receiver part of the channel, allowing the caller to wait for and retrieve the result of the task once it's completed.
    /// 
    pub fn wait(&self, handle: JoinHandle<Expression>) -> oneshot::Reciever<Expression> {
        let rv = oneshot::channel();
        self.inner().push((handle, rv.sender()));
        rv
    }
    /// Aborts all ongoing synthesis tasks managed by this instance. 
    pub fn abort_all(&self) {
        for (h, p) in self.inner() {
            h.abort();
        }
        *self.inner() = Vec::new();
    }
    /// Checks and handles the status of ongoing tasks and their results. 
    pub fn check(&self) {
        let vec = std::mem::take(self.inner());
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

