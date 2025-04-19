use std::{future::Future, process::Output, rc::{Rc, Weak}, pin::Pin, cell::{RefCell, Cell, UnsafeCell}, borrow::BorrowMut, task::{Poll, Context, Waker}, ops::{Deref, DerefMut}};
use derive_more::{ From, Into, DebugCustom, Deref };
use pin_cell::{PinCell, PinMut};
use pin_weak::rc::PinWeak;

use crate::{debg, debg2, forward::future::taskrc::{TaskRc, TaskWeak}, warn};

use super::taskrc::{self, TaskTRc};

pub type Fut<'a, T> = PinCell<dyn Future<Output = T> + 'a>;

#[derive(DebugCustom)]
#[debug(fmt = "task#{}{}", id, r#"if self.is_ready() { "✓" } else { "" }"#)]
/// A structure that represents a task with an associated future in the synthesis process. 
/// 
/// It contains several fields to manage the state and execution of the task. 
/// 
/// 
/// The `id` field is a unique identifier for the task, which helps in tracking and managing multiple tasks. 
/// The `next` field is a weak reference to another task, allowing for non-owning links between tasks without preventing cleanup, supporting efficient task chaining or scheduling. 
/// The `result` field utilizes a `Cell` to store the polling state of the task's output, allowing for interior mutability so that the state can be updated even through shared references. 
/// The `fut` field, which uses `PinCell`, stores the pinned state of the future associated with this task, ensuring that the future is not moved in memory, thus maintaining safe interaction with asynchronous operations.
pub struct TaskT<T: Future + 'static> {
    pub id: usize,
    // Tasks are hooked when created !
    next: TaskWeak,
    pub result: Cell<Poll<T::Output>>,
    fut: PinCell<T>,
}


impl<T: Future  + 'static> TaskT<T> {
    /// Creates a new instance with a specified identifier and future, linking it into the task chain. 
    /// 
    /// It initializes the task by capturing the current task from a thread-local context, cloning this current task to maintain continuity of the task chain, and setting it back into the thread-local variable for future access. 
    /// The `result` field is initialized to `Poll::Pending` to indicate that the task has not yet completed or been executed, and the `fut` field is set with the provided future encapsulated in a pinned state, ensuring that the future's memory location remains constant throughout its lifetime. 
    /// This process effectively hooks the task into the broader asynchronous task management system by establishing its place in a linked series of tasks.
    /// 
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

/// Spawns a new asynchronous task from a given future. 
/// 
/// This function takes a future `fut` of type `T`, which must implement the `Future` trait and have a static lifetime. 
/// It creates a new task by generating a unique task identifier and initializing a `TaskT` object with the future. 
/// The task is then pinned and converted into a reference-counted task type, `TaskTRc<T>`. 
/// Once the task is created, it is immediately polled for execution using the `poll_task` method, allowing it to start processing its operations. 
/// The function returns the `TaskTRc<T>`, acting as a handle to the spawned task for further interaction or management.
/// 
pub fn spawn<T: Future + 'static>(fut: T) -> TaskTRc<T> {
    let res: TaskTRc<_> = Rc::pin(TaskT::new(generate_task_id(), fut)).into();
    res.poll_task();
    res
}


/// This code snippet defines a trait for managing asynchronous tasks within a concurrent system. 
/// 
/// 
/// The trait includes functions to check task readiness, execute polling, notify the system of task status changes, and retrieve the task's unique identifier. 
/// The `is_ready` function checks and returns whether the task is ready to be executed. 
/// The `poll_task` function attempts to perform the task and returns a success status, making use of a pinned reference-counted pointer to ensure memory safety during asynchronous operations. 
/// The `notify` function informs the system about updates or changes in the task's state, potentially influencing scheduling or execution flow, and it returns a `Result` indicating success or failure. 
/// The `id` function returns an identifier to uniquely identify each task within the system.
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
/// An optional static variable that potentially holds a reference-counted task. 
/// 
/// This mutable static variable is intended to store an instance of `TaskRc`, which allows shared access to a task object across different parts of the code. 
/// By being wrapped in an `Option`, it indicates that the variable can either store an existing task or represent a `None` state, implying the absence of a task. 
/// Care must be taken when accessing this mutable static to ensure safe concurrent usage due to Rust's usual restrictions on shared mutable state, especially since it risks data races without proper synchronization mechanisms.
/// 
static mut TOP_TASK: Option<TaskRc> = None;

