use std::collections::HashSet;

use chrono::{NaiveDate, Datelike, Month};
use itertools::Itertools;
use regex::Regex;

use crate::{galloc::AllocForExactSizeIter, expr::{Expr, ops}, impl_basic, impl_op1_opt, new_op1_opt, value::{ConstValue, Value}};
use crate::galloc::TryAllocForExactSizeIter;
use super::ParsingOp;


impl_basic!(ParseMonth, "month.parse");
impl crate::forward::enumeration::Enumerator1 for ParseMonth {
    fn enumerate(&self, this: &'static ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(())}
}

impl_op1_opt!(ParseMonth, "month.parse",
    Str -> Int { |s1| -> Option<i64> {
        todo!()
    }}
);

impl ParsingOp for ParseMonth {

    fn parse_into(&self, input: &'static str) -> std::vec::Vec<(&'static str, ConstValue)> {
        let months = [ "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let mut result: Vec<(&'static str, ConstValue)> = Vec::new();
        let month_literal = "(?<month>Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|(Nov|Dec)(?:ember)?)";
        let regex5 = Regex::new(format!(r"{month_literal}").as_str()).unwrap();
        let iter = regex5.captures_iter(input);
        for m in iter {
            if m.name("month").is_some() {
                let month = months.iter().enumerate().find(|(i, s)| ***s == m.name("month").unwrap().as_str()[0..3]).unwrap().0 as u32 + 1;
                result.push((m.get(0).unwrap().as_str(), month.into()));
            }
        }
        result
    }

}
