use std::{
    cell::UnsafeCell,
    collections::{hash_map, HashMap},
    task::Poll, ops::Index,
};

use derive_more::{Constructor, Deref, From, Into, TryInto};

use crate::{
    galloc::AllocForAny,
    expr::Expr,
    forward::future::{task::currect_task, eventbus::{EventBus, EventBusRc}},
    utils::UnsafeCellExt,
    value::Value, log, info, debg,
};


#[derive(From, TryInto)]
pub enum DataStatus {
    Expr(&'static Expr),
    Event(EventBusRc<&'static Expr>),
}

impl DataStatus {
    #[inline(always)]
    fn try_notify(&self, e: Expr) -> Result<Option<&'static Expr>, ()> {
        match self {
            DataStatus::Event(a) if !a.is_ready() => {
                let e = e.galloc();
                a.set_value(e)?;
                Ok(Some(e))
            }
            _ => Ok(None),
        }
    }
    fn get_event(&self) -> EventBusRc<&'static Expr> {
        match self {
            DataStatus::Event(a) => a.clone(),
            DataStatus::Expr(e) => EventBusRc::new_ready(e),
        }
    }
}

#[derive(From, Deref)]
pub struct Data(UnsafeCell<HashMap<Value, DataStatus>>);

impl Data {
    pub fn new() -> Self { Self(HashMap::new().into()) }
    #[inline(always)]
    pub fn count(&self) -> usize { unsafe{self.as_mut().len()} }
    #[inline(always)]
    pub fn set(&self, v: Value, e: Expr) -> Result<Option<&'static Expr>, ()> {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(o) => o.get().try_notify(e),
            hash_map::Entry::Vacant(v) => {
                let e = e.galloc();
                v.insert(e.into());
                Ok(Some(e))
            }
        }
    }

    #[inline(always)]
    pub fn acquire(&self, v: Value) -> EventBusRc<&'static Expr> {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(o) => o.get().get_event(),
            hash_map::Entry::Vacant(v) => {
                let ev = EventBusRc::new();
                v.insert(ev.clone().into());
                ev
            }
        }
    }
    #[inline(always)]
    pub fn acquire_is_first(&self, v: Value) -> (bool, EventBusRc<&'static Expr>) {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(o) => (false, o.get().get_event()),
            hash_map::Entry::Vacant(v) => {
                let ev = EventBusRc::new();
                v.insert(ev.clone().into());
                (true, ev)
            }
        }
    }
    #[inline(always)]
    pub fn try_acquire(&self, v: Value) -> Option<EventBusRc<&'static Expr>> {
        match unsafe{ self.as_mut().entry(v) } {
            hash_map::Entry::Occupied(o) => {
                match o.get() {
                    DataStatus::Event(e) if !e.is_ready() => Some(e.clone()),
                    _ => None
                }
            }
            hash_map::Entry::Vacant(v) => {
                let ev = EventBusRc::new();
                v.insert(ev.clone().into());
                Some(ev)
            }
        }
    }
    pub fn get(&self, index: Value) -> &'static Expr {
        match unsafe{ &self.as_mut()[&index] } {
            DataStatus::Expr(e) => e,
            DataStatus::Event(e) => {
                match e.result() {
                    Poll::Ready(e) => e,
                    Poll::Pending => panic!("no such entry"),
                }
            }
        }
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
