use css_lex::*;
use speculate_lib::*;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::{cmp, vec};

static LOOKBACK: usize = 10;

/**
 * Collects the loop body results into a 2d vector.
 *
 * Each loop body sends its index and an optional result type. If the message
 * received over the `port` is `None`, then stop listening and push the result
 * over `chan`. If the message received is `Some(i, None)`, then clear the
 * `i`-th result vector. If the message is `Some(i, Some(t))`, then add `t` to
 * the `i`-th result vector.
 */
fn spawn_result_collector<T: Send + Clone + 'static>(
    receiver: mpsc::Receiver<Option<(usize, Option<Vec<T>>)>>,
    sender: mpsc::Sender<Vec<T>>,
    size: usize,
) {
    thread::spawn(move || {
        let mut results = vec![Vec::new(); size];
        while let Some(message) = receiver.recv().unwrap() {
            match message {
                (idx, Some(val)) => results[idx] = val,
                (idx, None) => results[idx].clear(),
                // No need for a None case outside of the loop condition due to `while let`
            }
        }
        let flattened_results = results.into_iter().flatten().collect();
        sender.send(flattened_results).unwrap();
    });
}

/**
 * Find the start of the next token at or after `start`.
 *
 * Backs up `LOOKBACK` characters and begins lexing until reaching or passing
 * `start`.
 *
 * Assumes `input` has already been preprocessed.
 */
pub fn next_token_start(input: Arc<String>, start: usize) -> usize {
    let mut tokenizer = Tokenizer::new(input.clone());
    tokenizer.position = if start < LOOKBACK {
        0
    } else {
        cmp::min(start - LOOKBACK, tokenizer.length)
    };
    while tokenizer.position < start && tokenizer.next().is_some() {}
    tokenizer.position
}

pub fn spec_tokenize(input: String, num_iters: usize) -> (SpecStats, Vec<Node>) {
    let input = preprocess(&input); // Assuming preprocess() adapts to String -> String
    let css_len = input.len();
    let str_arc = Arc::new(input);
    let iter_size: usize = (css_len + num_iters - 1) / num_iters; // round up

    let (tx, rx) = mpsc::channel();
    let (res_tx, res_rx) = mpsc::channel();

    // LOOP_BODY
    let (arc_tx, arc_rx) = mpsc::channel();
    let arc_rx = Arc::new(Mutex::new(arc_rx));
    arc_tx.send(str_arc.clone()).unwrap();
    let body_tx = tx.clone();

    let loop_body = move |idx: usize, token_start: usize| {
        let upper = std::cmp::min((idx + 1) * iter_size, css_len);
        let string = arc_rx.lock().unwrap().recv().unwrap();
        let mut tokenizer = Tokenizer::new(string); // Assuming Tokenizer::new() now takes a string reference
        tokenizer.position = token_start;
        let mut results: Vec<Node> = Vec::with_capacity(10);

        // Example adaptation, assuming functionality of sending to the channel
        body_tx.send(Some((idx, None))).unwrap();
        while tokenizer.position < upper {
            match tokenizer.next() {
                Some(node) => results.push(node),
                None => break,
            }
        }
        body_tx.send(Some((idx, Some(results)))).unwrap();
        tokenizer.position
    };

    // PREDICTOR
    let (predictor_tx, predictor_rx) = mpsc::channel();
    let predictor_rx = Arc::new(Mutex::new(predictor_rx));
    let str_arc_clone = Arc::clone(&str_arc);
    predictor_tx
        .send(str_arc_clone)
        .expect("Failed to send on channel");
    let predictor = move |idx| {
        next_token_start(
            predictor_rx
                .lock()
                .unwrap()
                .recv()
                .expect("Failed to receive from channel"),
            idx * iter_size,
        )
    };

    spawn_result_collector(rx, res_tx, num_iters);
    let res = specfold(num_iters, loop_body, predictor);
    tx.send(None).unwrap();
    (res, res_rx.recv().unwrap())
}