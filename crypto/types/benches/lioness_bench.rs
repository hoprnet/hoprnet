use cipher::Block;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{
    crypto_traits::{Iv, Key, KeyIvInit},
    lioness::LionessBlake3ChaCha20,
};
use typenum::{U1024, Unsigned};

// Avoid musl's default allocator due to degraded performance
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 100_000;
type BlockSize = U1024;

pub fn lioness_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("lioness_bench");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Bytes(BlockSize::USIZE as u64));

    group.bench_function("lioness_encrypt", |b| {
        let mut k = Key::<LionessBlake3ChaCha20<BlockSize>>::default();
        let mut iv = Iv::<LionessBlake3ChaCha20<BlockSize>>::default();
        hopr_crypto_random::random_fill(&mut k);
        hopr_crypto_random::random_fill(&mut iv);

        let lioness = hopr_crypto_types::lioness::LionessBlake3ChaCha20::<BlockSize>::new(&k, &iv);
        let mut data = Block::<LionessBlake3ChaCha20<BlockSize>>::default();
        hopr_crypto_random::random_fill(&mut data);
        b.iter(|| {
            lioness.encrypt_block((&mut data).into());
        })
    });

    group.bench_function("lioness_decrypt", |b| {
        let mut k = Key::<LionessBlake3ChaCha20<BlockSize>>::default();
        let mut iv = Iv::<LionessBlake3ChaCha20<BlockSize>>::default();
        hopr_crypto_random::random_fill(&mut k);
        hopr_crypto_random::random_fill(&mut iv);

        let lioness = hopr_crypto_types::lioness::LionessBlake3ChaCha20::<BlockSize>::new(&k, &iv);
        let mut data = Block::<LionessBlake3ChaCha20<BlockSize>>::default();
        hopr_crypto_random::random_fill(&mut data);
        b.iter(|| {
            lioness.decrypt_block((&mut data).into());
        })
    });
}

criterion_group!(benches, lioness_bench);
criterion_main!(benches);
