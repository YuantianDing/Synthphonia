use std::{borrow::Borrow, cell::{Cell, Ref, RefCell, RefMut}, future::Future, pin::Pin, rc::Rc, task::{Poll, Context}};
use derive_more::{ From, Into, DebugCustom, Deref };
use pin_cell::PinCell;

use super::{taskrc::TaskWeak, task::currect_task};

enum Status<T: Clone> {
    Ready(T),
    Waiting(Vec<TaskWeak>),
}

impl<T: Clone> Status<T> {
    pub fn clone_as_poll(&self) -> Poll<T> {
        match self {
            Status::Ready(a) => Poll::Ready(a.clone()),
            Status::Waiting(_) => Poll::Pending,
        }
    }
}

#[derive(From, Into, Clone)]
pub struct FutCell<T: Clone>(Pin<Rc<RefCell<Status<T>>>>);


impl<T: Clone> FutCell<T> {
    pub fn new() -> Self {
        Rc::pin(RefCell::new(Status::Waiting(Vec::new()))).into()
    }
    pub fn new_ready(result: T) -> Self {
        Rc::pin(RefCell::new(Status::Ready(result))).into()
    }
    pub fn inner(&self) -> Ref<'_, Status<T>> {
        (*self.0).borrow()
    }
    pub fn inner_mut(&self) -> RefMut<'_, Status<T>> {
        (*self.0).borrow_mut()
    }
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        matches!(&*self.inner(), Status::Ready(_))
    }

    #[inline(always)]
    pub fn set_value(&self, value: T) -> Result<(), ()> {
        if let Status::Waiting(vec) = (*self.0).replace(Status::Ready(value)) {
            vec.iter().try_for_each(|x| { x.notify()?; Ok(())} )?;
            Ok(())
        } else { panic!("Setting value to a completed FutCell."); }
    }
    #[inline(always)]
    pub fn result(&self) -> Poll<T> {
        self.inner().clone_as_poll()
    }
    pub fn add_listener(&self, task: TaskWeak) {
        if let Status::Waiting(vec) = &mut *self.inner_mut() {
            vec.push(task);
        }
    }
    pub fn add_cur_task(&self) {
        self.add_listener(currect_task());
    }
}


impl<T: Clone> Future for FutCell<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if let Status::Ready(r) = &*self.inner() { 
            return Poll::Ready(r.clone())
        }
        self.add_cur_task();
        Poll::Pending
    }
}



#[cfg(test)]
mod tests {
    use std::{rc::Rc, cell::Cell, pin::pin, task::Poll};

    use crate::{async_clone, forward::future::{futcell::FutCell, task::{self, Task, TaskO}}};


    #[test]
    fn test_notify1() {
        let cell = FutCell::<usize>::new();
        let task1 = task::spawn(async_clone! { [cell]
            cell.await
        });
        
        assert!(!task1.poll_task());
        cell.set_value(100).unwrap();
        assert!(task1.is_ready());
    }
    #[test]
    fn test_notify2() {
        use std::future::Future;
        let cell = FutCell::<usize>::new();
        let mut task1 = task::spawn(async_clone! { [cell]
            cell.await
        });
        
        assert!(!task1.is_ready());
        assert!(!task1.poll_task());
        cell.set_value(100).unwrap();
        assert!(task1.poll_task());
        assert_eq!(task1.poll_unpin(), Poll::Ready(100));
    }
}