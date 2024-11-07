use std::{collections::HashSet, sync::Arc};

use chrono::{NaiveDate, Datelike, Month};
use itertools::Itertools;
use regex::Regex;
use spin::lazy;

use crate::{expr::{ops, Expr}, forward::executor::Enumerator, galloc::AllocForExactSizeIter, impl_basic, impl_op1_opt, new_op1_opt, value::{ConstValue, Value}};

use crate::galloc::TryAllocForExactSizeIter;
use super::ParsingOp;


impl_basic!(ParseDate, "date.parse");
impl crate::forward::enumeration::Enumerator1 for ParseDate {
    fn enumerate(&self, this: &'static ops::Op1Enum, exec: Arc<Enumerator>, opnt: [usize; 1]) -> Result<(), ()> { Ok(())}
}

impl crate::expr::ops::Op1 for ParseDate {
    fn cost(&self) -> usize {
        self.0
    }
    fn try_eval(&self, a1: crate::value::Value) -> (bool, crate::value::Value) {
        match a1 {
            crate::value::Value::Str(s1) => {
                let a = s1
                    .iter()
                    .map(|s1| {
                        let mut res = self.parse_into(*s1);
                        res.sort_by_key(|(a,b)| -(a.len() as isize));
                        res.first().map(|(s, c)| c.as_i64().unwrap()).unwrap_or(0 as i64)
                    }).galloc_scollect();
                (true, a.into())
            }
            _ => (false, Value::Null),
        }
    }
}

lazy_static::lazy_static!{
    static ref REGEXES : [Regex; 5] = {
        let month_literal = "(?<month>Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|(Nov|Dec)(?:ember)?)";
        let month = r"((?<m>\d{1,2})|(?<month>Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|(Nov|Dec)(?:ember)?))";
        let day = r"((?<d>\d{1,2})(st|nd|rd|th)?)";
        let year = r"(?<y>\d{2,4})";
        let regex1 = Regex::new(format!(r"{month}[\- /.,]*{day}?[\- /.,]*{year}?").as_str()).unwrap();
        let regex2 = Regex::new(format!(r"{year}[ \-/.,]+{month}[\- /.,]*{day}?").as_str()).unwrap();
        let regex3 = Regex::new(format!(r"{day}[ \-/.,]*{month}[\- /.,]*{year}?").as_str()).unwrap();
        let regex4 = Regex::new(format!(r"{month}[\- /.,]+{year}?").as_str()).unwrap();
        let regex5 = Regex::new(format!(r"{month_literal}").as_str()).unwrap();
        [regex1, regex2, regex3, regex4, regex5]
    };
}


impl ParsingOp for ParseDate {

    fn parse_into(&self, input: &'static str) -> std::vec::Vec<(&'static str, ConstValue)> {
        let months = [ "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let mut result: Vec<(&'static str, ConstValue)> = Vec::new();
        let [regex1, regex2, regex3, regex4, regex5] = &*REGEXES;
        let iter = regex1.captures_iter(input).chain(regex2.captures_iter(input)).chain(regex3.captures_iter(input)).chain(regex4.captures_iter(input)).chain(regex5.captures_iter(input));
        for m in iter {
            let mut year = if m.name("y").is_none() { 2000 } else { m.name("y").unwrap().as_str().parse::<i32>().unwrap()};
            if m.name("m").is_some() || m.name("month").is_some() {
                let month = if m.name("m").is_some() {
                    m.name("m").unwrap().as_str().parse::<u32>().unwrap()
                } else {
                    months.iter().enumerate().find(|(i, s)| ***s == m.name("month").unwrap().as_str()[0..3]).unwrap().0 as u32 + 1
                };
                let day = if m.name("d").is_none() { 1 } else { m.name("d").unwrap().as_str().parse::<u32>().unwrap()};
                if m.name("d").is_none() && m.name("y").is_none() { continue; }
                if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                    result.push((m.get(0).unwrap().as_str(), (d.num_days_from_ce() as i64).into() ));
                }
            }
        }
        result
    }

}


#[cfg(test)]
mod tests {
    use crate::{text::parsing::{ParseDate, ParsingOp}};

    #[test]
    fn test1() {
        let scanner = ParseDate(1);
        println!("{:?}", scanner.parse_into("Jan"))           ;
        println!("{:?}", scanner.parse_into("Jan 1st, 2034")) ;
        println!("{:?}", scanner.parse_into("03042241"))      ;
        println!("{:?}", scanner.parse_into("10/6/2143"))     ;
        println!("{:?}", scanner.parse_into("06-Oct-2143"))   ;
        println!("{:?}", scanner.parse_into("Mar 30 2002"))   ;
        println!("{:?}", scanner.parse_into("01311846"))      ;
        println!("{:?}", scanner.parse_into("22 Apr 1953"))   ;
        println!("{:?}", scanner.parse_into("03302241"))      ;
        println!("{:?}", scanner.parse_into("02-Aug-2160"))   ;
        println!("{:?}", scanner.parse_into("23 May 1984"))   ;
        println!("{:?}", scanner.parse_into("15 August 1740"));
        println!("{:?}", scanner.parse_into("Jul 08 2237"))   ;
        println!("{:?}", scanner.parse_into("3 Nov 1904"))    ;
        println!("{:?}", scanner.parse_into("5 April 2088"))  ;
        println!("{:?}", scanner.parse_into("05302131"))      ;
        println!("{:?}", scanner.parse_into("May 25 1817"))   ;
        println!("{:?}", scanner.parse_into("31 May 1963"))   ;
        println!("{:?}", scanner.parse_into("24-Nov-2098"))   ;
        println!("{:?}", scanner.parse_into("22 Oct 1815"))   ;
        println!("{:?}", scanner.parse_into("26 May 2155"))   ;
        println!("{:?}", scanner.parse_into("26-Mar-1816"))   ;
        println!("{:?}", scanner.parse_into("26 Apr 2090"))   ;
        println!("{:?}", scanner.parse_into("14-Aug-2089"))   ;
        println!("{:?}", scanner.parse_into("Apr 20 1957"))   ;
        println!("{:?}", scanner.parse_into("11 Sep 1952"))   ;
        println!("{:?}", scanner.parse_into("03-Nov-2114"))   ;
        println!("{:?}", scanner.parse_into("21 June 2059"))  ;
        println!("{:?}", scanner.parse_into("21-Jan-1818"))   ;
        println!("{:?}", scanner.parse_into("16 Sep 2075"))   ;
        println!("{:?}", scanner.parse_into("Oct 2 2204"))    ;
        println!("{:?}", scanner.parse_into("02 Sep 1747"))   ;
        println!("{:?}", scanner.parse_into("29 Jan 2218"))   ;
        println!("{:?}", scanner.parse_into("03 Apr 2008"))   ;
    }
}

