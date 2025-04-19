use enum_dispatch::enum_dispatch;

use super::*;
use crate::parser::config::Config;
use crate::{value::Value, expr};
use std::future::Future;
use std::path::Display;

pub mod str;
use self::context::Context;
pub use self::str::*;

use crate::text::parsing::*;
use crate::text::formatting::*;
pub mod base;
pub use self::base::*;

pub mod int;
pub use self::int::*;
pub mod float;
pub use self::float::*;

pub mod list;
pub use self::list::*;


pub mod date;
pub use date::*;
pub mod macros;

#[enum_dispatch]
/// Defines a trait for unary operations that supports cloning and formatting. 
/// 
/// This trait requires implementing a method to determine the operation's cost and another to attempt evaluation on a single input value. 
/// The `cost` method returns the computational expense of the operation as a `usize`, providing a way to assess the relative resource consumption. 
/// The `try_eval` method takes a single argument, `a1`, of type `Value`, and returns a tuple, where the first element is a boolean indicating success or failure of the evaluation, and the second element contains the resulting `Value`. 
/// This trait layout allows for flexibility and modularity when defining unary operations within the string synthesis framework. 
/// 
/// 
pub trait Op1: Clone + std::fmt::Display {
    fn cost(&self) -> usize;
    fn try_eval(&self, a1: Value) -> (bool, Value);
}

impl Op1Enum {
    /// Evaluates a unary operation on a given value and returns the result. 
    /// 
    /// This function takes a reference to self as an `Op1Enum` instance and a `Value` representing the operand for the unary operation. 
    /// It attempts to evaluate the operation by calling the `try_eval` method with the provided argument and returns the second element of the resulting tuple, which is the computed `Value`. 
    /// This method assumes that the operation is successfully executed, as it directly takes the result part of the tuple from `try_eval`.
    /// 
    pub fn eval(&self, a1: Value) -> Value {
        
        self.try_eval(a1).1
    }
}

#[enum_dispatch]
/// Defines a trait for binary operations in the string synthesis framework. 
/// 
/// 
/// The trait requires implementors to include methods for calculating the cost associated with executing the operation and attempting evaluation with two input values, returning a tuple with a boolean indicating success or failure and the resultant value. 
/// It also requires implementors to derive clone and display functionalities, ensuring that all binary operations can be easily duplicated and formatted to strings for display purposes.
pub trait Op2 : Clone + std::fmt::Display {
    fn cost(&self) -> usize;
    fn try_eval(&self, a1: Value, a2: Value) -> (bool, Value);
}

impl Op2Enum {
    /// Evaluates a binary operation encapsulated by the `Op2Enum`. 
    /// 
    /// It takes two arguments, both of type `Value`, and returns the result of attempting the operation. 
    /// The method utilizes the `try_eval` function internally, discarding the primary component of its result and only retaining the secondary element, which represents the successfully evaluated output. 
    /// This signifies that while the `try_eval` function may return additional diagnostic information or status, this method focuses solely on obtaining the computed value from the operation.
    /// 
    pub fn eval(&self, a1: Value, a2: Value) -> Value { self.try_eval(a1, a2).1 }
}

#[enum_dispatch]
/// A trait defining a ternary operation. 
/// 
/// It represents operations that take three argument values and provides methods to evaluate and determine the cost of performing the operation. 
/// Implementations of this trait must provide a `cost` method that returns the operation's cost as an unsigned size. 
/// The `try_eval` method attempts to evaluate the operation with three input values, returning a tuple where the first element indicates success as a boolean and the second element contains the resultant value. 
/// This trait requires its implementers to be clonable and displayable, facilitating duplication and formatted output of operation instances.
/// 
pub trait Op3 : Clone + std::fmt::Display {
    fn cost(&self) -> usize;
    fn try_eval(&self, a1: Value, a2: Value, a3: Value) -> (bool, Value);
}

