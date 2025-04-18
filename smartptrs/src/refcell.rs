use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::cell::Cell;

#[derive(Clone, Copy)]
enum BorrowState {
    UnBorrow,
    SharedBorrow(usize),
    ExclusiveBorrow,
}

// Send if T: Send !Sync
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<BorrowState>,
}

impl<T> RefCell<T> {
    pub const fn new(value: T) -> RefCell<T> {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(BorrowState::UnBorrow),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        match self.state.get() {
            BorrowState::UnBorrow => {
                self.state.set(BorrowState::SharedBorrow(1));
                Ref {
                    value: unsafe { NonNull::new_unchecked(self.value.get()) },
                    state: &self.state,
                }
            }
            BorrowState::SharedBorrow(count) => {
                self.state.set(BorrowState::SharedBorrow(count + 1));
                Ref {
                    value: unsafe { NonNull::new_unchecked(self.value.get()) },
                    state: &self.state,
                }
            }
            BorrowState::ExclusiveBorrow => {
                panic!("fuck you no way!")
            }
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        match self.state.get() {
            BorrowState::UnBorrow => {
                self.state.set(BorrowState::ExclusiveBorrow);
                RefMut {
                    value: unsafe { NonNull::new_unchecked(self.value.get()) },
                    state: &self.state,
                    _marker: PhantomData,
                }
            }
            BorrowState::SharedBorrow(_) | BorrowState::ExclusiveBorrow => {
                panic!("fuck you no way!")
            }
        }
    }
}

pub struct Ref<'a, T: 'a> {
    value: NonNull<T>,
    state: &'a Cell<BorrowState>,
}

impl<'a, T: 'a> std::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.value.as_ref() }
    }
}

impl<'a, T: 'a> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        match self.state.get() {
            BorrowState::UnBorrow | BorrowState::ExclusiveBorrow => unreachable!(),
            BorrowState::SharedBorrow(count) => {
                if count == 1 {
                    self.state.set(BorrowState::UnBorrow);
                } else {
                    self.state.set(BorrowState::SharedBorrow(count - 1));
                }
            }
        }
    }
}

pub struct RefMut<'a, T: 'a> {
    value: NonNull<T>,
    state: &'a Cell<BorrowState>,
    _marker: PhantomData<&'a mut T>, // invariance need
}

impl<'a, T: 'a> std::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<'a, T: 'a> std::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<'a, T: 'a> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        match self.state.get() {
            BorrowState::UnBorrow | BorrowState::SharedBorrow(_) => unreachable!(),
            BorrowState::ExclusiveBorrow => {
                self.state.set(BorrowState::UnBorrow);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_work0() {
        let z = RefCell::new(String::from("sdfs"));
        let a = z.borrow();
        assert_eq!(4, a.len());
    }

    #[test]
    fn it_work1() {
        let z = RefCell::new(String::from("fuck"));
        {
            let mut a = z.borrow_mut();
            a.push_str(" you!");
        }
        let a = z.borrow();
        assert_eq!(*a, "fuck you!");
    }

    #[test]
    #[should_panic]
    fn it_panic() {
        let z = RefCell::new(String::from("sdfs"));
        let a = z.borrow();
        let b = z.borrow_mut();
        println!("{} {}", a.len(), b.len());
    }
}
