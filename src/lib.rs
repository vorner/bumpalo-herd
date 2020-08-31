#![feature(thread_id_value)]
use std::iter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use bumpalo::Bump;
use crossbeam_utils::CachePadded;

struct Slot {
    arena: Bump,
    locked: AtomicBool,
}

pub struct Herd {
    slots: Vec<CachePadded<Slot>>,
}

unsafe impl Sync for Herd { }

impl Herd {
    pub fn new() -> Self {
        let slots = iter::repeat_with(|| Slot {
                arena: Bump::new(),
                locked: AtomicBool::new(false),
            })
            .map(CachePadded::new)
            .take(num_cpus::get())
            .collect();
        Self {
            slots,
        }
    }

    pub fn reset(&mut self) {
        for s in &mut self.slots {
            s.arena.reset();
        }
    }

    fn choose<R, F: FnOnce(&Bump) -> R>(&self, f: F) -> R {
        let offset = thread::current().id().as_u64().get() as usize % self.slots.len();
        let seq = iter::repeat(self.slots.iter())
            .flatten()
            .skip(offset);
        for s in seq {
            if s.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                let result = f(&s.arena);
                s.locked.store(false, Ordering::Release);
                return result;
            }
        }
        unreachable!();
    }

    pub fn alloc<T>(&self, val: T) -> &T {
        let res = self.choose(|arena| arena.alloc(val) as *const _);
        unsafe { &*res }
    }
}
