

use itertools::Itertools;

use crate::{expr::{cfg::Cfg, context::Context, Expr}, text::parsing::TextObjData, value::{Type, Value}};

use self::size::{VecEv, EV};

use super::executor::Executor;



pub mod substr;
pub mod all_eq;
pub mod size;
pub mod listsubseq;
pub mod len;

pub struct Data {
    pub size: size::Data,
    pub all_eq: all_eq::Data,
    pub substr: substr::Data,
    pub listsubseq: listsubseq::Data,
    pub len: Option<len::Data>,
    pub to: TextObjData,
} 

impl Data {
    pub fn new(cfg: & Cfg, ctx: & Context) -> Vec<Self> {
        let substr_sample = std::cmp::min(ctx.output.len(), cfg.config.substr_samples);
        let listsubseq_sample = std::cmp::min(ctx.output.len(), cfg.config.listsubseq_samples);
        cfg.iter().map(|nt| {
            let substr_sample = if !nt.start || nt.ty != Type::Str { 0 } else { substr_sample };
            let listsubseq_sample = if nt.ty != Type::ListStr { 0 } else { listsubseq_sample };
            Self {
                size: size::Data::new(cfg),
                all_eq: all_eq::Data::new(),
                substr: substr::Data::new(ctx.output, (0..substr_sample).collect_vec().as_slice()),
                listsubseq: listsubseq::Data::new(ctx.output, (0..listsubseq_sample).collect_vec().as_slice() ),
                len: if nt.ty != Type::ListStr { None } else { Some(len::Data::new()) },
                to: TextObjData::new()
            }
        }).collect_vec()
    }
    #[inline(always)]
    pub fn update(&self, exec: &'static Executor, e: Expr, v: Value) -> Result<Option<&'static Expr>, ()> {
        if let Some(e) = self.all_eq.set(v, e)? {
            self.substr.update(v)?;
            self.listsubseq.update(v)?;
            if let Some(l) = &self.len { l.update(v)? };
            self.to.update(exec, e, v);
            Ok(Some(e))
        } else {
            Ok(None)
        }
    }
}
