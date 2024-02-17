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
pub struct FormatTime(usize);

impl FormatTime {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1))
    }
}
impl_name!(FormatTime, "time.fmt");

impl Enumerator1 for FormatTime {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl_op1!(FormatTime, "time.fmt",
    Str -> Int { |s1| {
        todo!()
    }}
);

lazy_static::lazy_static!{
    static ref REGEX: Regex = Regex::new(r"^(?<h>\d{1,2})(:(?<m>\d{1,2}))?(:(?<s>\d{1,2}))?(\s*(?<pm>p\.?m\.?|P\.?M\.?|a\.?m\.?|A\.?M\.?))?").unwrap();
}

impl FormattingOp for FormatTime {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        if let Some(caps) = REGEX.captures(input) {
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
                    return Some((*self, a.num_seconds_from_midnight().into(), &input[caps.get(0).unwrap().as_str().len()..]))
                }
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