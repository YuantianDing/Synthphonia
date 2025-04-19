use std::{
    cell::{RefCell, UnsafeCell},
    task::Poll,
};

use crate::{galloc::AllocForAny, utils::UnsafeCellExt};

use super::{taskrc::TaskWeak, task::currect_task};

use derive_more::{Deref, From, Into};
use futures_core::Future;

/// A structure that encapsulates the core behavior of a communication channel. 
/// 
/// It includes a `result`, which holds a `Poll<T>` to represent the result of an asynchronous computation. 
/// The generic type `T` must implement the `Clone` trait, allowing for its value to be duplicated when needed without transferring ownership. 
/// Additionally, it maintains a list of `listeners`, represented as a `Vec<TaskWeak>`, which are weak references to tasks that are interested in the channel's state. 
/// These listeners can be notified of changes to the channel state, enabling efficient task synchronization and coordination in an asynchronous environment. 
/// The combination of these elements allows for managing task communication and synchronization robustly.
/// 
pub struct ChannelInner<T: Clone> {
    result: Poll<T>,
    listeners: Vec<TaskWeak>,
}

impl<T: Clone> ChannelInner<T> {
    /// Creates a new instance of the encapsulating structure within a static context and returns a reference to it. 
    /// 
    /// This method initializes the structure with a pending poll result and an empty list of listeners, encapsulated within an `UnsafeCell`. 
    /// The `galloc` function is then used to manage the allocation of this cell, suggesting that it handles some global or static allocation to ensure the returned reference is valid for the entire duration of the application runtime. 
    /// This approach implies a specialized context where safe access assumptions or a particular memory management strategy is intended, typically needed in highly concurrent or system-level scenarios.
    /// 
    pub fn new() -> &'static UnsafeCell<Self> { UnsafeCell::new(Self { result: Poll::Pending, listeners: Vec::new() }).galloc() }

    #[inline(always)]
    /// Checks whether the associated result is ready. 
    /// 
    /// This function returns a boolean indicating the readiness of the result stored within the structure. 
    /// It inspects the `result` field of the type, which employs the `Poll` type, and calls its `is_ready` method to determine if the result is available, allowing consumers to proceed if the result is ready.
    /// 
    pub fn is_ready(&self) -> bool { self.result.is_ready() }
    #[inline(always)]
    /// Notifies all listeners registered with the channel. 
    /// 
    /// This function clones the current result stored in the channel and temporarily replaces the `listeners` vector with a new, empty one. 
    /// It then iterates over the previous list of listeners, attempting to notify each one. 
    /// For each listener, it reassigns the cloned result back to `self.result` and calls `notify()` on the listener. 
    /// If all notifications succeed, the function returns `Ok(())`; otherwise, it returns an error if any notification fails.
    /// 
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
    /// Sends a value to the channel and notifies all listeners of this event. 
    /// 
    /// The method sets the `result` field to `Poll::Ready` with the provided value, indicating that the channel is prepared with a new item. 
    /// It then calls `notify()` to inform all registered listeners of the change. 
    /// After notification, the `result` is reset to `Poll::Pending`, indicating that the channel is waiting for further actions or input. 
    /// The function returns a `Result`, which is `Ok(())` if the operation is successful, or an error `()` if the notification fails.
    /// 
    pub fn send(&mut self, value: T) -> Result<(), ()> {
        self.result = Poll::Ready(value);
        self.notify()?;
        self.result = Poll::Pending;
        Ok(())
    }
    /// Adds a listener to the channel's list of listeners. 
    /// 
    /// This function takes a weak reference to a task and appends it to the vector of listeners maintained by the channel. 
    /// This design allows the channel to notify tasks that are interested in the results or events associated with this channel. 
    /// By using weak references, it ensures that the tasks can be collected by the Rust garbage collector if they are no longer needed, preventing memory leaks even if the channel outlives some of its listeners.
    /// 
    pub fn add_listener(&mut self, task: TaskWeak) { self.listeners.push(task); }
    /// Adds the current task to the list of listeners. 
    /// 
    /// This function retrieves the current task using `currect_task()` and then adds it as a listener to the internal vector of `listeners`. 
    /// By doing so, it enables the task to be notified or updated based on the operations carried out within the channel, facilitating asynchronous control flow. 
    /// The function modifies the state of `ChannelInner` by appending the current task to manage event-driven interactions effectively. 
    /// 
    /// 
    pub fn add_cur_task(&mut self) { self.add_listener(currect_task()); }
}

