#![feature(thread_id_value)]
use std::iter;
use std::sync::{Mutex, TryLockError};
use std::thread;

use bumpalo::Bump;

pub struct Herd {
    slots: Vec<Mutex<Bump>>,
}

unsafe impl Sync for Herd { }

impl Herd {
    pub fn new() -> Self {
        let slots = iter::repeat_with(Bump::new)
            .map(Mutex::new)
            .take(num_cpus::get())
            .collect();
        Self {
            slots,
        }
    }

    pub fn reset(&mut self) {
        for s in &mut self.slots {
            s.get_mut().unwrap().reset();
        }
    }

    fn choose<R, F: FnOnce(&Bump) -> R>(&self, f: F) -> R {
        let offset = thread::current().id().as_u64().get() as usize % self.slots.len();
        let seq = iter::repeat(self.slots.iter())
            .flatten()
            .skip(offset)
            .take(self.slots.len());
        for s in seq {
            match s.try_lock() {
                Ok(lock) => {
                    return f(&lock);
                }
                Err(TryLockError::WouldBlock) => (),
                Err(TryLockError::Poisoned(_)) => panic!("Poisoned herd member"),
            }
        }

        // Should not really happen, but can't rule out really.
        f(&self.slots[offset].lock().unwrap())
    }

    pub fn alloc<T>(&self, val: T) -> &T {
        let res = self.choose(|arena| arena.alloc(val) as *const _);
        unsafe { &*res }
    }
}
