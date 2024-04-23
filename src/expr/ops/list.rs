use std::cmp::min;
use std::ops::Not;

use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter};
use crate::utils::F64;
use crate::{new_op1, new_op2, new_op3, new_op2_opt};
use itertools::izip;



use super::{Op1, Op3, Op2};

pub fn to_index(len: usize, i: i64) -> usize {
    if i >= 0 {
        min(i as usize, len - 1)
    } else {
        len.saturating_sub(i.saturating_neg() as usize)
    }
}

new_op2_opt!(At, "list.at",
    (Str, Int) -> Str { |(s1, s2)| {
        if s1.len() > 0 {
            let i = to_index(s1.len(), *s2);
            Some(&*s1[i..=i].galloc_str())
        } else { None }
    }},
    (ListInt, Int) -> Int { |(s1, s2)| {
        if s1.len() > 0 {
            let i = to_index(s1.len(), *s2);
            Some(s1[i])
        } else { None }
    }},
    (ListStr, Int) -> Str { |(s1, s2)| {
        if s1.len() > 0 {
            let i = to_index(s1.len(), *s2);
            Some(s1[i])
        } else { None }
    }},
    (Str, Float) -> Str { |(s1, s2)| {
        if s1.len() > 0 {
            let i = to_index(s1.len(), **s2 as i64);
            Some(&*s1[i..=i].galloc_str())
        } else { None }
    }},
    (ListInt, Float) -> Int { |(s1, s2)| {
        if s1.len() > 0 {
            let i = to_index(s1.len(), **s2 as i64);
            Some(s1[i])
        } else { None }
    }},
    (ListStr, Float) -> Str { |(s1, s2)| {
        if s1.len() > 0 {
            let i = to_index(s1.len(), **s2 as i64);
            Some(s1[i])
        } else { None }
    }}
);

new_op2!(StrAt, "str.at",
    (Str, Int) -> Str { |(s1, s2)| {
        if s1.len() > 0 {
            if *s2 >= 0 && (*s2 as usize) < s1.len() {
                let i = *s2 as usize;
                &*s1[i..=i].galloc_str()
            } else { "" }
        } else { "" }
    }}
);

new_op1!(Len, "list.len", 
    Str -> Int { |s| s.len() as i64 },
    ListInt -> Int { |s| s.len() as i64 },
    ListStr -> Int { |s| s.len() as i64 }
);

new_op1!(FLen, "list.flen", 
    Str -> Float { |s| F64::from_usize(s.len()) },
    ListInt -> Float { |s| F64::from_usize(s.len()) },
    ListStr -> Float { |s| F64::from_usize(s.len()) }
);

new_op2!(Filter, "list.filter",
    (ListStr, Bool) -> Int { |s| panic!("Could not execuate Filter") }
);

new_op2!(Map, "list.map",
    (ListStr, Str) -> ListStr { |s| panic!("Could not execuate Filter") },
    (ListStr, Str) -> Str { |s| panic!("Could not execuate Filter") },
    (Str, Str) -> ListStr { |s| panic!("Could not execuate Filter") },
    (Str, Str) -> Str { |s| panic!("Could not execuate Filter") }
);
