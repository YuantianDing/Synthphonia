use itertools::Itertools;
use pest::iterators::Pair;

use crate::{
    galloc::AllocForStr, utils::TryRetain, value::{ConstValue, Type}
};

use super::{problem::{new_custom_error_span, Error, Rule}, config::Config};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum ProdRule {
    Var(String, Config),
    Const(ConstValue, Config),
    Op1(String, String, Config),
    Op2(String, String, String, Config),
    Op3(String, String, String, String, Config),
}

impl ConstValue {
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let [value]: [_; 1] = pair.into_inner().into_iter().collect_vec().try_into().unwrap();
        match value.as_rule() {
            Rule::numeral => {
                if value.as_str().contains(".") {
                    let f = value.as_str().parse::<f64>().map_err(|_| new_custom_error_span("Can not parse float".into(), value.as_span()))?;
                    Ok(Self::Float(f.into()))
                } else {
                    let f = value.as_str().parse::<i64>().map_err(|_| new_custom_error_span("Can not parse int".into(), value.as_span()))?;
                    Ok(Self::Int(f))
                }
            }
            Rule::strlit => Ok(Self::Str(value.as_str()[1..(value.as_str().len() - 1)].galloc_str())),
            Rule::boollit => match value.as_str() {
                "true" => Ok(Self::Bool(true)),
                "false" => Ok(Self::Bool(false)),
                "null" => Ok(Self::Null),
                _ => Err(new_custom_error_span("Can not parse the Boolean".into(), value.as_span())),
            },
            _ => panic!("should not reach here"),
        }
    }
}

impl ProdRule {
    pub fn const_value(&self) -> Option<&ConstValue> {
        match self {
            ProdRule::Const(i, _) => Some(i),
            _ => None,
        }
    }
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let mut vec = pair.into_inner().into_iter().collect_vec();
        let mut config = Config::new();
        vec.try_retain(|x| {
            if x.as_rule() == Rule::config {
                config.merge(Config::parse(x.clone())?);
                Ok(false)
            } else { Ok(true) }
        })?;
        if vec.len() == 1 {
            let [value]: [_; 1] = vec.try_into().unwrap();
            match value.as_rule() {
                Rule::value => Ok(Self::Const(ConstValue::parse(value)?, config)),
                Rule::symbol => Ok(Self::Var(value.as_str().into(), config)),
                _ => panic!("should not reach here"),
            }
        } else {
            match vec.as_slice() {
                [op, a1] => Ok(Self::Op1(op.as_str().into(), a1.as_str().into(), config)),
                [op, a1, a2] => Ok(Self::Op2(op.as_str().into(), a1.as_str().into(), a2.as_str().into(), config)),
                [op, a1, a2, a3] => Ok(Self::Op3(op.as_str().into(), a1.as_str().into(), a2.as_str().into(), a3.as_str().into(), config)),
                _ => panic!("should not reach here"),
            }
        }
    }
}

impl std::fmt::Debug for ProdRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(arg0, _) => write!(f, "{}", arg0),
            Self::Const(arg0, _) => write!(f, "{}", arg0),
            Self::Op1(arg0, arg1, c) => write!(f, "({} {}{:?})", arg0, arg1, c),
            Self::Op2(arg0, arg1, arg2, c) => write!(f, "({} {} {}{:?})", arg0, arg1, arg2, c),
            Self::Op3(arg0, arg1, arg2, arg3, c) => write!(f, "({} {} {} {}{:?})", arg0, arg1, arg2, arg3, c),
        }
    }
}
