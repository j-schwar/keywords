use criterion::{Criterion, criterion_group, criterion_main};
use keywords::KeywordMap;
use std::hint::black_box;

const TEXT: &str = include_str!("lorem_ipsum.txt");

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("keyword_map insert", |b| {
        b.iter(|| {
            let mut map = KeywordMap::new();
            for word in TEXT.split_whitespace() {
                map.insert(word, black_box(1));
            }
        })
    });

    c.bench_function("keyword_map find_by_partial_keyword", |b| {
        let mut map = KeywordMap::new();
        for word in TEXT.split_whitespace() {
            map.insert(word, black_box(1));
        }

        b.iter(|| {
            let _ = map.find_by_partial_keyword(black_box("temp"));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
