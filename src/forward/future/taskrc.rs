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
    /// Provides a method to create a copy of an instance of the associated type. 
    /// 
    /// The `clone` function takes a reference to the current instance and returns a new instance that is a clone of the original. 
    /// This is achieved by utilizing the `clone` method on the `Rc<TaskT<T>>`, allowing the smart pointer to increment its reference count. 
    /// As a result, the new instance will share ownership of the managed task with the original, enabling multiple owners to exist for the same underlying data without duplicating the data itself.
    /// 
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Future + 'static> Future for TaskTRc<T> {
    type Output = T::Output;

    /// Polls the future wrapped within the `TaskTRc` instance. 
    /// 
    /// This function takes a pinned mutable reference to the `TaskTRc` and a mutable reference to a `Context` and delegates the polling operation to the `poll_unpin` method. 
    /// It integrates seamlessly into the asynchronous Rust ecosystem by enabling the `TaskTRc` to be used as a future, determining if the task is ready to produce an output or if it must be polled again at a later stage.
    /// 
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_unpin()
    }
}

impl<T: Future + 'static> TaskTRc<T> {

    /// Provides functionality to poll the task wrapped within the current instance. 
    /// 
    /// The `poll_task` function attempts to check the readiness of the task by invoking the `poll_task` method on the inner `TaskT<T>` object. 
    /// This is achieved by first cloning the instance to ensure exclusive access and then delegating the polling operation to the encapsulated task, ultimately returning a boolean indicating whether the task is ready.
    /// 
    pub fn poll_task(&self) -> bool{
        self.clone().0.poll_task()
    }
    /// Provides a method to notify a task represented by this instance. 
    /// 
    /// The `notify` method attempts to trigger the task it encapsulates, returning a `Result`. 
    /// Upon success, it returns `Result::Ok(true)` if the task was successfully notified and a retry or wake-up is required. 
    /// If the notification does not succeed or is unnecessary, it may return `Result::Ok(false)`. 
    /// In case of failure, it returns `Result::Err(())`. 
    /// This implementation leverages cloning of the `TaskTRc` instance to ensure the task can be safely notified by working with a pinned and reference-counted version of the task.
    /// 
    pub fn notify(&self) -> Result<bool, ()> {
        self.clone().0.notify()
    }
    /// Provides a method to convert a `TaskTRc` instance into a `TaskRc`. 
    /// This method enables seamless transitioning between the two types, leveraging Rust's type conversion mechanisms to facilitate the interoperability between these task representations.
    pub fn task(self) -> TaskRc {
        self.into()
    }
    /// Transforms the instance into a `TaskORc` type. 
    /// 
    /// This method consumes the current `TaskTRc` instance and converts it into a `TaskORc` by utilizing the inner `Rc<TaskT<T>>`. 
    /// The `TaskORc` will hold a reference to the output type of the future `T`, preserving the reference-counted nature of the task while potentially allowing different operations or transformations to be applied to the task's result once it is resolved.
    /// 
    pub fn tasko(self) -> TaskORc<T::Output> {
        TaskORc(self.0)
    }
}

#[derive(From, Into, Deref, Debug)]
/// A wrapper around a reference-counted task object. 
/// 
/// This structure encapsulates a `Pin` around a `Rc` to ensure that the task cannot be moved, providing a safe way to manage tasks that must uphold certain invariants. 
/// The `TaskORc` is generic over a type `T`, representing the type associated with the encapsulated task operation. 
/// By using `Rc`, this structure supports shared ownership of the task across multiple components, providing automatic memory management as tasks are used and shared in different parts of the system. 
/// The use of a dynamic trait object (`dyn TaskO<T>`) allows for polymorphic task operations, enabling flexible implementations of task logic to be used interchangeably.
/// 
pub struct TaskORc<T>(pub Pin<Rc<dyn TaskO<T>>>);