/// Returns a weak reference to the current task. 
/// 
/// 
/// The function temporarily takes ownership of the current task stored in `CUR_TASK`, clones this reference, and then immediately restores the original task back. 
/// By performing this clone, it returns a weak reference to the current task, ensuring that the reference does not affect the task's lifecycle or ownership semantics.
pub fn currect_task() -> TaskWeak {
    let a = CUR_TASK.take();
    CUR_TASK.set(a.clone());
    a
}
/// Returns the current task's unique identifier. 
/// 
/// This function retrieves the current task from a thread-local storage, takes a temporary snapshot of the task, extracts its unique identifier, and then restores the task back to its original state in the thread-local storage. 
/// It ensures the accurate fetching of the task's identifier by leveraging shared ownership and safe concurrency management, making use of Rust's interior mutability features.
/// 
pub fn currect_task_id() -> usize {
    let a = CUR_TASK.take();
    let id = a.upgrade().unwrap().0.id();
    CUR_TASK.set(a);
    id
}
/// Retrieves the current task ID if available. 
/// 
/// This function temporarily takes the current task from a thread-local context, attempts to upgrade it to a strong reference, and if successful, retrieves the task's ID. 
/// The task is then reset back into the thread-local context. 
/// If the task is unavailable, the function returns 0, indicating the absence of a current task ID.
/// 
pub fn currect_task_id_opt() -> usize {
    let a = CUR_TASK.take();
    a.upgrade().map(|x| {
        let id = x.0.id();
        CUR_TASK.set(a);
        id
    }).unwrap_or(0)
}
/// Returns a cloned reference-counted task from the global, top-level task. 
/// 
/// It uses unsafe operations to clone and unwrap the task reference from `TOP_TASK`. 
/// The function assumes that `TOP_TASK` is set and non-null, directly cloning the task without checks for existing valid state or initializing conditions. 
/// This approach bypasses safe Rust's guarantee, reflecting a design choice where the responsibility for ensuring the validity and safety of `TOP_TASK` is managed externally.
/// 
pub fn top_task() -> TaskRc {
    unsafe { TOP_TASK.clone().unwrap() }
}
#[inline(always)]
/// Returns a boolean indicating whether the top task is ready. 
/// 
/// 
/// This function attempts to safely check the readiness of a top-level task by accessing a static variable, `TOP_TASK`, which might represent a globally defined task structure. 
/// `TOP_TASK` is accessed using an unsafe block, suggesting its nature of being a mutable static or an external variable. 
/// It uses the `as_ref` method to get an immutable reference, if available, and then applies the `map` function to call `is_ready` on the task. 
/// If the static variable is not accessible, or the reference is not valid, the function defaults to returning `false`, indicating that the top task is not ready.
pub fn top_task_ready() -> bool {
    unsafe{ TOP_TASK.as_ref() }.map(|x| x.is_ready()).unwrap_or(false)
}
/// Executes a given function within the context of a specified top-level task. 
/// 
/// It temporarily replaces the current top-level task reference with the provided `t` for the duration of the function `f`, then restores the original top-level task. 
/// The function `f` is called and its result is returned after restoring the original task. 
/// This setup allows for the management of task contexts, making sure that the task reference is switched only temporarily and that any operation within `f` recognizes the passed task `t` as the top-level task. 
/// The usage of unsafe code indicates direct manipulation of global state, which requires careful handling to maintain safety and correctness guarantees.
/// 
pub fn with_top_task<T>(t: TaskRc, f: impl FnOnce() -> T) -> T {
    let orig = unsafe { TOP_TASK.replace(t) };
    let result = f();
    unsafe { TOP_TASK = orig; }
    result
}

#[thread_local]
/// A static mutable variable that tracks the number of tasks currently active or being managed. 
/// 
/// This counter is defined with global scope and mutable access, allowing its value to be modified across different parts of the module or the program. 
/// However, as it uses unsafe Rust (`static mut`), caution is necessary to ensure race conditions or undefined behavior do not occur when accessing or modifying its value concurrently. 
/// Proper synchronization mechanisms, such as locks, should be implemented when manipulating this counter in multi-threaded environments.
/// 
static mut TASK_COUNTER : usize = 0;

