use std::cmp::min;
use std::ops::Not;

use bumpalo::collections::CollectIn;
use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter, AllocForIter, AllocForCharIter};
use crate::utils::F64;
use crate::{new_op1, new_op2, new_op3, new_op3_opt, new_op2_opt};
use itertools::izip;



use super::list::to_index;
use super::{Op1, Op3, Op2};


new_op2!(Concat, "str.++",
    (Str, Str) -> Str { |(s1, s2)| {
        (s1.galloc_owned_str() + s2).into_bump_str()
    }}
);

mod replace;
pub use replace::*;


new_op3!(SubStr, "str.substr",
    (Str, Int, Int) -> Str { |(s1, s2, s3)| {
        if s1.is_empty() { return ""; }
        if *s2 >= 0 && (*s2 as usize) < s1.len() && *s3 >= 0 {
            let i = *s2 as usize;
            let j = std::cmp::min(i + *s3 as usize, s1.len());
            s1[i..j].galloc_str()
        } else { "" }
    }}
);

new_op2_opt!(Head, "str.head",
    (Str, Int) -> Str { |(s1, s2)| {
        if s1.len() <= 1 { return None; }
        let i = to_index(s1.len(), *s2);
        if i == 0 || i == s1.len() { return None; }
        Some(s1[0..i].galloc_str())
    }},
    (Str, Float) -> Str { |(s1, s2)| {
        if s1.len() <= 1 { return None; }
        let i = to_index(s1.len(), **s2 as i64);
        if i == 0 || i == s1.len() { return None; }
        Some(s1[0..i].galloc_str())
    }}
);

new_op2_opt!(Tail, "str.tail",
    (Str, Int) -> Str { |(s1, s2)| {
        if s1.len() <= 1 { return None; }
        let i = to_index(s1.len(), *s2);
        if i == 0 || i == s1.len() { return None; }
        Some(s1[i..].galloc_str())
    }},
    (Str, Float) -> Str { |(s1, s2)| {
        if s1.len() <= 1 { return None; }
        let i = to_index(s1.len(), **s2 as i64);
        if i == 0 || i == s1.len() { return None; }
        Some(s1[i..].galloc_str())
    }}
);

new_op1!(ToInt, "str.to.int",
    Str -> Int { |s1| {
        s1.parse::<i64>().unwrap_or(0)
    }}
);

pub fn str_index_of_f(s1: &str, s2: &str, s3: usize) -> i64 {
    let mut result: usize = 0;
    for _ in 0..=s3 {
        if result >= s1.len() { return -1; }
        if let Some(r) = s1[result..].find(s2) {
            result += r + 1;
        } else {return -1;}
    }
    result as i64 - 1
}

pub fn str_index_of_b(s1: &str, s2: &str, s3: usize) -> i64 {
    let mut result: usize = s1.len();
    for _ in 0..s3 {
        if result == 0 { return -1; }
        if let Some(r) = s1[0..result].rfind(s2) {
            result = r;
        } else {return -1;}
    }
    result as i64
}

new_op3!(IndexOf, "str.indexof",
    (Str, Str, Int) -> Int { |(s1, s2, s3)| {
        if *s3 < 0 || *s3 as usize > s1.len() { return -1i64; }
        if let Some(r) = s1[*s3 as usize..].find(s2) {
            *s3 + r as i64
        } else { -1i64 }
    }}
);

new_op2!(PrefixOf, "str.prefixof",
    (Str, Str) -> Bool { |(s1, s2)| {
        s2.starts_with(s1)
    }}
);
new_op2!(SuffixOf, "str.suffixof",
    (Str, Str) -> Bool { |(s1, s2)| {
        s2.ends_with(s1)
    }}
);
new_op2!(Contains, "str.contains",
    (Str, Str) -> Bool { |(s1, s2)| {
        s1.contains(s2)
    }}
);


new_op2!(Split, "str.split",
    (Str, Str) -> ListStr { |(s1, s2)| {
        s1.split(s2).galloc_collect()
    }}
);

