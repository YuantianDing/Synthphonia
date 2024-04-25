use enum_dispatch::enum_dispatch;
use itertools::Itertools;



// pub mod simple;
// use simple::*;

use crate::{expr::{cfg::ProdRule, ops::{Op1, Op1Enum, Op2, Op2Enum, Op3, Op3Enum}, Expr}, galloc::AllocForAny};
use ext_trait::extension;
use super::executor::Executor;


pub trait Enumerator1 : Op1 {
    #[inline(always)]
    fn enumerate(&self, this: &'static Op1Enum, exec: &'static Executor, opnt: [usize; 1]) -> Result<(), ()> {
        enumerate1(self, this, exec, opnt)
    }
}
#[inline(always)]
pub fn enumerate1(s: &impl Op1, this: &'static Op1Enum, exec: &'static Executor, opnt: [usize; 1]) -> Result<(), ()> {
    if exec.size() <= s.cost() { return Ok(()); }
    for (e, v) in exec.data[opnt[0]].size.get_all(exec.size() - s.cost()) {
        let expr = Expr::Op1(this, *e);
        if let (true, value) = s.try_eval(*v) {
            exec.enum_expr(expr, value)?;
        }
    }
    Ok(())
}

pub trait Enumerator2 : Op2 {
    #[inline(always)]
    fn enumerate(&self, this: &'static Op2Enum, exec: &'static Executor, nt: [usize; 2]) -> Result<(), ()> {
        enumerate2(self, this, exec, nt)
    }
}
#[inline(always)]
pub fn enumerate2(s: &impl Op2, this: &'static Op2Enum, exec: &'static Executor, nt: [usize; 2]) -> Result<(), ()> {
    if exec.size() <= s.cost() { return Ok(()); }
    let total = exec.size() - s.cost();
    for (i, (e1, v1)) in exec.data[nt[0]].size.get_all_under(total) {
        for (e2, v2) in exec.data[nt[1]].size.get_all(total - i) {
            let expr = Expr::Op2(this, *e1, *e2);
            if let (true, value) = s.try_eval(*v1, *v2) {
                exec.enum_expr(expr, value)?;
            }
        }
    }
    Ok(())
}

pub trait Enumerator3 : Op3 {
    #[inline(always)]
    fn enumerate(&self, this: &'static Op3Enum, exec: &'static Executor, nt: [usize; 3]) -> Result<(), ()> {
        enumerate3(self, this, exec, nt)
    }
}
#[inline(always)]
pub fn enumerate3(s: &impl Op3, this: &'static Op3Enum, exec: &'static Executor, nt: [usize; 3]) -> Result<(), ()> {
    if exec.size() < s.cost() { return Ok(()); }
    let total = exec.size() - s.cost();
    for (i, (e1, v1)) in exec.data[nt[0]].size.get_all_under(total) {
        for (j, (e2, v2)) in exec.data[nt[1]].size.get_all_under(total - i) {
            for (e3, v3) in exec.data[nt[2]].size.get_all(total - i - j) {
                let expr = Expr::Op3(this, e1, e2, e3);
                if let (true, value) = s.try_eval(*v1, *v2, *v3) {
                    exec.enum_expr(expr, value)?;
                }
            }
        } 
    }
    Ok(())
}

impl Enumerator1 for Op1Enum {
    #[inline]
    fn enumerate(&self, this: &'static Op1Enum, exec: &'static Executor, opnt: [usize; 1]) -> Result<(), ()> {
        macro_rules! _do {($($op:ident)*) => {$(
            if let Self::$op(a) = self {
                return a.enumerate(this, exec, opnt);
            }
        )*};}
        crate::for_all_op1!();
        panic!()
    }
}

impl Enumerator2 for Op2Enum {
    #[inline]
    fn enumerate(&self, this: &'static Op2Enum, exec: &'static Executor, opnt: [usize; 2]) -> Result<(), ()> {
        macro_rules! _do {($($op:ident)*) => {$(
            if let Self::$op(a) = self {
                return a.enumerate(this, exec, opnt);
            }
        )*};}
        crate::for_all_op2!();
        panic!()
    }
}

impl Enumerator3 for Op3Enum {
    #[inline]
    fn enumerate(&self, this: &'static Op3Enum, exec: &'static Executor, opnt: [usize; 3]) -> Result<(), ()> {
        macro_rules! _do {($($op:ident)*) => {$(
            if let Self::$op(a) = self {
                return a.enumerate(this, exec, opnt);
            }
        )*};}
        crate::for_all_op3!();
        panic!()
    }
}

