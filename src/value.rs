use derive_more::DebugCustom;
use derive_more::Display;
use derive_more::TryInto;
use derive_more::From;
use itertools::Itertools;

use crate::expr::Expr;
use crate::galloc::AllocForExactSizeIter;
use crate::galloc::AllocForIter;
use crate::tree_learning::bits::BoxSliceExt;
use crate::tree_learning::Bits;
use crate::utils::F64;


#[derive(DebugCustom, PartialEq, Eq, Clone, Copy, Hash)]
/// Represents a comprehensive set of distinct type variants including basic and list-based types. 
/// 
/// 
/// Defines available kinds such as null, integer, boolean, string, float, and their corresponding list forms for integers and strings, with each variant accompanied by custom formatting annotations intended for debugging and display purposes.
pub enum Type {
    #[debug(fmt = "Null")]
    Null,
    #[debug(fmt = "Int")]
    Int,
    #[debug(fmt = "Bool")]
    Bool,
    #[debug(fmt = "String")]
    Str,
    #[debug(fmt = "Float")]
    Float,
    #[debug(fmt = "(List Int)")]
    ListInt,
    #[debug(fmt = "(List String)")]
    ListStr,
}

impl Type {
    /// Returns the corresponding basic type by extracting the underlying non-list variant from a list type, or leaves the type unchanged if it is already basic. 
    /// 
    /// 
    /// Evaluates the input type and, if it represents a list of integers or a list of strings, returns the respective integer or string type; otherwise, it returns the original value.
    pub fn basic(self) -> Type {
        match self {
            Type::ListInt => Self::Int,
            Type::ListStr => Self::Str,
            a => a,
        }
    }
    /// Converts a basic type into its corresponding list variant when applicable. 
    /// 
    /// 
    /// Returns an optional list type; if the input is a primitive integer or string type, it wraps the corresponding list type inside a Some, otherwise it returns None.
    pub fn to_list(self) -> Option<Type> {
        match self {
            Type::Int => Some(Type::ListInt),
            Type::Str => Some(Type::ListStr),
            _ => None
        }
    }
}

