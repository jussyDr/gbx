use crate::error::ReadResult;
use crate::reader::Reader;
use std::io::Read;

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
}
