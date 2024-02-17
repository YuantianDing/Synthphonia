use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::value::ConstValue;
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::AllocForExactSizeIter;

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatInt(usize);

impl FormatInt {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1))
    }
}
impl_name!(FormatInt, "int.fmt");

impl Enumerator1 for FormatInt {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl_op1!(FormatInt, "int.fmt",
    Str -> Int { |s1| {
        todo!()
    }}
);

impl FormattingOp for FormatInt {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let regex = Regex::new(format!(r"^ *(\-|\+)? *\d+").as_str()).unwrap();
        if let Some(a) = regex.find(input) {
            let cv: ConstValue = a.as_str().parse::<i64>().ok()?.into();
            Some((self.clone(), cv, &input[a.as_str().len()..]))
        } else { None }
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(other)
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Int(0)
    }
}