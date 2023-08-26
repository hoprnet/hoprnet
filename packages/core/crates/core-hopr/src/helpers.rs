use futures::stream::FuturesUnordered;

/// Add multiple futures into a FuturesUnordered object to setup concurrent execution of futures.
pub(super) fn to_futures_unordered<F>(mut fs: Vec<F>) -> FuturesUnordered<F> {
    let futures: FuturesUnordered<F> = FuturesUnordered::new();
    for f in fs.drain(..) {
        futures.push(f);
    }

    futures
}
