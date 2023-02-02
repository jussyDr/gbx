use anyhow::{anyhow, Result};
use base64::Engine;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::Path;

pub fn fetch_file(url: &str, hash_base64: &str, dir: &str) -> Result<File> {
    let path = Path::new(dir).join(hash_base64);

    let file = if path.exists() {
        let mut file = File::open(&path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

        if hash != hash_base64 {
            drop(file);
            fs::remove_file(path)?;

            return Err(anyhow!("incorrect file hash: {}", hash));
        }

        file.rewind()?;
        file
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
