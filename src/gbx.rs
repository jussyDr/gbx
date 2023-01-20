use crate::error::{ReadError, ReadResult};
use crate::header::{Compression, Header};
use crate::reader::{self, IdState, NodeState, Reader};
use crate::ref_table::RefTable;
use std::io::{Cursor, Read, Seek};

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
    fn header_chunks<'a>() -> &'a [(u32, ReadChunkFn<Self, R, I, N>)];
}

pub trait ReadBody<R, I, N>
where
    Self: Sized,
{
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

    let header = Header::read(&mut r, T::CLASS_ID)?;

    if !header.user_data.is_empty() {
        let mut r = Reader::new(header.user_data.as_slice());

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

    RefTable::read(&mut r)?;

    if header.body_compression == Compression::Compressed {
        let body_size = r.u32()?;
        let compressed_body_size = r.u32()?;
        let compressed_body = r.bytes(compressed_body_size as usize)?;
        let mut body = vec![0; body_size as usize];

        minilzo::decompress_to_slice(&compressed_body, &mut body)?;

        let mut r = Reader::with_id_and_node_state(
            Cursor::new(body),
            reader::IdState::new(),
            reader::NodeState::new(header.num_nodes as usize),
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
