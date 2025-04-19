use std::cmp::min;

use derive_more::DebugCustom;
use crate::{forward::enumeration::Enumerator3, galloc::{AllocForStr, AllocForExactSizeIter}, impl_op3, new_op2, new_op3, parser::config::Config, value::Value};
use itertools::izip;

use super::Op3;

new_op2!(Eq, "=",
    (Int, Int) -> Bool { |(s1, s2)| s1 == s2 },
    (Str, Str) -> Bool { |(s1, s2)| s1 == s2 }
);

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
/// A structured data representation used to denote a conditional expression with two components. 
/// 
/// The first component is a `usize`, which can be used to reference an index or identifier within a specific context. 
/// The second component is a `bool`, which typically serves as a condition to determine the control flow within an algorithm. 
/// This structure could be utilized in scenarios where conditional logic plays a role in decision-making processes, enabling elegant integration with indexed data or scenarios.
/// 
pub struct Ite(pub usize, pub bool);

impl Ite {
    /// Creates an instance of `Ite` from a provided `Config` object. 
    /// 
    /// It initializes the `Ite` with a `usize` value extracted from the configuration under the key `"cost"`, defaulting to `1` if not present, and a `bool` value under the key `"enum"`, defaulting to `false`. 
    /// This method allows the flexible creation of `Ite` structures based on configuration settings, facilitating custom behavior in synthesis tasks where conditional logic representations are used.
    /// 
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_usize("cost").unwrap_or(1), config.get_bool("enum").unwrap_or(false))
    }
    /// The method returns the static string "ite", which signifies the name of the operation represented by the struct. 
    /// This simple function provides a way to retrieve the identifier for instances of the struct, likely correlating with its function or behavior within the string synthesis framework.
    pub fn name() ->  &'static str {
        "ite"
    }
}
impl std::fmt::Display for Ite {
    /// Formats the `Ite` type for display. 
    /// 
    /// The method implements custom formatting logic by invoking the `name` method of the `Ite` type and formatting its result. 
    /// This enables `Ite` instances to be displayed as strings according to their name representation, integrating seamlessly with Rust's formatting mechanisms.
    /// 
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Self::name().fmt(f) }
}
impl Default for Ite {
    /// Provides a method that constructs a default instance. 
    /// 
    /// This method utilizes another function, `from_config`, by supplying it with a default configuration, retrieved from Rust's standard `Default` trait. 
    /// This default instance serves as a baseline configuration, which can then be adjusted as necessary for specific string synthesis tasks. 
    /// The implementation underlines its utility by abstracting away the initial setup process, promoting a more streamlined use of the type within the module.
    /// 
    fn default() -> Self { Self::from_config(&Default::default()) }
}
impl Enumerator3 for Ite {
    /// Enumerates possible expressions using a three-way conditional operator based on available execution data and constraints. 
    /// 
    /// This method first checks a flag and ensures there is sufficient size in the `Executor` before proceeding. 
    /// It iterates over all combinations of expressions and values from specified non-terminal indices, `nt`, ensuring that their combined size is within allowable limits determined by a threshold based on the current execution size minus the expression's cost. 
    /// For each valid combination, it constructs a ternary expression using the provided `Op3Enum` instance as the operator and evaluates it. 
    /// When evaluation succeeds, the expression and its resultant value are processed further via the `Executor`. 
    /// The method returns a `Result<(), ()>` to indicate completion or failure without a specific error.
    /// 
    fn enumerate(&self, this: &'static super::Op3Enum, exec: &'static crate::forward::executor::Executor, nt: [usize; 3]) -> Result<(), ()> {
        if !self.1 { return Ok(())}
        if exec.size() < self.cost() { return Ok(()); }
        let total = exec.size() - self.cost();
        for (i, (e1, v1)) in exec.data[nt[0]].size.get_all_under(total) {
            for (j, (e2, v2)) in exec.data[nt[1]].size.get_all_under(total - i) {
                for (_, (e3, v3)) in exec.data[nt[2]].size.get_all_under(total - i - j) {
                    let expr = super::Expr::Op3(this, e1, e2, e3);
                    if let (true, value) = self.try_eval(*v1, *v2, *v3) {
                        exec.enum_expr(expr, value)?;
                    }
                }
            } 
        }
        Ok(())
    }
}

impl_op3!(Ite, "ite",
    (Bool, Int, Int) -> Int { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }},
    (Bool, Str, Str) -> Str { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }},
    (Bool, Bool, Bool) -> Bool { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }},
    (Bool, Float, Float) -> Float { |(s1, s2, s3)| {
        if *s1 {*s2} else {*s3}
    }}
);
