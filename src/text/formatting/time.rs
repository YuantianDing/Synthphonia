use chrono::NaiveTime;
use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::utils::F64;
use crate::value::{ConstValue, Value};
use chrono::Timelike;
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::{AllocForExactSizeIter, AllocForStr};

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimeNumberFormat {
    None, Unknown, Padding, Default
}

impl TimeNumberFormat {
    pub fn format(&self, number: u32) -> String {
        match self {
            TimeNumberFormat::None => "".into(),
            TimeNumberFormat::Padding => format!("{:02}", number),
            TimeNumberFormat::Default | TimeNumberFormat::Unknown => format!("{}", number),
        }
    }
    pub fn format_colon(&self, number: u32) -> String {
        match self {
            TimeNumberFormat::None => "".into(),
            TimeNumberFormat::Padding => format!(":{:02}", number),
            TimeNumberFormat::Default | TimeNumberFormat::Unknown => format!(":{}", number),
        }
    }
    pub fn union(self, other: Self) -> Option<Self> {
        if self == other { Some(self) }
        else {
            match (self, other) {
                (Self::Unknown, Self::Padding) | (Self::Padding, Self::Unknown) => Some(Self::Padding),
                (Self::Unknown, Self::Default) | (Self::Default, Self::Unknown) => Some(Self::Default),
                _ => None,
            }
        }
    }
    pub fn from_name(name: &str) -> Self {
        match name {
            "none" => Self::None,
            "unknown" => Self::Unknown,
            "padding" => Self::Padding,
            "default" => Self::Default,
            _ => panic!()
        }
    }
    pub fn to_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Unknown => "unknown",
            Self::Padding => "padding",
            Self::Default => "default",
        }
    }
    pub fn get_format(input: &str) -> Self {
        if input.len() == 2 {
            if input.starts_with("0") { Self::Padding }
            else { Self::Unknown }
        } else if input.len() == 1 { Self::Default }
        else { Self::None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatTime {
    hour: TimeNumberFormat,
    minute: TimeNumberFormat,
    second: TimeNumberFormat,
    pm: Option<bool>,
}

impl FormatTime {
    pub fn from_config(config: &Config) -> Self {
        Self{
            hour: TimeNumberFormat::from_name(config.get_str("h").unwrap_or("default")),
            minute: TimeNumberFormat::from_name(config.get_str("m").unwrap_or("default")),
            second: TimeNumberFormat::from_name(config.get_str("s").unwrap_or("default")),
            pm: config.get_bool("pm"),
        }
    }
}
impl FormatTime {
    pub fn name() ->  &'static str {
        "time.fmt"
    }
}
impl std::fmt::Display for FormatTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "time.fmt #h:{} #m:{} #s:{}", self.hour.to_name(), self.minute.to_name(), self.second.to_name())?;
        if let Some(a) = self.pm {
            write!(f, " #pm:{}", a)?;
        }
        Ok(())
    }
}

impl Default for FormatTime {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator1 for FormatTime {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl crate::expr::ops::Op1 for FormatTime {
    fn cost(&self) -> usize { 1 }
    fn try_eval(&self,a1:Value) -> (bool, Value) {
        match a1 {
            Value::Int(s) => (true, Value::Str(s.iter().map(|&s1|{
                let time = NaiveTime::from_num_seconds_from_midnight_opt(s1 as u32, 0).unwrap_or_default();
                let mut h = time.hour();
                let mut pm = false;
                if self.pm.is_some() { 
                    let (a,b) = hour_to_pm(h);
                    h = a;
                    pm = b;}
                
                let hour = self.hour.format(h);
                let minute = self.minute.format_colon(time.minute());
                let second = self.second.format_colon(time.second());
                let mut result = hour + &minute + &second;
                if let Some(true) = self.pm {
                    if pm { result.push_str("PM") } else { result.push_str("AM")}
                } else if let Some(false) = self.pm {
                    if pm { result.push_str("pm") } else { result.push_str("am")}
                }
                result.galloc_str()
            }).galloc_scollect())),
            _ => (false, Value::Null),
        }
    }
}

lazy_static::lazy_static!{
    static ref REGEX: Regex = Regex::new(r"^(?<h>\d{1,2})(:(?<m>\d{1,2}))?(:(?<s>\d{1,2}))?((?<pm>pm|PM|am|AM))?").unwrap();
}

impl FormattingOp for FormatTime {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        if let Some(caps) = REGEX.captures(input) {
            let mut h = caps.name("h").unwrap().as_str().parse::<u32>().unwrap();
            let m = caps.name("m").map(|a| a.as_str().parse::<u32>().unwrap()).unwrap_or(0);
            let s = caps.name("s").map(|a| a.as_str().parse::<u32>().unwrap()).unwrap_or(0);
            if let Some(a) = caps.name("pm") {
                if h == 0 || h > 12 { return None; }
                h = convert_hour(a.as_str().starts_with('p') || a.as_str().starts_with('P'), h);
            }
            if caps.name("m").is_some() || caps.name("s").is_some() || caps.name("pm").is_some() {
                if let Some(a) = NaiveTime::from_hms_opt(h, m, s) {
                    let hfmt = TimeNumberFormat::get_format(caps.name("h").map(|x| x.as_str()).unwrap_or(""));
                    let mfmt = TimeNumberFormat::get_format(caps.name("m").map(|x| x.as_str()).unwrap_or(""));
                    let sfmt = TimeNumberFormat::get_format(caps.name("s").map(|x| x.as_str()).unwrap_or(""));
                    let pmfmt = caps.name("pm").map(|a| a.as_str() == "AM" || a.as_str() == "PM");
                    return Some((Self{ hour: hfmt, minute: mfmt, second: sfmt, pm: pmfmt}, a.num_seconds_from_midnight().into(), &input[caps.get(0).unwrap().as_str().len()..]))
                }
            }
        }
        None
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(Self{ 
            hour: self.hour.union(other.hour)?,
            minute: self.minute.union(other.minute)?,
            second: self.second.union(other.second)?,
            pm: if self.pm == other.pm { self.pm } else { return None },
        })
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Int(0.into())
    }
}

fn convert_hour(pm: bool, mut h: u32) -> u32 {
    if pm  {
        if h != 12 { h += 12; }
    } else if h == 12 { h = 0; }
    h
}
fn hour_to_pm(h: u32) -> (u32, bool) {
    if h == 0 { (12, false) }
    else if h <= 11 { (h, false) }
    else if h == 12 { (12, true) }
    else { (h - 12, true) }
}

fn conflict(a: Option<bool>, b: Option<bool>) -> Option<Option<bool>> {
    match (a, b) {
        (Some(x), Some(y)) if x != y => { None }
        (Some(x), _) | (None, Some(x)) => { Some(Some(x)) }
        (None, None) => { Some(None) }
    }
}