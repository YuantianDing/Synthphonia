
use std::cmp::min;
use std::ops::Not;

use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter, AllocForIter};
use crate::{new_op1, new_op2, new_op2_opt, new_op3};
use itertools::izip;



use super::list::to_index;
use super::{Op1, Op3, Op2};


new_op1!(ToStr, "int.to.str",
    Int -> Str { |s1| {
        s1.to_string().galloc_str()
    }}
);

new_op2!(Add, "int.+",
    (Int, Int) -> Int { |(s1, s2)|
        unsafe { s1.unchecked_add(*s2) }
    }
);
new_op2!(Sub, "int.-",
    (Int, Int) -> Int { |(s1, s2)|
        unsafe { s1.unchecked_sub(*s2) }
    }
);


new_op1!(Neg, "int.neg",
    Int -> Int { |s1| {
        unsafe { 0i64.unchecked_sub(*s1) }
    }}
);

new_op1!(IsPos, "int.is+",
    Int -> Bool { |s1| { s1 > &0 }}
);

new_op1!(IsZero, "int.is0",
    Int -> Bool { |s1| { s1 == &0 }}
);

new_op1!(IsNatural, "int.isN",
    Int -> Bool { |s1| { s1 >= &0 }}
);

new_op2_opt!(Floor, "int.floor",
    (Int, Int) -> Int { |(s1, s2)| {
        if *s2 == 0 { return None; }
        Some(s1.div_floor(*s2) * *s2)
    }}
);
new_op2_opt!(Round, "int.round",
    (Int, Int) -> Int { |(s1, s2)| {
        if *s2 == 0 { return None; }
        if (*s1 % *s2) * 2 >= *s2 {
            Some(s1.div_ceil(*s2) * *s2)
        } else {
            Some(s1.div_floor(*s2) * *s2)
        }
    }}
);
new_op2_opt!(Ceil, "int.ceil",
    (Int, Int) -> Int { |(s1, s2)| {
        if *s2 == 0 { return None; }
        Some(s1.div_ceil(*s2) * *s2)
    }}
);


#[cfg(test)]
mod tests {
    use crate::{expr::{ context::Context}, value::ConstValue};
    use crate::expr;

    #[test]
    fn test1() {
        let ctx = &Context::new(1, Vec::new(), Vec::new(), ConstValue::Int(0).value(1));
        let result = expr!(Add (Ceil 90 10) (Neg 1)).eval(ctx);
        println!("{result:?}");
    }
}