impl<T> Clone for TaskORc<T> {
    /// Implements a method to create a clone of a `TaskORc` instance. 
    /// 
    /// This method returns a new instance of the same type, containing a cloned pinned reference-counted pointer to the `TaskO` trait object. 
    /// This ensures that the shared ownership semantics and memory safety of the reference-counted object are maintained.
    /// 
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> TaskORc<T> {
    /// Provides a method to poll a task wrapped in the `TaskORc` type, determining its readiness. 
    /// 
    /// The `poll_task` function clones the `TaskORc` instance and invokes the `poll_task` method on the wrapped `TaskO` task, ultimately returning a boolean indicating whether the task is ready. 
    /// This approach enables querying the task's state in a non-blocking manner, utilising Rust's asynchronous capabilities by leveraging the underlying `TaskO` trait implementation.
    /// 
    pub fn poll_task(&self) -> bool{
        self.clone().0.poll_task()
    }
    /// Provides a method to notify the associated task. 
    /// 
    /// This function attempts to notify a task wrapped within the `TaskORc` instance. 
    /// It returns a `Result<bool, ()>`, where `Ok(true)` indicates a successful notification where some progress was made, `Ok(false)` signifies that no progress was possible, and `Err(())` represents a notification failure. 
    /// The internal logic involves cloning the current instance and invoking the `notify` method on the underlying task object.
    /// 
    pub fn notify(&self) -> Result<bool, ()> {
        self.clone().0.notify()
    }
    /// Provides a method to clone and return the underlying reference-counted task. 
    /// 
    /// This function creates a new `TaskRc` instance by cloning the internal `Rc` type stored in the `TaskORc` struct, ensuring that a new reference to the underlying task is produced. 
    /// This allows the task to be shared and accessed safely by other parts of the application without transferring ownership, maintaining Rust's safety guarantees related to reference counting.
    /// 
    pub fn task(&self) -> TaskRc {
        TaskRc(self.0.clone())
    }
}

impl<T> Future for TaskORc<T> {
    type Output = T;

    /// Provides a polling mechanism for the type, forwarding the polling operation using the `poll_unpin` method. 
    /// 
    /// This implementation takes a pinned mutable reference to itself and a mutable reference to a `Context`, and it returns a `Poll` indicating the completion state of the asynchronous computation. 
    /// This function delegates the actual polling logic to another method, efficiently managing the asynchronous task's state within the defined context.
    /// 
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<T> {
        self.poll_unpin()
    }
}

/// Fuck Orphan Rule
#[derive(From, Into, Deref, Debug)]
pub struct TaskRc(pub Pin<Rc<dyn Task>>);

impl Clone for TaskRc {
    /// Creates a new instance of this type by cloning the underlying `Rc<dyn Task>`. 
    /// 
    /// This method provides a way to create a duplicate of an existing instance, ensuring that the reference count to the inner task is incremented properly. 
    /// This allows for shared ownership of the task across multiple instances while maintaining correct memory management and lifetimes as managed by Rust's `Rc` smart pointer.
    /// 
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Future + 'static> From<TaskTRc<T>> for TaskRc {
    /// Constructs an instance by converting a given `TaskTRc<T>` into this type. 
    /// 
    /// The function takes ownership of the `TaskTRc<T>`'s inner value, which is a `Pin<Rc<TaskT<T>>>`, and uses it to instantiate a new object of this type. 
    /// This conversion facilitates integration between these two task representations, enabling seamless usage within systems that specifically require this type.
    /// 
    fn from(value: TaskTRc<T>) -> Self {
        TaskRc(value.0)
    }
}

impl<T: Future + 'static> From<TaskORc<T>> for TaskRc {
    /// Converts a `TaskORc<T>` into a `TaskRc`. 
    /// This implementation defines a method which takes an instance of `TaskORc<T>` and returns a `TaskRc` by extracting the pinned reference-counted pointer from the input and constructing a `TaskRc` with it.
    fn from(value: TaskORc<T>) -> Self {
        TaskRc(value.0)
    }
}

