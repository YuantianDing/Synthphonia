use std::task::Poll;

use futures::{future::select, FutureExt};
use futures_core::Future;
use smol::channel;

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
pub fn select_ret4<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin, f3: impl Future<Output=T> + Unpin, f4: impl Future<Output=T> + Unpin) -> 
    impl Future<Output = T> {
    select_ret(f1, select_ret(f2, select_ret(f3, f4)))
}
pub fn select_ret5<T>(f1: impl Future<Output=T> + Unpin, f2: impl Future<Output=T> + Unpin, f3: impl Future<Output=T> + Unpin, f4: impl Future<Output=T> + Unpin, f5: impl Future<Output=T> + Unpin) -> 
    impl Future<Output = T> {
    select_ret(f1, select_ret(f2, select_ret(f3, select_ret(f4, f5))))
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
    () => { futures::future::pending().await };
    ($t:ty) => { futures::future::pending::<$t>().await };
}

#[derive(Clone)]
pub enum MaybeReady<T: Clone> {
    Ready(T),
    Pending((async_broadcast::Sender<T>, async_broadcast::Receiver<T>)),
}

impl<T: Clone + std::fmt::Debug> MaybeReady<T> {
    pub fn pending() -> Self {
        Self::Pending(async_broadcast::broadcast::<T>(2))
    }
    pub fn pending_on(sdrv: (async_broadcast::Sender<T>, async_broadcast::Receiver<T>)) -> Self {
        Self::Pending(sdrv)
    }
    pub fn ready(t: T) -> Self {
        Self::Ready(t)
    }
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }
    pub fn set(&mut self, t: T) {
        if let Self::Pending(ref sd) = self {
            sd.0.try_broadcast(t.clone());
        }
        *self = Self::Ready(t);
    }
    pub fn sender(&mut self, t: T) -> Option<async_broadcast::Sender<T>> {
        let res = if let Self::Pending(ref sd) = self {
            Some(sd.0.clone())
        } else { None };
        *self = Self::Ready(t);
        res
    }
    pub fn poll(&self) -> Poll<T> {
        match self {
            MaybeReady::Ready(a) => Poll::Ready(a.clone()),
            MaybeReady::Pending(_) => Poll::Pending,
        }
    }
    pub fn poll_opt(&self) -> Option<T> {
        match self {
            MaybeReady::Ready(a) => Some(a.clone()),
            MaybeReady::Pending(_) => None,
        }
    }
    pub async fn get(&self) -> T {
        match self {
            MaybeReady::Ready(a) => a.clone(),
            MaybeReady::Pending(sender) => sender.1.clone().recv().await.unwrap(),
        }
    }
}