
#[macro_export]
/// Defines a macro for creating structures with basic functionality. 
/// 
/// This macro generates a structure that includes a single `usize` field, implements several traits, and provides methods related to configuration and naming. 
/// For any given identifier and a name, the generated structure derives traits for debugging, cloning, copying, equality comparison, and hashing. 
/// It includes a method to create an instance from a configuration, specifically fetching a "cost" value, defaulting to 1 if unspecified. 
/// There’s also a method for obtaining the name of the structure as a static string. 
/// Additionally, it implements the `Display` trait for formatted output of the structure's name, and provides a default implementation that creates an instance using a default configuration.
/// 
macro_rules! impl_basic {
    ($s:ident, $name:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $s(pub usize);
        impl $s {
            pub fn from_config(config: &$crate::parser::config::Config) -> Self {
                Self(config.get_usize("cost").unwrap_or(1))
            }
            pub fn name() -> &'static str {$name}
        }
        impl std::fmt::Display for $s {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Self::name().fmt(f)
            }
        }
        impl Default for $s {
            fn default() -> Self {
                Self::from_config(&Default::default())
            }
        }
    };
}

#[macro_export]
/// Expands to provide a default value for various data types. 
/// 
/// The macro accepts a type identifier—`Str`, `Int`, `Bool`, `Float`, `ListStr`, or `ListInt`—and emits a corresponding default value: an empty string, integer zero as a 64-bit signed integer, a boolean false, a float wrapped in the project's `F64` utility, a reference to an empty slice of string slices, or an empty array of integers, respectively. 
/// This aids in initializing variables with predetermined standard values across different types within the synthesis framework.
/// 
macro_rules! default_value {
    (Str) => { "" }; 
    (Int) => { 0i64 }; 
    (Bool) => { false }; 
    (Float) => { $crate::utils::F64(0.0) }; 
    (ListStr) => { &[] as &[&str] }; 
    (ListInt) => { [] }; 
}
#[macro_export]
/// Defines a macro that automates the implementation of several traits for a given type. 
/// 
/// This macro takes an identifier and a string literal as inputs, implementing functionalities related to the name of the type. 
/// The macro allows the type to return a static string as its name and implements the `Display` trait to facilitate formatted printing, using the name as the displayed representation. 
/// Additionally, it provides a default implementation for the type, initializing it using a configuration obtained from the `Default` trait. 
/// This macro is designed to reduce repetitive code for types that require a static name representation and a standardized construction approach.
/// 
macro_rules! impl_name {
    ($s:ident, $name:expr) => {
        impl $s {
            pub fn name() -> &'static str {$name}
        }
        impl std::fmt::Display for $s {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Self::name().fmt(f)
            }
        }
        impl Default for $s {
            fn default() -> Self {
                Self::from_config(&Default::default())
            }
        }
    };
}


#[macro_export]
/// Implements a macro to define unary operations for expression types. 
/// 
/// This macro provides a structured way to implement the `Op1` trait for a given type, enabling the definition of operation costs and evaluation methods. 
/// Specifically, it defines the `cost` method to return a constant value associated with the operation type and the `try_eval` method to attempt evaluation of the operation on an input value.
/// 
/// Within the implementation, the evaluation is performed by matching on the input value type. 
/// For each specified type conversion, it maps an expression over the input's iterable values, collecting results using `galloc_scollect`. 
/// If the input type matches one of the specified conversions, the operation applies and returns `true` along with the resulting value; otherwise, it returns `false` and a `Null` value. 
/// This macro thus facilitates efficient and reusable implementations of unary operations over various value types in the string synthesis framework.
macro_rules! impl_op1 {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        impl $crate::expr::ops::Op1 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value) -> (bool, $crate::value::Value) {
                match a1 {
                    $(
                        crate::value::Value::$t1(s) => (true, crate::value::Value::$rt(s.iter().map($f).galloc_scollect())),
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    }
}
#[macro_export]
/// Defines a macro to simplify the creation and implementation of unary operations in the synthesis framework. 
/// 
/// This macro automates the process of setting up a new unary operation by invoking several components. 
/// It takes an identifier, a name, and a series of type transformation definitions, associating each with an expression used for transformation. 
/// It uses the `impl_basic!` macro to provide basic trait implementations for the operation, sets up the operation as an implementation of `Enumerator1` from the `forward::enumeration` module, and then employs the `impl_op1!` macro to complete the detailed implementation needed for the operation including how it transforms various input types to the result type using the provided expression. 
/// This macro reduces repetitive code and ensures consistency when creating new operations.
/// 
macro_rules! new_op1 {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator1 for $s {}
        $crate::impl_op1!($s, $name, $($t1 -> $rt { $f }),*);
    };
}

