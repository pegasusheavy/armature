#![allow(clippy::all)]
#![allow(deprecated)]
#![allow(clippy::needless_question_mark)]
#![allow(unused_imports)]

//! Storage benchmarks for armature-storage

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

use armature_storage::{FileInfo, FileValidator, LocalStorage, LocalStorageConfig, UploadedFile};
use bytes::Bytes;

fn file_validation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_validation");

    // Create a validator with common rules
    let validator = FileValidator::new()
        .max_size(10 * 1024 * 1024) // 10 MB
        .allowed_types(&["image/jpeg", "image/png", "image/gif", "application/pdf"])
        .allowed_extensions(&["jpg", "jpeg", "png", "gif", "pdf"]);

    // Create test files of different sizes
    let small_file = UploadedFile::from_bytes(vec![0u8; 1024], "test.jpg");
    let medium_file = UploadedFile::from_bytes(vec![0u8; 100 * 1024], "test.png");
    let large_file = UploadedFile::from_bytes(vec![0u8; 1024 * 1024], "test.pdf");

    group.bench_function("validate_small_file", |b| {
        let file = small_file.clone();
        b.iter(|| {
            let result = validator.validate(black_box(&file));
            black_box(result)
        });
    });

    group.bench_function("validate_medium_file", |b| {
        let file = medium_file.clone();
        b.iter(|| {
            let result = validator.validate(black_box(&file));
            black_box(result)
        });
    });

    group.bench_function("validate_large_file", |b| {
        let file = large_file.clone();
        b.iter(|| {
            let result = validator.validate(black_box(&file));
            black_box(result)
        });
    });

    group.finish();
}

fn uploaded_file_creation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("uploaded_file_creation");

    group.bench_function("from_bytes_small", |b| {
        let data = vec![0u8; 1024];
        b.iter(|| {
            let file = UploadedFile::from_bytes(black_box(data.clone()), "test.jpg");
            black_box(file)
        });
    });

    group.bench_function("from_bytes_medium", |b| {
        let data = vec![0u8; 100 * 1024];
        b.iter(|| {
            let file = UploadedFile::from_bytes(black_box(data.clone()), "test.png");
            black_box(file)
        });
    });

    group.bench_function("from_bytes_large", |b| {
        let data = vec![0u8; 1024 * 1024];
        b.iter(|| {
            let file = UploadedFile::from_bytes(black_box(data.clone()), "document.pdf");
            black_box(file)
        });
    });

    group.finish();
}

fn file_info_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_info");

    let image_file = UploadedFile::from_bytes(vec![0u8; 1024], "photo.jpg");
    let video_file = UploadedFile::from_bytes(vec![0u8; 1024], "video.mp4");
    let text_file = UploadedFile::from_bytes(vec![0u8; 1024], "document.txt");

    group.bench_function("is_image", |b| {
        let file = image_file.clone();
        b.iter(|| {
            let result = file.info.is_image();
            black_box(result)
        });
    });

    group.bench_function("is_video", |b| {
        let file = video_file.clone();
        b.iter(|| {
            let result = file.info.is_video();
            black_box(result)
        });
    });

    group.bench_function("is_text", |b| {
        let file = text_file.clone();
        b.iter(|| {
            let result = file.info.is_text();
            black_box(result)
        });
    });

    group.bench_function("extension_lowercase", |b| {
        let file = image_file.clone();
        b.iter(|| {
            let result = file.info.extension_lowercase();
            black_box(result)
        });
    });

    group.finish();
}

fn validator_builder_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("validator_builder");

    group.bench_function("create_simple", |b| {
        b.iter(|| {
            let validator = FileValidator::new().max_size(10 * 1024 * 1024);
            black_box(validator)
        });
    });

    group.bench_function("create_complex", |b| {
        b.iter(|| {
            let validator = FileValidator::new()
                .max_size(10 * 1024 * 1024)
                .allowed_types(&["image/jpeg", "image/png", "image/gif", "image/webp"])
                .allowed_extensions(&["jpg", "jpeg", "png", "gif", "webp"]);
            black_box(validator)
        });
    });

    group.finish();
}

fn file_size_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_sizes");

    let sizes = [
        ("1KB", 1024usize),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("create_file", name), &size, |b, &size| {
            let data = vec![0u8; size];
            b.iter(|| {
                let file = UploadedFile::from_bytes(black_box(data.clone()), "test.bin");
                black_box(file)
            });
        });
    }

    group.finish();
}

fn bytes_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_operations");

    let file = UploadedFile::from_bytes(vec![0u8; 100 * 1024], "test.bin");

    group.bench_function("data_access", |b| {
        b.iter(|| {
            let data = file.data();
            black_box(data)
        });
    });

    group.bench_function("size", |b| {
        b.iter(|| {
            let size = file.size();
            black_box(size)
        });
    });

    group.bench_function("is_empty", |b| {
        b.iter(|| {
            let empty = file.is_empty();
            black_box(empty)
        });
    });

    group.bench_function("name", |b| {
        b.iter(|| {
            let name = file.name();
            black_box(name)
        });
    });

    group.bench_function("content_type", |b| {
        b.iter(|| {
            let ct = file.content_type();
            black_box(ct)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    file_validation_benchmark,
    uploaded_file_creation_benchmark,
    file_info_benchmark,
    validator_builder_benchmark,
    file_size_benchmark,
    bytes_operations_benchmark,
);

criterion_main!(benches);
