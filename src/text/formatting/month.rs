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
pub struct FormatMonth(usize);

impl FormatMonth {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1))
    }
}
impl_name!(FormatMonth, "month.fmt");

impl Enumerator1 for FormatMonth {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl_op1!(FormatMonth, "month.fmt",
    Str -> Int { |s1| {
        todo!()
    }}
);

lazy_static::lazy_static!{
    static ref REGEX: Regex = {
        let month_literal = "(?<month>Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|(Nov|Dec)(?:ember)?)";
        Regex::new(format!(r"^{month_literal}").as_str()).unwrap()
    };
}

impl FormattingOp for FormatMonth {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let months = [ "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        if let Some(caps) = REGEX.captures(input) {
            if caps.name("month").is_some() {
                let month = months.iter().enumerate().find(|(i, s)| ***s == caps.name("month").unwrap().as_str()[0..3]).unwrap().0 as u32 + 1;
                return Some((*self, month.into(), &input[caps.get(0).unwrap().as_str().len()..]))
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