#[macro_export]
/// Defines a macro to implement the `Op1` trait for a given type, focusing on optional transformations. 
/// 
/// 
/// This macro, `impl_op1_opt`, takes a type identifier and a series of transformation patterns, generating an implementation of the `Op1` trait for the specified type. 
/// The trait requires two key methods: `cost`, which returns the associated cost from the tuple stored within the type, and `try_eval`, which attempts to evaluate a unary operation on the provided `Value`. 
/// In `try_eval`, the macro iterates over values of specific types, applying a transformation function that returns an optional result. 
/// It collects results, tracking whether all transformations succeeded, and constructs a new `Value` of the result type, falling back to a default value if necessary. 
/// If the transformation can't be applied to the input type, it returns a flag of `false` with a `Null` value. 
/// This macro streamlines the implementation of the `Op1` trait for operations that may or may not succeed based on optional transformations. 
/// 
macro_rules! impl_op1_opt {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        impl $crate::expr::ops::Op1 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value) -> (bool, $crate::value::Value) {
                match a1 {
                    $(
                        crate::value::Value::$t1(s1) => {
                            let mut flag = true;
                            let v = s1.iter().map($f).map(|f| { flag &= f.is_some(); f.unwrap_or($crate::default_value![$rt]) }).galloc_scollect();
                            (flag, crate::value::Value::$rt(v))
                        }
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}
#[macro_export]
/// Defines a macro for creating a new unary operation. 
/// 
/// This macro simplifies the process of defining new unary operators by automating the implementation of necessary traits and logic. 
/// It first abstracts over the implementation of basic traits and characteristics by invoking `impl_basic!` with the specified identifier and name. 
/// Then, it ensures the new type implements `Enumerator1`, which likely facilitates iterating over or evaluating expressions. 
/// Finally, the macro invokes `impl_op1_opt!`, passing along the type, operation name, and mapping of input types to result types along with associated function logic. 
/// Through this structured approach, multiple unary operations with various input types and results can be efficiently defined and integrated into the synthesis framework.
/// 
macro_rules! new_op1_opt {
    ($s:ident, $name:expr, $($t1:ident -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator1 for $s {}
        $crate::impl_op1_opt!($s, $name, $($t1 -> $rt { $f }),*);
    };
}

#[macro_export]
/// Defines a macro for creating new binary operations within the expression framework. 
/// 
/// This macro, `new_op2`, takes as input a series of tokens to set up a binary operation type, specifically an identifier `$s`, a name `$name`, and multiple tuples of input types (`$t1` and `$t2`), return type `$rt`, and a functional expression `$f`. 
/// 
/// 
/// The macro performs several tasks: it invokes an implementation macro, `impl_basic!`, for basic setup using the provided identifier and name. 
/// It implements the `Enumerator2` trait for the operation using the `impl` syntax. 
/// Finally, it calls another implementation macro, `impl_op2!`, which probably performs the core operation setup, defining how each type of binary operation works using the provided type and functional expression pairs. 
/// This structure aids in generating consistent and concise definitions of new binary operations throughout the synthesis module.
macro_rules! new_op2 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator2 for $s {}
        $crate::impl_op2!($s, $name, $(($t1, $t2) -> $rt { $f }),*);
    };
}

#[macro_export]
/// This macro facilitates the creation of a new binary operation with optional behavior. 
/// 
/// It integrates a binary operation into the framework by implementing necessary traits and logic through various components. 
/// Initially, it utilizes `$crate::impl_basic!` to establish a basic setup for a given operation `$s`, associating it with a specified `$name`. 
/// The macro then implements the `Enumerator2` trait for `$s`, effectively enabling the operation to be part of the enumeration process utilized in forward synthesis tasks. 
/// 
/// 
/// Following this, it calls `$crate::impl_op2_opt!` to define additional implementation details for the operation, identifying the input types `$t1` and `$t2`, the return type `$rt`, and the expression `$f` that dictates the operation's computation. 
/// This approach ensures the cohesive integration of optional binary operations that can be used efficiently within the domain-specific synthesis framework, allowing for complex operations to be handled flexibly and dynamically.
macro_rules! new_op2_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator2 for $s {}
        $crate::impl_op2_opt!($s, $name, $(($t1, $t2) -> $rt { $f }),*);
    };
}

