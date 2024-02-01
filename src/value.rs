use derive_more::DebugCustom;
use derive_more::Display;
use derive_more::TryInto;
use derive_more::From;
use itertools::Itertools;

use crate::galloc::AllocForExactSizeIter;
use crate::galloc::AllocForIter;
use crate::utils::F64;


#[derive(DebugCustom, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Type {
    #[debug(fmt = "Null")]
    Null,
    #[debug(fmt = "Int")]
    Int,
    #[debug(fmt = "Bool")]
    Bool,
    #[debug(fmt = "Str")]
    Str,
    #[debug(fmt = "Float")]
    Float,
    #[debug(fmt = "(List Int)")]
    ListInt,
    #[debug(fmt = "(List Str)")]
    ListStr,
}

impl Type {
    pub fn basic(self) -> Type {
        match self {
            Type::ListInt => Self::Int,
            Type::ListStr => Self::Str,
            a => a,
        }
    }
    pub fn to_list(self) -> Option<Type> {
        match self {
            Type::Int => Some(Type::ListInt),
            Type::Str => Some(Type::ListStr),
            _ => None
        }
    }
}

#[derive(DebugCustom, Clone, TryInto, Copy, PartialEq, Eq, Hash, From)]
pub enum Value {
    #[debug(fmt = "{:?}", _0)]
    Int(&'static [i64]),
    #[debug(fmt = "{:?}", _0)]
    Float(&'static [F64]),
    #[debug(fmt = "{:?}", _0)]
    Bool(&'static [bool]),
    #[debug(fmt = "{:?}", _0)]
    Str(&'static [&'static str]),
    #[debug(fmt = "{:?}", _0)]
    ListInt(&'static [&'static [i64]]),
    #[debug(fmt = "{:?}", _0)]
    ListStr(&'static [&'static [&'static str]]),
}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Bool(_) => Type::Bool,
            Self::Str(_) => Type::Str,
            Self::Float(_) => Type::Float,
            Self::ListInt(_) => Type::ListInt,
            Self::ListStr(_) => Type::ListStr,
        }
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        match self {
            Value::Int(a) => a.len(),
            Value::Bool(b) => b.len(),
            Value::Str(s) => s.len(),
            Value::Float(s) => s.len(),
            Value::ListInt(l) => l.len(),
            Value::ListStr(l) => l.len(),
        }
    }
    #[inline(always)]
    pub fn length_inside(&self) -> Option<Vec<usize>> {
        match self {
            Value::Int(a) => None,
            Value::Bool(b) => None,
            Value::Float(s) => None,
            Value::Str(s) => Some(s.iter().map(|x| x.len()).collect_vec()),
            Value::ListInt(l) => Some(l.iter().map(|x| x.len()).collect_vec()),
            Value::ListStr(l) => Some(l.iter().map(|x| x.len()).collect_vec()),
        }
    }
    #[inline(always)]
    pub fn flatten_leak(&self) -> &'static [&'static str] {
        // Memory Leak !!!
        match self {
            Value::Str(s) => s.iter().flat_map(|x| (0..x.len()).map(|i| &x[i..i+1]) ).galloc_collect(),
            Value::ListStr(l) => l.iter().flat_map(|x| x.iter().map(|p| *p)).galloc_collect(),
            _ => panic!("Mismatched type: to_liststr_leak")
        }
    }

    pub fn from_const(ty: Type, constants: impl ExactSizeIterator<Item=ConstValue>) -> Self {
        match ty {
            Type::Bool => Value::Bool(constants.map(|p| p.as_bool().unwrap()).galloc_scollect()),
            Type::Int => Value::Int(constants.map(|p| p.as_i64().unwrap()).galloc_scollect()),
            Type::Str => Value::Str(constants.map(|p| p.as_str().unwrap()).galloc_scollect()),
            Type::Float => Value::Float(constants.map(|p| p.as_float().unwrap()).galloc_scollect()),
            _ => panic!("should not reach here"),
        }
    }
    pub fn substr(&self, other: &Value) -> bool{
        match (self, other) {
            (Value::Str(s), Value::Str(o)) => s.iter().zip(o.iter()).all(|(a,b)| b.contains(a)),
            _ => false,
        }
    }
    pub fn some_substr(&self, other: &Value) -> bool{
        match (self, other) {
            (Value::Str(s), Value::Str(o)) => s.iter().zip(o.iter()).any(|(a,b)| b.contains(a)),
            _ => false,
        }
    }
    pub fn to_str(self) -> &'static [&'static str] {
        self.try_into().unwrap()
    }
}


#[derive(DebugCustom, Display, PartialEq, Eq, Hash, Clone, Copy, From)]
pub enum ConstValue {
    #[debug(fmt = "null")]
    #[display(fmt = "null")]
    Null,
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Bool(bool),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Int(i64),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Str(&'static str),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Float(F64)
}

impl From<usize> for ConstValue {
    fn from(value: usize) -> Self {
        Self::Int(value as i64)
    }
}
impl From<u32> for ConstValue {
    fn from(value: u32) -> Self {
        Self::Int(value as i64)
    }
}

impl ConstValue {
    pub fn ty(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Bool(_) => Type::Bool,
            Self::Str(_) => Type::Str,
            Self::Float(_) => Type::Float,
            Self::Null => Type::Null,
        }
    }
    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> { if let Self::Bool(b) = self { Some(*b) } else { None }}
    #[inline(always)]
    pub fn as_i64(&self) -> Option<i64> { if let Self::Int(b) = self { Some(*b) } else { None }}
    pub fn as_usize(&self) -> Option<usize> { if let Self::Int(b) = self { Some(*b as usize) } else { None }}
    pub fn as_str(&self) -> Option<&'static str> { if let Self::Str(b) = self { Some(*b) } else { None }}
    pub fn as_float(&self) -> Option<F64> { if let Self::Float(b) = self { Some(*b) } else { None }}
    pub fn as_f64(&self) -> Option<f64> { if let Self::Float(b) = self { Some(**b) } else { None }}
    pub fn is_null(&self) -> bool { matches!(self, Self::Null) }
    pub fn value(&self, len: usize) -> Value {
        match self {
            ConstValue::Bool(t) => Value::Bool((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Int(t) => Value::Int((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Str(t) => Value::Str((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Float(f) => Value::Float((0..len).map(|_| *f).galloc_scollect()),
            ConstValue::Null => panic!("Unable to convert Null to Value"),
        }
    }

}

pub fn consts_to_value(consts: Vec<ConstValue>) -> Value {
    match consts[0] {
        ConstValue::Null => todo!(),
        ConstValue::Bool(_) => Value::Bool(consts.into_iter().map(|a| a.as_bool().unwrap()).galloc_scollect()),
        ConstValue::Int(_) => Value::Int(consts.into_iter().map(|a| a.as_i64().unwrap()).galloc_scollect()),
        ConstValue::Str(_) => Value::Str(consts.into_iter().map(|a| a.as_str().unwrap()).galloc_scollect()),
        ConstValue::Float(_) => Value::Float(consts.into_iter().map(|a| a.as_float().unwrap()).galloc_scollect()),
    }
}

#[macro_export]
macro_rules! const_value {
    (true) => {crate::value::ConstValue::Bool(true)};
    (false) => {crate::value::ConstValue::Bool(false)};
    ($l:literal) => { 
        if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<&str>() {
            crate::value::ConstValue::Str(f)
        } else if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<i32>() {
            crate::value::ConstValue::Int(*f as i64)
        } else if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<f64>() {
            crate::value::ConstValue::Float((*f as f64).into())
        } else { panic!("Invalid literal {}", $l) }
    };
}
