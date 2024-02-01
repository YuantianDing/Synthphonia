use std::{
    cell::{RefCell, UnsafeCell},
    task::Poll,
};

use crate::{galloc::AllocForAny, utils::UnsafeCellExt};

use super::{taskrc::TaskWeak, task::currect_task};

use derive_more::{Deref, From, Into};
use futures_core::Future;

pub struct ChannelInner<T: Clone> {
    result: Poll<T>,
    listeners: Vec<TaskWeak>,
}

impl<T: Clone> ChannelInner<T> {
    pub fn new() -> &'static UnsafeCell<Self> { UnsafeCell::new(Self { result: Poll::Pending, listeners: Vec::new() }).galloc() }

    #[inline(always)]
    pub fn is_ready(&self) -> bool { self.result.is_ready() }
    #[inline(always)]
    fn notify(&mut self) -> Result<(), ()> {
        let result = self.result.clone();
        let listeners = std::mem::replace(&mut self.listeners, Vec::new());
        listeners.iter().try_for_each(|x| {
            self.result = result.clone();
            x.notify()?;
            Ok(())
        })?;
        Ok(())
    }
    #[inline(always)]
    pub fn send(&mut self, value: T) -> Result<(), ()> {
        self.result = Poll::Ready(value);
        self.notify()?;
        self.result = Poll::Pending;
        Ok(())
    }
    pub fn add_listener(&mut self, task: TaskWeak) { self.listeners.push(task); }
    pub fn add_cur_task(&mut self) { self.add_listener(currect_task()); }
}

#[derive(From, Into, Deref, Clone, Copy)]
pub struct Channel<T: Clone + 'static>(&'static UnsafeCell<ChannelInner<T>>);

impl<T: Clone + 'static> std::fmt::Debug for Channel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:p}", self.0 as *const _)
    }
}


impl<T: Clone + 'static> Channel<T> {
    pub fn new() -> Self { ChannelInner::new().into() }
    pub fn get(&self) -> &mut ChannelInner<T> { unsafe { self.0.as_mut() } }
}

impl<T: Clone + 'static> Future for Channel<T> {
    type Output = T;

    #[inline(always)]
    fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let r = unsafe { self.0.as_mut() };
        if r.is_ready() {
            std::mem::replace(&mut r.result, Poll::Pending)
        } else {
            r.add_cur_task();
            Poll::Pending
        }
    }
}

impl<T: Clone + 'static> PartialEq for Channel<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl<T: Clone + 'static> Eq for Channel<T> {}

impl<T: Clone + 'static> std::hash::Hash for Channel<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}
