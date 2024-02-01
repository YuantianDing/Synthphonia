use pin_weak::rc::PinWeak;

use crate::forward::future::task::Task;

use derive_more::{ From, Into, DebugCustom, Deref };
use std::task::Poll;

use std::task::Context;

use crate::forward::future::task::TaskT;

use std::rc::Rc;

use std::pin::Pin;

use std::future::Future;

use super::task::TaskO;


/// Fuck Orphan Rule
#[derive(From, Into, Deref)]
pub struct TaskTRc<T: Future + 'static>(Pin<Rc<TaskT<T>>>);

impl<T: Future> Clone for TaskTRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Future + 'static> Future for TaskTRc<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_unpin()
    }
}

impl<T: Future + 'static> TaskTRc<T> {

    pub fn poll_task(&self) -> bool{
        self.clone().0.poll_task()
    }
    pub fn notify(&self) -> Result<bool, ()> {
        self.clone().0.notify()
    }
    pub fn task(self) -> TaskRc {
        self.into()
    }
    pub fn tasko(self) -> TaskORc<T::Output> {
        TaskORc(self.0)
    }
}

#[derive(From, Into, Deref, Debug)]
pub struct TaskORc<T>(pub Pin<Rc<dyn TaskO<T>>>);

impl<T> Clone for TaskORc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> TaskORc<T> {
    pub fn poll_task(&self) -> bool{
        self.clone().0.poll_task()
    }
    pub fn notify(&self) -> Result<bool, ()> {
        self.clone().0.notify()
    }
    pub fn task(&self) -> TaskRc {
        TaskRc(self.0.clone())
    }
}

impl<T> Future for TaskORc<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<T> {
        self.poll_unpin()
    }
}

/// Fuck Orphan Rule
#[derive(From, Into, Deref, Debug)]
pub struct TaskRc(pub Pin<Rc<dyn Task>>);

impl Clone for TaskRc {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Future + 'static> From<TaskTRc<T>> for TaskRc {
    fn from(value: TaskTRc<T>) -> Self {
        TaskRc(value.0)
    }
}

impl<T: Future + 'static> From<TaskORc<T>> for TaskRc {
    fn from(value: TaskORc<T>) -> Self {
        TaskRc(value.0)
    }
}

impl TaskRc {
    pub fn downgrade(self) -> TaskWeak {
        PinWeak::downgrade(self.0).into()
    }
    pub fn poll_task(&self) -> bool{
        self.clone().0.poll_task()
    }
    #[inline(always)]
    pub fn notify(&self) -> Result<bool, ()> {
        self.clone().0.notify()
    }
}

#[derive(Clone, From, Into, Deref, Default)]
pub struct TaskWeak(Option<PinWeak<dyn Task>>);

impl std::fmt::Debug for TaskWeak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(a) = self.upgrade() {
            write!(f, "{:?}", a)
        } else { write!(f, "null") }
    }
}

impl From<PinWeak<dyn Task>> for TaskWeak {
    fn from(value: PinWeak<dyn Task>) -> Self {
        Some(value).into()
    }
}

/// Fuck Orphan Rule
impl TaskWeak {
    pub fn new() -> Self { None.into() }
    #[inline(always)]
    pub fn upgrade(&self) -> Option<TaskRc> {
        self.as_ref().and_then(|x| x.upgrade().map(|x| x.into()))
    }
    #[inline(always)]
    pub fn notify(&self) -> Result<bool, ()> {
        self.upgrade().map(|x| x.notify()).unwrap_or(Ok(false))
    }
}
