use std::{borrow::Borrow, cell::{Cell, Ref, RefCell, RefMut}, future::Future, pin::Pin, rc::Rc, task::{Poll, Context}};
use derive_more::{ From, Into, DebugCustom, Deref };
use pin_cell::PinCell;

use super::{taskrc::TaskWeak, task::currect_task};

/// An enum that represents the status of a task or process in a synthesis operation. 
/// 
/// It has two variants: `Ready` and `Waiting`. 
/// 
/// 
/// The `Ready` variant holds a value of a generic type `T`, which is required to implement the `Clone` trait, indicating that the task or process is completed and the result is available. 
/// The `Waiting` variant contains a vector of weak references to tasks, represented by `Vec<TaskWeak>`, indicating that the task or process is currently waiting for other dependencies or conditions to be fulfilled before it can proceed. 
/// This design enables both the immediate availability of results and the management of dependencies in task execution workflows.
enum Status<T: Clone> {
    Ready(T),
    Waiting(Vec<TaskWeak>),
}

impl<T: Clone> Status<T> {
    /// Transforms a `Status` into a `Poll`. 
    /// 
    /// Depending on the variant, it returns either `Poll::Ready` with a cloned value if the status is `Ready`, or `Poll::Pending` if the status is `Waiting`. 
    /// This method is useful for interfacing with asynchronous operations, where the task may either be immediately available or require further conditions to be met, handled by signaling readiness or the need to await.
    /// 
    pub fn clone_as_poll(&self) -> Poll<T> {
        match self {
            Status::Ready(a) => Poll::Ready(a.clone()),
            Status::Waiting(_) => Poll::Pending,
        }
    }
}

#[derive(From, Into, Clone)]
/// A container designed to hold a value of type `T`, which must implement the `Clone` trait, providing safe shared access in a concurrency context. 
/// 
/// This type is a layered construct combining Rust's `Pin`, `Rc` (reference counting), and `RefCell` paradigms. 
/// It ensures memory safety during asynchronous operations by holding a `Pin` to prevent it from being moved while interior mutability is achieved through a `RefCell`, allowing mutation of its contained value even when the `Rc` is immutable. 
/// This combination makes it ideal for scenarios requiring both shared access and modification of values across asynchronous tasks, ensuring thread-safe operation and memory management.
/// 
pub struct FutCell<T: Clone>(Pin<Rc<RefCell<Status<T>>>>);


