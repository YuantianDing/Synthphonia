
use super::*;

#[macro_export]
macro_rules! for_all_op1 {
    () => {
        _do!(Len ToInt ToStr Neg IsPos IsZero IsNatural RetainLl RetainLc RetainN RetainL RetainLN Uppercase Lowercase ParseDate AsMonth AsDay AsYear AsWeekDay FormatDate ParseTime FormatTime FormatMonth FormatWeekday);
    };
}
#[macro_export]
macro_rules! for_all_op2 {
    () => { 
        _do!(Concat Eq At PrefixOf SuffixOf Contains Split Join Count Add Sub Head Tail Filter Map TimeFloor TimeAdd Floor Round Ceil)
    };
}
#[macro_export]
macro_rules! for_all_op3 {
    () => {
        _do!(Replace Ite SubStr IndexOf)
    };
}
