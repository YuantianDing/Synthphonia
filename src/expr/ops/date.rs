use chrono::NaiveDate;
use chrono::Datelike;

use std::cmp::min;
use std::ops::Not;

use bumpalo::collections::CollectIn;
use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter, AllocForIter, AllocForCharIter};
use crate::{new_op1, new_op2, impl_op3, impl_op3_opt, impl_op2_opt, new_op1_opt};
use itertools::izip;


use super::list::to_index;
use super::{Op1, Op3, Op2};




new_op1_opt!(FormatDate, "date.fmt",
    Int -> Str { |s1| {
        todo!()
    }}
);

new_op1_opt!(FormatMonth, "date.month.fmt",
    Int -> Str { |s1| {
        todo!()
    }}
);

new_op1_opt!(FormatWeekday, "date.weekday.fmt",
    Int -> Str { |s1| {
        todo!()
    }}
);

new_op1_opt!(AsMonth, "date.month",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        if p.is_none() { return None; }
        if let Some(date) = NaiveDate::from_num_days_from_ce_opt(p.unwrap()) {
            Some(date.month() as i64)
        } else { None }
    }}
);

new_op1_opt!(AsDay, "date.day",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        if p.is_none() { return None; }
        if let Some(date) = NaiveDate::from_num_days_from_ce_opt(p.unwrap()) {
            Some(date.day() as i64)
        } else { None }
    }}
);

new_op1_opt!(AsYear, "date.year",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        if p.is_none() { return None; }
        if let Some(date) = NaiveDate::from_num_days_from_ce_opt(p.unwrap()) {
            Some(date.year() as i64)
        } else { None }
    }}
);

new_op1_opt!(AsWeekDay, "date.weekday",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        if p.is_none() { return None; }
        if let Some(date) = NaiveDate::from_num_days_from_ce_opt(p.unwrap()) {
            Some(date.weekday().number_from_sunday() as i64)
        } else { None }
    }}
);


new_op1_opt!(FormatTime, "time.fmt",
    Str -> Int { |s1| {
        todo!()
    }}
);

new_op2!(TimeFloor, "time.floor",
    (Int, Int) -> Int { |(s1, s2)| {
        s1.div_floor(*s2) * *s2
    }}
);

new_op2!(TimeAdd, "time.+",
    (Int, Int) -> Int { |(s1, s2)| {
        (s1 + s2) % (60 * 60 * 60)
    }}
);