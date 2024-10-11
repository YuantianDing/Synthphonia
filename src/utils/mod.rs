
use derive_more::{From, Into, Deref, DerefMut, Display, DebugCustom};
use futures::{future::select, FutureExt};
use futures_core::Future;

pub mod join;
pub mod nested;
pub mod fut;

#[derive(From, Into, Deref, DerefMut, DebugCustom, Display, PartialEq, PartialOrd, Clone, Copy)]
#[debug(fmt = "{:?}", _0)]
#[display(fmt = "{:?}", _0)]
pub struct F64(pub f64);
impl F64 {
    pub fn new(value: f64) -> Self {
        Self((value * 1e10).round() / 1e10)
    }
    pub fn from_usize(value: usize) -> Self {
        Self(value as f64)
    }
}
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

#[extension(pub trait TryRetain)]
impl<T> Vec<T> {
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