impl<T: Clone> FutCell<T> {
    /// Creates a new instance in a waiting state. 
    /// 
    /// Initializes the instance with a `Status::Waiting` variant, encapsulating an empty vector of weak references to tasks (`TaskWeak`). 
    /// This setup indicates that the new instance is initially unset and listening for tasks that will complete and eventually provide a value. 
    /// The use of `Rc::pin` within the `new` function ensures that the inner `RefCell` is safely pinned, preventing it from being moved and maintaining its location in memory, which is crucial for safely managing mutable and shared state in concurrent contexts.
    /// 
    pub fn new() -> Self {
        Rc::pin(RefCell::new(Status::Waiting(Vec::new()))).into()
    }
    /// Creates a new instance of this type that is immediately ready with a specified result. 
    /// 
    /// This function wraps the provided result into a `Status::Ready` variant within a `RefCell`, then pins it using `Rc::pin`. 
    /// It subsequently converts this pinned reference into the appropriate structure for use as a `FutCell`. 
    /// The method illustrates an efficient approach for initializing `FutCell` instances that are in a ready state without additional waiting tasks.
    /// 
    pub fn new_ready(result: T) -> Self {
        Rc::pin(RefCell::new(Status::Ready(result))).into()
    }
    /// Provides access to the inner status of the `FutCell`. 
    /// 
    /// This function returns a borrowed reference to the `Status<T>` contained within the `FutCell`. 
    /// By calling this method, users can inspect the current state, whether it is ready or waiting, along with any associated data or tasks, without consuming or modifying the `FutCell` itself. 
    /// This operation depends on the borrowing rules of Rust, ensuring safe concurrent access.
    /// 
    pub fn inner(&self) -> Ref<'_, Status<T>> {
        (*self.0).borrow()
    }
    /// Provides a method to obtain a mutable reference to the internal `Status` of a `FutCell`. 
    /// This method enables access to modify the encapsulated `Status<T>` by borrowing a mutable reference, ensuring thread-safe manipulation within the constraints of Rust's borrowing rules.
    pub fn inner_mut(&self) -> RefMut<'_, Status<T>> {
        (*self.0).borrow_mut()
    }
    #[inline(always)]
    /// Determines if the underlying status is ready. 
    /// 
    /// This function checks whether the inner state of the object is in the `Ready` variant of the `Status` enum. 
    /// It returns `true` if the state is a `Ready` variant, indicating that the encapsulated value is available, otherwise it returns `false`.
    /// 
    pub fn is_ready(&self) -> bool {
        matches!(&*self.inner(), Status::Ready(_))
    }

    #[inline(always)]
    /// Sets the value of the `FutCell`, transitioning its status from `Waiting` to `Ready`. 
    /// 
    /// It replaces the current status of the `FutCell` with `Status::Ready` containing the provided value. 
    /// If the original status was `Status::Waiting`, it iterates over the collection of tasks, attempting to notify each task via the `notify` method. 
    /// If all notifications are successful, the function returns `Ok(())`. 
    /// If the `FutCell` is already in a `Ready` state, indicating it has been completed previously, the function panics to prevent overwriting a completed state.
    /// 
    pub fn set_value(&self, value: T) -> Result<(), ()> {
        if let Status::Waiting(vec) = (*self.0).replace(Status::Ready(value)) {
            vec.iter().try_for_each(|x| { x.notify()?; Ok(())} )?;
            Ok(())
        } else { panic!("Setting value to a completed FutCell."); }
    }
    #[inline(always)]
    /// Provides a method to retrieve the current status of the cell as a polling result. 
    /// 
    /// This retrieves the inner value of the FutCell, clones it, and returns it as a `Poll<T>`. 
    /// This enables evaluating the current state of the asynchronous computation, whether ready or pending, encapsulated within the FutCell structure.
    /// 
    pub fn result(&self) -> Poll<T> {
        self.inner().clone_as_poll()
    }
    /// Adds a listener task to a `FutCell`. 
    /// 
    /// This method checks if the current status of the cell is `Waiting`, which includes a vector of tasks, and if so, it appends the given weak reference to a task, `task`, to this vector. 
    /// This operation allows the system to keep track of tasks that are awaiting notification or further processing contingent on the future cell's state changes.
    /// 
    pub fn add_listener(&self, task: TaskWeak) {
        if let Status::Waiting(vec) = &mut *self.inner_mut() {
            vec.push(task);
        }
    }
    /// Adds the current task as a listener to the `FutCell`. 
    /// 
    /// This method retrieves the current task and invokes `add_listener` to ensure that the task is notified when the state of the `FutCell` changes. 
    /// By linking the current task to the cell, it facilitates coordination or synchronization across tasks that depend on the state encapsulated by `FutCell`.
    /// 
    pub fn add_cur_task(&self) {
        self.add_listener(currect_task());
    }
}


impl<T: Clone> Future for FutCell<T> {
    type Output = T;

    /// Provides the implementation for polling the wrapped status of a `FutCell`. 
    /// 
    /// When invoked, it checks whether the status inside the `FutCell` is currently `Ready`. 
    /// If `Ready`, it returns a `Poll::Ready` with the contained value cloned. 
    /// If the status is not `Ready`, it registers the current task to be notified for future polling and returns `Poll::Pending`. 
    /// This mechanism allows the `FutCell` to integrate into asynchronous tasks, operating within Rust's async runtime environment by leveraging the `Context` parameter to manage task state and transitions efficiently.
    /// 
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