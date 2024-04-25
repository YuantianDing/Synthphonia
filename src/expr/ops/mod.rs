use enum_dispatch::enum_dispatch;

use super::*;
use crate::parser::config::Config;
use crate::{value::Value, expr};
use std::future::Future;
use std::path::Display;

pub mod str;
use self::context::Context;
pub use self::str::*;

use crate::text::parsing::*;
use crate::text::formatting::*;
pub mod base;
pub use self::base::*;

pub mod int;
pub use self::int::*;
pub mod float;
pub use self::float::*;

pub mod list;
pub use self::list::*;


pub mod date;
pub use date::*;
pub mod macros;

#[enum_dispatch]
pub trait Op1: Clone + std::fmt::Display {
    fn cost(&self) -> usize;
    fn try_eval(&self, a1: Value) -> (bool, Value);
}

impl Op1Enum {
    pub fn eval(&self, a1: Value) -> Value {
        let a = self.try_eval(a1).1;
        a
    }
}

#[enum_dispatch]
pub trait Op2 : Clone + std::fmt::Display {
    fn cost(&self) -> usize;
    fn try_eval(&self, a1: Value, a2: Value) -> (bool, Value);
}

impl Op2Enum {
    pub fn eval(&self, a1: Value, a2: Value) -> Value { self.try_eval(a1, a2).1 }
}

#[enum_dispatch]
pub trait Op3 : Clone + std::fmt::Display {
    fn cost(&self) -> usize;
    fn try_eval(&self, a1: Value, a2: Value, a3: Value) -> (bool, Value);
}

impl Op3Enum {
    pub fn eval(&self, a1: Value, a2: Value, a3: Value) -> Value { self.try_eval(a1, a2, a3).1 }
}

#[enum_dispatch(Op1)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op1Enum {
    Len,
    ToInt,
    ToStr,
    Neg,
    IsPos,
    IsZero,
    IsNatural,
    RetainLl,
    RetainLc,
    RetainN,
    RetainL,
    RetainLN,
    Uppercase,
    Lowercase,
    AsMonth,
    AsDay,
    AsYear,
    AsWeekDay,
    ParseTime,
    ParseDate,
    ParseInt,
    ParseMonth,
    ParseWeekday,
    ParseFloat,
    FormatInt,
    FormatFloat,
    FormatTime,
    FormatMonth,
    FormatWeekday,
    FNeg,
    FAbs,
    FIsPos,
    FExp10,
    IntToFloat,
    FloatToInt,
    StrToFloat,
    FIsZero,
    FNotNeg,
    FLen,
}
impl std::fmt::Display for Op1Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(a) = self {
                    return write!(f, "{a}");
                }
            )*
        }}
        crate::for_all_op1!();
        Ok(())
    }
}

impl Op1Enum {
    pub fn from_name(name: &str, config: &Config) -> Self {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if $op::name() == name {
                    return $op::from_config(config).into();
                }
            )*
        }}
        crate::for_all_op1!();
        match name {
            "str.len" => Len::from_config(config).into(),
            "str.from_int" => ToStr::from_config(config).into(),
            "str.to_int" => ToInt::from_config(config).into(),
            _ => panic!("Unknown Operator {}", name),
        }
    }
    pub fn name(&self) -> &'static str {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(_) = self {
                    return $op::name();
                }
            )*
        }}
        crate::for_all_op1!();
        panic!()
    }
}

#[enum_dispatch(Op2)]
#[derive(Clone, PartialEq, Eq)]
pub enum Op2Enum {
    Concat,
    Eq,
    At,
    PrefixOf,
    SuffixOf,
    Contains,
    Split,
    Join,
    Count,
    Add,
    Sub,
    Head,
    Tail,
    Filter,
    Map,
    TimeFloor,
    TimeAdd,
    Floor, Round, Ceil,
    FAdd, FSub, FFloor, FRound, FCeil, FCount, FShl10, TimeMul, StrAt
}

impl std::fmt::Display for Op2Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(a) = self {
                    return write!(f, "{a}");
                }
            )*
        }}
        crate::for_all_op2!();
        Ok(())
    }
}

impl Op2Enum {
    pub fn from_name(name: &str, config: &Config) -> Self {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if $op::name() == name {
                    return $op::from_config(config).into();
                }
            )*
        }}
        crate::for_all_op2!();
        match name {
            "+" => Add::from_config(config).into(),
            "-" => Sub::from_config(config).into(),
            _ => panic!("Unknown Operator: {}", name),
        }
    }
    pub fn name(&self) -> &'static str {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(_) = self {
                    return $op::name();
                }
            )*
        }}
        crate::for_all_op2!();
        panic!()
    }
}

#[enum_dispatch(Op3)]
#[derive(Clone, PartialEq, Eq)]
pub enum Op3Enum {
    Replace,
    Ite,
    SubStr,
    IndexOf,
}

impl std::fmt::Display for Op3Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(a) = self {
                    return write!(f, "{a}");
                }
            )*
        }}
        crate::for_all_op3!();
        Ok(())
    }
}

impl Op3Enum {
    pub fn from_name(name: &str, config: &Config) -> Self {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if $op::name() == name {
                    return $op::from_config(config).into();
                }
            )*
        }}
        crate::for_all_op3!();
        panic!("Unknown Operator: {}", name);
    }
    pub fn name(&self) -> &'static str {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(_) = self {
                    return $op::name();
                }
            )*
        }}
        crate::for_all_op3!();
        panic!()
    }
}

pub mod op_impl;
