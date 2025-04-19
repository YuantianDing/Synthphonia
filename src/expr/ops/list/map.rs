use std::cmp::min;
use std::ops::Not;

use crate::expr::context::Context;
use crate::expr::Expr;
use crate::galloc::{AllocForExactSizeIter, AllocForStr, TryAllocForExactSizeIter};
use crate::parser::config::Config;
use crate::utils::F64;
use crate::{impl_op2, new_op1, new_op2, new_op2_opt, new_op3};
use derive_more::DebugCustom;
use itertools::izip;
use crate::value::Value;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A struct representing a mapping configuration for expressions. 
/// 
/// This structure contains a single field, an optional reference to a static `Expr` instance. 
/// The `Option` type encapsulates the possibility of this reference being `None`, indicating that the mapping may or may not point to a valid expression at any given time. 
/// This flexibility allows for dynamic adjustments within the synthesis process, where certain mappings might be deferred or omitted based on the current context or requirements.
/// 
pub struct Map(pub Option<&'static Expr>);

impl std::hash::Hash for Map {
    /// Provides a method to calculate the hash of the `Map` structure. 
    /// 
    /// The method takes a mutable reference to a generic hasher `H` and computes the hash by first accessing the internal `Option<&'static Expr>` value. 
    /// If the `Option` is `Some`, it maps the contained expression reference to a raw pointer and incorporates it into the hash calculation. 
    /// This ensures that the hashing process uniquely accounts for the memory address of the expression's reference, enhancing the precision of hash-based collections or algorithms using `Map` instances.
    /// 
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.map(|x| x as *const Expr).hash(state);
    }
}

impl Map {
    /// Creates a `Map` instance using the provided configuration. 
    /// 
    /// It retrieves an expression associated with the key `"f"` from the given `Config` object, which may be `None` if the key is not found, and wraps this expression in a `Map`. 
    /// This function assists in initializing a `Map` based on pre-defined configurations, facilitating customizable synthesis processes.
    /// 
    pub fn from_config(config: &Config) -> Self {
        Self(config.get_expr("f"))
    }
    /// Provides a method to retrieve the name associated with the `Map`. 
    /// 
    /// It returns a static string slice representing the operation's name within the context of the Synthphonia module, specifically identifying the `list.map` functionality. 
    /// This method is likely used to ensure consistency and readability when referring to the mapping operation internally or in debug outputs.
    /// 
    pub fn name() ->  &'static str {
        "list.map"
    }
}

impl std::fmt::Display for Map {
    /// Formats the `Map` structure for output. 
    /// 
    /// The function checks if the `Map` contains an `Expr` and, if so, formats the output to include the expression with a "list.map" prefix decorated by the expression's debug representation. 
    /// If no expression is present, the output simply includes "list.map". 
    /// This functionality is useful for logging or debugging when visual representation of the mapping operation is required.
    /// 
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(e) = self.0 {
            write!(f, "list.map #f:{:?}", e)
        } else {
            write!(f, "list.map")
        }
    }
}
impl Default for Map {
    /// Creates a default instance of the type by invoking the `from_config` method with a default configuration. 
    /// This method serves as a convenient way to generate an instance with standard settings, ensuring that if no specific configuration is provided, the type is still initialized in a consistent manner using default parameters.
    fn default() -> Self {
        Self::from_config(&Default::default())
    }
}

impl crate::forward::enumeration::Enumerator1 for Map {
    /// Provides a method for the `Map` structure to perform enumeration with given parameters, but currently returns a placeholder result. 
    /// 
    /// This method, `enumerate`, takes as parameters a reference to an instance of `Op1Enum`, a static reference to an `Executor`, and an array of one usize operand. 
    /// However, in its current implementation, it performs no actual enumeration or transformation logic and immediately returns a successful `Result` wrapped in `Ok(())` with an empty tuple as the error state. 
    /// This suggests that further enhancements or a use-case-specific implementation might be forthcoming for this structure within the application's context.
    /// 
    fn enumerate(&self, this: &'static crate::expr::ops::Op1Enum, exec: &'static crate::forward::executor::Executor, opnt: [usize; 1]) -> Result<(), ()> { Ok(()) }
}

impl crate::expr::ops::Op1 for Map {
    /// Calculates and returns the cost of the operation as a constant value. 
    /// 
    /// In this implementation, the method consistently returns `1`, indicating a fixed cost for any instance of the struct, regardless of the contents of the `Option<&'static Expr>` field. 
    /// This could be useful for basic cost computation scenarios where each `Map` instance contributes a uniform cost in the overall synthesis process.
    /// 
    fn cost(&self) -> usize { 1 }
    /// Provides a method to evaluate an expression contained within a `Map` structure using a given `Value`. 
    /// 
    /// If the input `Value` is of the type `ListStr`, it iterates over the list, creating a context for each string element that includes the length of the string and the string itself. 
    /// The expression is then evaluated within this context, converting the result to a string. 
    /// The output `Value` is adjusted to be a collection of these string results, and the method returns a tuple indicating success with a `true` boolean and the resultant `Value`. 
    /// If the input `Value` is not a `ListStr`, the method returns `false` and a `Null` `Value`.
    /// 
    fn try_eval(&self, a1: Value) -> (bool, Value) {
        let e = self.0.unwrap();
        if let Value::ListStr(a) = a1 {
            let a = a.iter().map(|&x| {
                let ctx = Context::new(x.len(), vec![x.into()], vec![], Value::Null);
                e.eval(&ctx).to_str()
            }).galloc_scollect();
            (true, a.into())
        } else { (false, Value::Null)}
    }
}
