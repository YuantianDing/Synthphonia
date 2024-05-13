

use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use regex::Regex;

use crate::{expr::{ops::{Op1Enum, Op2Enum, Op3Enum}, Expr}, galloc::AllocForAny, utils::TryRetain, value::{ConstValue, Type}};
use derive_more::Display;

use super::{config::Config, ioexamples::IOExamples, problem::{new_custom_error_span, Error, FunSig, ProblemParser, Rule}};


impl Expr {
    pub fn parse(pair: Pair<'_, Rule>, sig: Option<&FunSig>) -> Result<&'static Expr, Error> {
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
                Rule::value => Ok(Self::Const(ConstValue::parse(value)?).galloc()),
                Rule::symbol => {
                    let regex1 = Regex::new(format!(r"^<[0-9]>$").as_str()).unwrap();
                    if let Some(v) = sig.and_then(|x| x.index(value.as_str())) {
                        Ok(Self::Var(v as _).galloc())
                    } else if regex1.is_match(value.as_str()) {
                        Ok(Self::Var(value.as_str()[1..2].parse::<_>().unwrap()).galloc())
                    } else {
                        return Err(new_custom_error_span("Not an input variable".into(), value.as_span()));
                    }
                }
                _ => panic!("should not reach here"),
            }
        } else {
            match vec.as_slice() {
                [op, a1] => {
                    let op = Op1Enum::from_name(op.as_str(), &config);
                    Ok(Self::Op1(op.galloc(), Expr::parse(a1.clone(), sig)?).galloc())
                }
                [op, a1, a2] => {
                    let op = Op2Enum::from_name(op.as_str(), &config);
                    Ok(Self::Op2(op.galloc(), Expr::parse(a1.clone(), sig)?, Expr::parse(a2.clone(), sig)?).galloc())
                }
                [op, a1, a2, a3] => {
                    let op = Op3Enum::from_name(op.as_str(), &config);
                    Ok(Self::Op3(op.galloc(), Expr::parse(a1.clone(), sig)?, Expr::parse(a2.clone(), sig)?, Expr::parse(a3.clone(), sig)?).galloc())
                }
                _ => panic!("should not reach here"),
            }
        }
    }
}

#[derive(Debug, Display, Clone)]
#[display(fmt = "(define-fun {} {})", "sig", "expr.format(&sig)")]
pub struct DefineFun {
    pub sig: FunSig,
    pub expr: &'static Expr,
}

impl DefineFun {
    pub fn parse<'i>(pairs: Pair<'_, Rule>) -> Result<DefineFun, Error> {
        let [name, arglist, typ, expr]: [_; 4] = pairs.into_inner().collect_vec().try_into().unwrap();
        let args: Vec<(String, Type)> = arglist
            .into_inner()
            .map(|x| {
                let [name, typ]: [_; 2] = x.into_inner().collect_vec().try_into().unwrap();
                Ok((name.as_str().to_owned(), Type::parse(typ)?))
            })
            .try_collect()?;
        let rettype = Type::parse(typ)?;
        let sig = FunSig{name: name.as_str().into(), args, rettype};
        
        let expr = Expr::parse(expr, Some(&sig))?;
        Ok(Self{sig, expr})
    }
}

#[derive(Debug, Clone)]
pub struct CheckProblem {
    pub logic: String,
    pub definefun: DefineFun,
    pub examples: IOExamples,
}

impl CheckProblem {
    pub fn parse<'i>(input: &'i str) -> Result<CheckProblem, Error> {
        let [file]: [_; 1] = ProblemParser::parse(Rule::smtfile, input)?.collect_vec().try_into().unwrap();
        let [_, logic, definefun, examples, checksat]: [_; 5] = file.into_inner().collect_vec().try_into().unwrap();
        let [logic]: [_; 1] = logic.into_inner().collect_vec().try_into().unwrap();
        let definefun = DefineFun::parse(definefun)?;
        let examples = IOExamples::parse(examples, &definefun.sig, false)?;

        Ok(CheckProblem {
            logic: logic.as_str().to_owned(),
            definefun,
            examples,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::CheckProblem;

    #[test]
    fn test() {
        let s = fs::read_to_string("a.smt2").unwrap();
        let a = CheckProblem::parse(s.as_str()).unwrap();
        println!("{:?}", a);
    }
}
