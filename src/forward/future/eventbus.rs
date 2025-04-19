use std::{task::{Poll, Context}, cell::{Cell, RefCell}, future::Future, rc::Rc, pin::Pin};
use derive_more::{ From, Into, DebugCustom, Deref };

use super::{taskrc::TaskWeak, task::currect_task};


/// A structure representing a publish-subscribe mechanism for handling events asynchronously. 
/// 
/// This type specializes in managing events that can be cloned, leveraging generics to remain flexible across different data types. 
/// 
/// 
/// Internally, it maintains a `result` field that uses a `Cell` to hold a `Poll<T>` state, enabling mutable access to the value without requiring `mut` dereferencing, which is useful for operations across asynchronous tasks. 
/// The `listeners` field, encapsulated within a `RefCell`, stores a vector of weak references to tasks (`TaskWeak`), allowing dynamic borrowing and modification of the listener list. 
/// This design facilitates efficient broadcasting of events to multiple subscribers by maintaining the state of awaited tasks, which are hooked when the `await` keyword is used.
pub struct EventBus<T: Clone> {
    pub result: Cell<Poll<T>>,
    // EventBus are hooked when `await` !
    listeners: RefCell<Vec<TaskWeak>>,
}


impl<T: Clone> EventBus<T> {
    /// Creates a new instance with an initial state that represents a pending poll and an empty list of listeners. 
    /// 
    /// The `result` field is initialized to `Poll::Pending`, indicating that no event has occurred yet. 
    /// The `listeners` field is initialized with an empty vector, ready to store weak references to tasks that are awaiting events. 
    /// This setup allows the `EventBus` to manage and notify multiple asynchronous tasks when an event occurs.
    /// 
    pub fn new() -> Self {
        Self {
            result: Poll::Pending.into(),
            listeners: Vec::new().into(),
        }
    }

    // pub fn new_cur_task() -> Self {
    //     Self { result: Poll::Pending.into(), listeners: vec![currect_task()].into() }
    // }
    /// Constructs a new instance with a pre-defined result. 
    /// 
    /// This initializes the `EventBus` by creating a new instance where the `result` is immediately set to a pre-determined value wrapped in a `Poll::Ready`. 
    /// The `listeners`, which presumably keep track of tasks waiting for an event, are initialized as an empty vector wrapped in a `RefCell`, ready to store future listeners. 
    /// This setup is useful for scenarios where an event is already completed or predetermined at the instantiation, allowing listeners to be notified immediately when hooked.
    /// 
    pub fn new_ready(result: T) -> EventBus<T> {
        Self { result: Poll::Ready(result).into(), listeners: Vec::new().into() }
    }
    
    #[inline(always)]
    /// Checks whether the result in the EventBus is in a ready state. 
    /// 
    /// It temporarily replaces the current state of the `result` with `Poll::Pending` to inspect its readiness without altering its state permanently. 
    /// The original state is restored afterward, ensuring the check is non-destructive. 
    /// This method returns a boolean indicating whether the `Poll<T>` stored in `result` is ready, reflecting whether an associated computation is complete.
    /// 
    pub fn is_ready(&self) -> bool {
        let a = self.result.replace(Poll::Pending);
        let p = a.is_ready();
        self.result.set(a);
        p
    }
    #[inline(always)]
    /// Notifies all listeners of the `EventBus`. 
    /// 
    /// The function retrieves the current listeners from the `listeners` field, which is a `RefCell` containing a vector of `TaskWeak` instances. 
    /// It replaces this vector with a new one to take ownership of the current listener set. 
    /// The function then iterates over each listener, invoking the `notify` method on them and ensuring that each notification call succeeds, returning an `Ok(())` if all listeners are successfully notified, or an error if any of the notifications fail.
    /// 
    fn notify(&self) -> Result<(), ()> {
        let mut vec = self.listeners.replace(Vec::new());
        vec.iter().try_for_each(|x| { x.notify()?; Ok(())} )?;
        Ok(())
    }
    #[inline(always)]
    /// Sets a new value in the `EventBus` and notifies listeners. 
    /// 
    /// This function updates the `result` field by setting it to `Poll::Ready` with the provided value. 
    /// After setting the value, it calls a notify function to inform all registered listeners about the update. 
    /// If successful, the function returns `Ok(())`; otherwise, it returns an error indicated by `Err(())`.
    /// 
    pub fn set_value(&self, value: T) -> Result<(), ()> {
        self.result.set(Poll::Ready(value));
        self.notify()
    }
    #[inline(always)]
    /// Returns a clone of the current result stored in the `EventBus`. 
    /// 
    /// This function retrieves the current polling state by replacing it temporarily with `Poll::Pending`, then sets the previous state back into the `result` field. 
    /// The function ensures that the stored polling state is not altered, providing the caller with the most recent state while maintaining the internal consistency of the `result` field.
    /// 
    pub fn result(&self) -> Poll<T> {
        let a = self.result.replace(Poll::Pending);
        self.result.set(a.clone());
        a
    }
    /// Adds a listener to the event bus. 
    /// 
    /// This operation appends a given weak reference to a task into the list of listeners maintained by the event bus. 
    /// The method asserts that the event bus is not in a ready state before adding the listener, ensuring that listeners are only added when the event bus is actively awaiting.
    /// 
    pub fn add_listener(&self, task: TaskWeak) {
        debug_assert!(!self.is_ready());
        self.listeners.borrow_mut().push(task);
    }
    /// Adds the current task to the `EventBus` as a listener. 
    /// 
    /// This function retrieves the current task using the `currect_task()` function and then adds it to the listeners of the `EventBus`. 
    /// By doing this, the current task is subscribed to receive notifications or results from the `EventBus`, enabling it to react when the event it is waiting for is triggered or completed.
    /// 
    pub fn add_cur_task(&self) {
        self.add_listener(currect_task());
    }
}

