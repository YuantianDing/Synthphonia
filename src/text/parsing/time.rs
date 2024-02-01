use std::collections::HashSet;

use chrono::{Datelike, Month, NaiveDate, NaiveTime};
use itertools::Itertools;
use regex::Regex;
use crate::galloc::TryAllocForExactSizeIter;

use crate::value::ConstValue;
use crate::{galloc::AllocForExactSizeIter, expr::{Expr, ops}, impl_basic, impl_op1_opt, new_op1_opt, value::Value};

use super::ParsingOp;


use chrono::Timelike;

impl_basic!(ParseTime, "time.parse");
impl crate::forward::enumeration::Enumerator1 for ParseTime {
    fn enumerate(&self, this: &'static ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(())}
}

impl_op1_opt!(ParseTime, "time.parse",
    Str -> Int { |s1| {
        todo!()
    }}
);

impl ParsingOp for ParseTime {

    fn parse_into(&self, input: &'static str) -> std::vec::Vec<(&'static str, ConstValue)> {
        let mut result: Vec<(&'static str, ConstValue)> = Vec::new();
        let regex1 = Regex::new(r"(?<h>\d{1,2})(:(?<m>\d{1,2}))?(:(?<s>\d{1,2}))?(\s*(?<pm>p\.?m\.?|P\.?M\.?|a\.?m\.?|A\.?M\.?))?").unwrap();
        for caps in regex1.captures_iter(input) {
            let mut h = caps.name("h").unwrap().as_str().parse::<u32>().unwrap();
            let m = caps.name("m").map(|a| a.as_str().parse::<u32>().unwrap()).unwrap_or(0);
            let s = caps.name("s").map(|a| a.as_str().parse::<u32>().unwrap()).unwrap_or(0);
            if let Some(a) = caps.name("pm") {
                if a.as_str().chars().next().unwrap() == 'p' || a.as_str().chars().next().unwrap() == 'P'  {
                    if h != 12 { h += 12; }
                } else {
                    if h == 12 { h = 0; }
                }
            }
            if caps.name("m").is_some() || caps.name("s").is_some() || caps.name("pm").is_some() {
                if let Some(a) = NaiveTime::from_hms_opt(h, m, s) {
                    result.push((caps.get(0).unwrap().as_str(), a.num_seconds_from_midnight().into()))
                }
            }

        }
        result
    }

}

#[cfg(test)]
mod tests {

    use crate::text::parsing::{ParseTime, ParsingOp};


    #[test]
    fn test1() {
        let scanner = ParseTime(1);
        println!("{:?}", scanner.parse_into("6:25PM"));
        println!("{:?}", scanner.parse_into("6:25:12 PM"));
        println!("{:?}", scanner.parse_into("12:0:1 AM"));
        println!("{:?}", scanner.parse_into("12am"));
    }
}




