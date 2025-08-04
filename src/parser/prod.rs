use itertools::Itertools;
use pest::iterators::Pair;

use crate::{
    galloc::AllocForStr, utils::TryRetain, value::{ConstValue, Type}
};

use super::{problem::{new_custom_error_span, Error, Rule}, config::Config};

#[derive(PartialEq, Eq, Hash, Clone)]
/// A variant-rich enumeration representing different types of production rules used in string synthesis. 
/// 
/// It includes variants for handling variables, constants, and operations with differing arities. 
/// Each variant encapsulates a production rule as follows:
/// 
/// The `Var` variant takes a variable name and a configuration, associating this rule with a specific variable in the synthesis problem. 
/// The `Const` variant holds a constant value alongside its configuration, representing fixed values in the synthesis. 
/// Variants `Op1`, `Op2`, and `Op3` capture operations with one, two, and three operands respectively, each carrying the necessary operands as strings and concluding with a configuration object to manage operational parameters or constraints. 
/// This structure enables flexible representation of grammatical constructs in syntax-guided synthesis tasks, supporting a wide range of synthesis requirements.
pub enum ProdRule {
    Var(String, Config),
    Const(ConstValue, Config),
    Op1(String, String, Config),
    Op2(String, String, String, Config),
    Op3(String, String, String, String, Config),
}

impl ConstValue {
    /// Parses a `Pair` of `'_, Rule>` into a `ConstValue`, returning a result with either the parsed constant or an error. 

    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let [value]: [_; 1] = pair.into_inner().collect_vec().try_into().unwrap();
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
            Rule::hexnum => {
                let s = value.as_str().trim_start_matches("#x");
                let f = u64::from_str_radix(s, 16)
                    .map_err(|_| new_custom_error_span("Can not parse hex".into(), value.as_span()))?;
                Ok(Self::BitVector(s.len() * 4, f))
            }
            Rule::binnum => {
                let s = value.as_str().trim_start_matches("#b");
                let f = u64::from_str_radix(s, 2)
                    .map_err(|_| new_custom_error_span("Can not parse binary".into(), value.as_span()))?;
                Ok(Self::BitVector(s.len(), f))
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
    /// Returns the constant value associated with a specific production rule, if available. 

    pub fn const_value(&self) -> Option<&ConstValue> {
        match self {
            ProdRule::Const(i, _) => Some(i),
            _ => None,
        }
    }
    /// Parses a `Pair` object into a `ProdRule` variant. 

    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let mut vec = pair.into_inner().collect_vec();
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
