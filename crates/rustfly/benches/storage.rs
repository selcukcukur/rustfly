use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rustfly::{InMemoryAdapter, NativeAdapter, RustflyAdapter};
use std::hint::black_box;
use tempfile::tempdir;
use tokio::runtime::Builder;

static NEXT_PATH: AtomicUsize = AtomicUsize::new(0);

fn next_path(prefix: &str) -> String {
    let id = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}/{id}.txt")
}

fn bench_inmemory_sync(c: &mut Criterion) {
    c.bench_function("inmemory_sync_write_read", |bench| {
        bench.iter_batched(
            InMemoryAdapter::new,
            |adapter| {
                let path = next_path("sync");
                adapter
                    .write_sync(&path, Bytes::from_static(b"benchmark"))
                    .unwrap();
                black_box(adapter.read_sync(&path).unwrap());
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_inmemory_async(c: &mut Criterion) {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("inmemory_async_write_read", |bench| {
        bench.to_async(&runtime).iter_batched(
            InMemoryAdapter::new,
            |adapter| async move {
                let path = next_path("async");
                adapter
                    .write(&path, Bytes::from_static(b"benchmark"))
                    .await
                    .unwrap();
                black_box(adapter.read(&path).await.unwrap());
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_native_sync(c: &mut Criterion) {
    c.bench_function("native_sync_write_read", |bench| {
        bench.iter_batched(
            || {
                let dir = tempdir().unwrap();
                let adapter = NativeAdapter::new(dir.path());
                (adapter, dir)
            },
            |(adapter, _dir)| {
                let path = next_path("sync");
                adapter
                    .write_sync(&path, Bytes::from_static(b"benchmark"))
                    .unwrap();
                black_box(adapter.read_sync(&path).unwrap());
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_native_async(c: &mut Criterion) {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("native_async_write_read", |bench| {
        bench.to_async(&runtime).iter_batched(
            || {
                let dir = tempdir().unwrap();
                let adapter = NativeAdapter::new(dir.path());
                (adapter, dir)
            },
            |(adapter, _dir)| async move {
                let path = next_path("async");
                adapter
                    .write(&path, Bytes::from_static(b"benchmark"))
                    .await
                    .unwrap();
                black_box(adapter.read(&path).await.unwrap());
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_inmemory_list(c: &mut Criterion) {
    c.bench_function("inmemory_list_100_files", |bench| {
        bench.iter_batched(
            || {
                let adapter = InMemoryAdapter::new();
                for index in 0..100 {
                    adapter
                        .write_sync(
                            &format!("files/{index}.txt"),
                            Bytes::from_static(b"benchmark"),
                        )
                        .unwrap();
                }
                adapter
            },
            |adapter| {
                black_box(adapter.list_sync("files").unwrap());
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    storage,
    bench_inmemory_sync,
    bench_inmemory_async,
    bench_native_sync,
    bench_native_async,
    bench_inmemory_list
);
criterion_main!(storage);
