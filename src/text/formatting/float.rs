use regex::Regex;

use crate::forward::enumeration::Enumerator1;
use crate::utils::F64;
use crate::value::ConstValue;
use crate::{ impl_name, impl_op1, parser::config::Config};

use crate::galloc::AllocForExactSizeIter;

use super::FormattingOp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatFloat(usize);

impl FormatFloat {
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1))
    }
}
impl_name!(FormatFloat, "float.fmt");

impl Enumerator1 for FormatFloat {
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl_op1!(FormatFloat, "float.fmt",
    Str -> Int { |s1| {
        todo!()
    }}
);

impl FormattingOp for FormatFloat {
    fn format(&self, input: &'static str) -> Option<(Self, crate::value::ConstValue, &'static str)> {
        let regex = Regex::new(format!(r"^(\-|\+)?\d+(\.\d+([eE](\-|\+)?\d+)?)?").as_str()).unwrap();
        if let Some(a) = regex.find(input) {
            if let Ok(r) = a.as_str().parse::<f64>() {
                let cv: ConstValue = F64(r).into();
                Some((self.clone(), cv, &input[a.as_str().len()..]))
            } else { None }
        } else { None }
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(other)
    }

    fn bad_value() -> crate::value::ConstValue {
        crate::value::ConstValue::Float(0.0.into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        println!("{:?}", "123".parse::<f64>());
    }
}