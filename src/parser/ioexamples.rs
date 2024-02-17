// use crate::galloc::alloc_iter;

use std::collections::HashSet;

use crate::galloc::{self, AllocForIter};

use crate::value::ConstValue;

use super::problem::{new_custom_error_span, SynthFun};

use super::prod::ProdRule;

use super::problem::Error;

use crate::value::Type;

use super::problem::Rule;

use itertools::Itertools;
use pest::iterators::Pair;

use crate::value::Value;
use derive_more::DebugCustom;
#[derive(DebugCustom)]
#[debug(fmt = "{:?} -> {:?}", inputs, output)]
pub struct IOExamples {
    pub(crate) inputs: Vec<Value>,
    pub(crate) output: Value,
}

impl IOExamples {
    pub(crate) fn parse(examples: Pair<'_, Rule>, synthfun: &SynthFun) -> Result<Self, Error> {
        let name = synthfun.name.as_str();
        let args = synthfun.args.as_slice();
        let rettype = synthfun.rettype;
        let mut types = args.iter().map(|x| x.1).collect_vec();
        types.push(rettype);
        let v: HashSet<_> = examples
            .into_inner()
            .map(|x| {
                let span = x.as_span();
                let v = x.into_inner().skip(1).collect_vec();
                let v: Vec<_> = v.into_iter().map(|x| ConstValue::parse(x)).try_collect()?;
                if v.len() != types.len() {
                    return Err(new_custom_error_span(format!("wrong number of arguments for {}: expected", name), span));
                }
                for (value, typ) in v.iter().zip(types.iter()) {
                    if value.ty() != *typ {
                        return Err(new_custom_error_span(format!("wrong type for {}", name), span));
                    }
                }
                Ok(v)
            }).try_collect()?;

        let mut inputs = types.iter().enumerate().map(|(i, ty)| Value::from_const(*ty, v.iter().map(|input| &input[i]).cloned())).collect_vec();
        let output = inputs.pop().unwrap();
        Ok(Self { inputs, output })
    }
}
