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
    /// Provides a method to allocate an instance of `T` on the heap with a static lifetime. 
    /// 
    /// This implementation of `galloc` takes ownership of the `T` instance and uses the `alloc` function to place it in a location with a static lifetime, presumably managing it in a way that ensures its persistence for the duration of the program. 
    /// This can be particularly useful for scenarios where a static lifetime is required, such as when interfacing with systems or patterns that necessitate global state or long-lived data.
    /// 
    fn galloc(self) -> &'static T {
        alloc(self)
    }
    #[inline(always)]
    /// Provides a method that moves the instance and returns a reference to it allocated with a static lifetime. 
    /// 
    /// This method utilizes `alloc_mut` to perform the allocation, likely involving allocating the resource in a manner that ensures it lives for the entire duration of the application. 
    /// These semantics allow the user to safely assume that the reference will not expire during the program's execution, making it suitable for long-lived data structures or operations that require such guarantees.
    /// 
    fn galloc_mut(self) -> &'static T {
        alloc_mut(self)
    }
}

#[extension(pub trait AllocForExactSizeIter)]
impl<T: ExactSizeIterator> T {
    #[inline(always)]
    /// Transforms the implementor type by invoking `alloc_iter` and returns a static slice of items. 
    /// 
    /// This method consumes the instance of the type, calling `alloc_iter` which is expected to allocate or collect items into a static slice. 
    /// This process typically involves aggregating items from the iterator and providing a reference that persists for the 'static lifetime, enabling access to the collected items beyond the scope of the function execution.
    /// 
    fn galloc_scollect(self) -> &'static [T::Item] {
        alloc_iter(self)
    }
}

#[extension(pub trait TryAllocForExactSizeIter)]
impl<T: ExactSizeIterator<Item=Option<F>>, F> T {
    #[inline(always)]
    /// Provides a method to attempt to allocate an iterator's result and collect it into a static slice. 
    /// 
    /// It utilizes `try_alloc_iter` to perform the allocation, returning an `Option` containing a reference to a slice of type `F` if successful or `None` if the allocation fails. 
    /// This operation is likely part of a collection mechanism within the `T` type, which supports the controlled transformation of iterator results into persistent slices, possibly for optimized data handling in memory-sensitive contexts.
    /// 
    fn galloc_try_scollect(self) -> Option<&'static [F]> {
        try_alloc_iter(self)
    }
}

#[extension(pub trait AllocForIter)]
impl<T: Iterator> T {
    #[inline(always)]
    /// Collects the items of the iterator and stores them in a static slice. 
    /// 
    /// 
    /// This method consumes the iterator from which it is called and collects all of its items using an allocation function. 
    /// The items are stored in a statically allocated slice, which means that the lifetime of the collected items is tied to the entire duration of the program. 
    /// This could be particularly useful in contexts where the collected items need to be immutable and available for the entire runtime, though caution should be taken to ensure memory usage is within acceptable bounds considering the static lifetime.
    fn galloc_collect(self) -> &'static [T::Item] {
        alloc_iter2(self)
    }
}


#[extension(pub trait AllocForStr)]
impl str {
    #[inline(always)]
    /// Provides a method to convert a `str` slice into a `BString` with a static lifetime. 
    /// 
    /// It does so by calling the `as_owned` function on the string slice, which is likely intended to produce an owned version of the string that can be utilized where a borrowed string is not sufficient. 
    /// This transformation is useful in scenarios where a persistent, non-borrowed string is required, for example, for storage in data structures that manage their own memory lifecycle. 
    /// 
    /// 
    fn galloc_owned_str(&self) -> BString<'static> {
        as_owned(self)
    }
    #[inline(always)]
    /// Returns a reference to a static string by allocating storage for the given string slice. 
    /// 
    /// This method takes a reference to a string slice (`&self`) and allocates it using the `alloc_str` function. 
    /// The returned value is a reference to a static string, which implies that the allocated storage has a `'static` lifetime and remains valid for the entire duration of the program. 
    /// This function is useful when a longer-lived string reference is needed, potentially at the cost of additional memory allocation.
    /// 
    fn galloc_str(&self) -> &'static str {
        alloc_str(self)
    }
}

