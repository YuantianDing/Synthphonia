use std::cell::UnsafeCell;

use itertools::Itertools;

use crate::{
    expr::{cfg::Cfg, Expr},
    utils::UnsafeCellExt,
    value::Value,
};

pub type EV = (&'static Expr, Value);
pub type VecEv = Vec<EV>;
type SizeVec = Vec<VecEv>;

/// Term Dispatcher for a specific size of expression
pub struct Data(UnsafeCell<SizeVec>);

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum Size: {}, {:?}", self.len(), self.unsafe_inner().iter().map(|x| x.len()).collect_vec())
    }
}

impl Data {
    fn unsafe_inner(&self) -> &mut SizeVec { unsafe { self.0.as_mut() } }
    pub fn new(cfg: &Cfg) -> Self { Self(vec![vec![]].into()) }
    pub fn len(&self) -> usize { self.unsafe_inner().len() }
    pub fn get_all(&self, size: usize) -> &[EV] { self.unsafe_inner()[size].as_slice() }
    #[inline(always)]
    pub fn get_all_under(&self, size: usize) -> impl Iterator<Item = (usize, &EV)> + '_ {
        (1..size).flat_map(move |i| self.unsafe_inner()[i].iter().map(move |x| (i, x)))
    }
    #[inline(always)]
    pub fn add(&self, size: usize, vec: VecEv) {
        assert!(self.len() == size, "{size}, {}", self.len());
        self.unsafe_inner().push(vec);
    }
}
