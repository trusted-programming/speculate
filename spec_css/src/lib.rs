use css_lex::*;
use speculate_lib::*;
use std::sync::{mpsc, Arc};
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
    let mut tokenizer = Tokenizer::new(input);
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
    let str_arc = Arc::new(input.clone());

    let iter_size: usize = (css_len + num_iters - 1).div_ceil(num_iters); // round up

    let (tx, rx) = mpsc::channel();
    let (res_tx, res_rx) = mpsc::channel();

    // LOOP_BODY
    let body_tx = tx.clone();

    let loop_body = move |idx: usize, token_start: &usize| {
        let upper = std::cmp::min((idx + 1) * iter_size, css_len);
        let mut tokenizer = Tokenizer::new(Arc::clone(&str_arc));
        tokenizer.position = *token_start;
        let mut results: Vec<Node> = Vec::with_capacity(10);
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
    let str_arc = Arc::new(input);

    // PREDICTOR

    let predictor = move |idx| next_token_start(Arc::clone(&str_arc), idx * iter_size);
    spawn_result_collector(rx, res_tx, num_iters);
    let res = specfold(num_iters, loop_body, predictor);
    tx.send(None).unwrap();
    (res, res_rx.recv().unwrap())
}
