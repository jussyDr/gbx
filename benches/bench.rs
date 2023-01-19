use anyhow::{anyhow, Result};
use base64::Engine;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gbx::{Block, Map};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Seek, Write};
use std::path::Path;

fn fetch_file(url: &str, hash_base64: &str) -> Result<File> {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(hash_base64);

    let file = if path.try_exists()? {
        File::open(path)?
    } else {
        let bytes = reqwest::blocking::Client::builder()
            .user_agent("gbx-rs")
            .build()?
            .get(url)
            .send()?
            .bytes()?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

        if hash != hash_base64 {
            return Err(anyhow!("incorrect file hash: {}", hash));
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        file.write_all(&bytes)?;
        file.rewind()?;

        file
    };

    Ok(file)
}

fn bench(c: &mut Criterion) {
    let block_id = 44867;
    let url = format!("https://item.exchange/item/download/{block_id}");
    let file = fetch_file(&url, "U6JKwKAv62gS_KLHuJpaSc0Ri5mHvbitGodiceC-5qI").unwrap();
    let mut reader = BufReader::new(file);

    c.bench_function(&format!("read block {block_id}"), |b| {
        b.iter_with_large_drop(|| {
            reader.rewind().unwrap();
            black_box(Block::read_from(&mut reader).unwrap());
        })
    });

    let map_id = 31080;
    let url = format!("https://trackmania.exchange/maps/download/{map_id}");
    let file = fetch_file(&url, "QkvIruZgolwog5meQDgd3xqFEuZmLXWXG_n68YjPh5M").unwrap();
    let mut reader = BufReader::new(file);

    c.bench_function(&format!("read map {map_id}"), |b| {
        b.iter_with_large_drop(|| {
            reader.rewind().unwrap();
            black_box(Map::read_from(&mut reader).unwrap());
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
