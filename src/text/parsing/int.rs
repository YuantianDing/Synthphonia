use std::collections::HashSet;
use std::sync::Arc;

use chrono::{NaiveDate, Datelike, Month};
use itertools::Itertools;
use regex::Regex;
use crate::forward::executor::Enumerator;
use crate::galloc::TryAllocForExactSizeIter;

use crate::value::ConstValue;
use crate::{galloc::AllocForExactSizeIter, expr::{Expr, ops}, impl_basic, impl_op1_opt, new_op1_opt, value::Value};

use super::ParsingOp;


impl_basic!(ParseInt, "int.parse");
impl crate::forward::enumeration::Enumerator1 for ParseInt {
    fn enumerate(&self, this: &'static ops::Op1Enum, exec: Arc<Enumerator>, opnt: [usize; 1]) -> Result<(), ()> { Ok(())}
}

impl_op1_opt!(ParseInt, "int.parse",
    Str -> Int { |s1: &&str| -> Option<i64> {
        todo!()
    }}
);

impl ParsingOp for ParseInt {

    fn parse_into(&self, input: &'static str) -> std::vec::Vec<(&'static str, ConstValue)> {
        let regex = Regex::new(format!(r"(\-|\+)?\d+").as_str()).unwrap();
        let iter = regex.captures_iter(input);
        let mut result = Vec::new();
        for m in iter {
            let a = m.get(0).unwrap().as_str();
            if let Ok(i) = a.parse::<i64>() {
                result.push((a, i.into()));
            }
        }
        result
    }

}