#[macro_export]
/// Creates a new ternary operation macro for use within the string synthesis framework. 
/// 
/// The macro accepts the name of the operation and a list of tuples, each representing a combination of three argument types and a return type, along with a closure that defines the operation's functionality. 
/// It expands to implement basic operation utilities and enumeration traits for the specified operation, invoking helper macros `impl_basic!`, which likely sets up foundational properties, and `impl_op3!`, which embeds the provided logic into the operational infrastructure. 
/// Overall, this macro streamlines the definition and integration of complex ternary operations within the module.
/// 
macro_rules! new_op3 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator3 for $s {}
        $crate::impl_op3!($s, $name, $(($t1, $t2, $t3) -> $rt { $f }),*);
    };
}

#[macro_export]
/// Defines a macro that simplifies the creation and implementation of three-input (ternary) operations with optional functionality.
/// 
/// This macro-rule is structured to declare and implement a new ternary operation by accepting an identifier, a name, and a sequence of type transformations that results in a function. 
/// It leverages other internal macros, such as `impl_basic!` and `impl_op3_opt!`, to automatically generate and bind necessary implementations. 
/// The `impl_basic!` macro assists in setting up basic structure and namespacing for the operation, while `impl_op3_opt!` is likely responsible for the implementation details and handling optional features across different inputs. 
/// The macro also implements the `Enumerator3` trait from the `forward::enumeration` module for the specified identifier, providing enumeration capabilities for the new operation in a modular and reusable manner.
macro_rules! new_op3_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {
        $crate::impl_basic!($s, $name);
        impl $crate::forward::enumeration::Enumerator3 for $s {}
        $crate::impl_op3_opt!($s, $name, $(($t1, $t2, $t3) -> $rt { $f }),*);
    };
}
#[macro_export]
/// This macro facilitates the implementation of the `Op2` trait for a given struct, allowing it to define binary operations on values. 
/// 
/// It enables configuration of the operation by specifying patterns for value types, corresponding result types, and a closure for computation. 
/// When invoked, the macro takes the struct name, operation name, and a series of type pattern mappings where each pair of input types maps to a result type and an associated function. 
/// For each specified type combination, the generated implementation of the `try_eval` method attempts to evaluate the operation by zipping and transforming the input iterables using the provided closure function. 
/// If the types match, it returns a successful evaluation with the transformed result; otherwise, it returns a failure with a null value. 
/// This macro centralizes the repetitive logic for implementing binary operations across different value types, promoting code reuse and reducing boilerplate.
/// 
macro_rules! impl_op2 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op2 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2) { 
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2)) => (true, crate::value::Value::$rt(itertools::izip!(s1.iter(), s2.iter()).map($f).galloc_scollect())),
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}

