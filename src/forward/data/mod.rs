

use std::{cell::{RefCell, UnsafeCell}, sync::Arc};

use itertools::Itertools;
use spin::Mutex;

use crate::{expr::{cfg::Cfg, context::Context, Expr}, utils::UnsafeCellExt, value::{Type, Value}};

use self::size::{VecEv, EV};

use super::executor::Enumerator;


pub mod substr;
pub mod all_eq;
pub mod size;
pub mod prefix;
// pub mod contains;
pub mod len;

pub struct Data {
    pub size: size::Data,
    pub all_eq: all_eq::Data,
    pub substr: Option<Mutex<substr::Data>>,
    pub prefix: Option<Mutex<prefix::Data>>,
    // pub listsubseq: listsubseq::Data,
    pub len: Option<Mutex<len::Data>>,
    // pub to: Mutex<TextObjData>,
    pub new_ev: Mutex<Vec<(&'static Expr, Value)>>,
} 

impl Data {
    pub fn new(cfg: & Cfg, ctx: & Context) -> Vec<Self> {
        cfg.iter().enumerate().map(|(i, nt)| {
            Self {
                size: size::Data::new(cfg),
                all_eq: all_eq::Data::new(),
                substr: substr::Data::new(ctx.output, cfg.config.substr_limit),
                prefix: prefix::Data::new(ctx.output, usize::MAX),
                // listsubseq: listsubseq::Data::new(ctx.output, (0..listsubseq_sample).collect_vec().as_slice() ),
                len: if nt.ty != Type::ListStr && cfg[i].get_op1("list.map").is_some() { None } else { Some(len::Data::new().into()) },
                // to: TextObjData::new().into(),
                new_ev: Vec::<(&'static Expr, Value)>::new().into()
            }
        }).collect_vec()
    }
    pub fn substr(&self) -> Option<spin::MutexGuard<'_, substr::Data, >> {
        self.substr.as_ref().map(|a| a.lock() )
    }
    pub fn prefix(&self) -> Option<spin::MutexGuard<'_, prefix::Data, >> {
        self.prefix.as_ref().map(|a| a.lock() )
    }
    pub fn len(&self) -> Option<spin::MutexGuard<'_, len::Data, >> {
        self.len.as_ref().map(|a| a.lock() )
    }
    
    #[inline(always)]
    pub fn update(&self, exec: Arc<Enumerator>, e: Expr, v: Value) -> Result<Option<&'static Expr>, ()> {
        let new_ev = std::mem::replace(&mut *self.new_ev.lock(), Vec::new());
        for (e,v) in new_ev {
            self.all_eq.set_ref(v, e);
        }

        if let Some(e) = self.all_eq.set(v, e) {
            {
                if let Some(mut s) = self.substr() { s.update(v, exec.clone()); }
            }
            {
                if let Some(mut s) = self.prefix() { s.update(v, exec.clone()); }
            }
            {
                if let Some(mut l) = self.len() { l.update(v, exec.clone()); };
            }
            // self.listsubseq.update(v)?;
            // self.to.lock().update(exec, e, v);
            Ok(Some(e))
        } else {
            Ok(None)
        }
    }
    pub fn add_ev(&self, e: &'static Expr, v: Value) {
        self.new_ev.lock().push((e, v));
    }
}
