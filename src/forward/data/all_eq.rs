use std::{
    cell::UnsafeCell,
    collections::{hash_map},
    task::Poll, ops::Index,
};

use derive_more::{Constructor, Deref, From, Into, TryInto};
use futures::Future;
use rc_async::sync::broadcast::MaybeReady;

use crate::{
    galloc::AllocForAny,
    expr::Expr,
    utils::UnsafeCellExt,
    value::Value, log, info, debg,
};
use ahash::AHashMap as HashMap;

#[derive(From, Deref)]
pub struct Data(UnsafeCell<HashMap<Value, MaybeReady<&'static Expr>>>);

impl Data {
    pub fn new() -> Self { Self(HashMap::new().into()) }

    #[inline(always)]
    pub fn count(&self) -> usize { unsafe{self.as_mut().len()} }

    #[inline(always)]
    pub fn set(&self, v: Value, e: Expr) -> Option<&'static Expr> {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(mut p) => {
                if p.get().is_ready() {
                    None
                } else {
                    let e = e.galloc();
                    p.get_mut().set(e);
                    Some(e)
                }
            }
            hash_map::Entry::Vacant(v) => {
                let e = e.galloc();
                v.insert(MaybeReady::Ready(e));
                Some(e)
            }
        }
    }

    #[inline(always)]
    pub fn set_ref(&self, v: Value, e: &'static Expr) {
        let mut sd = None;
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(mut p) => {
                if !p.get().is_ready() { sd = p.get_mut().sender(e); }
            }
            hash_map::Entry::Vacant(v) => {
                v.insert(MaybeReady::Ready(e));
            }
        }
        sd.map(|x| x.send(e));
    }

    #[inline(always)]
    pub async fn acquire(&self, v: Value) -> &'static Expr {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(o) => o.get().get().await,
            hash_map::Entry::Vacant(v) => v.insert(MaybeReady::pending()).get().await,
        }
    }

    #[inline(always)]
    pub fn is_pending(&self, v: Value) -> bool {
        if let Some(a) = unsafe{ self.as_mut().get(&v) } {
            !a.is_ready()
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn contains<'a>(&'a self, v: Value) -> bool {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(o) => true,
            hash_map::Entry::Vacant(v) => false,
        }
    }
    // #[inline(always)]
    // pub fn try_acquire(&self, v: Value) -> Option<EventBusRc<&'static Expr>> {
    //     match unsafe{ self.as_mut().entry(v) } {
    //         hash_map::Entry::Occupied(o) => {
    //             match o.get() {
    //                 DataStatus::Event(e) if !e.is_ready() => Some(e.clone()),
    //                 _ => None
    //             }
    //         }
    //         hash_map::Entry::Vacant(v) => {
    //             let ev = EventBusRc::new();
    //             v.insert(ev.clone().into());
    //             Some(ev)
    //         }
    //     }
    // }
    pub fn at(&self, index: Value) -> Option<&'static Expr> {
        unsafe{ &self.as_mut().get(&index) }.and_then(|x| {
            x.poll_opt()
        })
    }
    pub fn get(&self, index: Value) -> &'static Expr {
        self.at(index).expect("No such entry")
    }
}

// thread_local!{
//     static DATA : UnsafeCell<HashMap<Value, DataStatus>> = HashMap::new().into();
// }

// #[inline(always)]
// pub fn set(v: Value, e: Expr) -> bool {
//     DATA.with(|data|{
//         match unsafe { data.as_mut().entry(v)} {
//             hash_map::Entry::Occupied(o) => {
//                 o.get().try_notify(e)
//             }
//             hash_map::Entry::Vacant(v) => {
//                 v.insert(e.galloc().into());
//                 true
//             }
//         }
//     })
// }

// #[inline(always)]
// pub fn listen_to(v: Value) -> EventBusRc<&'static Expr> {
//     DATA.with(|data|{
//         match unsafe{ data.as_mut().entry(v) } {
//             hash_map::Entry::Occupied(o) => o.get().try_add_task(),
//             hash_map::Entry::Vacant(v) => {
//                 let ev = EventBusRc::new_cur_task();
//                 v.insert(ev.clone().into());
//                 ev
//             }
//         }
//     })
// }