impl Op3Enum {
    /// Provides an evaluation method for the `Op3Enum` operations. 
    /// 
    /// Invokes the `try_eval` method with three provided `Value` arguments and returns the second element of the tuple resulting from the `try_eval` call. 
    /// This method abstracts the direct invocation of operation logic encapsulated in `try_eval`, emphasizing the resultant value of the operation within ternary operation contexts.
    /// 
    pub fn eval(&self, a1: Value, a2: Value, a3: Value) -> Value { self.try_eval(a1, a2, a3).1 }
}

#[enum_dispatch(Op1)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An enum that defines unary operations for expressions within the string synthesis framework. 
/// 
/// This enumeration, `Op1Enum`, includes a wide array of operations that can be applied to a single operand. 
/// The operations cover a diverse set of functionalities such as conversions between data types (e.g., `ToInt`, `ToStr`, `IntToFloat`, `FloatToInt`, `StrToFloat`), string manipulations like changing case (`Uppercase`, `Lowercase`) and retaining specific character types (`RetainLl`, `RetainLc`, `RetainN`, `RetainL`, `RetainLN`). 
/// 
/// 
/// Additionally, the enum supports various mathematical and logical checks (`IsPos`, `IsZero`, `IsNatural`, `FIsPos`, `FIsZero`, `FNotNeg`), numerical operations (`Neg`, `FNeg`, `FAbs`, `FExp10`), formatting (`FormatInt`, `FormatFloat`, `FormatTime`, `FormatMonth`, `FormatWeekday`), and parsing (`ParseTime`, `ParseDate`, `ParseInt`, `ParseMonth`, `ParseWeekday`, `ParseFloat`). 
/// It also includes utilities like `Len` for measuring length and several date-related transformations (`AsMonth`, `AsDay`, `AsYear`, `AsWeekDay`). 
/// This diverse suite of operations enables flexible and efficient manipulation of data types required for string synthesis challenges.
pub enum Op1Enum {
    Len,
    ToInt,
    ToStr,
    Neg,
    IsPos,
    IsZero,
    IsNatural,
    RetainLl,
    RetainLc,
    RetainN,
    RetainL,
    RetainLN,
    Map,
    Uppercase,
    Lowercase,
    AsMonth,
    AsDay,
    AsYear,
    AsWeekDay,
    ParseTime,
    ParseDate,
    ParseInt,
    ParseMonth,
    ParseWeekday,
    ParseFloat,
    FormatInt,
    FormatFloat,
    FormatTime,
    FormatMonth,
    FormatWeekday,
    FNeg,
    FAbs,
    FIsPos,
    FExp10,
    IntToFloat,
    FloatToInt,
    StrToFloat,
    FIsZero,
    FNotNeg,
    FLen,
}
impl std::fmt::Display for Op1Enum {
    /// Formats the operation represented by `Op1Enum` for printing. 
    /// 
    /// This implementation attempts to match the enum variant of the `Op1Enum` and writes its argument to the provided formatter. 
    /// The macro `crate::for_all_op1!()` is used to iterate over all possible unary operation variants, facilitating the matching process. 
    /// If an appropriate match is found, it writes the associated argument to the formatter. 
    /// If no match is found using the macro, it completes without any action. 
    /// This approach enables a concise and consistent output for each variant when formatted.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(a) = self {
                    return write!(f, "{a}");
                }
            )*
        }}
        crate::for_all_op1!();
        Ok(())
    }
}

