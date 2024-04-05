use std::thread;

#[derive(Debug)]
pub struct SpecStats {
    pub iters: usize,
    pub mispredictions: Vec<bool>,
}
//    let threads = thread::available_parallelism().unwrap().get() as f64;

/**
 * Speculatively execute consumer using the guessed value.
 */
pub fn spec<A: Eq + Send + Clone + 'static, B: Send + 'static>(
    producer: impl Fn() -> A + Send + 'static,
    predictor: impl Fn() -> A + Send + 'static,
    consumer: impl Fn(A) -> B + Send + 'static,
) -> B {
    let producer_result = thread::spawn(producer);
    let prediction = predictor();

    let speculative_result = consumer(prediction.clone());
    let real_value = producer_result.join().unwrap();

    if real_value == prediction {
        speculative_result
    } else {
        consumer(real_value)
    }
}

/**
 * Iteratively execute `loop_body` by guessing a value.
 *
 * the &fn() would close over the Arc, and then it would .clone it for each new
 * dyn Fn
 */
pub fn specfold<A: Eq + Clone + Send + 'static>(
    iters: usize,
    loop_body: impl Fn(usize, A) -> A + Send + Clone + 'static,
    predictor: impl Fn(usize) -> A + Send + Clone + 'static,
) -> SpecStats {
    let mut results = Vec::with_capacity(iters);

    let mut stats = SpecStats {
        iters,
        mispredictions: vec![false; iters],
    };

    for i in 0..iters {
        let loop_body_clone = loop_body.clone();
        let predictor_clone = predictor.clone();

        let thread = thread::spawn(move || {
            let prediction = predictor_clone(i);
            let res = loop_body_clone(i, prediction.clone());
            (prediction, res)
        });
        results.push(thread);
    }

    let mut previous: Option<A> = None;
    for (i, handle) in results.into_iter().enumerate() {
        let (prediction, _) = handle.join().unwrap();
        if let Some(prev) = &previous {
            if *prev != prediction {
                stats.mispredictions[i] = true;
                let res = loop_body(i, prev.clone());
                previous = Some(res);
            }
        }
    }
    stats
}
