use crate::error::{ReadResult, WriteResult};
use crate::reader::Reader;
use crate::writer::Writer;
use std::io::{Read, Write};

pub struct RefTable;

impl RefTable {
    pub fn read<R>(r: &mut Reader<R>) -> ReadResult<()>
    where
        R: Read,
    {
        let num_node_refs = r.u32()?;

        if num_node_refs > 0 {
            todo!()
        }

        Ok(())
    }

    pub fn write<W>(w: &mut Writer<W>) -> WriteResult
    where
        W: Write,
    {
        w.u32(0)?;

        Ok(())
    }
}