#[macro_export]
/// Defines a macro for implementing a two-operand operation trait with optional evaluation. 
/// 
/// 
/// This macro generates implementations for the `Op2` trait for a specified type by providing a framework where specific pairs of input types can define conversion logic and evaluation. 
/// For each tuple of input types and a resulting type provided within the macro invocation, the macro matches on the value types and applies a provided closure `f` to corresponding elements of the input values. 
/// It utilizes the `itertools::izip!` macro to pair elements from both input lists, applying the transformation function `f`, and collecting the results into a new list. 
/// If the operation fails or an element cannot be converted, it uses a default value for the resulting type. 
/// The generated implementation returns a flag indicating whether the operation was successful for all elements and the new value. 
/// If the inputs do not match any specified type combinations, the operation returns `false` with a `Null` value. 
/// This approach facilitates the flexible application of operations over compatible pairings of types while maintaining robust default behavior.
macro_rules! impl_op2_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op2 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2) {
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2)) => {
                            let mut flag = true;
                            let a = itertools::izip!(s1.iter(), s2.iter()).map($f).map(|f| { flag &= f.is_some(); f.unwrap_or($crate::default_value![$rt]) }).galloc_scollect();
                            (flag, crate::value::Value::$rt(a))
                        }
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}

#[macro_export]
/// Defines a macro for implementing a ternary operation trait. 
/// 
/// This macro, when invoked, generates an implementation of the `Op3` trait for a given structure. 
/// The trait includes two functions: `cost`, which returns a constant cost value associated with the operation, and `try_eval`, which attempts to evaluate the operation based on three input values. 
/// 
/// 
/// The `try_eval` function utilizes Rust's pattern matching to handle specific combinations of value types, applying a provided function using `itertools::izip` to iterate over the input values in tandem, mapping them to a result that is collected using `galloc_scollect`. 
/// If the provided input types do not match any of the specified patterns, the function returns a false success flag and a `Null` value, indicating an unsuccessful evaluation. 
/// This macro simplifies the creation of complex operations by automating the repetitive parts of defining the `Op3` implementations.
macro_rules! impl_op3 {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op3 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value, a3 : crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2, a3) {
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2), crate::value::Value::$t3(s3)) =>
                            (true, crate::value::Value::$rt(itertools::izip!(s1.iter(), s2.iter(), s3.iter()).map($f).galloc_scollect())),
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}

#[macro_export]
/// A macro for implementing ternary operations with optional characteristics. 
/// 
/// This macro, `impl_op3_opt!`, simplifies the process of defining implementations for the `Op3` trait on a specified type (designated by `$s`). 
/// For each combination of argument types and return type specified, it generates the operation's logic by defining the `cost` and `try_eval` methods for the `Op3` trait. 
/// 
/// 
/// In `try_eval`, the macro handles different matching patterns of input argument types. 
/// It uses pattern matching to destructure given values into tuples of variant types defined in `$t1`, `$t2`, and `$t3`. 
/// The evaluation iterates over combined elements of these tuples using the `itertools::izip!` macro, applying an expression `$f` on them. 
/// This produces an output optionally handled via a `flag`, which determines success. 
/// If any element evaluation yields `None`, the default value specified for the return type `$rt` is used instead. 
/// The resulting values are collected using `galloc_scollect()` into the expected result type wrapped in `crate::value::Value::$rt`. 
/// If none of the specified patterns match, the operation defaults to returning a `false` flag and a `Value::Null`.
macro_rules! impl_op3_opt {
    ($s:ident, $name:expr, $(($t1:ident, $t2:ident, $t3:ident) -> $rt:ident { $f:expr }),*) => {

        impl $crate::expr::ops::Op3 for $s {
            fn cost(&self) -> usize { self.0 }
            fn try_eval(&self, a1 : $crate::value::Value, a2 : $crate::value::Value, a3 : crate::value::Value) -> (bool, crate::value::Value) {
                match (a1, a2, a3) {
                    $(
                        (crate::value::Value::$t1(s1), crate::value::Value::$t2(s2), crate::value::Value::$t3(s3)) => {
                            let mut flag = true
                            let a = itertools::izip!(s1.iter(), s2.iter(), s3.iter()).map($f).map(|f| { flag &= f.is_some(); f.unwrap_or($crate::default_value![$rt]) }).galloc_scollect();
                            (flag, crate::value::Value::$rt(a))
                        }
                    )*
                    _ => (false, crate::value::Value::Null),
                }
            }
        }
    };
}
