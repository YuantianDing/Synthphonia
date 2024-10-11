use std::cell::UnsafeCell;

use itertools::Itertools;

use crate::{
    expr::{cfg::Cfg, Expr},
    utils::UnsafeCellExt,
    value::Value,
};

pub type EV = (&'static Expr, Value);
pub type VecEv = Vec<EV>;
pub struct Data(boxcar::Vec<VecEv>);

impl Data {
    pub fn new(cfg: &Cfg) -> Self { Self(boxcar::vec![vec![]]) }
    pub fn len(&self) -> usize { self.0.count() }
    pub fn get_all(&self, size: usize) -> &[EV] { self.0[size].as_slice() }
    #[inline(always)]
    pub fn get_all_under(&self, size: usize) -> impl Iterator<Item = (usize, &EV)> + '_ {
        (1..size).flat_map(move |i| self.0[i].iter().map(move |x| (i, x)))
    }
    #[inline(always)]
    pub fn add(&self, size: usize, vec: VecEv) {
        assert!(self.len() == size, "{size}, {}", self.len());
        self.0.push(vec);
    }
}
