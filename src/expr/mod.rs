use enum_dispatch::enum_dispatch;

use crate::value::{ConstValue, Value};


pub mod context;
pub mod cfg;
pub mod ops;

use derive_more::DebugCustom;

use self::{context::Context, ops::{Op1Enum, Op2Enum, Op3Enum}};
#[derive(DebugCustom, PartialEq, Eq, Clone)]
pub enum Expr {
    #[debug(fmt = "{:?}", _0)]
    Const(ConstValue),
    #[debug(fmt = "<{:?}>", _0)]
    Var(i64),
    #[debug(fmt = "({} {:?})", _0, _1)]
    Op1(&'static Op1Enum, &'static Expr),
    #[debug(fmt = "({} {:?} {:?})", _0, _1, _2)]
    Op2(&'static Op2Enum, &'static Expr, &'static Expr),
    #[debug(fmt = "({} {:?} {:?} {:?})", _0, _1, _2, _3)]
    Op3(&'static Op3Enum, &'static Expr, &'static Expr, &'static Expr),
}

impl Expr {
    pub fn eval(&self, ctx: &Context) -> Value {
        match self {
            Expr::Const(c) => c.value(ctx.len()),
            Expr::Var(index) => ctx[*index],
            Expr::Op1(op1, a1) => op1.eval(a1.eval(ctx)),
            Expr::Op2(op2, a1, a2) => op2.eval(a1.eval(ctx), a2.eval(ctx)),
            Expr::Op3(op3, a1, a2, a3) => op3.eval(a1.eval(ctx), a2.eval(ctx), a3.eval(ctx)),
        }
    }
}

#[macro_export]
macro_rules! expr_no_use {
    ($l:literal) => { crate::expr::Expr::Const(crate::const_value!($l))};
    ([$l:literal]) => { crate::expr::Expr::Var($l)};
    ({$l:expr}) => { $l };
    ($op:ident $a1:tt) => { 
        crate::expr::Expr::Op1(Op1Enum::$op($op::default()).galloc(), expr![$a1].galloc())
    };
    ($op:ident $a1:tt $a2:tt) => { 
        crate::expr::Expr::Op2(Op2Enum::$op($op::default()).galloc(), expr![$a1].galloc(), expr![$a2].galloc())
    };
    ($op:ident $a1:tt $a2:tt $a3:tt) => {
        crate::expr::Expr::Op3(Op3Enum::$op($op::default()).galloc(), expr![$a1].galloc(), expr![$a2].galloc(), expr![$a3].galloc())
    };
    ( ($( $inner:tt )*) ) => { $crate::expr_no_use!($($inner)*) };
}

#[macro_export]
macro_rules! expr {
    ( $( $inner:tt )*) => { {
        use $crate::galloc::AllocForAny;
        use $crate::expr::ops::str::*;
        use $crate::expr::ops::*;
        $crate::expr_no_use!($($inner)*) 
    }};
}

#[cfg(test)]
mod tests {
    use crate::{value::Value, galloc, expr::{ops::str::{Replace, Concat}, context::Context}, const_value};
    use crate::galloc::AllocForAny;

    #[test]
    fn test1() {
        let input = const_value!("938-242-504").value(1);
        let output = const_value!("938.242.504").value(1);
        let ctx = Context::new(1, vec![input], vec![], output);
        let e = expr!{ (Replace (Replace [0] "-" ".") "-" ".") };
        assert_eq!(e.eval(&ctx), output);
    }
}


