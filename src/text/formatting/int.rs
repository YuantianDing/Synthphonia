use std::cmp::max;
use std::sync::Arc;

use itertools::Format;
use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::forward::executor::Enumerator;
use crate::value::{ConstValue, Value};
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::{AllocForExactSizeIter, AllocForStr};

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatInt(usize, usize);

impl FormatInt {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1), config.get_usize("width").unwrap_or(1))
    }
    pub fn format_single(&self, value: i64) -> String {
        if self.1 > 0 {
            format!("{:0left$}", value, left= self.1)
        } else { format!("{}", value) }
    }
    pub fn get_format(input: &str) -> Self {
        let startzero = input.starts_with("+0") || input.starts_with("-0") || input.starts_with("0");
        let before_dot = if startzero { input.len() } else { 0 };
        Self(1, before_dot)
    }
}

impl FormatInt {
    pub fn name() ->  &'static str {
        "int.fmt"
    }
}

impl std::fmt::Display for FormatInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "int.fmt #left:{}", self.1)
    }
}

impl Default for FormatInt {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator1 for FormatInt {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: Arc<Enumerator>, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

crate::impl_formatop!(FormatInt, Int, |this: &FormatInt| this.0);

fn conflict(a: usize, b: usize) -> Option<usize> {
    if a > 0 && b > 0 && a != b { return None; }
    Some(max(a, b))
}

impl FormattingOp for FormatInt {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let regex = Regex::new(format!(r"^ *(\-|\+)? *\d+").as_str()).unwrap();
        if let Some(a) = regex.find(input) {
            let cv: ConstValue = a.as_str().parse::<i64>().ok()?.into();
            Some((FormatInt::get_format(a.as_str()), cv, &input[a.as_str().len()..]))
        } else { None }
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(Self(1, conflict(self.1, other.1)?))
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Int(0)
    }
}