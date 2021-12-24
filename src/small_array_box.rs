use std::mem::{transmute, ManuallyDrop, MaybeUninit};
use std::ptr::NonNull;
use std::slice::{from_raw_parts, from_raw_parts_mut};

use std::iter::IntoIterator;
use std::iter::{ExactSizeIterator, Iterator};

use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};

use array_init::array_init;

union SmallArrayBoxInner<T, const INLINE_LEN: usize> {
    ptr: NonNull<T>,
    inline_storage: ManuallyDrop<[MaybeUninit<T>; INLINE_LEN]>,
}

/// * `INLINE_LEN` - Number of elements that can be stored inline.
pub struct SmallArrayBox<T, const INLINE_LEN: usize> {
    storage: SmallArrayBoxInner<T, INLINE_LEN>,
    len: usize,
}

impl<T, const INLINE_LEN: usize> Default for SmallArrayBox<T, INLINE_LEN> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<T, const INLINE_LEN: usize> From<Box<[T]>> for SmallArrayBox<T, INLINE_LEN> {
    fn from(boxed: Box<[T]>) -> Self {
        Self::from_box(boxed)
    }
}

impl<T, const INLINE_LEN: usize> From<Vec<T>> for SmallArrayBox<T, INLINE_LEN> {
    fn from(vec: Vec<T>) -> Self {
        vec.into_boxed_slice().into()
    }
}

impl<T, const INLINE_LEN: usize> SmallArrayBox<T, INLINE_LEN> {
    pub const fn new_empty() -> Self {
        Self {
            storage: SmallArrayBoxInner {
                ptr: NonNull::dangling(),
            },
            len: 0,
        }
    }

    pub fn new<I>(iter: impl IntoIterator<IntoIter = I>) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        let iter = iter.into_iter();

        let len = iter.len();

        if len <= INLINE_LEN {
            let mut this = Self {
                storage: SmallArrayBoxInner {
                    inline_storage: ManuallyDrop::new(array_init(|_| MaybeUninit::uninit())),
                },
                len,
            };

            let inline_storage = unsafe { this.storage.inline_storage.deref_mut() };

            iter.zip(inline_storage).for_each(|(src, dst)| {
                dst.write(src);
            });

            this
        } else {
            let vec: Vec<T> = iter.collect();
            let array_ptr = Box::into_raw(vec.into_boxed_slice());
            let slice = unsafe { &mut *array_ptr };
            let ptr = unsafe { NonNull::new_unchecked(&mut slice[0]) };

            Self {
                storage: SmallArrayBoxInner { ptr },
                len,
            }
        }
    }

    pub fn from_box(boxed: Box<[T]>) -> Self {
        let len = boxed.len();

        if len <= INLINE_LEN {
            let vec: Vec<T> = boxed.into();
            Self::new(vec)
        } else {
            let array_ptr = Box::into_raw(boxed);
            let slice = unsafe { &mut *array_ptr };
            let ptr = unsafe { NonNull::new_unchecked(&mut slice[0]) };

            debug_assert_eq!(slice.len(), len);

            Self {
                storage: SmallArrayBoxInner { ptr },
                len,
            }
        }
    }

    pub fn into_boxed_slice(self) -> Box<[T]> {
        let len = self.len;

        let mut this = ManuallyDrop::new(self);

        if len <= INLINE_LEN {
            let inline_storage = unsafe { &mut this.storage.inline_storage };
            let mut vec = Vec::with_capacity(len);

            for elem in inline_storage[..len].iter_mut() {
                let ptr = elem.as_mut_ptr();
                vec.push(unsafe { ptr.read() });
            }

            debug_assert_eq!(vec.len(), len);

            vec.into_boxed_slice()
        } else {
            let ptr = unsafe { this.storage.ptr }.as_ptr();
            let slice = unsafe { from_raw_parts_mut(ptr, len) };
            unsafe { Box::from_raw(slice) }
        }
    }
}

impl<T, const INLINE_LEN: usize> Deref for SmallArrayBox<T, INLINE_LEN> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let len = self.len;

        if len <= INLINE_LEN {
            let inline_storage = unsafe { self.storage.inline_storage.deref() };
            unsafe { transmute(&inline_storage[..len]) }
        } else {
            let ptr = unsafe { self.storage.ptr }.as_ptr();
            unsafe { from_raw_parts(ptr, len) }
        }
    }
}

impl<T, const INLINE_LEN: usize> DerefMut for SmallArrayBox<T, INLINE_LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len;

        if len <= INLINE_LEN {
            let inline_storage = unsafe { self.storage.inline_storage.deref_mut() };
            unsafe { transmute(&mut inline_storage[..len]) }
        } else {
            let ptr = unsafe { self.storage.ptr }.as_ptr();
            unsafe { from_raw_parts_mut(ptr, len) }
        }
    }
}

impl<T, const INLINE_LEN: usize> Drop for SmallArrayBox<T, INLINE_LEN> {
    fn drop(&mut self) {
        let len = self.len;

        if len <= INLINE_LEN {
            let inline_storage = unsafe { self.storage.inline_storage.deref_mut() };

            inline_storage[..len].iter_mut().for_each(|elem| {
                let ptr = elem.as_mut_ptr();
                unsafe {
                    ptr.drop_in_place();
                }
            });
        } else {
            let ptr = unsafe { self.storage.ptr }.as_ptr();
            let slice = unsafe { from_raw_parts_mut(ptr, len) };
            drop(unsafe { Box::from_raw(slice) });
        }
    }
}

impl<T: Debug, const INLINE_LEN: usize> Debug for SmallArrayBox<T, INLINE_LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.deref())
    }
}

#[cfg(test)]
mod tests {
    type SmallArrayBox = super::SmallArrayBox<u8, 8>;

    use std::ops::{Deref, DerefMut};
    use std::ptr;

    fn assert_ptr_eq(x: *const [u8], y: *const [u8]) {
        assert!(ptr::eq(x, y));
    }

    #[test]
    fn test_new_empty() {
        let mut empty_array = SmallArrayBox::new_empty();

        let empty: &[u8] = &[];

        assert_eq!(empty_array.deref(), empty);
        assert_eq!(empty_array.deref_mut(), empty);
        assert_ptr_eq(empty_array.deref(), empty_array.deref_mut());

        let boxed = empty_array.into_boxed_slice();

        assert_eq!(&*boxed, empty);
    }
}
