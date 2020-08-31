#![feature(test)]
#![allow(dead_code)]

extern crate test;

use std::mem;
use std::sync::Mutex;

use bumpalo::Bump;
use bumpalo_herd::Herd;
use crossbeam_utils::thread;
use test::Bencher;

const CNT: usize = 1000_000;

struct NodeBoxed {
    value: usize,
    next: Option<Box<NodeBoxed>>,
}

impl Drop for NodeBoxed {
    fn drop(&mut self) {
        let mut buf = Vec::new();
        let mut head = self.next.take();
        while let Some(mut n) = head {
            head = n.next.take();
            buf.push(n);
        }
    }
}

fn boxed_sequence(cnt: usize) {
    let mut head = None;

    for i in 0..cnt {
        head = test::black_box(Some(Box::new(NodeBoxed {
            value: i,
            next: head,
        })));
    }
}

#[bench]
fn alloc_directly(b: &mut Bencher) {
    b.iter(|| {
        boxed_sequence(CNT);
    });
}

fn split<F: Fn(usize) + Sync>(f: F) {
    let n = num_cpus::get();
    let cnt = CNT / n;
    thread::scope(|s| {
        for _ in 0..n {
            s.spawn(|_| {
                let cnt = cnt;
                f(cnt);
            });
        }
    }).unwrap();
}

#[bench]
fn alloc_multi(b: &mut Bencher) {
    b.iter(|| {
        split(boxed_sequence);
    });
}

struct NodeRef<'a> {
    value: usize,
    next: Option<&'a NodeRef<'a>>,
}

#[bench]
fn single_threaded(b: &mut Bencher) {
    let mut arena = Bump::new();

    b.iter(|| {
        let mut head: Option<&_> = None;
        for i in 0..CNT {
            head = test::black_box(Some(arena.alloc(NodeRef {
                value: i,
                next: head,
            })));
        }

        arena.reset();
    });
}

unsafe fn extend<'a, 'b, T>(v: &'a T) -> &'b T {
    mem::transmute(v)
}

#[bench]
fn locked(b: &mut Bencher) {
    let arena = Mutex::new(Bump::new());

    b.iter(|| {
        split(|cnt| {
            let mut head: Option<&_> = None;
            for i in 0..cnt {
                unsafe {
                    head = test::black_box(Some(extend(arena.lock().unwrap().alloc(NodeRef {
                        value: i,
                        next: head,
                    }))));
                }
            }
        });

        arena.lock().unwrap().reset();
    });
}

#[bench]
fn herd_single(b: &mut Bencher) {
    let mut arena = Herd::new();

    b.iter(|| {
        let mut head: Option<&_> = None;
        for i in 0..CNT {
            head = test::black_box(Some(arena.alloc(NodeRef {
                value: i,
                next: head,
            })));
        }

        arena.reset();
    });
}

#[bench]
fn herd_multi(b: &mut Bencher) {
    let mut arena = Herd::new();

    b.iter(|| {
        split(|cnt| {
            let mut head: Option<&_> = None;

            for i in 0..cnt {
                head = test::black_box(Some(arena.alloc(NodeRef {
                    value: i,
                    next: head,
                })));
            }
        });

        arena.reset();
    });
}
