use std::cell::{Cell, UnsafeCell};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Condvar, Mutex};

// Flavors:
//  - Synchronous channels: Channel where send() can block. Limited capacity.
//   - Mutex + Condvar + VecDeque
//   - Atomic VecDeque (atomic queue) + thread::park + thread::Thread::notify
//  - Asynchronous channels: Channel where send() cannot block. Unbounded.
//   - Mutex + Condvar + VecDeque
//   - Mutex + Condvar + LinkedList
//   - Atomic linked list, linked list of T
//   - Atomic block linked list, linked list of atomic VecDeque<T>
//  - Rendezvous channels: Synchronous with capacity = 0. Used for thread synchronization.
//  - Oneshot channels: Any capacity. In practice, only one call to send().

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared::<T>::new());
    (
        Sender {
            shared: Arc::clone(&shared),
        },
        Receiver {
            shared: Arc::clone(&shared),
            buffer: UnsafeCell::new(VecDeque::default()),
            phantom: PhantomData,
        },
    )
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Sender<T> {}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        inner.disconnected = inner.senders == 0;
        if inner.disconnected {
            self.shared.condvar.notify_all();
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: UnsafeCell<VecDeque<T>>,
    phantom: PhantomData<Cell<()>>, // !Sync
}

unsafe impl<T: Send> Send for Receiver<T> {}

struct Shared<T> {
    inner: Mutex<Inner<T>>,
    condvar: Condvar,
}

impl<T> Shared<T> {
    fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
            condvar: Condvar::new(),
        }
    }
}

struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
    disconnected: bool,
}

impl<T> Inner<T> {
    fn new() -> Self {
        Self {
            queue: VecDeque::default(),
            senders: 1,
            disconnected: false,
        }
    }
}

#[derive(Debug)]
pub struct SendError<T>(pub T);

#[derive(Debug)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

#[derive(Debug)]
pub struct RecvError;

impl<T> Sender<T> {
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        let mut inner = self.shared.inner.lock().unwrap();
        if inner.disconnected {
            return Err(SendError(t));
        }
        inner.queue.push_back(t);
        if inner.queue.len() == 1 {
            self.shared.condvar.notify_one();
        }
        Ok(())
    }
}

impl<T> Receiver<T> {
    fn get_buffer(&self) -> &mut VecDeque<T> {
        // Safety:
        unsafe { &mut *self.buffer.get() }
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        if let Some(t) = self.get_buffer().pop_front() {
            return Ok(t);
        }

        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(t) => {
                    std::mem::swap(self.get_buffer(), &mut inner.queue);
                    return Ok(t);
                }
                None if inner.disconnected => {
                    return Err(RecvError);
                }
                None => {
                    inner = self.shared.condvar.wait(inner).unwrap();
                }
            }
        }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        if let Some(t) = self.get_buffer().pop_front() {
            return Ok(t);
        }

        let mut inner = self.shared.inner.lock().unwrap();

        match inner.queue.pop_front() {
            Some(t) => {
                std::mem::swap(self.get_buffer(), &mut inner.queue);
                Ok(t)
            }
            None if inner.disconnected => Err(TryRecvError::Disconnected),
            None => Err(TryRecvError::Empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn basic_send_recv() {
        let (tx, rx) = channel();
        tx.send(42).unwrap();
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn multiple_senders() {
        let (tx, rx) = channel();
        let tx1 = tx.clone();
        let tx2 = tx.clone();

        let handle = thread::spawn(move || {
            tx.send(1).unwrap();
            tx1.send(2).unwrap();
            tx2.send(3).unwrap();
        });

        handle.join().unwrap();

        let mut results = vec![];
        results.push(rx.recv().unwrap());
        results.push(rx.recv().unwrap());
        results.push(rx.recv().unwrap());
        results.sort();
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[test]
    fn sender_disconnect() {
        let (tx, rx) = channel::<i32>();
        let tx_clone = tx.clone();
        drop(tx);
        assert!(rx.try_recv().is_err());
        drop(tx_clone);
        assert!(matches!(rx.recv(), Err(RecvError)));
    }

    #[test]
    fn non_blocking_receive() {
        let (tx, rx) = channel();
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        tx.send(10).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 10);

        drop(tx);
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Disconnected)));
    }

    #[test]
    fn high_concurrency_stress() {
        let (tx, rx) = channel();
        let mut handles = vec![];

        for _ in 0..10 {
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    tx.send(i).unwrap();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut count = 0;
        while let Ok(num) = rx.try_recv() {
            count += 1;
            assert!(num >= 0 && num < 100);
        }
        assert_eq!(count, 10 * 100);
    }
}
