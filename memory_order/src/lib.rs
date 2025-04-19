use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {} // avoid false sharing
        }
        return SpinLockGuard { lock: self };
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct SpinLockGuard<'a, T: 'a> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn it_works() {
        const N: usize = 3;

        let data_spinlock = Arc::new(SpinLock::new(vec![1, 2, 3, 4]));
        let res_spinlock = Arc::new(SpinLock::new(0));

        let mut threads = Vec::with_capacity(N);

        (0..N).for_each(|_| {
            let data_spinlock_clone = Arc::clone(&data_spinlock);
            let res_spinlock_clone = Arc::clone(&res_spinlock);

            threads.push(thread::spawn(move || {
                let result = {
                    let mut data = data_spinlock_clone.lock();
                    let result = data.iter().fold(0, |acc, x| acc + x * 2);
                    data.push(result);
                    result
                };
                *res_spinlock_clone.lock() += result;
            }))
        });

        let mut data = data_spinlock.lock();
        let result = data.iter().fold(0, |acc, x| acc + x * 2);
        data.push(result);
        drop(data);
        *res_spinlock.lock() += result;

        threads
            .into_iter()
            .for_each(|thread| thread.join().expect("failed"));

        assert_eq!(*res_spinlock.lock(), 800);
    }
}
