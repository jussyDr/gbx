use crate::error::{ReadError, ReadResult, WriteResult};
use crate::reader::{self, Reader};
use crate::writer::{self, Writer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::io::{Cursor, Read, Seek, Write};

#[derive(PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Compression {
    Compressed = b'C',
    Uncompressed = b'U',
}

pub type ReadChunkFn<T, R, I, N> = fn(&mut T, &mut Reader<R, I, N>) -> ReadResult<()>;

pub enum ReadChunk<T, R, I, N> {
    Read(ReadChunkFn<T, R, I, N>),
    Skip,
    ReadSkippable(ReadChunkFn<T, R, I, N>),
}

pub trait Class {
    const CLASS_ID: u32;
}

pub trait ReadHeader<R, I, N> {
    #[allow(clippy::type_complexity)]
    fn header_chunks<'a>() -> &'a [(u32, ReadChunkFn<Self, R, I, N>)];
}

pub trait ReadBody<R, I, N>
where
    Self: Sized,
{
    #[allow(clippy::type_complexity)]
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)];
}

pub fn read<T, R>(reader: R) -> ReadResult<T>
where
    T: Default
        + Class
        + for<'a, 'b> ReadHeader<&'a [u8], &'b mut reader::IdState, ()>
        + ReadBody<Cursor<Vec<u8>>, reader::IdState, reader::NodeState>,
    R: Read,
{
    let mut node = T::default();

    let mut r = Reader::new(reader);

    if r.bytes(3)? != b"GBX" {
        return Err(ReadError(String::from("bad magic")));
    }

    match r.u16()? {
        6 => {}
        _ => return Err(ReadError(String::from("unsupported file version"))),
    }

    match r.u8()? {
        b'B' => {}
        _ => return Err(ReadError(String::from("file format not supported"))),
    }

    let ref_table_compression = Compression::try_from(r.u8()?)
        .map_err(|_err| ReadError(String::from("invalid compression type")))?;

    if matches!(ref_table_compression, Compression::Compressed) {
        return Err(ReadError(String::from(
            "compressed ref table not supported",
        )));
    }

    let body_compression = Compression::try_from(r.u8()?)
        .map_err(|_err| ReadError(String::from("invalid compression type")))?;

    match r.u8()? {
        b'R' => {}
        _unknown => return Err(ReadError(String::from("bad unknown byte"))),
    }

    r.class_id(T::CLASS_ID)?;
    let user_data_size = r.u32()?;
    let user_data = r.bytes(user_data_size as usize)?;
    let num_nodes = r.u32()?;

    if !user_data.is_empty() {
        let mut r = Reader::new(user_data.as_slice());

        let user_data_chunks = r.list(|r| {
            let chunk_id = r.u32()?;
            let size = r.u32()? & 0x7FFFFFFF;

            Ok((chunk_id, size))
        })?;

        let mut id_state = reader::IdState::new();
        let mut i = 0;

        for (chunk_id, size) in user_data_chunks {
            loop {
                let header_chunks = T::header_chunks();

                let (header_chunk_id, read_fn) = header_chunks
                    .get(i)
                    .ok_or_else(|| ReadError(format!("unknown chunk {chunk_id:08X}")))?;

                if *header_chunk_id == chunk_id {
                    let bytes = r.bytes(size as usize)?;
                    let mut r = Reader::with_id_state(bytes.as_slice(), &mut id_state);
                    read_fn(&mut node, &mut r)?;
                    break;
                } else {
                    i += 1;
                }
            }
        }
    }

    let num_node_refs = r.u32()?;

    if num_node_refs > 0 {
        todo!()
    }

    if body_compression == Compression::Compressed {
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

        read_body(&mut node, &mut r)?
    } else {
        todo!()
    }

    Ok(node)
}

pub fn read_body<T, R, I, N>(node: &mut T, r: &mut Reader<R, I, N>) -> ReadResult<()>
where
    T: ReadBody<R, I, N>,
    R: Read + Seek,
{
    let mut i = 0;

    loop {
        let chunk_id = r.u32()?;

        if chunk_id == 0xFACADE01 {
            break;
        }

        loop {
            let body_chunks = T::body_chunks();

            let (body_chunk_id, read_chunk) = body_chunks
                .get(i)
                .ok_or_else(|| ReadError(format!("unknown chunk {chunk_id:08X}")))?;

            if *body_chunk_id == chunk_id {
                match read_chunk {
                    ReadChunk::Read(read_fn) => read_fn(node, r)?,
                    ReadChunk::Skip => {
                        r.u32()?;
                        let size = r.u32()?;
                        r.skip(size as u64)?;
                    }
                    ReadChunk::ReadSkippable(read_fn) => {
                        r.u32()?;
                        let _size = r.u32()?;
                        read_fn(node, r)?;
                    }
                }
                break;
            } else {
                i += 1;
            }
        }
    }

    Ok(())
}

pub trait WriteHeader<W, I, N> {
    #[allow(clippy::type_complexity)]
    fn write_header_chunks<'a>() -> &'a [(u32, fn(&Self, Writer<W, I, N>) -> WriteResult)];
}

pub trait WriteBody<W, I, N> {
    fn write_body(&self, w: &mut Writer<W, I, N>) -> WriteResult;
}

pub fn write<T, W>(node: &T, writer: W) -> WriteResult
where
    T: Class
        + for<'a, 'b> WriteHeader<&'a mut Vec<u8>, &'b mut writer::IdState, ()>
        + for<'a, 'b> WriteBody<&'a mut Vec<u8>, writer::IdState, &'b mut writer::NodeState>,
    W: Write,
{
    let mut user_data = vec![];
    {
        let mut w = Writer::new(&mut user_data);
        let mut id_state = writer::IdState::new();
        let mut i = 0;
        let mut chunks = vec![];

        loop {
            let (chunk_id, write_fn) = T::write_header_chunks()[i];

            let mut chunk = vec![];
            let w = Writer::with_id_state(&mut chunk, &mut id_state);
            write_fn(node, w)?;
            chunks.push((chunk_id, chunk));

            i += 1;

            if i >= T::write_header_chunks().len() {
                break;
            }
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
        node.write_body(&mut w)?;

        w.u32(0xFACADE01)?;
    }

    let mut output = vec![0; lzo1x::worst_compress(body.len())];
    let compressed_body = lzo1x::compress_to_slice(&body, &mut output);

    //let compressed_body = lzo1x::compress_to_slice(&body, &mut output);

    let mut w = Writer::new(writer);

    w.bytes(b"GBX")?;
    w.u16(6)?;
    w.u8(b'B')?;
    w.u8(b'U')?;
    w.u8(b'C')?;
    w.u8(b'R')?;
    w.u32(T::CLASS_ID)?;
    w.u32(user_data.len() as u32)?;
    w.bytes(&user_data)?;
    w.u32(node_state.num_nodes())?;
    w.u32(0)?;
    w.u32(body.len() as u32)?;
    w.u32(compressed_body.len() as u32)?;
    w.bytes(compressed_body)?;

    Ok(())
}
