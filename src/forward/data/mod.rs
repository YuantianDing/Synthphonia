

use std::cell::{RefCell, UnsafeCell};

use itertools::Itertools;

use crate::{expr::{cfg::Cfg, context::Context, Expr}, text::parsing::TextObjData, utils::UnsafeCellExt, value::{Type, Value}};

use self::size::{VecEv, EV};

use super::executor::Executor;

/// Term Dispatcher for SubString
pub mod substr;

/// Term Dispatcher for Equal
pub mod all_eq;

/// Term Dispatcher for Size
pub mod size;

/// Term Dispatcher for Prefix
pub mod prefix;

/// Term Dispatcher for Contains
pub mod contains;
/// Term Dispatcher for Len
pub mod len;

/// All Term Dispatchers
pub struct Data {
    pub size: size::Data,
    pub all_eq: all_eq::Data,
    pub substr: Option<UnsafeCell<substr::Data>>,
    pub prefix: Option<UnsafeCell<prefix::Data>>,
    pub contains: Option<contains::Data>,
    pub len: Option<UnsafeCell<len::Data>>,
    pub to: TextObjData,
    pub new_ev: RefCell<Vec<(&'static Expr, Value)>>,
} 

impl Data {
    /// Create a instance of all term dispatchers
    pub fn new(cfg: & Cfg, ctx: & Context) -> Vec<Self> {
        cfg.iter().enumerate().map(|(i, nt)| {
            Self {
                size: size::Data::new(cfg),
                all_eq: all_eq::Data::new(),
                substr: substr::Data::new(ctx.output, cfg.config.substr_limit),
                prefix: prefix::Data::new(ctx.output, usize::MAX),
                contains: contains::Data::new(ctx.output.len(), nt.ty),
                len: if nt.ty != Type::ListStr && cfg[i].get_op1("list.map").is_some() { None } else { Some(len::Data::new().into()) },
                to: TextObjData::new(),
                new_ev: Vec::<(&'static Expr, Value)>::new().into()
            }
        }).collect_vec()
    }
    /// Get substr dispatcher
    pub fn substr(&self) -> Option<&mut substr::Data> {
        self.substr.as_ref().map(|a| unsafe { a.as_mut() } )
    }
    /// Get prefix dispatcher
    pub fn prefix(&self) -> Option<&mut prefix::Data> {
        self.prefix.as_ref().map(|a| unsafe { a.as_mut() } )
    }
    /// Get len dispatcher
    pub fn len(&self) -> Option<&mut len::Data> {
        self.len.as_ref().map(|a| unsafe { a.as_mut() } )
    }
    
    #[inline(always)]
    /// Add an new expression and value pair into the term dispatcher
    pub fn update(&self, exec: &'static Executor, e: Expr, v: Value) -> Result<Option<&'static Expr>, ()> {
        let new_ev = std::mem::take(&mut *self.new_ev.borrow_mut());
        for (e,v) in new_ev {
            self.all_eq.set_ref(v, e);
        }

        if let Some(e) = self.all_eq.set(v, e) {
            if let Some(s) = self.substr() { s.update(v, exec); }
            if let Some(s) = self.prefix() { s.update(v, exec); }
            if let Some(l) = self.len() { l.update(v, exec); };
            if let Some(c) = self.contains.as_ref() { c.update(v); }
            // self.listsubseq.update(v)?;
            self.to.update(exec, e, v);
            Ok(Some(e))
        } else {
            Ok(None)
        }
    }
    pub fn add_ev(&self, e: &'static Expr, v: Value) {
        self.new_ev.borrow_mut().push((e, v));
    }
}
