#![allow(clippy::type_complexity)]

mod writer;

pub(crate) use writer::{IdState, NodeState, Writer};

use std::error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::result;

/// Write error.
#[derive(Debug)]
pub struct Error(pub(crate) String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for Error {}

/// Write result.
pub type Result = result::Result<(), Error>;

pub struct WriterBuilder<'a, T> {
    node: &'a T,
    class_id: u32,
    header_chunks: Vec<(u32, fn(&T, Writer<&mut Vec<u8>, &mut IdState>) -> Result)>,
    body: fn(&T, &mut Writer<&mut Vec<u8>, IdState, &mut NodeState>) -> Result,
}

impl<'a, T> WriterBuilder<'a, T> {
    pub(crate) fn new(
        node: &'a T,
        class_id: u32,
        header_chunks: Vec<(u32, fn(&T, Writer<&mut Vec<u8>, &mut IdState>) -> Result)>,
        body: fn(&T, &mut Writer<&mut Vec<u8>, IdState, &mut NodeState>) -> Result,
    ) -> Self {
        Self {
            node,
            class_id,
            header_chunks,
            body,
        }
    }

    pub fn write_to<W>(self, writer: W) -> Result
    where
        W: Write,
    {
        let mut user_data = vec![];
        {
            let mut w = Writer::new(&mut user_data);
            let mut id_state = IdState::new();
            let mut chunks = vec![];

            for (chunk_id, write_fn) in self.header_chunks {
                let mut chunk = vec![];
                let w = Writer::with_id_state(&mut chunk, &mut id_state);
                write_fn(self.node, w)?;
                chunks.push((chunk_id, chunk));
            }

            w.u32(chunks.len() as u32)?;

            for (chunk_id, chunk) in &chunks {
                w.u32(*chunk_id)?;
                if chunk.len() <= u8::MAX as usize {
                    w.u32(chunk.len() as u32)?;
                } else {
                    w.u32(chunk.len() as u32 | 0x80000000)?;
                }
            }

            for (_, chunk) in chunks {
                w.bytes(&chunk)?;
            }
        }

        let mut body = vec![];
        let mut node_state = NodeState::new();
        {
            let mut w = Writer::with_id_and_node_state(&mut body, IdState::new(), &mut node_state);
            (self.body)(self.node, &mut w)?;

            w.u32(0xFACADE01)?;
        }

        let mut output = vec![0; lzo1x::worst_compress(body.len())];
        let compressed_body = lzo1x::compress_to_slice(&body, &mut output);

        let mut w = Writer::new(writer);

        w.bytes(b"GBX")?;
        w.u16(6)?;
        w.u8(b'B')?;
        w.u8(b'U')?;
        w.u8(b'C')?;
        w.u8(b'R')?;
        w.u32(self.class_id)?;
        w.u32(user_data.len() as u32)?;
        w.bytes(&user_data)?;
        w.u32(node_state.num_nodes())?;
        w.u32(0)?;
        w.u32(body.len() as u32)?;
        w.u32(compressed_body.len() as u32)?;
        w.bytes(compressed_body)?;

        Ok(())
    }

    pub fn write_to_file<P>(self, path: P) -> Result
    where
        P: AsRef<Path>,
    {
        let file = File::create(path).map_err(|err| Error(format!("{err}")))?;
        let writer = BufWriter::new(file);
        self.write_to(writer)
    }
}
