use std::str::{from_utf8, from_utf8_unchecked};
use bumpalo::collections::{String as BString, CollectIn};
use bumpalo::Bump;
use ext_trait::extension;
use bumpalo::collections::Vec as BVec;


thread_local! {
    static THR_ARENA: Bump = Bump::new() // Use Bumpalo for speed. Global is too slow.
}

#[extension(pub trait AllocForAny)]
impl<T> T {
    #[inline(always)]
    fn galloc(self) -> &'static T {
        alloc(self)
    }
    #[inline(always)]
    fn galloc_mut(self) -> &'static mut T {
        alloc_mut(self)
    }
}

#[extension(pub trait AllocForExactSizeIter)]
impl<T: ExactSizeIterator> T {
    #[inline(always)]
    fn galloc_scollect(self) -> &'static [T::Item] {
        alloc_iter(self)
    }
}

#[extension(pub trait TryAllocForExactSizeIter)]
impl<T: ExactSizeIterator<Item=Option<F>>, F> T {
    #[inline(always)]
    fn galloc_try_scollect(self) -> Option<&'static [F]> {
        try_alloc_iter(self)
    }
}

#[extension(pub trait AllocForIter)]
impl<T: Iterator> T {
    #[inline(always)]
    fn galloc_collect(self) -> &'static [T::Item] {
        alloc_iter2(self)
    }
}


#[extension(pub trait AllocForStr)]
impl str {
    #[inline(always)]
    fn galloc_owned_str(&self) -> BString<'static> {
        as_owned(self)
    }
    #[inline(always)]
    fn galloc_str(&self) -> &'static str {
        alloc_str(self)
    }
}

#[inline(always)]
fn alloc<T>(t: T) -> &'static T {
    THR_ARENA.with(|arena| {
        let p = arena.alloc(t) as *mut T;
        unsafe { p.as_ref::<'static>().unwrap() }
    })
}

#[inline(always)]
fn alloc_mut<T>(t: T) -> &'static mut T {
    THR_ARENA.with(|arena| {
        let p = arena.alloc(t) as *mut T;
        unsafe { p.as_mut::<'static>().unwrap() }
    })
}

#[inline(always)]
fn alloc_iter<T>(iter: impl ExactSizeIterator<Item= T>) -> &'static [T] {
    THR_ARENA.with(|arena| {
        let p = arena.alloc_slice_fill_iter(iter) as *mut [T];
        unsafe { p.as_ref::<'static>().unwrap() }
    })
}

#[inline(always)]
fn try_alloc_iter<T>(iter: impl ExactSizeIterator<Item= Option<T>>) -> Option<&'static [T]> {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        let vec: Option<BVec<_>> = unsafe { iter.collect_in(p.as_ref::<'static>().unwrap()) };
        vec.map(|x| x.into_bump_slice())
    })
}

#[inline(always)]
fn alloc_iter2<T>(iter: impl Iterator<Item= T>) -> &'static [T] {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        let vec: BVec<_> = unsafe { iter.collect_in(p.as_ref::<'static>().unwrap()) };
        vec.into_bump_slice()
    })
}
#[inline(always)]
pub fn new_bvec<T>(cap: usize) -> BVec<'static, T> {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        unsafe { BVec::with_capacity_in(cap, p.as_ref().unwrap()) }
    })
}

#[inline(always)]
fn alloc_str(s: &str) -> &'static str {
    THR_ARENA.with(|arena| {
        let p = arena.alloc_str(s) as *mut str;
        unsafe { p.as_ref::<'static>().unwrap() }
    })
}

#[inline(always)]
fn as_owned(s: &str) -> BString<'static> {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        unsafe { BString::from_str_in(s, p.as_ref::<'static>().unwrap()) }
    })
}

// fn alloc_str_u8(s: impl ExactSizeIterator<Item=u8>) -> &'static str {
//     THR_ARENA.with(|arena| {
//         let p = unsafe { from_utf8_unchecked(arena.alloc_slice_fill_iter(s)) } as *const str;
//         unsafe { p.as_ref::<'static>().unwrap() }
//     })
// }

// fn alloc_iter_mut<T>(iter: impl ExactSizeIterator<Item= T>) -> &'static mut [T] {
//     THR_ARENA.with(|arena| {
//         let p = arena.alloc_slice_fill_iter(iter) as *mut [T];
//         unsafe { p.as_mut::<'static>().unwrap() }
//     })
// }

fn collect_str_in_arena0(iter: impl Iterator<Item= char>) -> &'static str {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        let vec: BString<'static> = unsafe { iter.collect_in::<BString>(p.as_ref::<'static>().unwrap()) };
        vec.into_bump_str()
    })
}

#[extension(pub trait AllocForCharIter)]
impl<T: Iterator<Item=char>> T {
    fn galloc_collect_str(self) -> &'static str {
        collect_str_in_arena0(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alloc() {
        let i = alloc(1isize);
        assert!(*i == 1)
    }
}

