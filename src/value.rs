use derive_more::DebugCustom;
use derive_more::Display;
use derive_more::TryInto;
use derive_more::From;
use itertools::Itertools;

use crate::expr::Expr;
use crate::galloc::AllocForExactSizeIter;
use crate::galloc::AllocForIter;
use crate::tree_learning::bits::BoxSliceExt;
use crate::tree_learning::Bits;
use crate::utils::F64;


#[derive(DebugCustom, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Type {
    #[debug(fmt = "Null")]
    Null,
    #[debug(fmt = "Int")]
    Int,
    #[debug(fmt = "Bool")]
    Bool,
    #[debug(fmt = "String")]
    Str,
    #[debug(fmt = "Float")]
    Float,
    #[debug(fmt = "(List Int)")]
    ListInt,
    #[debug(fmt = "(List String)")]
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
    #[debug(fmt = "null")]
    Null,
}

impl Value {

    pub fn with_examples(self, exs: &[usize]) -> Value {
        match self {
            Value::Int(a) => Value::Int(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Float(a) => Value::Float(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Bool(a) => Value::Bool(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Str(a) => Value::Str(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::ListInt(a) => Value::ListInt(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::ListStr(a) => Value::ListStr(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Null => Value::Null,
        }
    }
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
            Self::Null => Type::Null,
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
            Value::Null => 0,
        }
    }
    #[inline(always)]
    pub fn length_inside(&self) -> Option<Vec<usize>> {
        match self {
            Value::Int(a) => None,
            Value::Bool(b) => None,
            Value::Float(s) => None,
            Value::Null => None,
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
            Value::ListStr(l) => l.iter().flat_map(|x| x.iter().copied()).galloc_collect(),
            _ => panic!("Mismatched type: to_liststr_leak")
        }
    }
    #[inline(always)]
    pub fn try_flatten_leak(&self) -> Option<&'static [&'static str]> {
        // Memory Leak !!!
        match self {
            Value::Str(s) => Some(s.iter().flat_map(|x| (0..x.len()).map(|i| &x[i..i+1]) ).galloc_collect()),
            Value::ListStr(l) => Some(l.iter().flat_map(|x| x.iter().copied()).galloc_collect()),
            _ => None,
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
    pub fn to_liststr(self) -> &'static [&'static [&'static str]] {
        self.try_into().unwrap()
    }
    pub fn to_bool(self) -> &'static [bool] {
        self.try_into().unwrap()
    }
    pub fn to_bits(self) -> Bits {
        Bits::from_bit_siter(self.to_bool().iter().cloned())
    }
    pub fn is_all_true(&self) -> bool {
        if let Self::Bool(b) = self {
            b.iter().all(|x| *x)
        } else { false }
    }
    pub fn is_all_false(&self) -> bool {
        if let Self::Bool(b) = self {
            b.iter().all(|x| !(*x))
        } else { false }
    }
    pub fn is_all_empty(&self) -> bool {
        if let Self::Str(b) = self {
            b.iter().all(|x| x.is_empty())
        } else { false }
    }
    pub fn bool_not(self) -> Value {
        let this = self.to_bool();
        this.iter().map(|x| !x).galloc_scollect().into()
    }
    pub fn eq_count(&self, other: &Self) -> usize {
        match (self, other) {
            (Self::Int(a1), Self::Int(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::Str(a1), Self::Str(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::Float(a1), Self::Float(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::Bool(a1), Self::Bool(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::ListInt(a1), Self::ListInt(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::ListStr(a1), Self::ListStr(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            _ => 0,
        }
    }
    pub fn eq_bits(&self, other: &Self) -> Option<Bits> {
        match (self, other) {
            (Self::Int(a1), Self::Int(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::Str(a1), Self::Str(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::Float(a1), Self::Float(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::Bool(a1), Self::Bool(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::ListInt(a1), Self::ListInt(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::ListStr(a1), Self::ListStr(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            _ => None,
        }
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
    Float(F64),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Expr(&'static Expr)
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
            Self::Expr(_) => Type::Null,
        }
    }
    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> { if let Self::Bool(b) = self { Some(*b) } else { None }}
    #[inline(always)]
    pub fn as_i64(&self) -> Option<i64> { if let Self::Int(b) = self { Some(*b) } else { None }}
    pub fn as_usize(&self) -> Option<usize> { if let Self::Int(b) = self { Some(*b as usize) } else { None }}
    pub fn as_str(&self) -> Option<&'static str> { if let Self::Str(b) = self { Some(*b) } else { None }}
    pub fn as_float(&self) -> Option<F64> { if let Self::Float(b) = self { Some(*b) } else { None }}
    pub fn as_expr(&self) -> Option<&'static Expr> { if let Self::Expr(b) = self { Some(*b) } else { None }}
    pub fn as_f64(&self) -> Option<f64> { if let Self::Float(b) = self { Some(**b) } else { None }}
    pub fn is_null(&self) -> bool { matches!(self, Self::Null) }
    pub fn value(&self, len: usize) -> Value {
        match self {
            ConstValue::Bool(t) => Value::Bool((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Int(t) => Value::Int((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Str(t) => Value::Str((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Float(f) => Value::Float((0..len).map(|_| *f).galloc_scollect()),
            ConstValue::Null => panic!("Unable to convert Null to Value"),
            ConstValue::Expr(_) => panic!("Unable to convert Expr to Value"),
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
        ConstValue::Expr(_) => todo!(),
    }
}

#[macro_export]
macro_rules! const_value {
    (true) => {$crate::value::ConstValue::Bool(true)};
    (false) => {$crate::value::ConstValue::Bool(false)};
    ($l:literal) => { 
        if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<&str>() {
            $crate::value::ConstValue::Str(f)
        } else if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<i32>() {
            crate::value::ConstValue::Int(*f as i64)
        } else if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<f64>() {
            crate::value::ConstValue::Float((*f as f64).into())
        } else { panic!("Invalid literal {}", $l) }
    };
}