#[inline(always)]
/// Allocates a given value in a thread-local arena. 
/// 
/// 
/// This function takes a generic value and allocates it within a thread-local storage arena, then returns a static reference to the allocated object. 
/// It leverages the `THR_ARENA` thread-local variable, using its `alloc` method to store the value. 
/// The pointer to the allocated memory is then cast to a mutable pointer, and an unsafe operation is performed to convert it into a static reference. 
/// As Rust's safety guarantees are bypassed here, this code snippet assumes that the reference's lifetime requirements will be properly managed to prevent undefined behavior.
fn alloc<T>(t: T) -> &'static T {
    THR_ARENA.with(|arena| {
        let p = arena.alloc(t) as *mut T;
        unsafe { p.as_ref::<'static>().unwrap() }
    })
}

#[inline(always)]
/// Allocates a mutable reference to a value on a thread-local storage arena. 
/// 
/// 
/// This function takes an input of any type `T`, allocates it on a thread-local arena, and returns a mutable reference with a static lifetime. 
/// It uses a thread-local storage mechanism provided by `THR_ARENA` to ensure that the allocated object resides in memory local to the thread, improving performance for concurrent processes. 
/// The allocation is done by casting the allocated value to a mutable raw pointer, and then converting it to a mutable reference with a static lifetime using unsafe operations. 
/// The use of `unsafe` indicates that it is the programmer's responsibility to uphold the memory safety guarantees manually. 
/// This approach is typically employed when fine-grained control over memory allocation and lifetimes is necessary for performance-critical code.
fn alloc_mut<T>(t: T) -> &'static mut T {
    THR_ARENA.with(|arena| {
        let p = arena.alloc(t) as *mut T;
        unsafe { p.as_mut::<'static>().unwrap() }
    })
}

#[inline(always)]
/// Allocates a slice from an iterator and returns a reference with a static lifetime. 
/// 
/// 
/// This function takes an iterator implementing `ExactSizeIterator`, allocates a slice using elements from this iterator, and returns a reference to this slice with a `'static` lifetime. 
/// It utilizes a thread-local storage arena (`THR_ARENA`) to handle memory allocation. 
/// Inside the thread-local arena, it calls `alloc_slice_fill_iter` to allocate and fill the slice based on the provided iterator. 
/// The memory address of this allocated slice is then manipulated as a mutable pointer, which is unsafely coerced into a reference with a static lifetime and returned. 
/// This requires caution, as improper lifetime management may lead to undefined behavior.
fn alloc_iter<T>(iter: impl ExactSizeIterator<Item= T>) -> &'static [T] {
    THR_ARENA.with(|arena| {
        let p = arena.alloc_slice_fill_iter(iter) as *mut [T];
        unsafe { p.as_ref::<'static>().unwrap() }
    })
}

#[inline(always)]
/// Attempts to allocate items from an iterator within a thread-local memory arena. 
/// 
/// The function takes an iterator of optional items and tries to collect these items into a `Bump`-allocated vector within a thread-local storage `arena`. 
/// This is achieved by using the `collect_in` method, which collects items into a `Bump` allocator. 
/// If the collection is successful, the function converts the vector into a bump slice, returning a static reference to the slice. 
/// If any item in the iterator is `None`, or if the allocation fails, the function returns `None`, indicating that the items could not be allocated in the bump arena.
/// 
fn try_alloc_iter<T>(iter: impl ExactSizeIterator<Item= Option<T>>) -> Option<&'static [T]> {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        let vec: Option<BVec<_>> = unsafe { iter.collect_in(p.as_ref::<'static>().unwrap()) };
        vec.map(|x| x.into_bump_slice())
    })
}

