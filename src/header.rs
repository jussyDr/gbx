use crate::error::{ReadError, ReadResult};
use crate::reader::Reader;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::io::Read;

#[derive(TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Format {
    Binary = b'B',
    Text = b'T',
}

#[derive(PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Compression {
    Compressed = b'C',
    Uncompressed = b'U',
}

pub struct Header {
    pub body_compression: Compression,
    pub user_data: Vec<u8>,
    pub num_nodes: u32,
}

impl Header {
    pub fn read<R>(r: &mut Reader<R>, class_id: u32) -> ReadResult<Self>
    where
        R: Read,
    {
        if r.bytes(3)? != b"GBX" {
            return Err(ReadError::Generic(String::from("bad magic")));
        }

        match r.u16()? {
            6 => {}
            version => {
                return Err(ReadError::Generic(format!(
                    "unsupported file version {version}"
                )))
            }
        }

        let format = Format::try_from(r.u8()?)
            .map_err(|_err| ReadError::Generic(String::from("unknown format")))?;

        if matches!(format, Format::Text) {
            return Err(ReadError::Generic(String::from(
                "text file format not supported",
            )));
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

        r.class_id(class_id)?;
        let user_data_size = r.u32()?;
        let user_data = r.bytes(user_data_size as usize)?;
        let num_nodes = r.u32()?;

        Ok(Self {
            body_compression,
            user_data,
            num_nodes,
        })
    }
}
