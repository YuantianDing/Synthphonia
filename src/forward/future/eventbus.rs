use std::{task::{Poll, Context}, cell::{Cell, RefCell}, future::Future, rc::Rc, pin::Pin};
use derive_more::{ From, Into, DebugCustom, Deref };

use super::{taskrc::TaskWeak, task::currect_task};


pub struct EventBus<T: Clone> {
    pub result: Cell<Poll<T>>,
    // EventBus are hooked when `await` !
    listeners: RefCell<Vec<TaskWeak>>,
}


impl<T: Clone> EventBus<T> {
    pub fn new() -> Self {
        Self {
            result: Poll::Pending.into(),
            listeners: Vec::new().into(),
        }
    }

    // pub fn new_cur_task() -> Self {
    //     Self { result: Poll::Pending.into(), listeners: vec![currect_task()].into() }
    // }
    pub fn new_ready(result: T) -> EventBus<T> {
        Self { result: Poll::Ready(result).into(), listeners: Vec::new().into() }
    }
    
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        let a = self.result.replace(Poll::Pending);
        let p = a.is_ready();
        self.result.set(a);
        p
    }
    #[inline(always)]
    fn notify(&self) -> Result<(), ()> {
        let mut vec = self.listeners.replace(Vec::new());
        vec.iter().try_for_each(|x| { x.notify()?; Ok(())} )?;
        Ok(())
    }
    #[inline(always)]
    pub fn set_value(&self, value: T) -> Result<(), ()> {
        self.result.set(Poll::Ready(value));
        self.notify()
    }
    #[inline(always)]
    pub fn result(&self) -> Poll<T> {
        let a = self.result.replace(Poll::Pending);
        self.result.set(a.clone());
        a
    }
    pub fn add_listener(&self, task: TaskWeak) {
        debug_assert!(!self.is_ready());
        self.listeners.borrow_mut().push(task);
    }
    pub fn add_cur_task(&self) {
        self.add_listener(currect_task());
    }
}

#[derive(From, Into, Deref)]
pub struct EventBusRc<T: Clone>(Pin<Rc<EventBus<T>>>);

impl<T: Clone> EventBusRc<T> {
    pub fn new() -> EventBusRc<T> {
        Rc::pin(EventBus::new()).into()
    }
    
    pub fn new_ready(result: T) -> EventBusRc<T> {
        Rc::pin(EventBus::new_ready(result)).into()
    }
}

impl<T:Clone> Clone for EventBusRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> Future for EventBusRc<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_ready() { 
            let a = self.result.replace(Poll::Pending);
            self.result.set(a.clone());
            a
        }
        else {
            self.add_cur_task();
            Poll::Pending
        }
    }
}



#[cfg(test)]
mod tests {
    use std::{rc::Rc, cell::Cell, pin::pin, task::Poll};

    use crate::{forward::future::{taskrc::{TaskWeak, TaskTRc}, task::{Task, self, TaskO}, eventbus::{EventBus, EventBusRc, self}}, async_clone};

    #[test]
    fn test_notify1() {
        let eventbus = EventBusRc::<usize>::new();
        let task1 = task::spawn(async_clone! { [eventbus]
            eventbus.await
        });
        
        assert!(!task1.poll_task());
        eventbus.set_value(100).unwrap();
        assert!(task1.is_ready());
    }
    #[test]
    fn test_notify2() {
        use std::future::Future;
        let eventbus = EventBusRc::<usize>::new();
        let mut task1 = task::spawn(async_clone! { [eventbus]
            eventbus.await
        });
        
        assert!(!task1.is_ready());
        assert!(!task1.poll_task());
        eventbus.set_value(100).unwrap();
        assert!(task1.poll_task());
        assert_eq!(task1.poll_unpin(), Poll::Ready(100));
    }
}