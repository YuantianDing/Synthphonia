use std::{cmp::min, sync::Arc};

use derive_more::DebugCustom;
use crate::{forward::{enumeration::Enumerator3, executor::Enumerator}, galloc::{AllocForExactSizeIter, AllocForStr}, impl_op3, new_op2, new_op3, parser::config::Config, value::Value};
use itertools::izip;

use super::Op3;

new_op2!(Eq, "=",
    (Int, Int) -> Bool { |(s1, s2)| s1 == s2 },
    (Str, Str) -> Bool { |(s1, s2)| s1 == s2 }
);

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct Ite(pub usize, pub bool);

impl Ite {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1), config.get_bool("enum").unwrap_or(false))
    }
    pub fn name() ->  &'static str {
        "ite"
    }
}
impl std::fmt::Display for Ite {
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Self::name().fmt(f) }
}
impl Default for Ite {
    fn default() -> Self { Self::from_config(&Default::default()) }
}
impl Enumerator3 for Ite {
    fn enumerate(&self, this: &'static super::Op3Enum, exec: Arc<Enumerator>, nt: [usize; 3]) -> Result<(), ()> {
        if !self.1 { return Ok(())}
        if exec.size() < self.cost() { return Ok(()); }
        let total = exec.size() - self.cost();
        for (i, (e1, v1)) in exec.data[nt[0]].size.get_all_under(total) {
            for (j, (e2, v2)) in exec.data[nt[1]].size.get_all_under(total - i) {
                for (_, (e3, v3)) in exec.data[nt[2]].size.get_all_under(total - i - j) {
                    let expr = super::Expr::Op3(this, e1, e2, e3);
                    if let (true, value) = self.try_eval(*v1, *v2, *v3) {
                        exec.clone().enum_expr(expr, value)?;
                    }
                }
            } 
        }
        Ok(())
    }
}

impl_op3!(Ite, "ite",
    (Bool, Int, Int) -> Int { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }},
    (Bool, Str, Str) -> Str { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }},
    (Bool, Bool, Bool) -> Bool { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }},
    (Bool, Float, Float) -> Float { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }}
);
