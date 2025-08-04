use crate::{expr::ops::Op1Enum, galloc, value::{consts_to_value, ConstValue, Value}};


trait FormattingOp where Self: Sized {
    fn format(&self, input: &'static str) -> Option<(Self, ConstValue, &'static str)>;
    fn union(self, other: Self) -> Option<Self>;
    fn bad_value() -> ConstValue;
    fn format_all(&self, input: &'static [&'static str]) -> Option<(Self, Value, Value, Value)> {
        let mut a = Vec::with_capacity(input.len());
        let mut b = galloc::new_bvec(input.len());
        let mut cond = galloc::new_bvec(input.len());
        let mut newop: Option<Self> = None;
        for i in input {
            if let Some((op, x, y)) = self.format(i) {
                if let Some(no) = newop {
                    newop = no.union(op);
                    newop.as_ref()?;
                } else { newop = Some(op); }
                a.push(x);
                b.push(y);
                cond.push(true);
            } else {
                a.push(Self::bad_value());
                b.push(i);
                cond.push(false);
            }
        }
        newop.map(|no| (no, consts_to_value(a), Value::Str(b.into_bump_slice()), cond.into_bump_slice().into()))
    }
}

use ext_trait::extension;
pub mod int;
pub use int::*;
pub mod float;
pub use float::*;
pub mod time;
pub use time::*;
pub mod month;
pub use month::*;

pub mod weekday;
pub use weekday::*;

#[macro_export]
macro_rules! for_all_formatting_op {
    () => {
        _do!(FormatInt);
        _do!(FormatFloat);
        _do!(FormatTime);
        _do!(FormatMonth);
        _do!(FormatWeekday);
    };
}


impl Op1Enum {
    pub fn is_formatting_op(&self) -> bool {
        macro_rules! _do {($op:ident) => {
            if let Self::$op(_) = self { return true }
        };}
        crate::for_all_formatting_op!();
        false
    }
    pub fn format_all(&self, input: &'static [&'static str]) -> Option<(Op1Enum, Value, Value, Value)> {
        macro_rules! _do {($op:ident) => {
            if let Self::$op(op) = self { return op.format_all(input).map(|(a, b, c, d)| (a.into(), b, c, d)); }
        };}
        crate::for_all_formatting_op!();
        panic!();
    }
}

#[macro_export]
macro_rules! impl_formatop {
    ($opname:ident, $t:ident, $costf:expr) => {
        impl $crate::expr::ops::Op1 for $opname {
            fn cost(&self) -> usize {
                $costf(self)
            }
            fn try_eval(&self,a1:$crate::value::Value) -> (bool, $crate::value::Value) {
                match a1 {
                    Value::$t(s) => (true, Value::Str(s.iter().map(|s1| {
                        self.format_single(*s1).galloc_str()
                    }).galloc_scollect())),
                    _ => (false, Value::Null),
                }
            }
        }
    };
}