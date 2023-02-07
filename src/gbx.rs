#![allow(clippy::type_complexity)]

use crate::reader::{self, Reader};
use crate::writer::{self, Writer};
use crate::{read, write};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, Write};
use std::path::Path;

pub enum ReadBodyChunk<T, R, I, N> {
    Read(fn(&mut T, &mut Reader<R, I, N>) -> read::Result<()>),
    Skip,
    ReadSkippable(fn(&mut T, &mut Reader<R, I, N>) -> read::Result<()>),
}

pub struct ReaderBuilder<T> {
    read_user_data: bool,
    read_body: bool,

    default: fn() -> T,
    class_id: u32,
    header_chunks: Vec<(
        u32,
        fn(&mut T, &mut Reader<&[u8], &mut reader::IdState>) -> read::Result<()>,
    )>,
    body_chunks: Vec<(
        u32,
        ReadBodyChunk<T, Cursor<Vec<u8>>, reader::IdState, reader::NodeState>,
    )>,
}

impl<T> ReaderBuilder<T> {
    pub(crate) fn new(
        default: fn() -> T,
        class_id: u32,
        header_chunks: Vec<(
            u32,
            fn(&mut T, &mut Reader<&[u8], &mut reader::IdState>) -> read::Result<()>,
        )>,
        body_chunks: Vec<(
            u32,
            ReadBodyChunk<T, Cursor<Vec<u8>>, reader::IdState, reader::NodeState>,
        )>,
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

    pub fn read_from<R>(self, reader: R) -> read::Result<T>
    where
        R: Read,
    {
        let mut node = (self.default)();

        let mut r = Reader::new(reader);

        if r.bytes(3)? != b"GBX" {
            return Err(read::Error(String::from("bad magic")));
        }

        if r.u16()? != 6 {
            return Err(read::Error(String::from("version not supported")));
        }

        match r.u8()? {
            b'B' => {}
            b'T' => return Err(read::Error(String::from("text format not supported"))),
            _ => return Err(read::Error(String::from("bad format"))),
        }

        match r.u8()? {
            b'U' => {}
            b'C' => {
                return Err(read::Error(String::from(
                    "compressed ref table not supported",
                )))
            }
            _ => return Err(read::Error(String::from("bad compression"))),
        }

        let body_compressed = match r.u8()? {
            b'C' => true,
            b'U' => false,
            _ => return Err(read::Error(String::from("bad compression"))),
        };

        if r.u8()? != b'R' {
            return Err(read::Error(String::from("bad unknown byte")));
        }

        if r.u32()? != self.class_id {
            return Err(read::Error(String::from("unexpected node class")));
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
                let mut id_state = reader::IdState::new();

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
                    reader::IdState::new(),
                    reader::NodeState::new(num_nodes as usize),
                );

                read_body(&mut node, &mut r, self.body_chunks)?;
            } else {
                todo!()
            }
        }

        Ok(node)
    }

    pub fn read_from_file<P>(self, path: P) -> read::Result<T>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).map_err(|err| read::Error(format!("{err}")))?;
        let reader = BufReader::new(file);
        self.read_from(reader)
    }
}

pub fn read_body<T, R, I, N>(
    node: &mut T,
    r: &mut Reader<R, I, N>,
    body_chunks: Vec<(u32, ReadBodyChunk<T, R, I, N>)>,
) -> read::Result<()>
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

pub struct WriterBuilder<'a, T> {
    node: &'a T,
    class_id: u32,
    header_chunks: Vec<(
        u32,
        fn(&T, Writer<&mut Vec<u8>, &mut writer::IdState>) -> write::Result,
    )>,
    body:
        fn(&T, &mut Writer<&mut Vec<u8>, writer::IdState, &mut writer::NodeState>) -> write::Result,
}

impl<'a, T> WriterBuilder<'a, T> {
    pub(crate) fn new(
        node: &'a T,
        class_id: u32,
        header_chunks: Vec<(
            u32,
            fn(&T, Writer<&mut Vec<u8>, &mut writer::IdState>) -> write::Result,
        )>,
        body: fn(
            &T,
            &mut Writer<&mut Vec<u8>, writer::IdState, &mut writer::NodeState>,
        ) -> write::Result,
    ) -> Self {
        Self {
            node,
            class_id,
            header_chunks,
            body,
        }
    }

    pub fn write_to<W>(self, writer: W) -> write::Result
    where
        W: Write,
    {
        let mut user_data = vec![];
        {
            let mut w = Writer::new(&mut user_data);
            let mut id_state = writer::IdState::new();
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
        let mut node_state = writer::NodeState::new();
        {
            let mut w =
                Writer::with_id_and_node_state(&mut body, writer::IdState::new(), &mut node_state);
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

    pub fn write_to_file<P>(self, path: P) -> write::Result
    where
        P: AsRef<Path>,
    {
        let file = File::create(path).map_err(|err| write::Error(format!("{err}")))?;
        let writer = BufWriter::new(file);
        self.write_to(writer)
    }
}
