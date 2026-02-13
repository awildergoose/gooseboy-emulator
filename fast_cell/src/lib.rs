use std::{
    cell::{UnsafeCell},
    rc::Rc,
};
#[cfg(any(debug_assertions, feature = "borrow-check"))]
use std::cell::Cell;

/// Single-threaded mutable container, clonable by reference.
#[derive(Debug)]
pub struct FastCell<T> {
    value: Rc<UnsafeCell<T>>,

    #[cfg(any(debug_assertions, feature = "borrow-check"))]
    borrowed: Rc<Cell<bool>>,
}

impl<T> FastCell<T> {
    /// Create a new [`FastCell`]
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(UnsafeCell::new(value)),
            #[cfg(any(debug_assertions, feature = "borrow-check"))]
            borrowed: Rc::new(Cell::new(false)),
        }
    }

    /// Access the value mutably
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        #[cfg(any(debug_assertions, feature = "borrow-check"))]
        {
            struct Guard<'a> {
                flag: &'a Cell<bool>,
            }
            impl Drop for Guard<'_> {
                fn drop(&mut self) {
                    self.flag.set(false);
                }
            }
            assert!(
                !self.borrowed.get(),
                "FastCell::with called reentrantly (nested with)"
            );
            self.borrowed.set(true);
            let _guard = Guard {
                flag: &self.borrowed,
            };

            let ptr = self.value.get();
            unsafe { f(&mut *ptr) }
        }
        #[cfg(not(debug_assertions))]
        {
            let ptr = self.value.get();
            unsafe { f(&mut *ptr) }
        }
    }

    /// Get a mutable reference directly
    pub fn get_mut(&mut self) -> &mut T {
        #[cfg(any(debug_assertions, feature = "borrow-check"))]
        {
            debug_assert!(
                !self.borrowed.get(),
                "FastCell::get_mut called while a `with` borrow is active"
            );
        }
        unsafe { &mut *self.value.get() }
    }

    /// Consume the cell and return the inner value
    pub fn into_inner(self) -> Result<T, Self> {
        match Rc::try_unwrap(self.value) {
            Ok(cell) => Ok(cell.into_inner()),
            Err(rc) => Err(Self {
                value: rc,
                #[cfg(any(debug_assertions, feature = "borrow-check"))]
                borrowed: self.borrowed,
            }),
        }
    }
}

impl<T> Clone for FastCell<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            #[cfg(any(debug_assertions, feature = "borrow-check"))]
            borrowed: self.borrowed.clone(),
        }
    }
}

unsafe impl<T: Send> Send for FastCell<T> {}
unsafe impl<T: Send> Sync for FastCell<T> {}

#[cfg(test)]
mod tests {
    use super::FastCell;

    #[test]
    fn basic_mutation() {
        let mut cell = FastCell::new(10);
        cell.with(|v| *v += 5);
        assert_eq!(cell.get_mut(), &15);
    }

    #[test]
    fn multiple_with_calls() {
        let mut cell = FastCell::new(0);
        for i in 0..5 {
            cell.with(|v| *v += i);
        }
        assert_eq!(cell.get_mut(), &10);
    }

    #[test]
    fn sequential_with_calls() {
        let mut cell = FastCell::new(1);
        cell.with(|v| *v *= 2);
        cell.with(|v| *v += 3);
        assert_eq!(cell.get_mut(), &5);
    }

    #[test]
    fn get_mut_usage() {
        let mut cell = FastCell::new(100);
        let v = cell.get_mut();
        *v -= 50;
        assert_eq!(cell.get_mut(), &50);
    }

    #[test]
    fn into_inner_returns_value() {
        let cell = FastCell::new(String::from("hello"));
        let s = cell.into_inner().expect("failed to get inner");
        assert_eq!(s, "hello");
    }

    #[test]
    fn pointer_identity_check_reserved_capacity() {
        let mut v = Vec::with_capacity(4);
        v.extend_from_slice(&[1, 2, 3]);
        let mut cell = FastCell::new(v);

        let addr_before = cell.get_mut().as_ptr();
        cell.with(|vec| vec.push(4));
        let addr_after = cell.get_mut().as_ptr();
        assert_eq!(addr_before, addr_after);
        assert_eq!(cell.get_mut(), &[1, 2, 3, 4]);
    }

    #[test]
    fn reallocation_on_push() {
        let mut cell = FastCell::new(vec![1, 2, 3]);
        let addr_before = cell.get_mut().as_ptr();
        cell.with(|v| v.push(4));
        let addr_after = cell.get_mut().as_ptr();
        assert_ne!(addr_before, addr_after);
    }

    #[test]
    fn large_data_mutation() {
        let size = 1024 * 1024;
        let mut cell = FastCell::new(vec![0u8; size]);
        cell.with(|v| v[0] = 1);
        cell.with(|v| v[size - 1] = 255);
        let v = cell.get_mut();
        assert_eq!(v[0], 1);
        assert_eq!(v[size - 1], 255);
        assert_eq!(v.len(), size);
    }

    #[test]
    fn multiple_types() {
        let mut cell_int = FastCell::new(42);
        let mut cell_vec = FastCell::new(vec![1, 2, 3]);
        cell_int.with(|v| *v += 1);
        cell_vec.with(|v| v.push(4));
        assert_eq!(cell_int.get_mut(), &43);
        assert_eq!(cell_vec.get_mut(), &[1, 2, 3, 4]);
    }

    #[test]
    fn chained_with_calls() {
        let mut cell = FastCell::new(0);
        let mut sum = 0;
        for _ in 0..10 {
            cell.with(|v| {
                *v += 1;
                sum += *v;
            });
        }
        assert_eq!(*cell.get_mut(), 10);
        assert_eq!(sum, 55);
    }

    #[test]
    fn reuse_after_into_inner() {
        let cell = FastCell::new(99);
        let val = cell.into_inner().expect("failed to get inner");
        assert_eq!(val, 99);
    }

    #[test]
    #[should_panic(expected = "reentrantly")]
    fn nested_with_panics_in_debug() {
        let cell = FastCell::new(1);
        cell.with(|v| {
            *v += 1;
            cell.with(|w| *w += 1);
        });
    }

    #[test]
    fn sequential_with_is_ok() {
        let mut cell = FastCell::new(1);
        cell.with(|v| *v *= 2);
        cell.with(|v| *v += 3);
        assert_eq!(cell.get_mut(), &5);
    }
}
