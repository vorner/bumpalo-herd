#![doc(
    html_root_url = "https://docs.rs/bumpalo-herd/0.1.0/",
    test(attr(deny(warnings)))
)]
#![warn(missing_docs)]

use std::alloc::Layout;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Mutex;

// TODO: Prove that we don't ever drop the Bump when we shouldn't, even in case something panics.
// Can eg. push panic?
// TODO: Prove that our lifetime extending is correct.

use bumpalo::Bump;

type HerdInner = Vec<Box<Bump>>;

#[derive(Default)]
pub struct Herd(Mutex<HerdInner>);

pub struct Member<'h> {
    arena: ManuallyDrop<Box<Bump>>,
    owner: &'h Herd,
}

macro_rules! alloc_fn {
    ($(pub fn $name: ident<($($g: tt)*)>(&self, $($pname: ident: $pty: ty),*) -> $res: ty;)*) => {
        $(
            pub fn $name<$($g)*>(&self, $($pname: $pty),*) -> $res {
                self.extend(self.arena.$name($($pname),*))
            }
        )*
    }
}

impl<'h> Member<'h> {
    alloc_fn! {
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
        self.extend(self.arena.alloc_slice_fill_iter(iter))
    }

    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        self.arena.as_ref().alloc_layout(layout)
    }

    fn extend<'s, T: ?Sized>(&'s self, v: &'s mut T) -> &'h mut T {
        let result = v as *mut T;
        unsafe { &mut *result }
    }

    // Note: This *can't* return `&'h Bump`. That way one could keep a reference, drop the Member
    // and let another thread take it - that would allow both to allocate from the same Bump which
    // would be UB.
    pub fn as_bump(&self) -> &Bump {
        &self.arena
    }
}

impl Drop for Member<'_> {
    fn drop(&mut self) {
        let mut lock = self.owner.0.lock().unwrap();
        let member = unsafe { ManuallyDrop::take(&mut self.arena) };
        lock.push(member);
    }
}

impl Herd {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        for e in self.0.get_mut().unwrap().iter_mut() {
            e.reset();
        }
    }

    pub fn get(&self) -> Member<'_> {
        let mut lock = self.0.lock().unwrap();
        let bump = lock.pop().unwrap_or_default();
        Member {
            arena: ManuallyDrop::new(bump),
            owner: self,
        }
    }
}
