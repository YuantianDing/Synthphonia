use std::cmp::min;

use crate::{
    expr::{ops::Op3, Expr}, forward::enumeration::Enumerator3, galloc::{AllocForExactSizeIter, AllocForStr}, new_op3, parser::config::Config, value::Value
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A struct that represents a string replacement operation. 
/// 
/// It includes two public fields, both of type `usize`, which likely denote positions or lengths in a string where a replacement operation is intended to take place. 
/// The straightforward structure of this item suggests it serves as a utility to encapsulate parameters for a replace-like task within a larger synthesis or transformation process.
/// 
pub struct Replace(pub usize, pub usize);

impl Replace {
    /// Creates a new instance by extracting configuration values. 
    /// 
    /// It fetches the "cost" and "enum_replace_cost" from the given `Config` object, setting them as the first and second elements respectively. 
    /// If the configuration values are not present, it defaults to using 1 for "cost" and 3 for "enum_replace_cost". 
    /// This approach allows the object to be instantiated with specific costs based on the provided configuration, facilitating customizability within the synthesis framework.
    /// 
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1), config.get_usize("enum_replace_cost").unwrap_or(3))
    }
    /// Returns the name associated with the `Replace` operation. 
    /// 
    /// This functionality provides a static method that outputs the string `"str.replace"`, serving as an identifier for the operation within the synthesis framework. 
    /// This is useful for referencing the operation in logs, configuration, or other parts of the system where consistent naming is necessary.
    /// 
    pub fn name() -> &'static str {
        "str.replace"
    }
}

impl std::fmt::Display for Replace {
    /// Formats the `Replace` instance for display purposes. 
    /// 
    /// This implementation of the `fmt` function, part of the `std::fmt` module, uses the `name` method of the `Replace` struct itself to provide a formatted representation. 
    /// The formatted output is directed to a given formatter instance, which integrates the `Replace` instance into a formatted output stream.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Self::name().fmt(f)
    }
}

impl Default for Replace {
    /// Creates a new instance with default configuration. 
    /// 
    /// This method initializes the instance using the `from_config` function, passing a default configuration. 
    /// The implementation implies the `Replace` structure can be configured with external settings, yet defaults allow creating a baseline instance without specifying particular attributes.
    /// 
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl Enumerator3 for Replace {
    /// Enumerates possible expressions in the context of the synthesis problem. 
    /// 
    /// This function first checks if the executor's available size meets the minimum cost requirement, terminating early if it does not. 
    /// It calculates the total allowable size for enumeration and iterates over combinations of sub-expressions e1, e2, and e3 from the executor's data, constrained by the maximum specified size. 
    /// Within these loops, it constructs a ternary operation expression with the provided `Op3Enum`. 
    /// The function attempts to evaluate this new expression using given values, and if successful, it proceeds to enumerate the expression in the executor with its evaluated value. 
    /// The process ensures that only valid expressions with feasible evaluations are considered, thus optimizing the string synthesis tasks.
    /// 
    fn enumerate(&self, this: &'static crate::expr::ops::Op3Enum, exec: &'static crate::forward::executor::Executor, nt: [usize; 3]) -> Result<(), ()> {
        if exec.size() < self.cost() { return Ok(()); }
        let total = exec.size() - self.cost();
        for (i, (e2, v2)) in exec.data[nt[0]].size.get_all_under(min(total, self.1)) {
            for (j, (e3, v3)) in exec.data[nt[1]].size.get_all_under(min(total - i, self.1)) {
                for (e1, v1) in exec.data[nt[2]].size.get_all(total - i - j) {
                    let expr = Expr::Op3(this, e1, e2, e3);
                    if let (true, value) = self.try_eval(*v1, *v2, *v3) {
                        exec.enum_expr(expr, value)?;
                    }
                }
            } 
        }
        Ok(())
    }
}

impl Op3 for Replace {
    /// Provides functionality to calculate the cost of a `Replace` operation. 
    /// 
    /// The `cost` method, when called on an instance of `Replace`, returns the first element of the tuple. 
    /// This represents the operational cost or significance of the replacement process defined by the instance.
    /// 
    fn cost(&self) -> usize {
        self.0
    }
    /// Provides a method to attempt the evaluation of a replacement operation within a given context of string values. 
    /// 
    /// This method takes three `Value` parameters, `a1`, `a2`, and `a3`, assuming they are all strings. 
    /// It performs a replacement operation using the Rust `replacen` string method, which replaces the first occurrence of a substring (from `a1` and `a2` combinations) with a new string (`a3`). 
    /// The use of `itertools::izip!` allows iterating over the characters of the input strings in parallel, applying the replacement on each character triplet. 
    /// If the inputs match the expected string types, the method returns a tuple indicating success and the resulting string; otherwise, it returns a tuple indicating failure with a `Value::Null`. 
    /// The `galloc_str` and `galloc_scollect` methods are employed to efficiently handle memory allocation for the resulting strings.
    /// 
    fn try_eval(&self, a1: Value, a2: Value, a3: Value) -> (bool, Value) {
        match (a1, a2, a3) {
            (Value::Str(s1), Value::Str(s2), Value::Str(s3)) => (true, Value::Str(
                itertools::izip!(s1.iter(), s2.iter(), s3.iter())
                    .map(|(s1, s2, s3)| s1.replacen(*s2, s3, 1).galloc_str())
                    .galloc_scollect(),
            )),
            _ => (false, Value::Null),
        }
    }
}