#[inline(always)]
/// Allocates an iterator's items into a bump-allocated slice. 
/// 
/// 
/// This function leverages a thread-local bump arena to efficiently allocate memory for storing the items produced by the provided iterator. 
/// It uses `collect_in` to gather the iterator's elements within the context of the arena's bump allocation, ensuring the allocation remains valid for the program's lifetime. 
/// A reference to the bump allocator is obtained through unsafe pointer dereferencing, and the iterator's elements are collected into a bump vector (`BVec`). 
/// This vector is then converted into a static reference to a slice stored in the arena memory, offering a performance advantage by reducing heap allocation overhead.
fn alloc_iter2<T>(iter: impl Iterator<Item= T>) -> &'static [T] {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        let vec: BVec<_> = unsafe { iter.collect_in(p.as_ref::<'static>().unwrap()) };
        vec.into_bump_slice()
    })
}
#[inline(always)]
/// Creates a new `BVec` with a specified capacity. 
/// 
/// This function utilizes thread-local storage to access a bump allocator, referred to as `THR_ARENA`, and constructs a `BVec` with a given capacity (`cap`). 
/// It obtains a reference to this arena, which is a bump allocator instance, and uses it to allocate and manage the memory for the `BVec`. 
/// This approach leverages unsafe code to perform pointer dereferencing and ensures that the memory management is efficient for temporary allocations within concurrent environments.
/// 
pub fn new_bvec<T>(cap: usize) -> BVec<'static, T> {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        unsafe { BVec::with_capacity_in(cap, p.as_ref().unwrap()) }
    })
}

#[inline(always)]
/// Allocates a string in a thread-local arena and returns a reference with a `'static` lifetime. 
/// 
/// This function takes a string slice as input and uses a thread-local storage arena to allocate memory for it, ensuring that the memory is associated with the current thread's local context. 
/// The string is then converted into a raw mutable pointer, which is dereferenced into a reference with a `'static` lifetime. 
/// Unsafe operations are required to perform this operation due to manual memory management and lifetime extension, ensuring that the memory is valid throughout the entire program execution within that thread, without being subject to Rust's usual borrowing constraints.
/// 
fn alloc_str(s: &str) -> &'static str {
    THR_ARENA.with(|arena| {
        let p = arena.alloc_str(s) as *mut str;
        unsafe { p.as_ref::<'static>().unwrap() }
    })
}

#[inline(always)]
/// Creates an owned `BString` from a borrowed string slice using a thread-local memory arena. 
/// 
/// This function leverages the thread-local `THR_ARENA`, which provides access to a memory allocator (`Bump`) for efficient memory management. 
/// It temporarily takes a reference to this allocator, and within an unsafe block, uses it to convert the input string slice into a `BString`. 
/// The allocator is dereferenced as a static lifetime to ensure it persists as long as necessary, allowing for the creation of a `BString` that is not bound to the typical lifetime of the input slice. 
/// This approach improves performance by reducing heap allocations for short-lived objects in multi-threaded scenarios.
/// 
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

/// Converts an iterator of characters into a static string stored in a memory arena. 
/// 
/// 
/// This function uses a thread-local arena to safely collect characters from an iterator into a `BString`, a specialized string container. 
/// The `collec_in` method is invoked with an unsafe context to ensure that memory allocation aligns properly within the arena's boundaries. 
/// The function ultimately returns a static string reference derived from converting the `BString` into a bump-allocated string, ensuring efficient memory usage and lifetime management through the arena while retaining a 'static lifetime.
fn collect_str_in_arena0(iter: impl Iterator<Item= char>) -> &'static str {
    THR_ARENA.with(|arena| {
        let p = arena as *const Bump;
        let vec: BString<'static> = unsafe { iter.collect_in::<BString>(p.as_ref::<'static>().unwrap()) };
        vec.into_bump_str()
    })
}

#[extension(pub trait AllocForCharIter)]
impl<T: Iterator<Item=char>> T {
    /// Provides a method for collecting a string representation of the implementing item into a global memory arena and returning a static string reference. 
    /// 
    /// The `galloc_collect_str` function utilizes an underlying function, `collect_str_in_arena0`, to manage the allocation and lifetime of the string within a predefined arena, enabling efficient memory usage that allows the returned string to have a `'static` lifetime. 
    /// This approach is particularly useful in scenarios where the string's long-term immutability and accessibility are needed across different parts of the program without reallocation. 
    /// 
    /// 
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

