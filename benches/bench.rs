use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gbx::{Block, Map};
use std::io::Read;

fn bench(c: &mut Criterion) {
    let block_id = 50037;
    let url = format!("https://item.exchange/item/download/{block_id}");
    let mut file = test_util::fetch_file(
        &url,
        "fCmY6tchDnNmRIxQypzgEuOCzc5s4Avy0MjGZ6YsgtA",
        env!("CARGO_TARGET_TMPDIR"),
    )
    .unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();

    c.bench_function(&format!("read block {block_id}"), |b| {
        b.iter_with_large_drop(|| {
            black_box(Block::reader().read_from(buf.as_slice()).ok());
        })
    });

    let map_id = 46951;
    let url = format!("https://trackmania.exchange/maps/download/{map_id}");
    let mut file = test_util::fetch_file(
        &url,
        "RZY3fk02zYm2UrTSQx3xdl1omZ7GC1c5rY4CzD5WvXs",
        env!("CARGO_TARGET_TMPDIR"),
    )
    .unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();

    c.bench_function(&format!("read map {map_id}"), |b| {
        b.iter_with_large_drop(|| {
            black_box(Map::reader().read_from(buf.as_slice()).ok());
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
