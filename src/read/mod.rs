#![allow(clippy::type_complexity)]

mod reader;

pub(crate) use reader::{IdState, NodeState, Reader};

use std::error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;
use std::result;

/// Read error.
#[derive(Debug)]
pub struct Error(pub(crate) String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for Error {}

/// Read result.
pub type Result<T> = result::Result<T, Error>;

pub enum ReadBodyChunk<T, R, I, N> {
    Read(fn(&mut T, &mut Reader<R, I, N>) -> Result<()>),
    Skip,
    ReadSkippable(fn(&mut T, &mut Reader<R, I, N>) -> Result<()>),
}

pub struct ReaderBuilder<T> {
    read_user_data: bool,
    read_body: bool,

    default: fn() -> T,
    class_id: u32,
    header_chunks: Vec<(
        u32,
        fn(&mut T, &mut Reader<&[u8], &mut IdState>) -> Result<()>,
    )>,
    body_chunks: Vec<(u32, ReadBodyChunk<T, Cursor<Vec<u8>>, IdState, NodeState>)>,
}

impl<T> ReaderBuilder<T> {
    pub(crate) fn new(
        default: fn() -> T,
        class_id: u32,
        header_chunks: Vec<(
            u32,
            fn(&mut T, &mut Reader<&[u8], &mut IdState>) -> Result<()>,
        )>,
        body_chunks: Vec<(u32, ReadBodyChunk<T, Cursor<Vec<u8>>, IdState, NodeState>)>,
    ) -> Self {
        Self {
            read_user_data: true,
            read_body: true,
            default,
            class_id,
            header_chunks,
            body_chunks,
        }
    }

    pub fn user_data(mut self, read_user_data: bool) -> Self {
        self.read_user_data = read_user_data;
        self
    }

    pub fn body(mut self, read_body: bool) -> Self {
        self.read_body = read_body;
        self
    }

    pub fn read_from<R>(self, reader: R) -> Result<T>
    where
        R: Read,
    {
        let mut node = (self.default)();

        let mut r = Reader::new(reader);

        if r.bytes(3)? != b"GBX" {
            return Err(Error(String::from("bad magic")));
        }

        if r.u16()? != 6 {
            return Err(Error(String::from("version not supported")));
        }

        match r.u8()? {
            b'B' => {}
            b'T' => return Err(Error(String::from("text format not supported"))),
            _ => return Err(Error(String::from("bad format"))),
        }

        match r.u8()? {
            b'U' => {}
            b'C' => return Err(Error(String::from("compressed ref table not supported"))),
            _ => return Err(Error(String::from("bad compression"))),
        }

        let body_compressed = match r.u8()? {
            b'C' => true,
            b'U' => false,
            _ => return Err(Error(String::from("bad compression"))),
        };

        if r.u8()? != b'R' {
            return Err(Error(String::from("bad unknown byte")));
        }

        if r.u32()? != self.class_id {
            return Err(Error(String::from("unexpected node class")));
        }

        let user_data_size = r.u32()?;

        if user_data_size > 0 {
            let user_data = r.bytes(user_data_size as usize)?;

            if self.read_user_data {
                let mut r = Reader::new(user_data.as_slice());

                let user_data_chunks = r.list(|r| {
                    let chunk_id = r.u32()?;
                    let size = r.u32()? & 0x7FFFFFFF;

                    Ok((chunk_id, size))
                })?;

                let mut header_chunks = self.header_chunks.into_iter();
                let mut id_state = IdState::new();

                for (chunk_id, size) in user_data_chunks {
                    let (_, read_fn) = header_chunks.find(|(id, _)| *id == chunk_id).unwrap();

                    let bytes = r.bytes(size as usize)?;
                    let mut r = Reader::with_id_state(bytes.as_slice(), &mut id_state);

                    read_fn(&mut node, &mut r)?;
                }
            }
        }

        let num_nodes = r.u32()?;
        let num_node_refs = r.u32()?;

        if num_node_refs > 0 {
            todo!()
        }

        if self.read_body {
            if body_compressed {
                let body_size = r.u32()?;
                let compressed_body_size = r.u32()?;
                let compressed_body = r.bytes(compressed_body_size as usize)?;
                let mut body = vec![0; body_size as usize];

                lzo1x::decompress_to_slice(&compressed_body, &mut body).unwrap();

                let mut r = Reader::with_id_and_node_state(
                    Cursor::new(body),
                    IdState::new(),
                    NodeState::new(num_nodes as usize),
                );

                read_body(&mut node, &mut r, self.body_chunks)?;
            } else {
                todo!()
            }
        }

        Ok(node)
    }

    pub fn read_from_file<P>(self, path: P) -> Result<T>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).map_err(|err| Error(format!("{err}")))?;
        let reader = BufReader::new(file);
        self.read_from(reader)
    }
}

pub fn read_body<T, R, I, N>(
    node: &mut T,
    r: &mut Reader<R, I, N>,
    body_chunks: Vec<(u32, ReadBodyChunk<T, R, I, N>)>,
) -> Result<()>
where
    R: Read + Seek,
{
    let mut body_chunks = body_chunks.into_iter();

    loop {
        let chunk_id = r.u32()?;

        if chunk_id == 0xFACADE01 {
            break;
        }

        let (_, read_body_chunk) = body_chunks.find(|(id, _)| *id == chunk_id).unwrap();

        match read_body_chunk {
            ReadBodyChunk::Read(read_fn) => read_fn(node, r)?,
            ReadBodyChunk::Skip => {
                r.u32()?;
                let size = r.u32()?;
                r.skip(size as u64)?;
            }
            ReadBodyChunk::ReadSkippable(read_fn) => {
                r.u32()?;
                let _size = r.u32()?;
                read_fn(node, r)?
            }
        }
    }

    Ok(())
}
