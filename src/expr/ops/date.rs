use chrono::NaiveDate;
use chrono::Datelike;

use std::cmp::min;
use std::ops::Not;

use bumpalo::collections::CollectIn;
use derive_more::DebugCustom;
use crate::galloc::{AllocForStr, AllocForExactSizeIter, TryAllocForExactSizeIter, AllocForIter, AllocForCharIter};
use crate::new_op2_opt;
use crate::{new_op1, new_op2, impl_op3, impl_op3_opt, impl_op2_opt, new_op1_opt};
use itertools::izip;


use super::list::to_index;
use super::{Op1, Op3, Op2};





new_op1_opt!(AsMonth, "date.month",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        p?;
        NaiveDate::from_num_days_from_ce_opt(p.unwrap()).map(|date| date.month() as i64)
    }}
);

new_op1_opt!(AsDay, "date.day",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        p?;
        NaiveDate::from_num_days_from_ce_opt(p.unwrap()).map(|date| date.day() as i64)
    }}
);

new_op1_opt!(AsYear, "date.year",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        p?;
        NaiveDate::from_num_days_from_ce_opt(p.unwrap()).map(|date| date.year() as i64)
    }}
);

new_op1_opt!(AsWeekDay, "date.weekday",
    Int -> Int { |s1| {
        let p = i32::try_from(*s1).ok();
        p?;
        NaiveDate::from_num_days_from_ce_opt(p.unwrap()).map(|date| date.weekday().number_from_sunday() as i64)
    }}
);

new_op2_opt!(TimeFloor, "time.floor",
    (Int, Int) -> Int { |(s1, s2)| {
        if *s2 != 0 {
            Some(s1.div_floor(*s2) * *s2)
        } else { None }
    }}
);

new_op2!(TimeAdd, "time.+",
    (Int, Int) -> Int { |(s1, s2)| {
        (s1 + s2) % (60 * 60 * 60)
    }}
);

new_op2!(TimeMul, "time.*",
    (Int, Int) -> Int { |(s1, s2)| {
        (s1 * s2) % (60 * 60 * 60)
    }}
);