impl Op1Enum {
    /// Implements a method to create an instance of the enumeration from a string identifier. 
    /// 
    /// This method takes a string `name` representing the desired operation and a `config` reference, which helps configure certain operational aspects, to return the corresponding `Op1Enum` variant. 
    /// Internally, it uses a macro, `_do`, to iterate over potential operations and check if their names match the given string identifier. 
    /// If a match is found, it returns the operation configured with the supplied configuration. 
    /// For specific operations like "str.len", "str.from_int", and "str.to_int", direct matches that do not utilize the macro are provided for convenience. 
    /// If no operation matches the given string name, the method panics with an "Unknown Operator" error message.
    /// 
    pub fn from_name(name: &str, config: &Config) -> Self {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if $op::name() == name {
                    return $op::from_config(config).into();
                }
            )*
        }}
        crate::for_all_op1!();
        match name {
            "str.len" => Len::from_config(config).into(),
            "str.from_int" => ToStr::from_config(config).into(),
            "str.to_int" => ToInt::from_config(config).into(),
            _ => panic!("Unknown Operator {}", name),
        }
    }
    /// Provides a method to retrieve the name of a unary operation as a static string. 
    /// 
    /// The method uses a macro to match the current instance of the enumeration against all possible variants of unary operations defined by `Op1Enum`. 
    /// If the current instance matches one of these variants, it returns the corresponding name by invoking the `name()` method of the matched operation. 
    /// It leverages a macro named `for_all_op1!` to iterate through all operation variants, ensuring extensibility and maintainability when new operations are introduced. 
    /// If no match is found, the method panics, indicating an unexpected state or missing variant handling.
    /// 
    pub fn name(&self) -> &'static str {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(_) = self {
                    return $op::name();
                }
            )*
        }}
        crate::for_all_op1!();
        panic!()
    }
}

#[enum_dispatch(Op2)]
#[derive(Clone, PartialEq, Eq, Hash)]
/// An enum representing binary operations used in the expression manipulation framework. 
/// 
/// This enumeration includes a diverse set of operations applicable to strings, numbers, lists, and time-related data. 
/// Typical operations include string manipulations such as `Concat`, `PrefixOf`, and `Contains`, which allow for constructing and checking properties of strings. 
/// There are also numerical operations like `Add`, `Sub`, alongside floating-point specific operations like `FAdd`, `FSub`, and rounding techniques such as `Floor`, `Round`, and `Ceil`.
/// 
/// Moreover, the enum encapsulates list operations such as `Head`, `Tail`, and `Filter`, indicating capabilities to manipulate and traverse lists. 
/// Time-based operations like `TimeFloor`, `TimeAdd`, and `TimeMul` are included, reflecting tasks related to temporal data. 
/// `Split` and `Join` manage compound string or list structures, and `StrAt` and `At` facilitate index-based access in strings or lists. 
/// The enumeration is designed to accommodate various contexts and operations necessary for a comprehensive synthesis framework, supporting diverse data types and manipulation techniques.
pub enum Op2Enum {
    Concat,
    Eq,
    At,
    PrefixOf,
    SuffixOf,
    Contains,
    Split,
    Join,
    Count,
    Add,
    Sub,
    Head,
    Tail,
    Filter,
    TimeFloor,
    TimeAdd,
    Floor, Round, Ceil,
    FAdd, FSub, FFloor, FRound, FCeil, FCount, FShl10, TimeMul, StrAt
}

impl std::fmt::Display for Op2Enum {
    /// Provides a formatting implementation for the `Op2Enum` type, utilizing the Rust standard library's `fmt` trait. 
    /// 
    /// This method employs a macro to streamline the process of matching against all possible `Op2Enum` variants, applying the formatting operation uniformly. 
    /// Within the macro `_do`, each variant is checked using pattern matching, and upon a match, it writes the variant's name to the provided formatter. 
    /// The macro ultimately integrates with a custom crate-level macro `crate::for_all_op2!()` to encompass all operations within `Op2Enum`, ensuring the method comprehensively formats each operation, returning a `Result` to indicate success or failure of the formatting operation.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(a) = self {
                    return write!(f, "{a}");
                }
            )*
        }}
        crate::for_all_op2!();
        Ok(())
    }
}

