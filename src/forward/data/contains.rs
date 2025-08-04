use std::{cell::UnsafeCell, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use futures::StreamExt;
use itertools::Itertools;
use simple_rc_async::sync::broadcast::{self, Sender};

use crate::{utils::UnsafeCellExt, value::{Type, Value}};


pub type ListStr = &'static [&'static str];
/// Determines whether all elements of the first sequence appear in order within the second sequence. 
/// 
/// The function iterates over the second sequence while maintaining an iterator over the first sequence. 
/// For each element encountered in the second sequence, if it matches the next expected element from the first sequence, the iterator advances. 
/// The process terminates early and returns true if all elements of the first sequence have been matched; otherwise, it returns false.
/// 
pub fn listsubseq(that: ListStr, this: ListStr) -> bool {
    let mut iter = that.iter().peekable();
    for item in this {
        match iter.peek() {
            Some(i)  if i == &item => { iter.next(); }
            Some(_) => (),
            None => return true,
        }
    }
    return iter.peek() == None;
}

pub type ListData = HashMap<String, Vec<broadcast::Sender<Value>>>;

/// Term dispatcher for contains
pub struct Data(UnsafeCell<Vec<ListData>>);

impl Data {

    pub fn new(len: usize, ty: Type) -> Option<Self> {
        if let Type::ListStr = ty {
            Some(Data(vec![HashMap::new(); len].into()))
        } else { None }
    }
    fn get(&self) -> &mut Vec<ListData> {
        unsafe { self.0.as_mut() }
    }
    pub fn update(&self, value: Value) -> () {
        if let Value::ListStr(ls) = value {
            let mut iter = ls.iter().zip(self.get().iter());
            let mut senders = HashMap::<broadcast::Sender<Value>, usize>::new();
            let (sl0, data0) = iter.next().unwrap();
            let mut position = 1;
            for s in sl0.iter() {
                if let Some(a) = data0.get(*s) {
                    for sd in a {
                        senders.insert(sd.clone(), position);
                    }
                }
            }
            if senders.is_empty() { return; }
            
            for (sl, data) in iter {
                position <<= 1;
                for s in sl.iter() {
                    if let Some(a) = data.get(*s) {
                        for sd in a {
                            if let Some(mask) = senders.get_mut(sd) {
                                *mask |= position;
                            }
                        }
                    }
                }
            }

            for (sd, mask) in senders {
                if mask >= (1 << ls.len()) - 1 {
                    sd.send(value);
                }
            }
        }
    }

    pub fn listen_at(&self, value: Value) -> broadcast::Reciever<Value> {
        if let Value::ListStr(ls) = value {
            let sd = broadcast::channel();
            for (sl, data) in ls.iter().zip(self.get().iter_mut()) {
                for s in sl.iter() {
                    if let Some(a) = data.get_mut(*s) {
                        a.push(sd.clone());
                    } else {
                        data.insert((*s).to_string(), vec![sd.clone()]);
                    }
                }
            }
            sd.reciever()
        } else if let Value::Str(s) = value {
            let sd = broadcast::channel();
            for (s, data) in s.iter().zip(self.get().iter_mut()) {
                if let Some(a) = data.get_mut(*s) {
                    a.push(sd.clone());
                } else {
                    data.insert(s.to_string(), vec![sd.clone()]);
                }
            }
            sd.reciever()
        } else { panic!("Unsupported Type for Contains") }
    }

    #[inline(always)]
    pub async fn listen_for_each<T>(&self, value: Value, mut f: impl FnMut(Value) -> Option<T>) -> T {
        let mut rv = self.listen_at(value);
        loop {
            if let Some(t) = f(rv.next().await.unwrap()) { return t; }
        }
    }
}