#[derive(DebugCustom, Clone, TryInto, Copy, PartialEq, Eq, Hash, From)]
/// A collection of constant values representing various primitive and collection types. 
/// 
/// This enumeration encapsulates integers, floats, booleans, and strings as well as lists of integers and strings, with each variant storing its data as a static slice to ensure efficient access. 
/// Additionally, a null variant is provided to denote the absence of a value.
/// 
pub enum Value {
    #[debug(fmt = "{:?}", _0)]
    Int(&'static [i64]),
    #[debug(fmt = "{:?}", _0)]
    Float(&'static [F64]),
    #[debug(fmt = "{:?}", _0)]
    Bool(&'static [bool]),
    #[debug(fmt = "{:?}", _0)]
    Str(&'static [&'static str]),
    #[debug(fmt = "{:?}", _0)]
    ListInt(&'static [&'static [i64]]),
    #[debug(fmt = "{:?}", _0)]
    ListStr(&'static [&'static [&'static str]]),
    #[debug(fmt = "null")]
    Null,
}

impl Value {

    /// Transforms the current value by selecting elements at indices specified in the examples slice and produces a new value of the same variant. 
    /// 
    /// 
    /// Iterates over the elements in the internal collection corresponding to the variant and constructs a new value by extracting items at the given indices. 
    /// If the variant is Null, the result is also Null.
    pub fn with_examples(self, exs: &[usize]) -> Value {
        match self {
            Value::Int(a) => Value::Int(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Float(a) => Value::Float(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Bool(a) => Value::Bool(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Str(a) => Value::Str(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::ListInt(a) => Value::ListInt(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::ListStr(a) => Value::ListStr(exs.iter().cloned().map(|i| a[i]).galloc_scollect()),
            Value::Null => Value::Null,
        }
    }
}

impl Value {
    /// Returns the type corresponding to the variant of the value. 
    /// This function examines the value's variant and returns the associated type, ensuring that each kind of value consistently maps to its specific type.
    pub fn ty(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Bool(_) => Type::Bool,
            Self::Str(_) => Type::Str,
            Self::Float(_) => Type::Float,
            Self::ListInt(_) => Type::ListInt,
            Self::ListStr(_) => Type::ListStr,
            Self::Null => Type::Null,
        }
    }
    #[inline(always)]
    /// Returns the number of elements contained within the value. 
    /// 
    /// 
    /// Examines the variant of the value and computes its length accordingly, using the inherent length of the underlying slice or collection, with a length of zero returned for the null variant.
    pub fn len(&self) -> usize {
        match self {
            Value::Int(a) => a.len(),
            Value::Bool(b) => b.len(),
            Value::Str(s) => s.len(),
            Value::Float(s) => s.len(),
            Value::ListInt(l) => l.len(),
            Value::ListStr(l) => l.len(),
            Value::Null => 0,
        }
    }
    #[inline(always)]
    /// Returns an optional vector of element lengths contained within the value. 
    /// This method computes and returns a vector of lengths for each element when applicable, such as when the value holds strings or lists; for other cases, it yields None, indicating that individual element lengths are not defined.
    pub fn length_inside(&self) -> Option<Vec<usize>> {
        match self {
            Value::Int(a) => None,
            Value::Bool(b) => None,
            Value::Float(s) => None,
            Value::Null => None,
            Value::Str(s) => Some(s.iter().map(|x| x.len()).collect_vec()),
            Value::ListInt(l) => Some(l.iter().map(|x| x.len()).collect_vec()),
            Value::ListStr(l) => Some(l.iter().map(|x| x.len()).collect_vec()),
        }
    }
    #[inline(always)]
    /// Flattens the contained string(s) into a unified static slice of string references. 
    /// 
    /// This function operates by taking a value holding either a singular string or a list of strings and returns a reference to a static slice where each element corresponds to a single-character substring from the original strings. 
    /// It panics if the value is of any other type, ensuring that only supported string types are processed.
    /// 
    pub fn flatten_leak(&self) -> &'static [&'static str] {
        // Memory Leak !!!
        match self {
            Value::Str(s) => s.iter().flat_map(|x| (0..x.len()).map(|i| &x[i..i+1]) ).galloc_collect(),
            Value::ListStr(l) => l.iter().flat_map(|x| x.iter().copied()).galloc_collect(),
            _ => panic!("Mismatched type: to_liststr_leak")
        }
    }
    #[inline(always)]
    /// Converts a value holding strings into an optional flattened representation as a static slice of string slices. 
    /// 
    /// 
    /// Checks if the input value encapsulates either individual strings or a list of strings and produces a flattened collection where each element represents a single-character string slice or an element from the list, respectively. 
    /// If the value does not match these string types, it returns None.
    pub fn try_flatten_leak(&self) -> Option<&'static [&'static str]> {
        // Memory Leak !!!
        match self {
            Value::Str(s) => Some(s.iter().flat_map(|x| (0..x.len()).map(|i| &x[i..i+1]) ).galloc_collect()),
            Value::ListStr(l) => Some(l.iter().flat_map(|x| x.iter().copied()).galloc_collect()),
            _ => None,
        }
    }

    /// Creates a synthesized value from an iterator of constant values based on the specified type. 
    /// 
    /// 
    /// Converts each constant from the provided iterator to the corresponding native variant by mapping through type-specific conversion methods and collecting the results into the proper aggregated structure. 
    /// If the type is unsupported for such conversion, the function triggers a panic with an appropriate error message.
    pub fn from_const(ty: Type, constants: impl ExactSizeIterator<Item=ConstValue>) -> Self {
        match ty {
            Type::Bool => Value::Bool(constants.map(|p| p.as_bool().unwrap()).galloc_scollect()),
            Type::Int => Value::Int(constants.map(|p| p.as_i64().unwrap()).galloc_scollect()),
            Type::Str => Value::Str(constants.map(|p| p.as_str().unwrap()).galloc_scollect()),
            Type::Float => Value::Float(constants.map(|p| p.as_float().unwrap()).galloc_scollect()),
            _ => panic!("should not reach here"),
        }
    }
    /// Checks whether every string element in the first value is a substring of the corresponding string element in the second value.
    /// 
    /// Operates by iterating over paired elements from two string collections and returning true only if each element of the first collection is contained within the corresponding element of the second; if either value is not a string collection, it returns false.
    pub fn substr(&self, other: &Value) -> bool{
        match (self, other) {
            (Value::Str(s), Value::Str(o)) => s.iter().zip(o.iter()).all(|(a,b)| b.contains(a)),
            _ => false,
        }
    }
    /// Checks whether any string in the first value appears as a substring in the corresponding string of the second value. 
    /// This function compares two values and, when both are collections of strings, pairs each corresponding element using a zipper technique and returns true if at least one pair satisfies the substring condition; in all other cases it returns false.
    pub fn some_substr(&self, other: &Value) -> bool{
        match (self, other) {
            (Value::Str(s), Value::Str(o)) => s.iter().zip(o.iter()).any(|(a,b)| b.contains(a)),
            _ => false,
        }
    }
    /// Converts an internal value into a statically allocated slice of string slices. 
    /// 
    /// 
    /// Transforms the underlying data by attempting to convert the current value into the desired string slice representation and immediately unwrapping the result. 
    /// This operation guarantees that the conversion succeeds, provided the value is compatible with the expected type.
    pub fn to_str(self) -> &'static [&'static str] {
        self.try_into().unwrap()
    }
    /// Converts a value into a static list of string slices by performing a conversion using the TryInto trait. 
    /// Panics if the conversion fails, returning the resulting list of string slices upon success.
    pub fn to_liststr(self) -> &'static [&'static [&'static str]] {
        self.try_into().unwrap()
    }
    /// Converts the value into a static slice of booleans. 
    /// This function attempts to transform the value into the corresponding boolean slice using an internal conversion mechanism and returns the result, panicking if the conversion fails.
    pub fn to_bool(self) -> &'static [bool] {
        self.try_into().unwrap()
    }
    /// Converts the boolean representation contained in the receiver into a bit vector. 
    /// 
    /// 
    /// Transforms the boolean values of the instance into a sequential bit representation suitable for bitwise manipulation. 
    /// This function extracts a boolean slice from the instance and converts it into an aggregated Bits value.
    pub fn to_bits(self) -> Bits {
        Bits::from_bit_siter(self.to_bool().iter().cloned())
    }
    /// Checks whether all boolean values in the object are true. 
    /// 
    /// 
    /// Determines if the object, when representing boolean values, contains only true elements by iterating over the booleans and verifying each one is true; if the object does not represent booleans, it returns false.
    pub fn is_all_true(&self) -> bool {
        if let Self::Bool(b) = self {
            b.iter().all(|x| *x)
        } else { false }
    }
    /// Determines whether all boolean values in the instance evaluate to false.
    /// 
    /// Evaluates the content of the instance by checking if it represents a boolean collection, returning true only when every boolean in that collection is false; otherwise, it returns false, defaulting to false if the instance does not correspond to a boolean value.
    pub fn is_all_false(&self) -> bool {
        if let Self::Bool(b) = self {
            b.iter().all(|x| !(*x))
        } else { false }
    }
    /// Checks whether every string element within the value is empty.
    /// 
    /// Determines if the current instance contains a collection of strings and, if so, verifies that all strings have zero length, returning false if the value is of a different type.
    pub fn is_all_empty(&self) -> bool {
        if let Self::Str(b) = self {
            b.iter().all(|x| x.is_empty())
        } else { false }
    }
    /// Negates each boolean element in the input value and returns a new value with the negated booleans.
    /// 
    /// This function converts the current instance into a boolean slice, applies logical negation to every element, and then collects the results back into a new instance that encapsulates the transformed boolean values.
    pub fn bool_not(self) -> Value {
        let this = self.to_bool();
        this.iter().map(|x| !x).galloc_scollect().into()
    }
    /// Computes the number of pairwise equal elements shared between two values. 
    /// 
    /// This function compares the corresponding elements of two instances by iterating over their internal collections and counting pairs that are equal. 
    /// It supports several variants (such as integers, strings, floats, booleans, and lists) by aligning elements in parallel; when the compared types do not match, it returns zero.
    /// 
    pub fn eq_count(&self, other: &Self) -> usize {
        match (self, other) {
            (Self::Int(a1), Self::Int(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::Str(a1), Self::Str(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::Float(a1), Self::Float(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::Bool(a1), Self::Bool(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::ListInt(a1), Self::ListInt(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            (Self::ListStr(a1), Self::ListStr(a2)) => a1.iter().zip(a2.iter()).filter(|(a, b)| a == b).count(),
            _ => 0,
        }
    }
    /// Compares two values and computes a bit mask representing elementwise equality. 
    /// 
    /// This function performs an elementwise comparison between the contents of two values of the same specific variant and returns an optional bit mask where each bit indicates whether corresponding elements are equal. 
    /// If the two values do not belong to a matching variant that supports elementwise comparison (e.g., comparing an integer sequence to a boolean sequence), the function returns None.
    /// 
    pub fn eq_bits(&self, other: &Self) -> Option<Bits> {
        match (self, other) {
            (Self::Int(a1), Self::Int(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::Str(a1), Self::Str(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::Float(a1), Self::Float(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::Bool(a1), Self::Bool(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::ListInt(a1), Self::ListInt(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            (Self::ListStr(a1), Self::ListStr(a2)) => Some(Bits::from_bit_siter(a1.iter().zip(a2.iter()).map(|(a, b)| a == b))),
            _ => None,
        }
    }
}


#[derive(DebugCustom, Display, PartialEq, Eq, Hash, Clone, Copy, From)]
/// Represents a constant value that abstracts various literal types and expressions. 
/// This type encapsulates null, boolean, integer, string, floating-point, and expression values, each with respective formatting behavior for debugging and display purposes.
pub enum ConstValue {
    #[debug(fmt = "null")]
    #[display(fmt = "null")]
    Null,
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Bool(bool),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Int(i64),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Str(&'static str),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Float(F64),
    #[debug(fmt = "{:?}", _0)]
    #[display(fmt = "{:?}", _0)]
    Expr(&'static Expr)
}

impl From<usize> for ConstValue {
    /// Converts a usize value into a constant integer by casting it into a 64-bit integer. 
    /// 
    /// 
    /// Transforms a usize into its integer representation wrapped as a constant, enabling its use in contexts where a constant of integer type is expected.
    fn from(value: usize) -> Self {
        Self::Int(value as i64)
    }
}
impl From<u32> for ConstValue {
    /// Converts an unsigned 32-bit integer into a constant integer value.
    /// 
    /// This function takes a u32 as input and returns the corresponding constant integer value by converting the input to a 64-bit integer internally.
    fn from(value: u32) -> Self {
        Self::Int(value as i64)
    }
}

impl ConstValue {
    /// Returns the type corresponding to a constant value instance. 
    /// This function maps each variant of the constant value to its respective type: integer, boolean, string, float, or null, with the expression variant treated as null.
    pub fn ty(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Bool(_) => Type::Bool,
            Self::Str(_) => Type::Str,
            Self::Float(_) => Type::Float,
            Self::Null => Type::Null,
            Self::Expr(_) => Type::Null,
        }
    }
    #[inline(always)]
    /// Returns an optional boolean extracted from the constant. 
    /// This function checks if the constant holds a boolean value, and if so, returns it wrapped in Some; otherwise, it returns None.
    pub fn as_bool(&self) -> Option<bool> { if let Self::Bool(b) = self { Some(*b) } else { None }}
    #[inline(always)]
    /// Returns the contained integer value if the input represents one. 
    /// This function checks if the instance holds an integer and, if so, returns it inside an Option; otherwise, it yields None.
    pub fn as_i64(&self) -> Option<i64> { if let Self::Int(b) = self { Some(*b) } else { None }}
    /// Converts the integer variant of a constant value into a usize if applicable. 
    /// 
    /// 
    /// Returns an Option containing the usize representation when the constant is an integer; otherwise, it produces None.
    pub fn as_usize(&self) -> Option<usize> { if let Self::Int(b) = self { Some(*b as usize) } else { None }}
    /// Returns an Option wrapping a static string reference if the constant holds a string value, or None otherwise. 
    /// This method inspects the constant value and, when it represents a string, provides access to the underlying static string slice, enabling string-specific processing while gracefully handling non-string cases.
    pub fn as_str(&self) -> Option<&'static str> { if let Self::Str(b) = self { Some(*b) } else { None }}
    /// Returns the floating-point value contained in the constant, if available.
    /// 
    /// Checks whether the constant is of the float variant, and if so, returns its underlying value wrapped in an Option; otherwise, it produces None.
    pub fn as_float(&self) -> Option<F64> { if let Self::Float(b) = self { Some(*b) } else { None }}
    /// Returns an optional static reference to an expression if the constant value represents an expression variant. 
    /// This function checks whether the constant holds an expression and, if so, returns it wrapped in an option; otherwise, it returns None.
    pub fn as_expr(&self) -> Option<&'static Expr> { if let Self::Expr(b) = self { Some(*b) } else { None }}
    /// Extracts a floating-point value from a constant if it represents a float. 
    /// This function checks the internal variant and, when it holds a float, retrieves the underlying f64 value wrapped in an Option; otherwise, it returns None.
    pub fn as_f64(&self) -> Option<f64> { if let Self::Float(b) = self { Some(**b) } else { None }}
    /// Checks whether a constant value is null. 
    /// 
    /// 
    /// Determines if the constant value represents a null literal and returns true if so, otherwise false.
    pub fn is_null(&self) -> bool { matches!(self, Self::Null) }
    /// Converts a constant configuration into a collection value by replicating its underlying data over a specified length. 
    /// 
    /// 
    /// Matches on the constant variant to determine the appropriate data type and constructs a value by repeatedly inserting the constant element for each index in the specified range. 
    /// For boolean, integer, string, and float constants, a collection of repeated elements is created, while attempting to convert a null or expression constant results in a panic.
    pub fn value(&self, len: usize) -> Value {
        match self {
            ConstValue::Bool(t) => Value::Bool((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Int(t) => Value::Int((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Str(t) => Value::Str((0..len).map(|_| *t).galloc_scollect()),
            ConstValue::Float(f) => Value::Float((0..len).map(|_| *f).galloc_scollect()),
            ConstValue::Null => panic!("Unable to convert Null to Value"),
            ConstValue::Expr(_) => panic!("Unable to convert Expr to Value"),
        }
    }

}

/// Converts a vector of constant values into a value variant by deducing the target type from the first element. 
/// 
/// It traverses the provided vector, extracts the underlying primitive (boolean, integer, string, or float) for each constant, and collects the results into an allocated collection to form the corresponding value variant. 
/// For constant expressions and null values, the conversion remains unimplemented.
/// 
pub fn consts_to_value(consts: Vec<ConstValue>) -> Value {
    match consts[0] {
        ConstValue::Null => todo!(),
        ConstValue::Bool(_) => Value::Bool(consts.into_iter().map(|a| a.as_bool().unwrap()).galloc_scollect()),
        ConstValue::Int(_) => Value::Int(consts.into_iter().map(|a| a.as_i64().unwrap()).galloc_scollect()),
        ConstValue::Str(_) => Value::Str(consts.into_iter().map(|a| a.as_str().unwrap()).galloc_scollect()),
        ConstValue::Float(_) => Value::Float(consts.into_iter().map(|a| a.as_float().unwrap()).galloc_scollect()),
        ConstValue::Expr(_) => todo!(),
    }
}

#[macro_export]
/// This macro converts literal values into constant representations based on their type. 
/// 
/// It accepts the literal tokens true and false to produce boolean constants, and for a general literal it attempts runtime type checking to determine whether it is a string, an integer, or a floating-point value, converting it accordingly. 
/// If the literal does not match any of these supported types, the macro causes a panic with an invalid literal message.
/// 
macro_rules! const_value {
    (true) => {$crate::value::ConstValue::Bool(true)};
    (false) => {$crate::value::ConstValue::Bool(false)};
    ($l:literal) => { 
        if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<&str>() {
            $crate::value::ConstValue::Str(f)
        } else if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<i32>() {
            crate::value::ConstValue::Int(*f as i64)
        } else if let Some(f) = (&$l as &dyn std::any::Any).downcast_ref::<f64>() {
            crate::value::ConstValue::Float((*f as f64).into())
        } else { panic!("Invalid literal {}", $l) }
    };
}
