// use crate::galloc::alloc_iter;

use std::collections::HashSet;

use crate::galloc::{self, AllocForIter};

use crate::value::ConstValue;

use super::problem::{new_custom_error_span, FunSig, SynthFun};

use super::prod::ProdRule;

use super::problem::Error;

use crate::value::Type;

use super::problem::Rule;

use counter::Counter;
use itertools::Itertools;
use pest::error::InputLocation;
use pest::iterators::Pair;

use crate::value::Value;
use derive_more::DebugCustom;
#[derive(DebugCustom, Clone)]
#[debug(fmt = "{:?} -> {:?}", inputs, output)]
/// A struct that holds input and output examples for string synthesis problems. 
/// 
/// The structure consists of two fields: `inputs`, which is a vector containing multiple `Value` elements, and `output`, a single `Value` representing the expected result. 
/// This setup is designed to facilitate the storage and retrieval of example data necessary for evaluating and validating synthesis algorithms, by providing concrete cases of input-output relationships.
/// 
pub struct IOExamples {
    pub(crate) inputs: Vec<Value>,
    pub(crate) output: Value,
}

impl IOExamples {
    /// Parses a collection of input/output examples according to a specified function signature and optional deduplication flag, returning a structured set of examples or an error. 
    /// 
    /// It begins by extracting relevant metadata from the provided function signature, such as function name, argument types, and return type. 
    /// The function processes the provided examples by iterating over them, ensuring each example contains a correct number of arguments and matching types. 
    /// If the 'dedup' parameter is set to true, duplicates are removed using a `HashSet`. 
    /// Finally, the function constructs the `inputs` and `output`, organizing each example's inputs by type before returning the assembled `IOExamples` structure.
    /// 
    pub(crate) fn parse(examples: Pair<'_, Rule>, sig: &FunSig, dedup: bool) -> Result<Self, Error> {
        let name = sig.name.as_str();
        let args = sig.args.as_slice();
        let rettype = sig.rettype;
        let mut types = args.iter().map(|x| x.1).collect_vec();
        types.push(rettype);
        let mut v: Vec<_> = examples
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
            
        if dedup {
            let set: HashSet<_> = v.iter().cloned().collect();
            v = set.into_iter().collect_vec();
        }

        let mut inputs = types.iter().enumerate().map(|(i, ty)| Value::from_const(*ty, v.iter().map(|input| &input[i]).cloned())).collect_vec();
        let output = inputs.pop().unwrap();
        Ok(Self { inputs, output })
    }
    
    /// Extracts and returns a list of constant substrings identified in the input and output examples of string synthesis problems.
    /// 
    /// The method iterates over all input strings and the output string, treating them as a unified sequence. 
    /// For each string, it generates all possible substrings using `all_slices` and counts their occurrences. 
    /// It then evaluates each distinct substring, checking for specific filtering conditions: the substring must appear with sufficient frequency, must either be a significant length or show certain frequency patterns, and should not be simple numeric or alphanumeric characters. 
    /// Substrings meeting these criteria that are not already surpassed in count by longer substrings are added to the list of constants. 
    /// This approach helps in identifying significant repeating string patterns, which can play a crucial role in constructing string transformation rules.
    pub fn extract_constants(&self) -> Vec<&'static str> {
        let mut counter = Counter::<&str, usize>::new();
        let mut total_len = 0;
        for s1 in self.inputs.iter().chain(std::iter::once(&self.output)) {
            if let Value::Str(a) = s1 {
                for s2 in a.iter() {
                    for s in all_slices(s2) {
                        counter[&s] += 1;
                        total_len += s.len();
                    }
                }
            }
        }

        let mut constants: Vec<&'static str> = Vec::new();
        for (k, v) in counter.iter() {
            let mut flag = false;
            if *v >= std::cmp::max(3, total_len / 200) {
                if k.len() == 1 && k.chars().all(char::is_alphanumeric) {
                    continue;
                }
                if k.chars().all(char::is_numeric) {
                    continue;
                }
                if k.len() == 1 {
                    flag = true;
                }
                if k.len() >= 6 {
                    flag = true;
                } else if k.len() >= 4 && *v >= std::cmp::max(5, total_len / 100) {
                    flag = true;
                } else if *v >= std::cmp::max(8, total_len / 30) {
                    flag = true;
                }

                if flag && constants.iter().filter(|c| c.contains(k)).all(|c| counter[k] > counter[c] + 1) {
                    constants.retain(|c| !c.contains(k) || counter[k] + 1 < counter[c]);
                    constants.push(k);
                }
            }
        }

        constants
    }
}

/// Generates an iterator over all possible slices of the input string. 
/// 
/// 
/// This function takes a string slice as input and creates an iterator that yields each possible substring of the input string. 
/// It uses a range from 0 to the length of the string to initiate the starting index of each slice. 
/// For each starting index, it employs `flat_map` combined with `char_indices` and `skip` to navigate through the string, creating substrings from each starting index to each subsequent character index. 
/// The resulting iterator efficiently covers all contiguous substrings in the original string, ensuring comprehensive slice generation without allocating additional string storage.
fn all_slices(a: &str) -> impl Iterator<Item = &str> {
    (0..a.len()).flat_map(move |i| a.char_indices().skip(i).map(move |(j, _)| &a[i..j + 1]))
}
