use std::cmp::min;
use std::ops::Not;

use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, AllocForIter};
use crate::{new_op1, new_op3, new_op2};
use itertools::izip;


use super::list::to_index;
use super::{Op1, Op3, Op2};


new_op1!(ParseFloat, "float.fmt",
    Int -> Str { |s1| {
        todo!()
    }}
);

new_op1!(FormatFloat, "float.parse",
    Str -> Int { |s1| {
        todo!()
    }}
);

new_op2!(FAdd, "float.+",
    (Int, Int) -> Int { |(s1, s2)| {
        let f1 = f64::from_bits(*s1 as u64);
        let f2 = f64::from_bits(*s2 as u64);
        (f1 + f2).to_bits() as i64
    }}
);
new_op2!(FSub, "float.-",
    (Int, Int) -> Int { |(s1, s2)| {
        let f1 = f64::from_bits(*s1 as u64);
        let f2 = f64::from_bits(*s2 as u64);
        (f1 - f2).to_bits() as i64
    }}
);


new_op1!(FNeg, "float.neg",
    Int -> Int { |s1| {
        let f1 = f64::from_bits(*s1 as u64);
        (-f1).to_bits() as i64
    }}
);

new_op1!(FAbs, "float.abs",
    Int -> Int { |s1| {
        let f1 = f64::from_bits(*s1 as u64);
        f1.abs().to_bits() as i64
    }}
);

new_op1!(FIsPos, "float.is+",
    Int -> Bool { |s1| {
        let f1 = f64::from_bits(*s1 as u64);
        f1 > 0.0
    }}
);

new_op2!(FFloor, "float.floor",
    (Int, Int) -> Int { |(s1, s2)| {
        s1.div_floor(*s2) * *s2
    }}
);
new_op2!(FRound, "float.round",
    (Int, Int) -> Int { |(s1, s2)| {
        if (*s1 % *s2) * 2 >= *s2 {
            s1.div_ceil(*s2) * *s2
        } else {
            s1.div_floor(*s2) * *s2
        }
    }}
);
new_op2!(FCeil, "float.ceil",
    (Int, Int) -> Int { |(s1, s2)| {
        s1.div_ceil(*s2) * *s2
    }}
);