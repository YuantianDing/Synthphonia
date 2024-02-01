
use derive_more::{From, Into, Deref, DerefMut, Display};
use futures::{future::select, FutureExt};
use futures_core::Future;


#[derive(From, Into, Deref, DerefMut, Debug, Display, PartialEq, PartialOrd, Clone, Copy)]
pub struct F64(pub f64);

impl Eq for F64 {}

impl std::hash::Hash for F64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


use std::cell::UnsafeCell;

use ext_trait::extension;


#[extension(pub trait UnsafeCellExt)]
impl<T> UnsafeCell<T> {
    unsafe fn as_mut(&self) -> &mut T {
        &mut *self.get()
    }
    fn replace(&self, v : T) -> T {
        std::mem::replace(unsafe { self.as_mut() }, v)
    }
}

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

pub fn select_ret<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin) -> impl Future<Output=T> + Unpin {
    select(f1, f2).map(|a| {
        match a {
            futures::future::Either::Left(a) => a.0,
            futures::future::Either::Right(a) => a.0,
        }
    })
}
pub fn select_ret3<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin, f3: impl Future<Output=T> + Unpin) -> 
    impl Future<Output = T> {
    select_ret(f1, select_ret(f2, f3))
}

pub async fn pending_if<T>(condition: bool, fut: impl Future<Output=T>) -> T {
    if condition { fut.await } else { crate::never!() }
}


#[macro_export]
macro_rules! async_clone {
    ( [$( $x:ident )*] $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            async move { $y }
        }
    };
}

#[macro_export]
macro_rules! rebinding {
    (ref $x:ident) => { let $x = &$x; };
    (clone $x:ident) => { let $x = $x.clone(); };
    (move $x:ident) => { let $x = $x; };
    (mut $x:ident) => { let $x = &mut $x; };
}

#[macro_export]
macro_rules! closure {
    ( $( $x:ident $v:ident ),*; $y:expr ) => {
        {
            $(crate::rebinding!($x $v); )*
            { $y }
        }
    };
}
#[macro_export]
macro_rules! async_closure {
    ( [$( $x:expr );*] $y:expr ) => {
        {
            $(crate::rebinding!($x); )*
            async move { $y }
        }
    };
}

#[macro_export]
macro_rules! never {
    () => {
        futures::future::pending().await
    };
}






