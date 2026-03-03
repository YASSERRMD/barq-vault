use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use barq_compress::{compress, decompress, Codec};

fn bench_compress_lz4(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress/lz4");
    for size in [1_024usize, 10_240, 102_400, 1_048_576] {
        let data: Vec<u8> = (b'a'..=b'z').cycle().take(size).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, d| {
            b.iter(|| compress(black_box(d), Codec::Lz4).unwrap())
        });
    }
    group.finish();
}

fn bench_compress_zstd(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress/zstd");
    for size in [1_024usize, 10_240, 102_400] {
        let data: Vec<u8> = (b'a'..=b'z').cycle().take(size).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, d| {
            b.iter(|| compress(black_box(d), Codec::Zstd(3)).unwrap())
        });
    }
    group.finish();
}

fn bench_compress_lzma(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress/lzma");
    group.sample_size(20); // LZMA is slow — reduce samples
    for size in [1_024usize, 10_240] {
        let data: Vec<u8> = (b'a'..=b'z').cycle().take(size).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, d| {
            b.iter(|| compress(black_box(d), Codec::Lzma(3)).unwrap())
        });
    }
    group.finish();
}

fn bench_decompress_lz4(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress/lz4");
    for size in [1_024usize, 10_240, 102_400] {
        let data: Vec<u8> = (b'a'..=b'z').cycle().take(size).collect();
        let compressed = compress(&data, Codec::Lz4).unwrap();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &compressed, |b, c| {
            b.iter(|| decompress(black_box(c), Codec::Lz4, size).unwrap())
        });
    }
    group.finish();
}

fn bench_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress/ratio");
    let data: Vec<u8> = b"the quick brown fox jumps over the lazy dog. ".repeat(2000);
    let codecs = [("lz4", Codec::Lz4), ("zstd3", Codec::Zstd(3)), ("lzma3", Codec::Lzma(3))];
    for (name, codec) in codecs {
        group.bench_function(name, |b| {
            b.iter(|| compress(black_box(&data), codec.clone()).unwrap())
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_compress_lz4,
    bench_compress_zstd,
    bench_compress_lzma,
    bench_decompress_lz4,
    bench_compression_ratio,
);
criterion_main!(benches);
