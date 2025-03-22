use std::collections::HashSet;

use chrono::{NaiveDate, Datelike, Month};
use itertools::Itertools;
use regex::Regex;
use crate::galloc::TryAllocForExactSizeIter;

use crate::utils::F64;
use crate::value::ConstValue;
use crate::{galloc::AllocForExactSizeIter, expr::{Expr, ops}, impl_basic, impl_op1_opt, new_op1_opt, value::Value};

use super::ParsingOp;


impl_basic!(ParseFloat, "float.parse");
impl crate::forward::enumeration::Enumerator1 for ParseFloat {
    fn enumerate(&self, this: &'static ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(())}
}

impl_op1_opt!(ParseFloat, "float.parse",
    Str -> Int { |s1: &&str| -> Option<i64> {
        todo!()
    }}
);

impl ParsingOp for ParseFloat {

    fn parse_into(&self, input: &'static str) -> std::vec::Vec<(&'static str, ConstValue)> {
        let regex = Regex::new(r"(\-|\+)?[\d,]+(\.[\d,]+([eE](\-|\+)?\d+)?)?".to_string().as_str()).unwrap();
        let iter = regex.captures_iter(input);
        let mut result = Vec::new();
        for m in iter {
            let a = m.get(0).unwrap().as_str();
            if let Ok(i) = a.parse::<f64>() {
                result.push((a, F64::new(i).into()));
            }
        }
        result
    }

}

pub fn detector(input: &str) -> bool {
    let regex = Regex::new(r"(\-|\+)?[\d,]+(\.[\d,]+([eE](\-|\+)?\d+)?)".to_string().as_str()).unwrap();
    regex.is_match(input)
}

#[cfg(test)]
mod tests {
    use crate::text::parsing::{float::detector, ParsingOp};

    use super::ParseFloat;

    #[test]
    fn test1() {
        let scan = ParseFloat(1);
        println!("{:?}", scan.parse_into("123"));
        println!("{:?}", scan.parse_into("+123.321E3"));
    }
    #[test]
    fn test_detector() {
        assert!(!detector("123"));
        assert!(detector("123.0"));
        assert!(!detector("123E1"));
    }
}

