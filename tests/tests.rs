use speculate::*;
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

#[test]
fn test_spec() {
    assert!(spec(|| 2 + 2, || 4, |x| x + 2) == 6);
    assert!(spec(|| 2 + 2, || 1, |x| x + 2) == 6);
}

/// Spawns a thread to collect results sent over a channel.
///
/// # Arguments
///
/// * `receiver` - The receiving end of a channel from which to receive results.
/// * `sender` - The sending end of a channel to which collected results will be sent.
/// * `size` - The expected number of results to collect.
///
/// The function expects the `receiver` to receive `Option<(usize, T)>` messages,
/// where `Some((idx, val))` indicates a result `val` at index `idx`, and `None` indicates
/// that no more results will be sent.
fn spawn_result_collector<T: 'static + Send + Default + Clone>(
    receiver: mpsc::Receiver<Option<(usize, T)>>,
    sender: mpsc::Sender<Vec<T>>,
    size: usize,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut results = vec![T::default(); size];
        for received in receiver {
            match received {
                Some((idx, val)) => results[idx] = val,
                None => break,
            }
        }
        let _res = sender.send(results);
    })
}

#[test]
fn test_specfold_correct_prediction() {
    let (tx, rx) = mpsc::channel::<Option<(usize, isize)>>();
    let (res_tx, res_rx) = mpsc::channel::<Vec<isize>>();
    let tx_clone = tx.clone();
    // Spawn the result collector thread

    let loop_body = move |idx: usize, val: isize| -> isize {
        let res = idx as isize + val;
        tx.send(Some((idx, res))).unwrap();
        res
    };
    let loop_results = vec![0, 0, 1, 3, 6];
    let predictor = move |idx: usize| loop_results[idx];

    spawn_result_collector(rx, res_tx, 5);
    specfold(5, loop_body, predictor);
    tx_clone.send(None).unwrap();
    let expected_results = vec![0, 1, 3, 6, 10];
    assert!(res_rx.recv().unwrap() == expected_results);
}

#[test]
fn test_specfold_incorrect_prediction() {
    let (tx, rx) = mpsc::channel::<Option<(usize, isize)>>();
    let (res_tx, res_rx) = mpsc::channel::<Vec<isize>>();
    let tx_clone = tx.clone();

    let loop_body = move |idx: usize, val: isize| -> isize {
        let res = idx as isize + val + 5;
        tx.send(Some((idx, res))).unwrap();
        res
    };

    let predictor = move |_| 0;
    spawn_result_collector(rx, res_tx, 1);
    specfold(1, loop_body, predictor);
    tx_clone.send(None).unwrap();
    let expected_results = vec![5];
    assert!(res_rx.recv().unwrap() == expected_results);
}

#[test]
fn test_specfold_no_tasks() {
    let (tx, rx) = mpsc::channel::<Option<(usize, isize)>>();
    let (res_tx, res_rx) = mpsc::channel::<Vec<isize>>();
    let tx_clone = tx.clone();

    let loop_body = move |idx: usize, val: isize| -> isize {
        let res = idx as isize + val + 5;
        tx.send(Some((idx, res))).unwrap();
        res
    };

    let predictor = move |_| 0;
    spawn_result_collector(rx, res_tx, 0);
    specfold(0, loop_body, predictor);
    tx_clone.send(None).unwrap();
    let expected_results = vec![];
    assert!(res_rx.recv().unwrap() == expected_results);
}
