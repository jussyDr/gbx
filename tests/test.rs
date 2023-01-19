use anyhow::{anyhow, Result};
use base64::Engine;
use gbx::{Block, Map};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Seek, Write};
use std::path::Path;

fn fetch_mania_exchange_file(url: &str, hash_base64: &str) -> Result<File> {
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

#[test]
fn read_block() -> Result<()> {
    for line in include_str!("blocks.txt").lines() {
        if let [block_id, hash] = line.split_whitespace().collect::<Vec<_>>()[..] {
            println!("read block {block_id}");

            let url = format!("https://item.exchange/item/download/{block_id}");
            let file = fetch_mania_exchange_file(&url, hash)?;
            let reader = BufReader::new(file);

            Block::read_from(reader).unwrap();
        }
    }

    Ok(())
}

#[test]
fn read_map() -> Result<()> {
    for line in include_str!("maps.txt").lines() {
        if let [map_id, hash] = line.split_whitespace().collect::<Vec<_>>()[..] {
            println!("read map {map_id}");

            let url = format!("https://trackmania.exchange/maps/download/{map_id}");
            let file = fetch_mania_exchange_file(&url, hash)?;
            let reader = BufReader::new(file);

            Map::read_from(reader).unwrap();
        }
    }

    Ok(())
}
