
use super::*;

#[macro_export]
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
macro_rules! for_all_op2 {
    () => { 
        _do!(Concat Eq At PrefixOf SuffixOf Contains Split Join Count Add Sub Head Tail Filter TimeFloor TimeAdd Floor Round Ceil FAdd FSub FFloor FRound FCeil FCount FShl10
            TimeMul StrAt)
    };
}
#[macro_export]
macro_rules! for_all_op3 {
    () => {
        _do!(Replace Ite SubStr IndexOf)
    };
}
