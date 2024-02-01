
use std::cmp::min;
use std::ops::Not;

use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, AllocForIter};
use crate::{new_op1, new_op3, new_op2};
use itertools::izip;



use super::list::to_index;
use super::{Op1, Op3, Op2};


new_op1!(ToStr, "int.to.str",
    Int -> Str { |s1| {
        s1.to_string().galloc_str()
    }}
);

new_op1!(ParseInt, "int.fmt",
    Int -> Str { |s1| {
        todo!()
    }}
);

new_op1!(FormatInt, "int.parse",
    Str -> Int { |s1| {
        todo!()
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

new_op2!(Floor, "int.floor",
    (Int, Int) -> Int { |(s1, s2)| {
        s1.div_floor(*s2) * *s2
    }}
);
new_op2!(Round, "int.round",
    (Int, Int) -> Int { |(s1, s2)| {
        if (*s1 % *s2) * 2 >= *s2 {
            s1.div_ceil(*s2) * *s2
        } else {
            s1.div_floor(*s2) * *s2
        }
    }}
);
new_op2!(Ceil, "int.ceil",
    (Int, Int) -> Int { |(s1, s2)| {
        s1.div_ceil(*s2) * *s2
    }}
);