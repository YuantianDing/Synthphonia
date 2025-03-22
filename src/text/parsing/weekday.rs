use std::collections::HashSet;

use chrono::{Datelike, Month, NaiveDate, NaiveTime};
use itertools::Itertools;
use regex::Regex;
use crate::galloc::TryAllocForExactSizeIter;

use crate::value::ConstValue;
use crate::{galloc::AllocForExactSizeIter, expr::{Expr, ops}, impl_basic, impl_op1_opt, new_op1_opt, value::Value};

use super::ParsingOp;


use chrono::Timelike;

impl_basic!(ParseWeekday, "weekday.parse");
impl crate::forward::enumeration::Enumerator1 for ParseWeekday {
    fn enumerate(&self, this: &'static ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(())}
}

impl_op1_opt!(ParseWeekday, "weekday.parse",
    Str -> Int { |s1| -> Option<i64> {
        todo!()
    }}
);

impl ParsingOp for ParseWeekday {
    fn parse_into(&self, input: &'static str) -> std::vec::Vec<(&'static str, ConstValue)> {
        let months = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let mut result: Vec<(&'static str, ConstValue)> = Vec::new();
        let weekday_literal = r"Sun(day)?|Mon(day)?|Tue(sday)?|Wed(nesday)?|Thu(r|rsday)?|Fri(day)?|Sat(urday)?";
        let regex5 = Regex::new(weekday_literal.to_string().as_str()).unwrap();
        let iter = regex5.captures_iter(input);
        for m in iter {
            let month = months.iter().enumerate().find(|(i, s)| ***s == m.get(0).unwrap().as_str()[0..3]).unwrap().0 as u32 + 1;
            result.push((m.get(0).unwrap().as_str(), month.into()));
        }
        result
    }
}
