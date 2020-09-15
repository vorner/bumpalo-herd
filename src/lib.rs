use std::alloc::Layout;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Mutex;

use bumpalo::Bump;

#[derive(Default)]
struct HerdInner {
    extra: Vec<Box<Bump>>,
}

#[derive(Default)]
pub struct Herd(Mutex<HerdInner>);

pub struct Member<'h> {
    arena: ManuallyDrop<Box<Bump>>,
    owner: &'h Herd,
}

macro_rules! alloc {
    ($(pub fn $name: ident<($($g: tt)*)>(&self, $($pname: ident: $pty: ty),*) -> $res: ty;)*) => {
        $(
            pub fn $name<$($g)*>(&self, $($pname: $pty),*) -> $res {
                let result = self.arena.$name($($pname),*) as *mut _;
                unsafe { &mut *result }
            }
        )*
    }
}

impl<'h> Member<'h> {
    alloc! {
        pub fn alloc<(T)>(&self, val: T) -> &'h mut T;
        pub fn alloc_with<(T, F: FnOnce() -> T)>(&self, f: F) -> &'h mut T;
        pub fn alloc_str<()>(&self, src: &str) -> &'h mut str;
        pub fn alloc_slice_clone<(T: Clone)>(&self, src: &[T]) -> &'h mut [T];
        pub fn alloc_slice_copy<(T: Copy)>(&self, src: &[T]) -> &'h mut [T];
        pub fn alloc_slice_fill_clone<(T: Clone)>(&self, len: usize, value: &T) -> &'h mut [T];
        pub fn alloc_slice_fill_copy<(T: Copy)>(&self, len: usize, value: T) -> &'h mut [T];
        pub fn alloc_slice_fill_default<(T: Default)>(&self, len: usize) -> &'h mut [T];
        pub fn alloc_slice_fill_with<(T, F: FnMut(usize) -> T)>(&self, len: usize, f: F)
            -> &'h mut [T];
    }

    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> &'h mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let result = self.arena.as_ref().alloc_slice_fill_iter(iter) as *mut _;
        unsafe { &mut *result }
    }

    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        self.arena.as_ref().alloc_layout(layout)
    }
}

impl Drop for Member<'_> {
    fn drop(&mut self) {
        let mut lock = self.owner.0.lock().unwrap();
        let member = unsafe { ManuallyDrop::take(&mut self.arena) };
        lock.extra.push(member);
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

    pub fn get(&self) -> Member<'_> {
        let mut lock = self.0.lock().unwrap();
        let bump = lock.extra.pop().unwrap_or_default();
        Member {
            arena: ManuallyDrop::new(bump),
            owner: self,
        }
    }
}
