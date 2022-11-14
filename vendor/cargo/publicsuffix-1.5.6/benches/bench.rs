use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let list = publicsuffix::List::fetch().unwrap();
    c.bench_function("bench raw.github.com", |b| {
        b.iter(|| list.parse_domain(black_box("raw.github.com")).unwrap())
    });
    c.bench_function("bench www.city.yamanashi.yamanashi.jp", |b| {
        b.iter(|| {
            list.parse_domain(black_box("www.city.yamanashi.yamanashi.jp"))
                .unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
