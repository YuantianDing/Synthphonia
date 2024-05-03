

use std::cell::UnsafeCell;

use itertools::Itertools;

use crate::{expr::{cfg::Cfg, context::Context, Expr}, text::parsing::TextObjData, utils::UnsafeCellExt, value::{Type, Value}};

use self::size::{VecEv, EV};

use super::executor::Executor;


pub mod substr;
pub mod all_eq;
pub mod size;
pub mod prefix;
// pub mod listsubseq;
// pub mod len;

pub struct Data {
    pub size: size::Data,
    pub all_eq: all_eq::Data,
    pub substr: Option<UnsafeCell<substr::Data>>,
    // pub listsubseq: listsubseq::Data,
    // pub len: Option<len::Data>,
    pub to: TextObjData,
} 

impl Data {
    pub fn new(cfg: & Cfg, ctx: & Context) -> Vec<Self> {
        cfg.iter().map(|nt| {
            Self {
                size: size::Data::new(cfg),
                all_eq: all_eq::Data::new(),
                substr: substr::Data::new(ctx.output, cfg.config.substr_limit),
                // listsubseq: listsubseq::Data::new(ctx.output, (0..listsubseq_sample).collect_vec().as_slice() ),
                // len: if nt.ty != Type::ListStr { None } else { Some(len::Data::new()) },
                to: TextObjData::new()
            }
        }).collect_vec()
    }

    #[inline(always)]
    pub fn update(&self, exec: &'static Executor, e: Expr, v: Value) -> Result<Option<&'static Expr>, ()> {
        if let Some(e) = self.all_eq.set(v, e) {
            if let Some(s) = &self.substr { unsafe{ s.as_mut() }.update(v); }
            // self.listsubseq.update(v)?;
            // if let Some(l) = &self.len { l.update(v)? };
            self.to.update(exec, e, v);
            Ok(Some(e))
        } else {
            Ok(None)
        }
    }
}
