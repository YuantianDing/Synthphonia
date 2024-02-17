use std::{future::Future, process::Output, rc::{Rc, Weak}, pin::Pin, cell::{RefCell, Cell, UnsafeCell}, borrow::BorrowMut, task::{Poll, Context, Waker}, ops::{Deref, DerefMut}};
use derive_more::{ From, Into, DebugCustom, Deref };
use pin_cell::{PinCell, PinMut};
use pin_weak::rc::PinWeak;

use crate::{forward::future::taskrc::{TaskWeak, TaskRc}, debg};

use super::taskrc::{self, TaskTRc};

pub type Fut<'a, T> = PinCell<dyn Future<Output = T> + 'a>;

#[derive(DebugCustom)]
#[debug(fmt = "task#{}{}", id, r#"if self.is_ready() { "✓" } else { "" }"#)]
pub struct TaskT<T: Future + 'static> {
    pub id: usize,
    // Tasks are hooked when created !
    next: TaskWeak,
    pub result: Cell<Poll<T::Output>>,
    fut: PinCell<T>,
}


impl<T: Future  + 'static> TaskT<T> {
    fn new(id: usize, fut: T) -> Self {
        let task = CUR_TASK.take();
        CUR_TASK.set(task.clone());
        // Tasks are hooked when created !
        Self {
            id,
            next: task,
            result: Poll::Pending.into(),
            fut: fut.into(),
        }
    }
}

pub fn spawn<T: Future + 'static>(fut: T) -> TaskTRc<T> {
    let res: TaskTRc<_> = Rc::pin(TaskT::new(generate_task_id(), fut)).into();
    res.poll_task();
    res
}


pub trait Task: std::fmt::Debug {
    fn is_ready(&self) -> bool;
    fn poll_task(self: Pin<Rc<Self>>) -> bool;
    fn notify(self: Pin<Rc<Self>>) -> Result<bool, ()>;
    fn id(&self) -> usize;
}

thread_local! {
    static CUR_TASK: Cell<TaskWeak> = Cell::new(TaskWeak::new());
}
#[thread_local]
static mut TOP_TASK: Option<TaskRc> = None;

pub fn currect_task() -> TaskWeak {
    let a = CUR_TASK.take();
    CUR_TASK.set(a.clone());
    a
}
pub fn currect_task_id() -> usize {
    let a = CUR_TASK.take();
    let id = a.upgrade().unwrap().0.id();
    CUR_TASK.set(a);
    id
}
pub fn currect_task_id_opt() -> usize {
    let a = CUR_TASK.take();
    a.upgrade().map(|x| {
        let id = x.0.id();
        CUR_TASK.set(a);
        id
    }).unwrap_or(0)
}
pub fn top_task() -> TaskRc {
    unsafe { TOP_TASK.clone().unwrap() }
}
#[inline(always)]
pub fn top_task_ready() -> bool {
    unsafe{ TOP_TASK.as_ref() }.map(|x| x.is_ready()).unwrap_or(false)
}
pub fn with_top_task<T>(t: TaskRc, f: impl FnOnce() -> T) -> T {
    let orig = unsafe { TOP_TASK.replace(t) };
    let result = f();
    unsafe { TOP_TASK = orig; }
    result
}

#[thread_local]
static mut TASK_COUNTER : usize = 0;

pub fn generate_task_id() -> usize {
    unsafe {
        TASK_COUNTER += 1;
        TASK_COUNTER
    }
}

pub fn number_of_task() -> usize {
    unsafe { TASK_COUNTER }
}

impl <T: Future + 'static> Task for TaskT<T> {
    #[inline(always)]
    fn is_ready(&self) -> bool {
        let a = self.result.replace(Poll::Pending);
        let p = a.is_ready();
        self.result.set(a);
        p
    }
    fn poll_task(self: Pin<Rc<Self>>) -> bool {
        if self.is_ready() { return true; }
        debug_assert!(currect_task_id_opt() < self.id);
        
        let fut = unsafe { Pin::new_unchecked(&self.fut) };
        if let Ok(mut fut) = fut.try_borrow_mut() {
            let r = PinMut::as_mut(&mut fut);
            let waker = Waker::noop();
            let mut cx = Context::from_waker(&waker);
            
            let last_task = CUR_TASK.replace(taskrc::TaskRc(self.clone()).downgrade());
            let result = r.poll(&mut cx);
            CUR_TASK.set(last_task);
            self.result.set(result);

            self.is_ready()
        } else { false }
    }

    fn notify(self: Pin<Rc<Self>>) -> Result<bool, ()> {
        if self.clone().poll_task() {
            if top_task_ready() {
                return Err(());
            }
            if let Some(next) = &self.next.upgrade() {
                next.clone().0.notify()?;
            }
            Ok(true)
        } else { Ok(false) }
    }

    fn id(&self) -> usize {
        self.id
    }
}

pub trait TaskO<T>: Task {
    fn poll_unpin(self: &Self) -> Poll<T>;
}

impl<T: Future + 'static> TaskO<T::Output> for TaskT<T> {
    fn poll_unpin(&self) -> Poll<T::Output> {
        if self.is_ready() { self.result.replace(Poll::Pending) }
        else { Poll::Pending }
    }
}


#[cfg(test)]
mod tests {
    use std::{rc::Rc, pin::Pin, task::{Waker, Context}, cell::{RefCell, Cell}};

    use pin_cell::{PinCell, PinMut};
    use refbox::RefBox;
    use std::future::Future;

    use crate::forward::{future::task::{Task, TaskT, self}, future::taskrc::{TaskTRc, TaskWeak}};


    #[test]
    fn test_poll_task() {

        let fut = async {
            let mut x = 1;
            let y = &mut x;
            *y += 1;
            assert_eq!(*y, 2);
        };
        let task1 = task::spawn(fut);
        let task1c: TaskTRc<_> = task1.clone();
        let fut2 = async { task1c.await };
        let task2 = task::spawn(fut2);
        assert!(task1.poll_task());
        assert!(task2.poll_task());

    }
    #[test]
    fn test_notify() {
        let tasko2 = Rc::new(Cell::new(TaskWeak::new()));
        let tasko2copy = tasko2.clone();
        let task1 = task::spawn(async move {
            let task2 = task::spawn(async {1});
            tasko2copy.set(task2.clone().task().downgrade());
            task2.await
        });

        assert!(!task1.poll_task());
        assert!(tasko2.take().upgrade().unwrap().notify().unwrap());
        assert!(task1.is_ready());

    }
}



