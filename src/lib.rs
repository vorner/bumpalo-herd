use std::sync::Mutex;

use bumpalo::Bump;

#[derive(Default)]
struct HerdInner {
    extra: Vec<Box<Bump>>,
}

#[derive(Default)]
pub struct Herd(Mutex<HerdInner>);

pub struct Member<'h> {
    arena: Option<Box<Bump>>,
    owner: &'h Herd,
}

impl<'h> Member<'h> {
    pub fn alloc<T>(&self, val: T) -> &'h T {
        let result = self.arena.as_ref().unwrap().alloc(val) as *const _;
        unsafe { &*result }
    }
}

impl Drop for Member<'_> {
    fn drop(&mut self) {
        self.owner.0.lock().unwrap().extra.push(self.arena.take().unwrap());
    }
}

impl Herd {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        for e in &mut self.0.get_mut().unwrap().extra {
            e.reset();
        }
    }

    pub fn get<'h>(&'h self) -> Member<'h> {
        let mut lock = self.0.lock().unwrap();
        let bump = lock.extra.pop().unwrap_or_default();
        Member {
            arena: Some(bump),
            owner: self,
        }
    }
}