#[derive(From, Into, Deref)]
/// A structure that encapsulates a reference-counted event bus with pinning. 
/// 
/// It is designed to hold an `Rc` wrapped reference to an `EventBus` that manages events of type `T`, where `T` must implement the `Clone` trait. 
/// The use of `Pin` indicates that the stored `EventBus` is intended to maintain a fixed memory location, which is crucial for ensuring the integrity of the events' lifecycle semantics, particularly in scenarios involving asynchronous programming or when the event bus needs to interact with other pinned data structures. 
/// This design allows for safe and efficient event management and dispatching in contexts where shared ownership of the event bus is necessary.
/// 
pub struct EventBusRc<T: Clone>(Pin<Rc<EventBus<T>>>);

impl<T: Clone> EventBusRc<T> {
    /// Creates a new `EventBusRc` instance. 
    /// 
    /// This function initializes a new `EventBus` and wraps it in a `Rc` with `Pin`, ensuring that the `EventBus` cannot be moved in memory, and then converts it into an `EventBusRc`. 
    /// This construction allows the `EventBus` to be managed with reference counting, making it safe to share across multiple parts of the program while maintaining its position in memory.
    /// 
    pub fn new() -> EventBusRc<T> {
        Rc::pin(EventBus::new()).into()
    }
    
    /// Creates a new instance of `EventBusRc` that is immediately ready with the given result. 
    /// 
    /// The function takes a result of type `T` and constructs an `EventBus` already prepared with this result. 
    /// It then returns a pinned reference-counted version of this `EventBus` wrapped in an `EventBusRc`. 
    /// This method essentially sets up an `EventBusRc` with a pre-determined event result, allowing for immediate interaction and reducing the need for further initialization before use.
    /// 
    pub fn new_ready(result: T) -> EventBusRc<T> {
        Rc::pin(EventBus::new_ready(result)).into()
    }
}

impl<T:Clone> Clone for EventBusRc<T> {
    /// Provides a clone method for duplicating an `EventBusRc` instance. 
    /// 
    /// This method creates a new instance of `EventBusRc` that contains a clone of the inner `Rc<EventBus<T>>`, thereby maintaining shared ownership of the `EventBus` data. 
    /// The use of `Rc::clone` ensures that the reference count is incremented properly, allowing for safe, concurrent use of the same `EventBus` among multiple `EventBusRc` instances.
    /// 
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> Future for EventBusRc<T> {
    type Output = T;

    /// Implements polling for the EventBusRc type, checking if the event bus is ready. 
    /// 
    /// If it is ready, it temporarily replaces the current poll result with `Poll::Pending`, clones the previous result back in place, and returns it. 
    /// If it is not ready, the current task is added to the listeners, and it returns `Poll::Pending` to indicate that it is not complete. 
    /// This method effectively manages the state of the event bus, ensuring that tasks are appropriately scheduled and notified when they are ready.
    /// 
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