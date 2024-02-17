use chrono::NaiveTime;
use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::utils::F64;
use crate::value::ConstValue;
use chrono::Timelike;
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::AllocForExactSizeIter;

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatWeekday(usize);

impl FormatWeekday {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1))
    }
}
impl_name!(FormatWeekday, "weekday.fmt");

impl Enumerator1 for FormatWeekday {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl_op1!(FormatWeekday, "weekday.fmt",
    Str -> Int { |s1| {
        todo!()
    }}
);

lazy_static::lazy_static!{
    static ref REGEX: Regex = {
        let weekday_literal = r"Sun(day)?|Mon(day)?|Tue(sday)?|Wed(nesday)?|Thu(r|rsday)?|Fri(day)?|Sat(urday)?";
        Regex::new(format!(r"^{weekday_literal}").as_str()).unwrap()
    };
}

impl FormattingOp for FormatWeekday {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let weekdays = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        if let Some(caps) = REGEX.captures(input) {
            if caps.get(0).is_some() {
                let weekday = weekdays.iter().enumerate().find(|(i, s)| ***s == caps.get(0).unwrap().as_str()[0..3]).unwrap().0 as u32 + 1;
                return Some((*self, weekday.into(), &input[caps.get(0).unwrap().as_str().len()..]))
            }
        }
        None
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(other)
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Int(0.into())
    }
}