use std::cell::UnsafeCell;

// Send if T Send
// !Sync
pub struct Cell<T> {
    v: UnsafeCell<T>,
}

// unsafe impl<T: Send> Send for Cell<T> {} // unnecessary bacasue of UnsafeCell
// impl<T> !Sync for Cell<T> {} // unnecessary becasue of UnsafeCell

impl<T> Cell<T> {
    pub const fn new(value: T) -> Cell<T> {
        Self {
            v: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, val: T) {
        // SAFETY: only the thread can modify because of !Sync
        unsafe {
            *self.v.get() = val;
        }
    }
}

impl<T: Copy> Cell<T> {
    pub fn get(&self) -> T {
        // SAFETY:
        unsafe { *self.v.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_work() {
        let a = Cell::new('a');
        a.set('b');
        assert_eq!(a.get(), 'b');
    }
}
