use crate::{
    expr::ops, forward::enumeration, galloc::AllocForExactSizeIter, impl_basic, new_op1,
    value::Value,
};

fn mask(i: usize) -> u64 { 
    (1u64 << i) - 1
}
fn to_signed(i: usize, a: u64) -> i64 {
    if a & (1u64 << (i - 1)) != 0 {
        (a | !mask(i)) as i64
    } else {
        a as i64
    }
}

impl_basic!(BvNot, "bvnot");
impl enumeration::Enumerator1 for BvNot {}
impl ops::Op1 for BvNot {
    fn cost(&self) -> usize { self.0 }

    fn try_eval(&self, a1: Value) -> (bool, Value) {
        if let Value::BitVector(i, a1) = a1 {
            (true, Value::BitVector(i, a1.iter().map(|x| !x & mask(i)).galloc_scollect()))
        } else {
            (false, Value::Null)
        }
    }
}

impl_basic!(BvNeg, "bvneg");
impl enumeration::Enumerator1 for BvNeg {}
impl ops::Op1 for BvNeg {
    fn cost(&self) -> usize { self.0 }

    fn try_eval(&self, a1: Value) -> (bool, Value) {
        if let Value::BitVector(i, a1) = a1 {
            (true, Value::BitVector(i, a1.iter().map(|x| (0u64 - x) & mask(i)).galloc_scollect()))
        } else {
            (false, Value::Null)
        }
    }
}

macro_rules! impl_bvop2 {
    ($op:ident, $name:literal, $f:expr) => {
        impl_basic!($op, $name);
        impl enumeration::Enumerator2 for $op {}
        impl ops::Op2 for $op {
            fn cost(&self) -> usize { self.0 }

            fn try_eval(&self, a1: Value, a2: Value) -> (bool, Value) {
                if let (Value::BitVector(i1, a1), Value::BitVector(i2, a2)) = (a1, a2) {
                    $f(i1, a1, i2, a2)
                } else {
                    (false, Value::Null)
                }
            }
        }
    };
}

impl_bvop2!(BvAdd, "bvadd", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| (x + y) & mask(i)).galloc_scollect();
    (true, Value::BitVector(i, result))
});
impl_bvop2!(BvSub, "bvsub", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| (x - y) & mask(i)).galloc_scollect();
    (true, Value::BitVector(i, result))
});
impl_bvop2!(BvMul, "bvmul", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| (x * y) & mask(i)).galloc_scollect();
    (true, Value::BitVector(i, result))
});

impl_bvop2!(BvAnd, "bvand", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| x & y).galloc_scollect();
    (true, Value::BitVector(i, result))
});
impl_bvop2!(BvOr, "bvor", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| x | y).galloc_scollect();
    (true, Value::BitVector(i, result))
});
impl_bvop2!(BvXor, "bvxor", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| x ^ y).galloc_scollect();
    (true, Value::BitVector(i, result))
});

impl_bvop2!(BvShl, "bvshl", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| if *y >= 64 {0} else { (x << y) & mask(i) }).galloc_scollect();
    (true, Value::BitVector(i, result))
});
impl_bvop2!(BvLShr, "bvlshr", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| if *y >= 64 {0} else { x >> y }).galloc_scollect();
    (true, Value::BitVector(i, result))
});
impl_bvop2!(BvAShr, "bvashr", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| if *y >= 64 {0} else { (to_signed(i, *x) >> y) as u64 & mask(i) }).galloc_scollect();
    (true, Value::BitVector(i, result))
});

impl_bvop2!(BvUDiv, "bvudiv", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    if a2.iter().any(|&x| x == 0) {
        return (false, Value::Null);
    }
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| x / y).galloc_scollect();
    (true, Value::BitVector(i, result))
});

impl_bvop2!(BvSDiv, "bvsdiv", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    if a2.iter().any(|&x| x == 0) {
        return (false, Value::Null);
    }
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| to_signed(i, *x).overflowing_div(to_signed(i, *y)).0 as u64 & mask(i)).galloc_scollect();
    (true, Value::BitVector(i, result))
});

impl_bvop2!(BvURem, "bvurem", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    if a2.iter().any(|&x| x == 0) {
        return (false, Value::Null);
    }
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| x % y).galloc_scollect();
    (true, Value::BitVector(i, result))
});

impl_bvop2!(BvSRem, "bvsrem", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    if a2.iter().any(|&x| x == 0) {
        return (false, Value::Null);
    }
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| to_signed(i, *x).overflowing_rem(to_signed(i, *y)).0 as u64 & mask(i)).galloc_scollect();
    (true, Value::BitVector(i, result))
});


impl_bvop2!(BvSlt, "bvslt", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| to_signed(i, *x) < to_signed(i, *y)).galloc_scollect();
    (true, Value::Bool(result))
});
impl_bvop2!(BvUlt, "bvult", |i1, a1: &'static [u64], i2, a2: &'static [u64]| {
    let i = std::cmp::max(i1, i2);
    let result = a1.iter().zip(a2.iter()).map(|(x, y)| x < y).galloc_scollect();
    (true, Value::Bool(result))
});