new_op2!(Join, "str.join",
    (ListStr, Str) -> Str { |(s1, s2)| {
        s1.join(s2).galloc_str()
    }}
);

new_op2!(Count, "str.count",
    (Str, Str) -> Int { |(s1, s2)| {
        s1.matches(s2).count() as i64
    }}
);

new_op2!(FCount, "str.fcount",
    (Str, Str) -> Float { |(s1, s2)| {
        F64::from_usize(s1.matches(s2).count())
    }}
);

new_op1!(RetainLl, "str.retainLl",
    Str -> Str { |s1| {
        s1.chars().filter(|s| s.is_lowercase()).galloc_collect_str()
    }}
);

new_op1!(RetainLc, "str.retainLc",
    Str -> Str { |s1| {
        s1.chars().filter(|s| s.is_uppercase()).galloc_collect_str()
    }}
);

new_op1!(RetainN, "str.retainN",
    Str -> Str { |s1| {
        s1.chars().filter(|s| s.is_ascii_digit()).galloc_collect_str()
    }}
);

new_op1!(RetainL, "str.retainL",
    Str -> Str { |s1| {
        s1.chars().filter(|s| s.is_alphabetic()).galloc_collect_str()
    }}
);

new_op1!(RetainLN, "str.retainLN",
    Str -> Str { |s1| {
        s1.chars().filter(|s| s.is_alphanumeric()).galloc_collect_str()
    }}
);

new_op1!(Uppercase, "str.uppercase",
    Str -> Str { |s1| {
        s1.to_uppercase().galloc_str()
    }}
);

new_op1!(Lowercase, "str.lowercase",
    Str -> Str { |s1| {
        s1.to_lowercase().galloc_str()
    }}
);

#[cfg(test)]
mod tests {
    use crate::expr::ops::str::{str_index_of_f, str_index_of_b};

    #[test]
    fn test_str_index_of_inner() {
        assert!(str_index_of_f("s1asdf", "s1", 0) == 0);
        assert!(str_index_of_f("a s1s1s1", "s1", 0) == 2);
        assert!(str_index_of_f("a s1s1s1", "s1", 1) == 4);
        assert!(str_index_of_f("a s1s1s1", "s1", 2) == 6);
        assert!(str_index_of_f("a s1s1s1 s1", "s1", 3) == 9);
        assert!(str_index_of_f("a s1s1s1 s1", "s1", 4) == -1);

        assert!(str_index_of_f("s", "s", 0) == 0);
        assert!(str_index_of_f("s1asdf", "s", 0) == 0);
        assert!(str_index_of_f("a s1s1s1", "s", 0) == 2);
        assert!(str_index_of_f("a s1s1s1", "s", 1) == 4);
        assert!(str_index_of_f("a s1s1s1", "s", 2) == 6);
        assert!(str_index_of_f("a s1s1s1 s", "s", 3) == 9);
        assert!(str_index_of_f("a s1s1s1 s", "s", 4) == -1);

        assert!(str_index_of_b("s1asdf", "s1", 1) == 0);
        assert!(str_index_of_b("a s1s1s1", "s1", 3) == 2);
        assert!(str_index_of_b("a s1s1s1", "s1", 2) == 4);
        assert!(str_index_of_b("a s1s1s1", "s1", 1) == 6);
        assert!(str_index_of_b("a s1s1s1 s1", "s1", 1) == 9);
        assert!(str_index_of_b("a s1s1s1 s1", "s1", 5) == -1);

        assert!(str_index_of_b("s", "s", 1) == 0);
        assert!(str_index_of_b("s1asdf", "s", 2) == 0);
        assert!(str_index_of_b("a s1s1s1", "s", 3) == 2);
        assert!(str_index_of_b("a s1s1s1", "s", 2) == 4);
        assert!(str_index_of_b("a s1s1s1", "s", 1) == 6);
        assert!(str_index_of_b("a s1s1s1 s", "s", 1) == 9);
        assert!(str_index_of_b("a s1s1s1 s", "s", 5) == -1);
    }
}