impl Op2Enum {
    /// Converts a string name into an `Op2Enum` variant by matching the provided name with known operation names and configurations. 
    /// 
    /// The function utilizes a macro to iterate through all defined binary operations (`Op2`) and checks if the operation's name matches the input string. 
    /// If a match is found, it retrieves the operation with the given configuration and converts it into the `Op2Enum` type. 
    /// For specific operators like `"+"` and `"-"`, the function directly constructs their corresponding `Add` or `Sub` variants, respectively. 
    /// If no matching operation is found, it raises a panic with an error message indicating the unknown operator name.
    /// 
    pub fn from_name(name: &str, config: &Config) -> Self {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if $op::name() == name {
                    return $op::from_config(config).into();
                }
            )*
        }}
        crate::for_all_op2!();
        match name {
            "+" => Add::from_config(config).into(),
            "-" => Sub::from_config(config).into(),
            _ => panic!("Unknown Operator: {}", name),
        }
    }
    /// Returns the name of the operation represented by the given instance of the enumeration. 
    /// 
    /// This implementation utilizes a macro to iterate over possible operation variants defined in the enumeration, invoking the `name` method on each. 
    /// The `name` method of a specific operation is called, and its result is returned if the instance matches one of the variants. 
    /// If no match is found, the function will trigger a panic, indicating that the instance does not correspond to a recognized operation variant. 
    /// This design aims to streamline name retrieval across multiple operation types within the `Op2Enum`.
    /// 
    pub fn name(&self) -> &'static str {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(_) = self {
                    return $op::name();
                }
            )*
        }}
        crate::for_all_op2!();
        panic!()
    }
}

#[enum_dispatch(Op3)]
#[derive(Clone, PartialEq, Eq, Hash)]
/// An enum representing ternary operations in the string synthesis framework. 
/// 
/// This enum includes operations such as `Replace`, which substitutes a part of a string with another substring, and `Ite` (if-then-else), which selects between two expressions based on a condition. 
/// It also includes `SubStr`, which extracts a portion of a string specified by a starting index and length, and `IndexOf`, which determines the index of a substring within another string. 
/// These operations are essential for manipulating strings in complex synthesis tasks.
/// 
pub enum Op3Enum {
    Replace,
    Ite,
    SubStr,
    IndexOf,
}

impl std::fmt::Display for Op3Enum {
    /// Formats an `Op3Enum` variant into a string. 
    /// 
    /// The functionality uses a macro to match the variant of `Op3Enum` and writes its value (`a`) to the given formatter. 
    /// If the variant matches, the function returns the result of the formatted write operation. 
    /// If none of the variants match, it defaults to resolving successfully with an `Ok(())`. 
    /// The macro `crate::for_all_op3!()` is employed to handle all possible variants of `Op3Enum`, allowing concise and reusable code patterns for formatting each operation in the enumeration.
    /// 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(a) = self {
                    return write!(f, "{a}");
                }
            )*
        }}
        crate::for_all_op3!();
        Ok(())
    }
}

impl Op3Enum {
    /// Provides a method for creating an instance of an operation from its name and a specified configuration. 
    /// 
    /// The `from_name` function takes a string slice representing the name of the operation and a reference to a `Config` object. 
    /// Utilizing a macro named `_do`, it iterates over all possible operations within the `Op3Enum` by dynamically executing each operation's name comparison. 
    /// If a match is found, it returns the corresponding operator configured via the `from_config` method. 
    /// In case no matching operation name is found, the function will terminate execution and issue a panic with an error message indicating the unknown operator. 
    /// This method ensures that each operation can be instantiated from a configuration while providing runtime safety against undefined operations.
    /// 
    pub fn from_name(name: &str, config: &Config) -> Self {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if $op::name() == name {
                    return $op::from_config(config).into();
                }
            )*
        }}
        crate::for_all_op3!();
        panic!("Unknown Operator: {}", name);
    }
    /// Provides an implementation to retrieve the name of an operation represented by this item. 
    /// 
    /// This is achieved using a macro to iterate over a series of operations associated with the item, checking if the current instance matches any of these operations and returning its name via a helper function defined for each operation. 
    /// The logic ensures that for any valid instance of this item, the correct associated name is returned. 
    /// If none of the operations match, it results in a panic, indicating an unexpected state.
    /// 
    pub fn name(&self) -> &'static str {
        macro_rules! _do { ($($op:ident)*) => {
            $(
                if let Self::$op(_) = self {
                    return $op::name();
                }
            )*
        }}
        crate::for_all_op3!();
        panic!()
    }
}

pub mod op_impl;
