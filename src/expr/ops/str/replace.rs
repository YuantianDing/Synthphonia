use std::cmp::min;

use crate::{
    expr::{ops::Op3, Expr}, forward::enumeration::Enumerator3, galloc::{AllocForExactSizeIter, AllocForStr}, new_op3, parser::config::Config, value::Value
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Replace(pub usize, pub usize);

impl Replace {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1), config.get_usize("enum_replace_cost").unwrap_or(3))
    }
    pub fn name() -> &'static str {
        "str.replace"
    }
}

impl std::fmt::Display for Replace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Self::name().fmt(f)
    }
}

impl Default for Replace {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator3 for Replace {
    fn enumerate(&self, this: &'static crate::expr::ops::Op3Enum, exec: &'static crate::forward::executor::Executor, nt: [usize; 3]) -> Result<(), ()> {
        if exec.size() < self.cost() { return Ok(()); }
        let total = exec.size() - self.cost();
        for (i, (e2, v2)) in exec.data[nt[0]].size.get_all_under(min(total, self.1)) {
            for (j, (e3, v3)) in exec.data[nt[1]].size.get_all_under(min(total - i, self.1)) {
                for (e1, v1) in exec.data[nt[2]].size.get_all(total - i - j) {
                    let expr = Expr::Op3(this, e1, e2, e3);
                    if let (true, value) = self.try_eval(*v1, *v2, *v3) {
                        exec.enum_expr(expr, value)?;
                    }
                }
            } 
        }
        Ok(())
    }
}

impl Op3 for Replace {
    fn cost(&self) -> usize {
        self.0
    }
    fn try_eval(&self, a1: Value, a2: Value, a3: Value) -> (bool, Value) {
        match (a1, a2, a3) {
            (Value::Str(s1), Value::Str(s2), Value::Str(s3)) => (true, Value::Str(
                itertools::izip!(s1.iter(), s2.iter(), s3.iter())
                    .map(|(s1, s2, s3)| &*s1.replacen(*s2, s3, 1).galloc_str())
                    .galloc_scollect(),
            )),
            _ => (false, Value::Null),
        }
    }
}
