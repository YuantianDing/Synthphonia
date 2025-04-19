use enum_dispatch::enum_dispatch;

use crate::{debg2, galloc::AllocForAny, parser::problem::FunSig, value::{ConstValue, Value}};


/// Program running context
pub mod context;

/// Context-free grammar
pub mod cfg;

/// Operators
pub mod ops;

use derive_more::DebugCustom;

use self::{context::Context, ops::{Op1, Op1Enum, Op2, Op2Enum, Op3, Op3Enum}};
#[derive(DebugCustom, PartialEq, Eq, Clone, Hash)]
/// Expressions, statically referenced.
/// 
/// These expressions include constants, variables, and operations, each of which is encapsulated in the `Expr` enum. 
/// Constants are represented by the `Const` variant, holding a `ConstValue`. 
/// Variables are stored as a 64-bit integer with the `Var` variant. 
/// Operations are categorized into unary, binary, and ternary, represented by `Op1`, `Op2`, and `Op3` respectively, each associated with an operation enumeration and the relevant sub-expressions. 
/// 
/// 
/// The enum variants include formatting annotations used in debugging to display the expressions in a readable format, enhancing the ability to trace and verify expression states during execution. 
/// These representations ensure the flexibility and power required for manipulating and evaluating expressions within the synthesis tasks.
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
    /// Evaluates the expression within a given context to produce a `Value`. 
    pub fn eval(&self, ctx: &Context) -> Value {
        
        match self {
            Expr::Const(c) => c.value(ctx.len()),
            Expr::Var(index) => ctx[*index],
            Expr::Op1(op1, a1) => op1.eval(a1.eval(ctx)),
            Expr::Op2(op2, a1, a2) => op2.eval(a1.eval(ctx), a2.eval(ctx)),
            Expr::Op3(op3, a1, a2, a3) => op3.eval(a1.eval(ctx), a2.eval(ctx), a3.eval(ctx)),
        }
    }
    /// Calculates the cost of an expression. 
    pub fn cost(&self) -> usize {
        match self {
            Expr::Const(c) => 1,
            Expr::Var(index) => 1,
            Expr::Op1(op1, a1) => op1.cost() + a1.cost(),
            Expr::Op2(op2, a1, a2) => op2.cost() + a1.cost() + a2.cost(),
            Expr::Op3(op3, a1, a2, a3) => op3.cost() + a1.cost() + a2.cost() + a3.cost(),
        }
    }
    /// Determines whether an expression contains another expression. 
    pub fn contains(&self, other: &Expr) -> bool {
        if self == other { true } 
        else {
            match self {
                Expr::Const(_) => false,
                Expr::Var(_) => false,
                Expr::Op1(_, e1) => e1.contains(other),
                Expr::Op2(_, e1, e2) => e1.contains(other) || e2.contains(other),
                Expr::Op3(_, e1, e2, e3) => e1.contains(other) || e2.contains(other) || e3.contains(other),
            }
        }
    }
    /// Formats an expression into a string representation. 
    pub fn format(&self, sig: &FunSig) -> String {
        match self {
            Expr::Const(c) => format!("{:?}", c),
            Expr::Var(index) => sig.args[*index as usize].0.clone(),
            Expr::Op1(op1, a1) => format!("({} {})", op1, a1.format(sig)),
            Expr::Op2(op2, a1, a2) => format!("({} {} {})", op2, a1.format(sig), a2.format(sig)),
            Expr::Op3(op3, a1, a2, a3) => format!("({} {} {} {})", op3, a1.format(sig), a2.format(sig), a3.format(sig)),
        }
    }
    /// Construct a ternary expression that represents an if-then-else operation within the context of the `Expr` enum. 
    pub fn ite(&'static self, t: &'static Expr, f: &'static Expr) -> &'static Expr {
        crate::expr!(Ite {self} {t} {f}).galloc()
    }
    /// Converts an `Expr` into an `Expression`. 
    pub fn to_expression(&self) -> Expression {
        match self {
            Expr::Const(c) => Expression::Const(*c),
            Expr::Var(v) => Expression::Var(*v),
            Expr::Op1(op, a1) => Expression::Op1((*op).clone(), a1.to_expression().into()),
            Expr::Op2(op, a1, a2) => Expression::Op2((*op).clone(), a1.to_expression().into(), a2.to_expression().into()),
            Expr::Op3(op, a1, a2, a3) => Expression::Op3((*op).clone(), a1.to_expression().into(), a2.to_expression().into(), a3.to_expression().into()),
        }
    }
}

#[derive(DebugCustom, PartialEq, Eq, Clone, Hash)]
/// Expressions, owned.
pub enum Expression {
    #[debug(fmt = "{:?}", _0)]
    Const(ConstValue),
    #[debug(fmt = "<{:?}>", _0)]
    Var(i64),
    #[debug(fmt = "({} {:?})", _0, _1)]
    Op1(Op1Enum, Box<Expression>),
    #[debug(fmt = "({} {:?} {:?})", _0, _1, _2)]
    Op2(Op2Enum, Box<Expression>, Box<Expression>),
    #[debug(fmt = "({} {:?} {:?} {:?})", _0, _1, _2, _3)]
    Op3(Op3Enum, Box<Expression>, Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Converts an `Expression` into a statically allocated `Expr` reference. 
    pub fn alloc_local(self) -> &'static Expr {
        match self {
            Expression::Const(a) => Expr::Const(a).galloc(),
            Expression::Var(v) => Expr::Var(v).galloc(),
            Expression::Op1(op1, a1) => Expr::Op1(op1.galloc(), a1.alloc_local()).galloc(),
            Expression::Op2(op1, a1, a2) => Expr::Op2(op1.galloc(), a1.alloc_local(), a2.alloc_local()).galloc(),
            Expression::Op3(op1, a1, a2, a3) => Expr::Op3(op1.galloc(), a1.alloc_local(), a2.alloc_local(), a3.alloc_local()).galloc(),
        }
    }
}

#[macro_export]
macro_rules! expr_no_use {
    ($l:literal) => { $crate::expr::Expr::Const($crate::const_value!($l))};
    ([$l:literal]) => { $crate::expr::Expr::Var($l)};
    ({$l:expr}) => { $l };
    ($op:ident $a1:tt) => { 
        crate::expr::Expr::Op1(Op1Enum::$op($op::default()).galloc(), crate::expr![$a1].galloc())
    };
    ($op:ident $a1:tt $a2:tt) => { 
        crate::expr::Expr::Op2(Op2Enum::$op($op::default()).galloc(), crate::expr![$a1].galloc(), crate::expr![$a2].galloc())
    };
    ($op:ident $a1:tt $a2:tt $a3:tt) => {
        crate::expr::Expr::Op3(Op3Enum::$op($op::default()).galloc(), crate::expr![$a1].galloc(), crate::expr![$a2].galloc(), crate::expr![$a3].galloc())
    };
    ( ($( $inner:tt )*) ) => { $crate::expr_no_use!($($inner)*) };
}

#[macro_export]
/// Macro to create of expressions in the Synthphonia module. 
macro_rules! expr {
    ( $( $inner:tt )*) => { {
        use $crate::galloc::AllocForAny;
        use $crate::expr::ops::str::*;
        use $crate::expr::ops::*;
        use $crate::expr::ops::float::*;
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


