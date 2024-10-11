use std::cmp::min;
use std::ops::Not;

use crate::expr::context::Context;
use crate::expr::Expr;
use crate::galloc::{AllocForExactSizeIter, AllocForStr, TryAllocForExactSizeIter};
use crate::parser::config::Config;
use crate::utils::F64;
use crate::{impl_op2, new_op1, new_op2, new_op2_opt, new_op3};
use derive_more::DebugCustom;
use itertools::izip;
use crate::value::Value;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Map(pub Option<&'static Expr>);

impl std::hash::Hash for Map {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.map(|x| x as *const Expr).hash(state);
    }
}

impl Map {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_expr("f"))
    }
    pub fn name() ->  &'static str {
        "list.map"
    }
}

impl std::fmt::Display for Map {
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(e) = self.0 {
            write!(f, "list.map #f:{:?}", e)
        } else {
            write!(f, "list.map")
        }
    }
}
impl Default for Map {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl crate::forward::enumeration::Enumerator1 for Map {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Enumerator, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl crate::expr::ops::Op1 for Map {
    fn cost(&self) -> usize { 1 }
    fn try_eval(&self, a1: Value) -> (bool, Value) {
        let e = self.0.unwrap();
        if let Value::ListStr(a) = a1 {
            let a = a.iter().map(|&x| {
                let ctx = Context::new(x.len(), vec![x.into()], vec![], Value::Null);
                e.eval(&ctx).to_str()
            }).galloc_scollect();
            (true, a.into())
        } else { (false, Value::Null)}
    }
}
