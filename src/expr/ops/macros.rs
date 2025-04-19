
use super::*;

#[macro_export]
/// Defines a macro that generates a sequence of unary operations for expressions in the synthesis framework. 
/// 
/// This macro, when invoked, expands into a list of operation identifiers that correspond to various unary operations applicable within the framework. 
/// These include conversion operations (such as `ToInt`, `ToStr`, and `StrToFloat`), string manipulations (like `Uppercase` and `Lowercase`), and mathematical operations (for example, `Neg`, `IsPos`, and `FAbs`). 
/// Additional operations handle parsing and formatting tasks (`ParseDate`, `FormatMonth`, `ParseFloat`) and manage numeric transformations relevant to float and integer types. 
/// This macro serves as a central registry for unary operations, ensuring they are consistently defined and utilized throughout the synthesis module.
/// 
macro_rules! for_all_op1 {
    () => {
        _do!(Len ToInt ToStr Neg IsPos IsZero IsNatural RetainLl RetainLc RetainN RetainL RetainLN Uppercase Lowercase ParseDate AsMonth AsDay AsYear AsWeekDay ParseTime FormatFloat
            ParseInt 
            FormatInt
            ParseMonth
            ParseWeekday
            FormatTime
            FormatMonth
            FormatWeekday
            FormatFloat
            ParseFloat
            FNeg
            FAbs
            FIsPos
            FExp10
            IntToFloat
            FloatToInt
            StrToFloat
            FNotNeg
            FIsZero
            FLen
            Map);
    };
}
#[macro_export]
/// Expands to include a sequence of binary (two-operand) operations as part of the `Expr Ops` sub-module. 
/// 
/// These operations encompass a wide range of functionalities for string handling, mathematical computations, and date-time manipulations within the synthesis framework. 
/// The macro generates repetitive code structures for each operation defined within its body, listed in a `_do!` macro call. 
/// This list includes operations like `Concat` for concatenating strings, `Eq` for equality checks, `Add` and `Sub` for arithmetic, and `TimeAdd` for time adjustments, among others. 
/// Such a macro centralizes the operation definitions, simplifying maintenance and reducing redundancy in coding related to binary operations.
/// 
macro_rules! for_all_op2 {
    () => { 
        _do!(Concat Eq At PrefixOf SuffixOf Contains Split Join Count Add Sub Head Tail Filter TimeFloor TimeAdd Floor Round Ceil FAdd FSub FFloor FRound FCeil FCount FShl10
            TimeMul StrAt)
    };
}
#[macro_export]
/// Defines a macro that expands to include a set of three-element operations used within the expression handling of string synthesis tasks. 
/// 
/// This macro uses the `_do!` macro utility to list the operations `Replace`, `Ite`, `SubStr`, and `IndexOf`, which are likely shorthand references to more complex operations involving three arguments or parameters. 
/// The purpose is to simplify the repetitive task of writing out these operations each time they need to be applied, enabling more concise and modular code within areas of the project that handle ternary operations.
/// 
macro_rules! for_all_op3 {
    () => {
        _do!(Replace Ite SubStr IndexOf)
    };
}
