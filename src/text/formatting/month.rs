use std::collections::HashSet;

use chrono::NaiveTime;
use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::impl_op1_opt;
use crate::utils::F64;
use crate::value::ConstValue;
use crate::{impl_name, impl_op1, parser::config::Config};
use chrono::Timelike;

use crate::galloc::{AllocForExactSizeIter, TryAllocForExactSizeIter};

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatMonth(usize, Option<bool>);

impl FormatMonth {
    pub fn from_config(config: &Config) -> Self {
        Self(
            config.get_usize("cost").unwrap_or(1),
            config.get_bool("abbv"),
        )
    }
}

impl FormatMonth {
    pub fn name() -> &'static str {
        "month.fmt"
    }
}

impl std::fmt::Display for FormatMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(abbv) = self.1 {
            write!(f, "month.fmt #abbv:{}", abbv)
        } else {
            write!(f, "month.fmt")
        }
    }
}

impl Default for FormatMonth {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator1 for FormatMonth {
    fn enumerate(
        &self,
        this: &'static crate::expr::ops::Op1Enum,
        exec: &'static crate::forward::executor::Executor,
        opnt: [usize; 1],
    ) -> Result<(), ()> {
        Ok(())
    }
}

impl crate::expr::ops::Op1 for FormatMonth {
    fn cost(&self) -> usize {
        self.0
    }
    fn try_eval(&self, a1: crate::value::Value) -> Option<crate::value::Value> {
        match a1 {
            crate::value::Value::Int(s1) => {
                let a = s1.iter().map(|&s1| {
                    if !(s1 >= 1 && s1 <= 12) { return Some(""); }
                    let months_abbv = ["", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
                    let months_full = ["", "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
                    
                    if let Some(true) = self.1 {
                        Some(months_abbv[s1 as usize])
                    } else {
                        Some(months_full[s1 as usize])
                    }
                }).galloc_try_scollect();
                a.map(|a| crate::value::Value::Str(a))
            }
            _ => None,
        }
    }
}

lazy_static::lazy_static! {
    static ref REGEX: Regex = {
        let month_literal = "(?<month>Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|(Nov|Dec)(?:ember)?)";
        Regex::new(format!(r"^{month_literal}").as_str()).unwrap()
    };
}

impl FormattingOp for FormatMonth {
    fn format(
        &self,
        input: &'static str,
    ) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let months_arr = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let months_abbv = HashSet::from(["Jan", "Feb", "Mar", "Apr", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]);
        let months_full = HashSet::from(["January", "February", "March", "April", "June", "July", "August", "September", "October", "November", "December"]);
        if let Some(caps) = REGEX.captures(input) {
            if caps.name("month").is_some() {
                let m = caps.name("month").unwrap().as_str();
                let month = months_arr.iter().enumerate().find(|(_, s)| ***s == m[0..3]).unwrap().0 as u32 + 1;
                let abbv = if months_abbv.contains(m) { Some(true) } else if months_full.contains(m) { Some(false) } else { None } ;
                return Some((Self(1, abbv), month.into(), &input[caps.get(0).unwrap().as_str().len()..]));
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