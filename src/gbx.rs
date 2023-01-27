use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::{ReadError, ReadResult};
use crate::reader::{self, IdState, NodeState, Reader};
use std::io::{Cursor, Read, Seek};

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
        + for<'a, 'b> ReadHeader<&'a [u8], &'b mut IdState, ()>
        + ReadBody<Cursor<Vec<u8>>, IdState, NodeState>,
    R: Read,
{
    let mut node = T::default();

    let mut r = Reader::new(reader);

    if r.bytes(3)? != b"GBX" {
        return Err(ReadError::Generic(String::from("bad magic")));
    }

    match r.u16()? {
        6 => {}
        _ => return Err(ReadError::Generic(String::from("unsupported file version"))),
    }

    match r.u8()? {
        b'B' => {}
        _ => {
            return Err(ReadError::Generic(String::from(
                "file format not supported",
            )))
        }
    }

    let ref_table_compression = Compression::try_from(r.u8()?)
        .map_err(|_err| ReadError::Generic(String::from("unknown compression")))?;

    if matches!(ref_table_compression, Compression::Compressed) {
        return Err(ReadError::Generic(String::from(
            "compressed ref table not supported",
        )));
    }

    let body_compression = Compression::try_from(r.u8()?)
        .map_err(|_err| ReadError::Generic(String::from("unknown compression")))?;

    match r.u8()? {
        b'R' => {}
        _unknown => return Err(ReadError::Generic(String::from("bad unknown byte"))),
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

        let mut id_state = IdState::new();
        let mut i = 0;

        for (chunk_id, size) in user_data_chunks {
            loop {
                let header_chunks = T::header_chunks();

                let (header_chunk_id, read_fn) = header_chunks
                    .get(i)
                    .ok_or_else(|| ReadError::Generic(format!("Unknown chunk {chunk_id:08X}")))?;

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

        lzo1x::decompress_to_slice(&compressed_body, &mut body)?;

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
                .ok_or_else(|| ReadError::Generic(format!("Unknown chunk {chunk_id:08X}")))?;

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