impl TaskRc {
    /// Converts the internal strong reference of the struct into a weak reference. 
    /// 
    /// This function creates a `TaskWeak` instance from the current `TaskRc`, using the `PinWeak::downgrade` method on its pinned strong reference. 
    /// It enables the option to hold a reference to the task without preventing it from being deallocated when there are no more strong references, facilitating a non-owning handle that can be upgraded back to a `TaskRc` if needed, provided that the task still exists.
    /// 
    pub fn downgrade(self) -> TaskWeak {
        PinWeak::downgrade(self.0).into()
    }
    /// Implements a method to poll the task for completion. 
    /// 
    /// This function creates a clone of the `TaskRc`, and then calls the `poll_task` method on the underlying pinned reference-counted task. 
    /// The method typically evaluates the state of the task and returns a boolean indicating if the task is completed. 
    /// This provides a mechanism for checking if a background or asynchronous task has reached completion within a concurrent task management system.
    /// 
    pub fn poll_task(&self) -> bool{
        self.clone().0.poll_task()
    }
    #[inline(always)]
    /// Provides a method to notify a task and determine if it was successfully notified. 
    /// 
    /// This method attempts to notify the current task held within the `TaskRc` type by cloning itself and invoking the `notify` method on the internal task object. 
    /// It returns a `Result` indicating whether the notification was successful (`Ok(true)`) or encountered an error (`Err(())`), allowing callers to handle the result according to their needs in the context of task execution or scheduling.
    /// 
    pub fn notify(&self) -> Result<bool, ()> {
        self.clone().0.notify()
    }
}

#[derive(Clone, From, Into, Deref, Default)]
/// A wrapper around an optional weak reference to a `Task`. 
/// 
/// This structure encapsulates an `Option` containing a `PinWeak` reference to a dynamic `Task` trait. 
/// `PinWeak` enables safe, temporary references to objects within a pinning context, ensuring that the referenced task cannot be moved or invalidated unexpectedly while providing access to task-related operations when available. 
/// This design allows for efficient checking and operation on tasks without extending their lifetime unnecessarily.
/// 
pub struct TaskWeak(Option<PinWeak<dyn Task>>);

impl std::fmt::Debug for TaskWeak {
    /// Provides a custom implementation of the `fmt` method for formatting instances of the type. 
    /// 
    /// This method attempts to upgrade the weak reference encapsulated within the `TaskWeak` to a strong reference. 
    /// If the upgrade is successful, it outputs the formatted string of the resulting strong reference. 
    /// Otherwise, it outputs the string `"null"` to indicate that the upgrade failed, typically because the referred-to resource has been dropped.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(a) = self.upgrade() {
            write!(f, "{:?}", a)
        } else { write!(f, "null") }
    }
}

impl From<PinWeak<dyn Task>> for TaskWeak {
    /// Creates a `TaskWeak` from a given `PinWeak<dyn Task>`. 
    /// This function converts a `PinWeak` reference into an optional value and then wraps it into a `TaskWeak`, facilitating the safe handling of conditional ownership and potential deallocation of `Task` instances.
    fn from(value: PinWeak<dyn Task>) -> Self {
        Some(value).into()
    }
}

/// Fuck Orphan Rule
impl TaskWeak {
    pub fn new() -> Self { None.into() }
    #[inline(always)]
    /// Provides a method to attempt to upgrade a weak reference of a task to a strong reference. 
    /// 
    /// This method first checks if the inner `Option` is `Some`, and if so, it attempts to upgrade the contained weak reference into a strong reference of type `TaskRc`. 
    /// If the upgrade is successful, it converts the strong reference into a `TaskRc`, returning it wrapped in an `Option`. 
    /// If any step along this path fails, it returns `None`.
    /// 
    pub fn upgrade(&self) -> Option<TaskRc> {
        self.as_ref().and_then(|x| x.upgrade().map(|x| x.into()))
    }
    #[inline(always)]
    /// Provides a method to attempt notification of a task. 
    /// 
    /// This method attempts to upgrade the weak reference to a strong reference, allowing interaction with the underlying task. 
    /// If successful, it calls the `notify` method on the task, returning either `Ok(true)` or any result the task's `notify` method provides. 
    /// If the upgrade fails (i.e., if the task has already been dropped), it returns `Ok(false)`, indicating the task could not be notified.
    /// 
    pub fn notify(&self) -> Result<bool, ()> {
        self.upgrade().map(|x| x.notify()).unwrap_or(Ok(false))
    }
}