#[derive(From, Into, Deref, Clone, Copy)]
/// A struct representing a communication channel. 
/// 
/// It is generic over a type `T`, which is required to implement the `Clone` trait and have a `'static` lifetime. 
/// This structure holds a reference to an `UnsafeCell` containing an instance of `ChannelInner<T>`. 
/// The use of `UnsafeCell` suggests that this channel will permit interior mutability, allowing the stored data to be mutated even when the struct is immutable. 
/// This design is likely used in scenarios that involve low-level concurrency or other situations where controlled unsafe operations are necessary to encapsulate and synchronize access to mutable data.
/// 
pub struct Channel<T: Clone + 'static>(&'static UnsafeCell<ChannelInner<T>>);

impl<T: Clone + 'static> std::fmt::Debug for Channel<T> {
    /// Formats a `Channel` instance for display using a pointer address. 
    /// 
    /// This implementation of the `fmt` method for `Channel<T>` writes a formatted string representation of the channel's internal pointer to the provided formatter, displaying it as a pointer address. 
    /// This can be useful for debugging, providing visibility into the memory address of the `Channel` instance. 
    /// The method returns a `std::fmt::Result` indicating the success or failure of the formatting operation.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:p}", self.0 as *const _)
    }
}


impl<T: Clone + 'static> Channel<T> {
    /// Creates a new instance of the channel. 
    /// 
    /// This function initializes a new channel by creating an underlying `ChannelInner` and converting it into a `Channel` type. 
    /// The `new` function leverages the default setup of `ChannelInner`, ensuring that the channel is ready to handle tasks and listeners when in use.
    /// 
    pub fn new() -> Self { ChannelInner::new().into() }
    /// Provides a method to retrieve a mutable reference to the inner contents of the Channel. 
    /// 
    /// This method accesses the underlying `ChannelInner` structure, which holds the result and the list of listeners. 
    /// It uses an unsafe block to perform a mutable borrow of the inner value from the `UnsafeCell`, circumventing Rust's usual borrowing rules. 
    /// This allows for direct manipulation of the channel's internal data, such as updating the result or managing listeners, but requires careful use to avoid violating Rust’s aliasing rules, as doing so could lead to undefined behavior.
    /// 
    pub fn get(&self) -> &mut ChannelInner<T> { unsafe { self.0.as_mut() } }
}

impl<T: Clone + 'static> Future for Channel<T> {
    type Output = T;

    #[inline(always)]
    /// Implements a polling mechanism for asynchronous operations associated with the `Channel`. 
    /// 
    /// In the `poll` method, the `ChannelInner` encapsulated by the `Channel` is accessed unsafely to check if it is ready. 
    /// If the result within the `ChannelInner` is ready, the method will replace the stored result with `Poll::Pending` and return the previously stored result. 
    /// If the result is not ready, the current task is added to the `listeners` to be notified later, and `Poll::Pending` is returned, indicating that the operation should be retried at a later time. 
    /// This mechanism supports concurrent task scheduling by coordinating access to shared resources and managing task wake-up notifications.
    /// 
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
    /// Determines if two channels are identical. 
    /// 
    /// This method compares the memory locations of the internal `ChannelInner` components encapsulated within the channels to determine identicality. 
    /// It verifies pointer equality, meaning it checks whether the two `Channel` instances point to the same memory address, thereby confirming they are the exact same instance, rather than simply having equivalent contents.
    /// 
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl<T: Clone + 'static> Eq for Channel<T> {}

impl<T: Clone + 'static> std::hash::Hash for Channel<T> {
    /// Hashes the Channel by leveraging the memory address of its internal `UnsafeCell`. 
    /// 
    /// The method takes a generic hasher `H` and a mutable reference to it as parameters. 
    /// Inside this function, `std::ptr::hash` is employed, which computes a hash value based on the pointer address of the `UnsafeCell` contained within the `Channel`. 
    /// This facilitates unique identification of the channel instance within hashing contexts, such as when added to a hash set or used as a key in a hashmap.
    /// 
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state)
    }
}