#[extension(pub trait ProdRuleEnumerate)]
impl ProdRule {
    fn enumerate(&self, exec: &'static Executor) -> Result<(), ()> {
        match self {
            ProdRule::Const(c) => {
                if exec.size() == 1 {
                    exec.enum_expr(Expr::Const(*c), c.value(exec.ctx.len()))?;
                }
                Ok(())
            }
            ProdRule::Var(v) => {
                if exec.size() == 1 {
                    exec.enum_expr(Expr::Var(*v), exec.ctx.get(*v).unwrap().clone())?;
                }
                Ok(())
            }
            ProdRule::Op1(op1, nt1) => {
                op1.enumerate(op1, exec, [*nt1])
            }
            ProdRule::Op2(op2, nt1, nt2) => {
                op2.enumerate(op2, exec, [*nt1, *nt2])
            }
            ProdRule::Op3(op3, nt1, nt2, nt3) => {
                op3.enumerate(op3, exec, [*nt1, *nt2, *nt3])
            }
            ProdRule::Nt(_) => todo!(),
        }
    }
}


// #[derive(From, Into, Deref)]
// pub struct EnumerationCfg(Vec<Vec<EnumeratorEnum>>);

// impl EnumerationCfg {
//     pub fn from_cfg(cfg: & Cfg, ctx: &Context) -> Self {
//         cfg.iter().enumerate().map(|(i,x)| {
//             // if x.ty == Type::Bool && !cfg.condition_search { return Vec::new(); }
//             let first = EnumeratorEnum::from(TextObjEnumerator(i));
//             let mut v = vec![first];
//             v.append(&mut x.rules.iter().map(|r| EnumeratorEnum::from_rule(r.clone(), cfg, ctx)).collect_vec());
//             v
//     }).collect_vec().into()
//     }
//     pub fn enumerate(&self, data: &[Data], size: usize, nt: usize, mut f: impl EnumFn) -> Result<(), ()> {
//         this.0[nt].iter().try_for_each(|e| e.enumerate(data, size, &mut f))
//     }
// }


// #[enum_dispatch]
// pub trait Enumerator {
//     fn enumerate(&self, data: &[Data], size: usize, f: impl EnumFn) -> Result<(), ()>;
// }



// #[enum_dispatch(Enumerator)]
// pub enum EnumeratorEnum { NoEnumerator, TextObjEnumerator, Single, Enumerator1, Enumerator2, Enumerator3 }

// impl EnumeratorEnum {
//     fn from_rule(value: ProdRule, graph: &Cfg, ctx: &Context) -> Self {
//         match value {
//             ProdRule::Const(c) => {
//                 Single::new(Expr::Const(c), c.value(ctx.len()), 1).into()
//             }
//             ProdRule::Var(v) => {
//                 if v >= 0 { return NoEnumerator.into(); }
//                 Single::new(Expr::Var(v), ctx.get(v).unwrap().clone(), 1).into()
//             }
//             ProdRule::Op1(op1, nt1) => {
//                 // if op1.name().ends_with(".parse") { return NoEnumerator.into(); }
//                 // if op1.name().ends_with(".fmt") { return NoEnumerator.into(); }
//                 Enumerator1::new(op1.clone(), nt1, op1.cost()).into()
//             }
//             ProdRule::Op2(op2, nt1, nt2) => {
//                 if cfg.get_bool("condsearchonly").unwrap_or(false) && !graph.condition_search {
//                     return NoEnumerator.into();
//                 }
//                 Enumerator2::new(op2.clone(), (nt1, nt2), op2.cost()).into()
//             }
//             ProdRule::Op3(op3, nt1, nt2, nt3) => {
//                 if let Op3Enum::Ite(_) = op3 {
//                     return NoEnumerator.into();
//                 }
//                 Enumerator3::new(op3, (nt1, nt2, nt3), get_opsize(cfg, graph.condition_search)).into()
//             }
//             ProdRule::Nt(_, _) => todo!(),
//         }
//     }
// }

// fn get_opsize(cfg: Config, cond_search: bool) -> usize {
//     if cond_search {
//         cfg.get_i64("cond_opsize").unwrap_or(get_opsize(cfg, false) as i64) as usize
//     } else {
//         cfg.get_i64("opsize").unwrap_or(1) as usize
//     }
// }
