
use derive_more::{From, Into, Deref, DerefMut, Display, DebugCustom};
use futures::{future::select, FutureExt};
use futures_core::Future;

pub mod join;
pub mod nested;
#[derive(From, Into, Deref, DerefMut, DebugCustom, Display, PartialEq, PartialOrd, Clone, Copy)]
#[debug(fmt = "{:?}", _0)]
#[display(fmt = "{:?}", _0)]
/// A newtype wrapper encapsulating a 64-bit floating-point value. 
/// It provides an abstraction over the underlying primitive to support precise numerical handling.
/// 
/// This structure is designed to integrate seamlessly with various trait implementations, enabling convenient conversion, dereferencing, cloning, and formatted output operations based on its internal floating-point representation.
pub struct F64(pub f64);
impl F64 {
    /// Creates a new floating-point instance ensuring numerical precision by rounding the input to 10 decimal places.
    /// 
    /// Rounds the provided f64 value by multiplying it by 1e10, applying rounding, and then dividing by 1e10 to harmonize precision before encapsulating it within the new type.
    pub fn new(value: f64) -> Self {
        Self((value * 1e10).round() / 1e10)
    }
    /// Converts an unsigned integer into a floating-point representation encapsulated by the custom numeric wrapper. 
    /// 
    /// 
    /// Enables users to create a new instance of the f64-based type from a usize value by performing a straightforward type conversion.
    pub fn from_usize(value: usize) -> Self {
        Self(value as f64)
    }
}
impl Eq for F64 {}

impl std::hash::Hash for F64 {
    /// Computes a hash for the wrapped floating-point value based on its bit representation. 
    /// 
    /// 
    /// Hashes the underlying value by interpreting the bits of the inner floating-point as an integer and feeding it into the provided hasher state. 
    /// This ensures that two values with the same numerical representation produce identical hash codes.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


use std::cell::UnsafeCell;

use ext_trait::extension;


#[extension(pub trait UnsafeCellExt)]
impl<T> UnsafeCell<T> {
    /// Returns a mutable reference to the underlying data contained within an unsafe cell. 
    /// This function is marked as unsafe because it directly casts the internal pointer to a mutable reference, thereby bypassing Rust’s normal borrowing rules.
    unsafe fn as_mut(&self) -> &mut T {
        &mut *self.get()
    }
    /// Replaces the value contained within an internal cell with a new value while returning the previous content.
    /// 
    /// This function consumes a new value to update the internal state and returns the original value that was stored, effectively performing an atomic swap operation in the cell's memory.
    fn replace(&self, v : T) -> T {
        std::mem::replace(unsafe { self.as_mut() }, v)
    }
}

/// Awaits a collection of futures concurrently and returns the output of the first future that completes. 
/// 
/// 
/// Collects the input futures into a vector, then, if the collection is empty, it stalls by awaiting a pending future; otherwise, it races all the futures and returns the output from the one that finishes first.
pub async fn select_all<T>(
    futures: impl IntoIterator<Item = impl std::future::Future<Output = T>>,
) -> T {
    let futures = futures.into_iter().collect::<Vec<_>>();
    // Workaround against select_all's arbitrary assert
    if futures.is_empty() {
        return std::future::pending().await;
    }
    futures::future::select_all(futures.into_iter().map(Box::pin)).await.0
}

/// Selects between two futures and returns the output from the one that completes first. 
/// This function accepts two futures and, using a combinator, races them to yield the result of the first future that resolves.
pub fn select_ret<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin) -> impl Future<Output=T> + Unpin {
    select(f1, f2).map(|a| {
        match a {
            futures::future::Either::Left(a) => a.0,
            futures::future::Either::Right(a) => a.0,
        }
    })
}
/// Returns a future that resolves with the output of the first among three given futures to complete. 
/// This function concurrently races the three input futures and awaits the one that finishes first, effectively combining them into a single asynchronous computation whose result is that of the first completed future.
pub fn select_ret3<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin, f3: impl Future<Output=T> + Unpin) -> 
    impl Future<Output = T> {
    select_ret(f1, select_ret(f2, f3))
}
/// Returns the output of the first completed future among four concurrently evaluated futures. 
/// This function accepts four futures as parameters and returns a new future that resolves with the result of the first future to complete, ensuring a non-blocking selection process.
pub fn select_ret4<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin, f3: impl Future<Output=T> + Unpin, f4: impl Future<Output=T> + Unpin) -> 
    impl Future<Output = T> {
    select_ret(f1, select_ret(f2, select_ret(f3, f4)))
}
/// Returns a future that yields the output of the earliest completed one among five provided futures.
/// 
/// Selects five futures to run concurrently and returns a new future that resolves with the value from the first future to complete. 
/// This utility function chains the selections to ultimately combine all five inputs into a single asynchronous operation without blocking for the slower ones.
pub fn select_ret5<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin, f3: impl Future<Output=T> + Unpin, f4: impl Future<Output=T> + Unpin, f5: impl Future<Output=T> + Unpin) -> 
    impl Future<Output = T> {
    select_ret(f1, select_ret(f2, select_ret(f3, select_ret(f4, f5))))
}

