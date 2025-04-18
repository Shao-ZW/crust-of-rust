use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::cell::Cell;

// !Send and !Sync
pub struct Rc<T> {
    ptr: NonNull<RcInner<T>>,
    phantom: PhantomData<RcInner<T>>, // drop check
}

struct RcInner<T> {
    strong: Cell<usize>,
    value: T,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Rc<T> {
        Self {
            ptr: unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(RcInner {
                    strong: Cell::new(1),
                    value,
                })))
            },
            phantom: PhantomData,
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };
        inner.strong.set(inner.strong.get() + 1);
        Self {
            ptr: self.ptr, // NonNull is Copy
            phantom: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().value }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };
        let cnt = inner.strong.get();
        if cnt == 1 {
            drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
        } else {
            inner.strong.set(cnt - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_work() {
        let a = Rc::new(Cell::new(10));
        let b = a.clone();
        a.set(19);
        assert_eq!(19, b.get());
    }
}