/// Generates a unique task identifier by incrementing a global task counter. 
/// 
/// This function leverages an unsafe block to directly manipulate a global variable, `TASK_COUNTER`, ensuring each invocation yields a sequentially higher value. 
/// This approach is efficient for environments requiring fast and straightforward ID generation but necessitates caution due to the inherent risks associated with unsafe code, particularly concerning data races if accessed concurrently across threads.
/// 
pub fn generate_task_id() -> usize {
    unsafe {
        TASK_COUNTER += 1;
        TASK_COUNTER
    }
}

/// Returns the current number of tasks by accessing the `TASK_COUNTER` using an unsafe block. 
/// 
/// This function retrieves the value of `TASK_COUNTER`, a global counter that likely tracks the number of active or total tasks being handled. 
/// Employing an unsafe block indicates that `TASK_COUNTER` might involve shared mutable state or external operations that bypass Rust's usual safety checks, necessitating caution when using this function.
/// 
pub fn number_of_task() -> usize {
    unsafe { TASK_COUNTER }
}

impl <T: Future + 'static> Task for TaskT<T> {
    #[inline(always)]
    /// Determines if the task is ready to be executed by checking the current polling state stored in the `result`. 
    /// 
    /// It temporarily replaces the current polling state with `Poll::Pending`, checks if the previous state was ready, and then restores the original state. 
    /// The function returns a boolean indicating whether the task's previous state was ready for execution.
    /// 
    fn is_ready(&self) -> bool {
        let a = self.result.replace(Poll::Pending);
        let p = a.is_ready();
        self.result.set(a);
        p
    }
    /// Polls the asynchronous task contained within the structure to determine its readiness. 
    /// 
    /// The method first checks if the task is already complete by calling `is_ready()`, returning `true` if so. 
    /// If the task's ID is not current, it logs an attempt to poll and returns `false`. 
    /// The method uses an unsafe block to create a mutable pinned reference to the task's future, and then attempts to borrow and poll it within a newly created execution context. 
    /// A no-operation waker is employed to construct the context. 
    /// The method temporarily replaces the current task context, polls the future, then restores the previous task context, storing the result of the poll operation. 
    /// Finally, it updates the task's result and returns a boolean indicating the task's readiness.
    /// 
    fn poll_task(self: Pin<Rc<Self>>) -> bool {
        if self.is_ready() { return true; }
        if currect_task_id_opt() >= self.id {
            debg2!("TASK#{} attempted to poll TASK#{}", currect_task_id_opt(), self.id);
            return false;
        }
        
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

    /// Enables a task to notify the system about its readiness for execution. 
    /// 
    /// The method first attempts to poll the task using `poll_task`, which returns a boolean indicating if the task is in a ready state. 
    /// If the task is ready and the top-level task is not already in a ready state (checked via `top_task_ready`), it propagates the notification to the next task in the sequence, if available, by upgrading the weak reference and invoking its `notify` method. 
    /// If the top task is ready, the function returns an error to signal this condition. 
    /// Otherwise, it returns a success status reflecting whether the notification was successful.
    /// 
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

    /// Retrieves the unique identifier associated with a task instance. 
    /// 
    /// The function accesses the `id` field of a `TaskT` instance, returning an unsigned integer that serves as the identifier for this task. 
    /// This identifier can be used to distinguish between different task instances within the system.
    /// 
    fn id(&self) -> usize {
        self.id
    }
}

/// Defines functionality for a task that can be polled in a manner that does not require pinning. 
/// 
/// This trait extends the generic `Task` trait by introducing a method `poll_unpin` which attempts to resolve or advance the task without altering its pinned state. 
/// The `poll_unpin` method returns a `Poll<T>`, representing the task's readiness state, where `T` is the type of value produced upon successful completion of the task. 
/// This trait allows tasks to be designed with flexibility in their execution lifecycle, enabling them to be easily incorporated into environments that manage task states without additional allocation or state changes inherent to pinned tasks.
/// 
pub trait TaskO<T>: Task {
    fn poll_unpin(self: &Self) -> Poll<T>;
}

impl<T: Future + 'static> TaskO<T::Output> for TaskT<T> {
    /// Provides functionality for polling a task that is not pinned. 
    /// 
    /// This method checks if the task is ready by invoking `is_ready`. 
    /// If the task is indeed ready, it updates the `result` field to `Poll::Pending` and returns this updated state. 
    /// If the task is not ready, it simply returns `Poll::Pending`, effectively indicating that the task cannot make progress at this point and must be polled again in the future. 
    /// This behavior is integral within asynchronous task management, ensuring tasks only proceed when their prerequisites are satisfied.
    /// 
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



