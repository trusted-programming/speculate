#![feature(test)]

extern crate test;

use speculate::*;
use std::sync::{mpsc, Arc};
use test::{black_box, Bencher};

#[bench]
fn bench_direct_2048(b: &mut Bencher) {
    let v: Vec<usize> = (0..2048).collect();
    b.iter(|| {
        // Use black_box to prevent compiler optimizations regarding these computations
        let val = v.iter().fold(0, |old, new| old + *new);
        black_box(v.iter().fold(val, |old, new| old + *new));
    });
}

#[bench]
fn test_spec_correct_2048(b: &mut Bencher) {
    let v = (0..2048).collect::<Vec<_>>();
    let v_arc = Arc::new(v);

    b.iter(|| {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        tx.send(v_arc.clone()).unwrap();
        tx2.send(v_arc.clone()).unwrap();

        spec(
            move || {
                let local_arc = rx.recv().unwrap();
                local_arc.iter().fold(0, |old, &new| old + new)
            },
            || 2096128,
            move |x| {
                let local_arc = rx2.recv().unwrap();
                local_arc.iter().fold(x, |old, &new| old + new)
            },
        );
    });
}

#[bench]
fn bench_spec_wrong_2048(b: &mut Bencher) {
    let v = (0..2048).collect::<Vec<_>>();
    let v_arc = Arc::new(v);

    b.iter(|| {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        tx.send(v_arc.clone()).unwrap();
        tx2.send(v_arc.clone()).unwrap();
        tx2.send(v_arc.clone()).unwrap();

        spec(
            move || {
                let local_arc = rx.recv().unwrap();
                local_arc.iter().fold(0, |old, &new| old + new)
            },
            || 0, // Incorrect result of `fold`
            move |x| {
                let local_arc = rx2.recv().unwrap();
                local_arc.iter().fold(x, |old, &new| old + new)
            },
        );
    });
}

#[bench]
fn bench_direct_65536(b: &mut Bencher) {
    let v: Vec<usize> = (0..65536).collect();
    b.iter(|| {
        // Use black_box to prevent compiler optimizations regarding these computations
        let val = v.iter().fold(0, |old, new| old + *new);
        black_box(v.iter().fold(val, |old, new| old + *new));
    });
}

#[bench]
fn bench_spec_correct_65536(b: &mut Bencher) {
    let v = (0..65536).collect::<Vec<_>>();
    let v_arc = Arc::new(v);

    b.iter(|| {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        tx.send(v_arc.clone()).unwrap();
        tx2.send(v_arc.clone()).unwrap();

        spec(
            move || {
                let local_arc = rx.recv().unwrap();
                local_arc.iter().fold(0, |old, &new| old + new)
            },
            || 2147450880,
            move |x| {
                let local_arc = rx2.recv().unwrap();
                local_arc.iter().fold(x, |old, &new| old + new)
            },
        );
    });
}

#[bench]
fn bench_spec_wrong_65536(b: &mut Bencher) {
    let v = (0..65536).collect::<Vec<_>>();
    let v_arc = Arc::new(v);

    b.iter(|| {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        tx.send(v_arc.clone()).unwrap();
        tx2.send(v_arc.clone()).unwrap();
        tx2.send(v_arc.clone()).unwrap();

        spec(
            move || {
                let local_arc = rx.recv().unwrap();
                local_arc.iter().fold(0, |old, &new| old + new)
            },
            || 0, // Incorrect result of `fold`
            move |x| {
                let local_arc = rx2.recv().unwrap();
                local_arc.iter().fold(x, |old, &new| old + new)
            },
        );
    });
}
