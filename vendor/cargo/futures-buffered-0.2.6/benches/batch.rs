use divan::{AllocProfiler, Divan};
use std::time::Duration;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    Divan::from_args().main();
}

async fn sleep() {
    tokio::time::sleep(Duration::from_micros(100)).await
}

#[divan::bench_group]
mod futures_unordered {
    use futures_buffered::FuturesUnorderedBounded;
    use futures_util::{stream::FuturesUnordered, StreamExt};

    use crate::sleep;

    const SIZES: [usize; 3] = [16, 64, 256];

    #[divan::bench(args = SIZES)]
    fn futures(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut queue = FuturesUnordered::new();

        let total = n * n;
        for _ in 0..n {
            queue.push(sleep())
        }
        for _ in n..total {
            runtime.block_on(queue.next());
            queue.push(sleep())
        }
        for _ in 0..n {
            runtime.block_on(queue.next());
        }
    }

    #[divan::bench(args = SIZES)]
    fn futures_buffered(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut queue = FuturesUnorderedBounded::new(n);

        let total = n * n;
        for _ in 0..n {
            queue.push(sleep())
        }
        for _ in n..total {
            runtime.block_on(queue.next());
            queue.push(sleep())
        }
        for _ in 0..n {
            runtime.block_on(queue.next());
        }
    }
}

#[divan::bench_group]
mod buffer_unordered {
    use futures_buffered::BufferedStreamExt;
    use futures_util::{stream, StreamExt};

    use crate::sleep;

    const SIZES: [usize; 3] = [16, 64, 256];

    #[divan::bench(args = SIZES)]
    fn futures(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let total = n * n;
        let mut s = stream::iter((0..total).map(|_| sleep())).buffer_unordered(n);
        while runtime.block_on(s.next()).is_some() {}
    }

    #[divan::bench(args = SIZES)]
    fn futures_buffered(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let total = n * n;
        let mut s = stream::iter((0..total).map(|_| sleep())).buffered_unordered(n);
        while runtime.block_on(s.next()).is_some() {}
    }
}

#[divan::bench_group]
mod buffer_ordered {
    use futures_buffered::BufferedStreamExt;
    use futures_util::{stream, StreamExt};

    use crate::sleep;

    const SIZES: [usize; 3] = [16, 64, 256];

    #[divan::bench(args = SIZES)]
    fn futures(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let total = n * n;
        let mut s = stream::iter((0..total).map(|_| sleep())).buffered(n);
        while runtime.block_on(s.next()).is_some() {}
    }

    #[divan::bench(args = SIZES)]
    fn futures_buffered(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let total = n * n;
        let mut s = stream::iter((0..total).map(|_| sleep())).buffered_ordered(n);
        while runtime.block_on(s.next()).is_some() {}
    }
}

#[divan::bench_group]
mod join {
    use crate::sleep;

    const SIZES: [usize; 4] = [16, 64, 256, 1024];

    #[divan::bench(args = SIZES)]
    fn futures(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let futs = (0..n * 8).map(|_| sleep());
        runtime.block_on(futures::future::join_all(futs));
    }

    #[divan::bench(args = SIZES)]
    fn futures_buffered(n: usize) {
        // setup a tokio runtime for our tests
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let futs = (0..n * 8).map(|_| sleep());
        runtime.block_on(futures_buffered::join_all(futs));
    }
}
