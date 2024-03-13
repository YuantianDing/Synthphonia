use std::cmp::min;
use std::ops::Not;

use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter, AllocForIter};
use crate::utils::F64;
use crate::{new_op1, new_op1_opt, new_op2, new_op2_opt, new_op3};
use itertools::izip;


use super::list::to_index;
use super::{ Op1, Op2, Op3};


new_op2!(FAdd, "float.+",
    (Float, Float) -> Float { |(&s1, &s2)| {
        F64::new(*s1 + *s2)
    }}
);
new_op2!(FSub, "float.-",
    (Float, Float) -> Float { |(&s1, &s2)| {
        F64::new(*s1 - *s2)
    }}
);


new_op1!(FNeg, "float.neg",
    Float -> Float { |&s1| {
        F64::new(-*s1)
    }}
);

new_op1!(FAbs, "float.abs",
    Float -> Float { |&s1| {
        F64::new(s1.abs())
    }}
);

new_op1!(FIsPos, "float.is+",
    Float -> Bool { |s1| {
        **s1 > 0.0
    }}
);
new_op1!(FNotNeg, "float.not-",
    Float -> Bool { |s1| {
        **s1 >= 0.0
    }}
);
new_op1!(FIsZero, "float.is0",
    Float -> Bool { |s1| {
        **s1 == 0.0
    }}
);

new_op1!(FExp10, "float.exp10",
    Int -> Float { |s1| {
        F64::new(10.0f64.powi(*s1 as i32))
    }}
);

new_op2!(FShl10, "float.shl10",
    (Float, Int) -> Float { |(s1, s2)| {
        F64::new(**s1 * 10.0f64.powi(*s2 as i32))
    }}
);

new_op2_opt!(FFloor, "float.floor",
    (Float, Float) -> Float { |(&s1, &s2)| {
        if *s2 == 0.0 { return None }
        Some(F64::new((*s1 / *s2).floor() * *s2))
    }}
);
new_op2_opt!(FRound, "float.round",
    (Float, Float) -> Float { |(&s1, &s2)| {
        if *s2 == 0.0 { return None }
        Some(F64::new((*s1 / *s2).round() * *s2))
    }}
);
new_op2_opt!(FCeil, "float.ceil",
    (Float, Float) -> Float { |(&s1, &s2)| {
        if *s2 == 0.0 { return None }
        Some(F64::new((*s1 / *s2).ceil() * *s2))
    }}
);

new_op1!(IntToFloat, "int.to.float",
    Int -> Float { |&s1| {
        F64::new(s1 as f64)
    }}
);

new_op1!(FloatToInt, "float.to.int",
    Float -> Int { |&s1| {
        *s1 as i64
    }}
);

new_op1_opt!(StrToFloat, "str.to.float",
    Str -> Float { |&s1| {
        s1.parse::<f64>().ok().map(|x| F64::new(x))
    }}
);


#[cfg(test)]
mod tests {
    use crate::{expr::{ context::Context}, value::ConstValue};
    use crate::expr;

    #[test]
    fn test1() {
        let ctx = &Context::new(1, Vec::new(), Vec::new(), ConstValue::Int(0).value(1));
        let result = expr!(FFloor (FNeg (FExp10 1)) (FNeg (IntToFloat 0))).eval(ctx);
        println!("{result:?}");
    }
}