use cipher::Block;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_types::{crypto_traits::KeyIvInit, lioness::LionessBlake3ChaCha20};
use typenum::{U1022, Unsigned};

// Avoid musl's default allocator due to degraded performance
// Use mimalloc with secure feature for better performance
#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const SAMPLE_SIZE: usize = 100_000;
type BlockSize = U1022;

pub fn lioness_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("lioness_bench");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Bytes(BlockSize::USIZE as u64));

    group.bench_function("lioness_encrypt", |b| {
        let (k, iv) =
            hopr_crypto_types::lioness::LionessBlake3ChaCha20::<BlockSize>::generate_key_iv(hopr_crypto_random::rng());
        let lioness = hopr_crypto_types::lioness::LionessBlake3ChaCha20::<BlockSize>::new(&k, &iv);
        let mut data = Block::<LionessBlake3ChaCha20<BlockSize>>::default();
        hopr_crypto_random::random_fill(&mut data);
        b.iter(|| {
            lioness.encrypt_block((&mut data).into());
        })
    });

    group.bench_function("lioness_decrypt", |b| {
        let (k, iv) =
            hopr_crypto_types::lioness::LionessBlake3ChaCha20::<BlockSize>::generate_key_iv(hopr_crypto_random::rng());
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
