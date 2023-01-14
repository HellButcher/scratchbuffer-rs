#![warn(
    // missing_docs,
    // rustdoc::missing_doc_code_examples,
    future_incompatible,
    rust_2018_idioms,
    unused,
    trivial_casts,
    trivial_numeric_casts,
    unused_lifetimes,
    unused_qualifications,
    unused_crate_dependencies,
    clippy::cargo,
    clippy::multiple_crate_versions,
    clippy::empty_line_after_outer_attr,
    clippy::fallible_impl_from,
    clippy::redundant_pub_crate,
    clippy::use_self,
    clippy::suspicious_operation_groupings,
    clippy::useless_let_if_seq,
    // clippy::missing_errors_doc,
    // clippy::missing_panics_doc,
    clippy::wildcard_imports
)]
#![allow(clippy::comparison_chain)]
#![doc(html_no_source)]
#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;
use core::{
    alloc::Layout,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr, slice,
};

pub struct ScratchBuffer<T = u8> {
    buf: *mut u8,
    len_bytes: usize,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl ScratchBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: ptr::null_mut(),
            len_bytes: 0,
            capacity: 0,
            _phantom: PhantomData,
        }
    }

    /// clears the buffer, and allows to use it with a different type.
    ///
    /// # Requirements on `T`
    /// * `T` must implement `Copy` because the added entries will not be dropped!
    /// * `T` must have an alignment of `mem::align_of::<usize>()` or less (usually `1`, `2`, `4` or on a 64-bit target `8`),
    ///    because the memory is always allocated with an alignment of `mem::align_of::<usize>()`.
    ///
    pub fn clear_and_use_as<T: Copy>(&mut self) -> &mut ScratchBuffer<T> {
        assert!(mem::align_of::<T>() <= mem::align_of::<usize>());
        assert_ne!(0, mem::size_of::<T>());
        self.len_bytes = 0;
        unsafe { mem::transmute(self) }
    }
}

impl<T: Copy> ScratchBuffer<T> {
    const ALIGN: usize = mem::align_of::<usize>();
    const ITEM_LEN: usize = mem::size_of::<T>();

    #[inline]
    pub fn len(&self) -> usize {
        self.len_bytes / Self::ITEM_LEN
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len_bytes == 0
    }

    pub fn reserve(&mut self, _num_items: usize) {
        let new_len = self.len_bytes + Self::ITEM_LEN;
        if new_len > self.capacity {
            let new_capacity = new_len.next_power_of_two();
            unsafe {
                if self.buf.is_null() {
                    let layout = Layout::from_size_align(new_capacity, Self::ALIGN).unwrap();
                    self.buf = alloc::alloc::alloc(layout);
                } else {
                    let layout = Layout::from_size_align(self.capacity, Self::ALIGN).unwrap();
                    self.buf = alloc::alloc::realloc(self.buf, layout, new_capacity)
                }
            }
            self.capacity = new_capacity;
        }
    }

    pub fn push(&mut self, value: T) -> &mut T {
        self.reserve(1);
        let len = self.len();
        unsafe {
            let ptr = self.as_mut_ptr().add(len);
            ptr::write(ptr, value);
            self.len_bytes += Self::ITEM_LEN;
            &mut *ptr
        }
    }

    pub fn insert(&mut self, index: usize, value: T) -> &mut T {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (is {index}) should be <= len (is {len})");
        }

        self.reserve(1);
        let len = self.len();
        unsafe {
            let ptr = self.as_mut_ptr().add(len);
            if index < len {
                // copy bytes to the right to make space for the new value
                ptr::copy(ptr, ptr.add(1), len - index);
            } else if index > len {
                assert_failed(index, len);
            }
            ptr::write(ptr, value);
            self.len_bytes += Self::ITEM_LEN;
            &mut *ptr
        }
    }

    pub fn binary_search_insert_by_key<K: Ord>(
        &mut self,
        key: &K,
        get_key: impl Fn(&T) -> K,
    ) -> &mut T
    where
        T: Default,
    {
        self.binary_search_insert_by_key_with(key, get_key, Default::default)
    }

    pub fn binary_search_insert_by_key_with<K: Ord>(
        &mut self,
        key: &K,
        get_key: impl Fn(&T) -> K,
        create: impl FnOnce() -> T,
    ) -> &mut T {
        match self.binary_search_by_key(key, get_key) {
            Ok(i) => &mut self[i],
            Err(i) => self.insert(i, create()),
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.buf as *const T
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buf as *mut T
    }

    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        if len == 0 {
            return &[];
        }
        unsafe {
            let ptr = self.buf as *const T;
            slice::from_raw_parts(ptr, len)
        }
    }
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let len = self.len();
        if len == 0 {
            return &mut [];
        }
        unsafe {
            let ptr = self.buf as *mut T;
            slice::from_raw_parts_mut(ptr, len)
        }
    }
}

impl<T: Copy> Deref for ScratchBuffer<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Copy> DerefMut for ScratchBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<A: Copy> Extend<A> for ScratchBuffer<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T> Drop for ScratchBuffer<T> {
    fn drop(&mut self) {
        if !self.buf.is_null() {
            let layout = Layout::from_size_align(self.capacity, mem::align_of::<usize>()).unwrap();
            unsafe {
                alloc::alloc::dealloc(self.buf, layout);
            }
            self.buf = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ScratchBuffer;

    #[test]
    fn test_1() {
        let mut buf = ScratchBuffer::new();

        let u32buf = buf.clear_and_use_as::<u32>();
        u32buf.push(123);
        u32buf.push(456);
        assert_eq!(&[123u32, 456], u32buf.as_slice());

        let u16buf = buf.clear_and_use_as::<u16>();
        u16buf.push(345);
        u16buf.push(678);
        assert_eq!(&[345u16, 678], u16buf.as_slice());
    }
}
