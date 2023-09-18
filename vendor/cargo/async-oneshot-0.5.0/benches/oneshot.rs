use criterion::*;
use futures_micro::or;
use futures_lite::future::block_on;
use async_oneshot::oneshot;

pub fn create_destroy(c: &mut Criterion) {
    c.bench_function(
        "create_destroy",
        |b| b.iter(|| oneshot::<usize>())
    );
}

#[allow(unused_must_use)]
pub fn send(c: &mut Criterion) {
    let mut group = c.benchmark_group("send");
    group.bench_function(
        "success",
        |b| b.iter_batched(
            || oneshot::<usize>(),
            |(mut send, recv)| { (send.send(1).unwrap(), recv) },
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "closed",
        |b| b.iter_batched(
            || oneshot::<usize>().0,
            |mut send| send.send(1).unwrap_err(),
            BatchSize::SmallInput
        )
    );
}

pub fn try_recv(c: &mut Criterion) {
    let mut group = c.benchmark_group("try_recv");
    group.bench_function(
        "success",
        |b| b.iter_batched(
            || {
                let (mut send, recv) = oneshot::<usize>();
                send.send(1).unwrap();
                recv
            },
            |recv| recv.try_recv().unwrap(),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "empty",
        |b| b.iter_batched(
            || oneshot::<usize>(),
            |(send, recv)| (recv.try_recv().unwrap_err(), send),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "closed",
        |b| b.iter_batched(
            || oneshot::<usize>().1,
            |recv| recv.try_recv().unwrap_err(),
            BatchSize::SmallInput
        )
    );
}

pub fn recv(c: &mut Criterion) {
    let mut group = c.benchmark_group("async.recv");
    group.bench_function(
        "success",
        |b| b.iter_batched(
            || {
                let (mut send, recv) = oneshot::<usize>();
                send.send(42).unwrap();
                recv
            },
            |recv| block_on(recv).unwrap(),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "closed",
        |b| b.iter_batched(
            || {
                let (send, recv) = oneshot::<usize>();
                send.close();
                recv
            },
            |recv| block_on(recv).unwrap_err(),
            BatchSize::SmallInput
        )
    );
}

pub fn wait(c: &mut Criterion) {
    let mut group = c.benchmark_group("async.wait");
    group.bench_function(
        "success",
        |b| b.iter_batched(
            || oneshot::<usize>(),
            |(send, recv)| block_on(
                or!(async { recv.await.unwrap(); 1 },
                    async { send.wait().await.unwrap(); 2 }
                )
            ),
            BatchSize::SmallInput
        )
    );
    group.bench_function(
        "closed",
        |b| b.iter_batched(
            || oneshot::<usize>().0,
            |send| block_on(send.wait()).unwrap_err(),
            BatchSize::SmallInput
        )
    );
}

criterion_group!(
    benches
        , create_destroy
        , send
        , try_recv
        , recv
        , wait
);
criterion_main!(benches);