/// Awaits a provided future if a condition holds, otherwise yields a pending future that never resolves. 
/// This function evaluates a boolean and, when true, awaits and returns the output of the supplied future; if false, it defers execution indefinitely by awaiting a future that remains pending.
pub async fn pending_if<T>(condition: bool, fut: impl Future<Output=T>) -> T {
    if condition { fut.await } else { crate::never!() }
}


#[macro_export]
/// Creates an asynchronous block that clones a collection of variables before evaluating a given expression. 
/// 
/// 
/// This macro accepts a list of identifiers enclosed in square brackets and an expression. 
/// It clones each specified variable and produces an async move block that evaluates the expression, thereby reducing boilerplate code when working with asynchronous closures that require cloned values.
macro_rules! async_clone {
    ( [$( $x:ident )*] $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            async move { $y }
        }
    };
}

#[macro_export]
/// Provides utility for rebinding a variable with different semantics. 
/// This macro accepts a pattern specifying the rebinding mode—ref, clone, move, or mut—followed by an identifier, and expands it into a let binding that creates a new binding corresponding to that mode (borrowing immutably, cloning, moving, or borrowing mutably).
macro_rules! rebinding {
    (ref $x:ident) => { let $x = &$x; };
    (clone $x:ident) => { let $x = $x.clone(); };
    (move $x:ident) => { let $x = $x; };
    (mut $x:ident) => { let $x = &mut $x; };
}

#[macro_export]
/// Creates a closure-like block that performs variable rebinding before evaluating an expression. 
/// This macro takes a comma-separated list of identifier pairs, where each pair specifies a rebinding using a helper mechanism, and a trailing expression; it expands to a block that first applies the rebinding operations and then evaluates the provided expression within that modified scope.
macro_rules! closure {
    ( $( $x:ident $v:ident ),*; $y:expr ) => {
        {
            $($crate::rebinding!($x $v); )*
            { $y }
        }
    };
}
#[macro_export]
/// Generates an asynchronous closure that rebinding the provided expressions and evaluates a given expression within the async context. 
/// This macro takes a list of expressions to be rebound using a helper rebinding macro, then returns an async move block that evaluates the specified expression while capturing the rebound variables.
macro_rules! async_closure {
    ( [$( $x:expr );*] $y:expr ) => {
        {
            $($crate::rebinding!($x); )*
            async move { $y }
        }
    };
}

#[macro_export]
/// Generates an asynchronous expression that yields a future which never resolves. 
/// This macro conditionally produces a pending future, either using the default type or a specified type parameter, blocking execution indefinitely when awaited.
macro_rules! never {
    () => { futures::future::pending().await };
    ($t:ty) => { futures::future::pending::<$t>().await };
}
#[extension(pub trait TryRetain)]
impl<T> Vec<T> {
    /// Filters vector elements in-place by applying a predicate function that can return an error. 
    /// 
    /// 
    /// Evaluates each element using the provided function, retaining or removing elements based on the returned boolean value when successful; if the function returns an error for an element, the error is recorded and the element is retained, with the final result reflecting any encountered error.
    fn try_retain<E, F>(&mut self, mut f: F) -> Result<(), E>
        where F: FnMut(&T) -> Result<bool, E> {
        
        let mut result = Ok(());
        self.retain(|v| match f(v) {
            Ok(b) => b,
            Err(e) => {
                result = Err(e);
                true
            }
        });

        result
    }
}







