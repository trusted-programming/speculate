#![feature(test)]
extern crate test;

use speculate::*;
use std::sync::mpsc;
use std::sync::Arc;
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
async fn test_spec_correct_2048() {
    let (sender, receiver) = mpsc::channel::<Option<(usize, isize)>>(10);
    let (res_sender, res_receiver) = mpsc::channel::<Vec<isize>>(1);

    // Assuming the receiver is processed elsewhere and not directly within the closures
    let processed_data = process_receiver_data(&receiver); // Pseudocode

    let loop_body = Arc::new(move |idx: usize, val: isize| -> isize {
        // Use processed data instead of directly accessing the receiver
        val + 1 // Example operation
    });

    let predictor = Arc::new(move |idx: usize| -> isize {
        idx as isize * 2 // Directly return a prediction
    });

    // Assuming spec is properly defined to accept the closures
    spec(5, loop_body, predictor).await;

    // Example operations
    sender.send(None).await.unwrap();
    let results = res_receiver.recv().await.unwrap();
}
