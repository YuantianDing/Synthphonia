use std::cmp::{max, min};

use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::utils::F64;
use crate::value::{ConstValue, Value};
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::{AllocForExactSizeIter, AllocForStr};

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatFloat{
    cost: usize,
    padding: (usize, usize),
    min_size : (usize, usize),
}

impl FormatFloat {
    pub fn from_config(config: &Config) -> Self {
        Self{
            cost: config.get_usize("cost").unwrap_or(1),
            padding: (config.get_usize("left").unwrap_or(0), config.get_usize("right").unwrap_or(0)),
            min_size: (0, 0)
        }
    }
    pub fn format_single(&self, value: F64) -> String {
        let value = *value;
        let value_int = if value >= 0.0 { value.floor() } else { value.ceil() };
        let left = if self.padding.0 > 0 {
            format!("{:0left$}", value_int, left= self.padding.0)
        } else { format!("{}", value_int) };

        if let Some(mut right) = format!("{}", value).split_once('.').map(|x| x.1.to_string()) {
            while right.len() < self.padding.1 {
                right.push('0');
            }
            left + "." + &right
        } else if self.padding.1 > 0 {
            left + "." + &"0".repeat(self.padding.1)
        } else { left }
    }
    pub fn get_format(input: &str) -> Self {
        let endzero = input.ends_with("0") && input.contains(".");
        let startzero = input.starts_with("+0") || input.starts_with("-0") || input.starts_with("0");
        let min_left = input.chars().position(|x| x == '.').unwrap_or(input.len());
        let min_right = input.chars().position(|x| x == '.').map(|x| input.len() - 1 - x).unwrap_or(0);
        let before_dot = if startzero { min_left } else { 0 };
        let after_dot = if endzero { min_right } else { 0 };
        Self { cost: 1, padding: (before_dot, after_dot), min_size: (min_left, min_right) }
    }
}

impl FormatFloat {
    pub fn name() ->  &'static str {
        "float.fmt"
    }
}

impl std::fmt::Display for FormatFloat {
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "float.fmt #left:{} #right:{}", self.padding.0, self.padding.1)
    }
}

impl Default for FormatFloat {
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator1 for FormatFloat {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Enumerator, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

crate::impl_formatop!(FormatFloat, Float, |this: &FormatFloat| this.cost);

impl FormattingOp for FormatFloat {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let regex = Regex::new(format!(r"^\-?\d+(\.\d*)?").as_str()).unwrap();
        if let Some(a) = regex.find(input) {
            if a.as_str().ends_with(".") { return None; }
            if let Ok(r) = a.as_str().parse::<f64>() {
                let cv: ConstValue = F64::new(r).into();
                Some((Self::get_format(a.as_str()), cv, &input[a.as_str().len()..]))
            } else { None }
        } else { None }
    }

    fn union(self, other: Self) -> Option<Self> {
        let left = conflict(self.padding.0, other.padding.0)?;
        let right = conflict(self.padding.1, other.padding.1)?;
        let min_left = min(self.min_size.0, other.min_size.0);
        let min_right = min(self.min_size.1, other.min_size.1);
        if left > min_left { return None; }
        if right > min_right { return None; }
        Some(Self{ cost: 1, padding: (left, right), min_size: (min_left, min_right)})
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Float(0.0.into())
    }
}

fn conflict(a: usize, b: usize) -> Option<usize> {
    if a > 0 && b > 0 && a != b { return None; }
    Some(max(a, b))
}

#[cfg(test)]
mod tests {
    use crate::text::formatting::FormatFloat;

    #[test]
    fn format() {
        let a = "001234000.01010";
        assert_eq!(FormatFloat::get_format(a).format_single(a.parse::<f64>().unwrap().into()), a);
        let a = "001234000.0101";
        assert_eq!(FormatFloat::get_format(a).format_single(a.parse::<f64>().unwrap().into()), a);
        let a = "1234000.01010";
        assert_eq!(FormatFloat::get_format(a).format_single(a.parse::<f64>().unwrap().into()), a);
        let a = "1234000.01010";
        assert_eq!(FormatFloat::get_format(a).format_single(a.parse::<f64>().unwrap().into()), a);
        let a = "-01234000.01010";
        assert_eq!(FormatFloat::get_format(a).format_single(a.parse::<f64>().unwrap().into()), a);
        let a = "-1234000.0101000";
        assert_eq!(FormatFloat::get_format(a).format_single(a.parse::<f64>().unwrap().into()), a);
    }
}