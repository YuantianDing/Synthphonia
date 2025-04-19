use std::cmp::min;
use std::ops::Not;

use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter};
use crate::utils::F64;
use crate::{new_op1, new_op2, new_op3, new_op2_opt};
use itertools::izip;



use super::{Op1, Op3, Op2};

/// Converts an integer to a valid index within a bounded length. 
/// 
/// For non-negative input `i`, it returns the minimum of `i` or `len - 1`, ensuring the result does not exceed the upper bound of the available length. 
/// For negative `i`, it computes an index by subtracting the negated value of `i` from the total length using saturating operations to avoid overflow, effectively wrapping around from the end of the array-like sequence. 
/// This function handles out-of-bounds indices gracefully, ensuring the resulting index always falls within the valid range of 0 to `len - 1`.
/// 
pub fn to_index(len: usize, i: i64) -> usize {
    if i >= 0 {
        min(i as usize, len - 1)
    } else {
        len.saturating_sub(i.saturating_neg() as usize)
    }
}

new_op2_opt!(At, "list.at",
    (Str, Int) -> Str { |(s1, s2)| {
        if !s1.is_empty() {
            let i = to_index(s1.len(), *s2);
            Some(s1[i..=i].galloc_str())
        } else { None }
    }},
    (ListInt, Int) -> Int { |(s1, s2)| {
        if !s1.is_empty() {
            let i = to_index(s1.len(), *s2);
            Some(s1[i])
        } else { None }
    }},
    (ListStr, Int) -> Str { |(s1, s2)| {
        if !s1.is_empty() {
            let i = to_index(s1.len(), *s2);
            Some(s1[i])
        } else { None }
    }},
    (Str, Float) -> Str { |(s1, s2)| {
        if !s1.is_empty() {
            let i = to_index(s1.len(), **s2 as i64);
            Some(s1[i..=i].galloc_str())
        } else { None }
    }},
    (ListInt, Float) -> Int { |(s1, s2)| {
        if !s1.is_empty() {
            let i = to_index(s1.len(), **s2 as i64);
            Some(s1[i])
        } else { None }
    }},
    (ListStr, Float) -> Str { |(s1, s2)| {
        if !s1.is_empty() {
            let i = to_index(s1.len(), **s2 as i64);
            Some(s1[i])
        } else { None }
    }}
);

new_op2!(StrAt, "str.at",
    (Str, Int) -> Str { |(s1, s2)| {
        if !s1.is_empty() {
            if *s2 >= 0 && (*s2 as usize) < s1.len() {
                let i = *s2 as usize;
                s1[i..=i].galloc_str()
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

pub mod map;
pub use map::Map;