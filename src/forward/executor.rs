use std::{
    cell::{Cell, UnsafeCell, RefCell}, collections::{hash_map::Entry, HashMap, HashSet}, default, fs, future::Future, task::Poll
};

use derive_more::{Constructor, Deref, From, Into};
use itertools::Itertools;

use crate::{
    debg,
    expr::{
        cfg::{Cfg, ProdRule},
        context::Context,
        Expr,
    },
    info,
    utils::UnsafeCellExt,
    value::{ConstValue, Value}, log, parser::problem::PBEProblem, forward::{data::{size, substr}, enumeration::ProdRuleEnumerate, executor}, backward::{ Deducer, DeducerEnum}, galloc::AllocForAny, debg2,
};

use super::{
    future::taskrc::{TaskRc, TaskTRc, TaskORc},
    data::{self, all_eq, size::EV, Data},
    future::task,
};

pub trait EnumFn = FnMut(Expr, Value) -> Result<(), ()>;

pub struct OtherData {
    pub all_str_const: HashSet<&'static str>,
    pub problems: UnsafeCell<HashMap<(usize, Value), TaskORc<&'static Expr>>>,
}

pub struct Executor {
    pub counter: Cell<usize>,
    pub cur_size: Cell<usize>,
    pub cur_nt: Cell<usize>,
    pub ctx: Context,
    pub cfg: Cfg,
    pub deducers: Vec<DeducerEnum>,
    pub data: Vec<Data>,
    pub other: OtherData,
    expr_collector: UnsafeCell<Vec<EV>>,
}

impl Executor {
    pub fn new(ctx: Context, cfg: Cfg) -> Self {
        let all_str_const = cfg[0].rules.iter().flat_map(|x| if let ProdRule::Const(ConstValue::Str(s)) = x { Some(*s) } else { None }).collect();
        let data = Data::new(&cfg, &ctx);
        let deducers = (0..cfg.len()).map(|i, | DeducerEnum::from_nt(&cfg, &ctx, i)).collect_vec();
        let other = OtherData { all_str_const, problems: UnsafeCell::new(HashMap::new()) };
        Self { counter: 0.into(), ctx, cfg, data, other, deducers, expr_collector: Vec::new().into(), cur_size: 0.into(), cur_nt: 0.into() }
    }
    pub fn collect_expr(&self, e: &'static Expr, v: Value) {
        unsafe { self.expr_collector.as_mut().push((e, v)) }
    }
    pub fn extract_expr_collector(&self) -> Vec<EV> {
        self.expr_collector.replace(Vec::new())
    }
    pub fn cur_data(&self) -> &Data {
        &self.data[self.cur_nt.get()]
    }
    pub fn spawn_task(&'static self, nt: usize, value: Value) -> TaskORc<&'static Expr> {
        let problems = unsafe { self.other.problems.as_mut() };
        match problems.entry((nt, value)) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(e) => {
                let t = task::spawn(self.deducers[nt].deduce(self, value)).tasko();
                e.insert(t.clone());
                t
            }
        }
    }

    pub fn size(&self) -> usize { self.cur_size.get() }
    pub fn nt(&self) -> usize { self.cur_nt.get() }
    pub fn count(&self) -> usize { self.counter.get() }
    
    #[inline]
    pub fn enum_expr(&self, e: Expr, v: Value) -> Result<(), ()> {
        if self.counter.get() % 10000 == 0 {
            info!("Searching size={} [{}] - {:?} {:?}", self.cur_size.get(), self.counter.get(), e, v);
        }
        self.counter.update(|x| x + 1);
        
        if let Some(e) = self.cur_data().update(e, v)? {
            self.collect_expr(e,v)
        }
        Ok(())
    }
    fn run(&'static self) -> Result<(), ()> {
        let _ = self.extract_expr_collector();
        for size in 1 ..self.cfg.config.size_limit {
            for (nt, ntdata) in self.cfg.iter().enumerate() {
                self.cur_size.set(size);
                self.cur_nt.set(nt);
                info!("Enumerating size={} nt={} with - {}", size, ntdata.name, self.counter.get());
                
                for rule in &ntdata.rules {
                    rule.enumerate(self)?;
                }
                
                self.cur_data().size.add(size, self.extract_expr_collector());
            }
        }
        Ok(())
    }
    // pub fn get_problem(&'static self, p: Problem) -> TaskORc<&'static Expr> {
    //     let hash = unsafe { self.other.problems.as_mut() };
    //     match hash.entry(p.clone()) {
    //         std::collections::hash_map::Entry::Occupied(p) => p.get().clone(),
    //         std::collections::hash_map::Entry::Vacant(v) => {
    //             let t = task::spawn(p.deduce(self)).tasko();
    //             v.insert(t.clone());
    //             t
    //         }
    //     }
    // }
    pub fn block_on<T>(&'static self, t: TaskORc<T>) -> Option<T> {
        task::with_top_task(t.clone().task(), || {
            let _ = self.run();
        });
        match t.poll_unpin() {
            Poll::Ready(res) => Some(res),
            Poll::Pending => None,
        }
    }
    // pub fn run_with(&'static self, p: Problem) -> Option<&'static Expr> {
    //     let t = self.get_problem(p);
    //     task::with_top_task(t.task(), || {
    //         let _ = self.run();
    //     });
    //     match t.poll_unpin() {
    //         Poll::Ready(res) => Some(res),
    //         Poll::Pending => None,
    //     }
    // }
}

