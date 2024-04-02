use std::sync::Arc;
use tokio::task; // Ensure tokio's async runtime is used.

#[derive(Debug)]
pub struct SpecStats {
    pub iters: usize,
    pub mispredictions: Vec<bool>,
}

/**
 * Speculatively execute consumer using the guessed value.
 */
pub async fn spec<A, B>(
    producer: impl Fn() -> A + Send + 'static,
    predictor: impl Fn() -> A + Send + 'static,
    consumer: impl Fn(A) -> B + Send + 'static,
) -> B
where
    A: Eq + Send + Clone + 'static,
    B: Send + 'static,
{
    let producer_result = task::spawn(async move { producer() });
    let prediction = predictor();

    // TODO: might spawn a task here as well
    let speculative_result = consumer(prediction.clone());
    let real_value = producer_result
        .await
        .expect("Failed to await producer result");

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
pub async fn specfold<A: Eq + Clone + Send + 'static>(
    iters: usize,
    loop_body: Arc<dyn Fn(usize, A) -> A + Send + Sync>,
    predictor: Arc<dyn Fn(usize) -> A + Send + Sync>,
) -> SpecStats {
    let mut results = Vec::with_capacity(iters);

    let mut stats = SpecStats {
        iters,
        mispredictions: vec![false; iters],
    };

    for i in 0..iters {
        let loop_body_clone = loop_body.clone();
        let predictor_clone = predictor.clone();

        let fut = task::spawn(async move {
            let prediction = predictor_clone(i);
            let res = loop_body_clone(i, prediction.clone());
            (prediction, res)
        });
        results.push(fut);
    }

    let mut previous: Option<A> = None;
    for (i, handle) in results.iter_mut().enumerate() {
        if let Ok((prediction, result)) = handle.await {
            if let Some(prev) = &previous {
                if *prev != prediction {
                    stats.mispredictions[i] = true;
                }
            }
            previous = Some(result);
        }
    }

    stats
}
