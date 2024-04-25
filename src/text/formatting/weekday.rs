use std::collections::HashSet;

use chrono::NaiveTime;
use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::utils::F64;
use crate::value::{ConstValue, Value};
use chrono::Timelike;
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::{AllocForExactSizeIter, TryAllocForExactSizeIter};

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatWeekday(usize, Option<bool>);

impl FormatWeekday {
    pub fn from_config(config: &Config) -> Self {
        Self(
            config.get_usize("cost").unwrap_or(1),
            config.get_bool("abbv"),
        )
    }
}
impl FormatWeekday {
    pub fn name() -> &'static str {
        "weekday.fmt"
    }
}
impl std::fmt::Display for FormatWeekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(abbv) = self.1 {
            write!(f, "weekday.fmt #abbv:{}", abbv)
        } else {
            write!(f, "weekday.fmt")
        }
    }
}

impl Default for FormatWeekday {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator1 for FormatWeekday {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}


impl crate::expr::ops::Op1 for FormatWeekday {
    fn cost(&self) -> usize {
        self.0
    }
    fn try_eval(&self, a1: crate::value::Value) -> (bool, crate::value::Value) {
        match a1 {
            crate::value::Value::Int(s1) => {
                let a = s1.iter().map(|&s1| {
                    if !(s1 >= 1 && s1 <= 7) { return ""; }
                    let weekday_abbv = ["", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                    let weekday_full = ["", "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
                    
                    if let Some(true) = self.1 {
                        weekday_abbv[s1 as usize]
                    } else {
                        weekday_full[s1 as usize]
                    }
                }).galloc_scollect();
                (true, a.into())
            }
            _ => (false, Value::Null),
        }
    }
}

lazy_static::lazy_static!{
    static ref REGEX: Regex = {
        let weekday_literal = r"Sun(day)?|Mon(day)?|Tue(sday)?|Wed(nesday)?|Thu(r|rsday)?|Fri(day)?|Sat(urday)?";
        Regex::new(format!(r"^{weekday_literal}").as_str()).unwrap()
    };
}

impl FormattingOp for FormatWeekday {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let weekdays = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let weekday_abbv = HashSet::from(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
        let weekday_full = HashSet::from(["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]);
        if let Some(caps) = REGEX.captures(input) {
            if caps.get(0).is_some() {
                let m = caps.get(0).unwrap().as_str();
                let weekday = weekdays.iter().enumerate().find(|(i, s)| ***s == caps.get(0).unwrap().as_str()[0..3]).unwrap().0 as u32 + 1;
                let abbv = if weekday_abbv.contains(m) { Some(true) } else if weekday_full.contains(m) { Some(false) } else { None } ;
                return Some((*self, weekday.into(), &input[caps.get(0).unwrap().as_str().len()..]))
            }
        }
        None
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(Self(1, conflict(self.1, other.1)?))
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Int(0.into())
    }
}

fn conflict(a: Option<bool>, b: Option<bool>) -> Option<Option<bool>> {
    match (a, b) {
        (Some(x), Some(y)) if x != y => { None }
        (Some(x), _) | (None, Some(x)) => { Some(Some(x)) }
        (None, None) => { Some(None) }
    }
}