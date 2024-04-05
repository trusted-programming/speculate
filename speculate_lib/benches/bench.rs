use criterion::{black_box, criterion_group, criterion_main, Criterion};
use speculate_lib::*;
use std::sync::{mpsc, Arc};

fn bench_2048(c: &mut Criterion) {
    let v: Vec<usize> = (0..2048).collect();
    let v_arc = Arc::new(v.clone());

    c.bench_function("2048 direct", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations regarding these computations
            let val = v.iter().fold(0, |old, new| old + *new);
            black_box(v.iter().fold(val, |old, new| old + *new));
        })
    });
    c.bench_function("2048 speculate correct", |b| {
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
    });
    c.bench_function("2048 speculate wrong", |b| {
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
    });
}

fn bench_65536(c: &mut Criterion) {
    let v: Vec<usize> = (0..65536).collect();
    let v_arc = Arc::new(v.clone());

    c.bench_function("65536 direct", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations regarding these computations
            let val = v.iter().fold(0, |old, new| old + *new);
            black_box(v.iter().fold(val, |old, new| old + *new));
        });
    });

    c.bench_function("65536 speculate correct", |b| {
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
    });

    c.bench_function("65536 speculate wrong", |b| {
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
    });
}

criterion_group!(b_2048, bench_2048);
criterion_group!(b_65536, bench_65536);
criterion_main!(b_2048, b_65536);
