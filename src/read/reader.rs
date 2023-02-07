use crate::read::{Error, Result};
use crate::types::{ExternalFileRef, FileRef, Id, InternalFileRef};
use crate::Vec3;
use std::any::Any;
use std::borrow::BorrowMut;
use std::io::{Read, Seek, SeekFrom, Take};
use std::iter;
use std::mem::size_of;

#[derive(Default)]
pub struct IdState {
    seen_id: bool,
    ids: Vec<Id>,
}

impl IdState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct NodeState {
    nodes: Vec<Option<Box<dyn Any>>>,
}

impl NodeState {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            nodes: iter::repeat_with(|| None).take(num_nodes).collect(),
        }
    }
}

pub struct Reader<R, I = (), N = ()> {
    inner: R,
    id_state: I,
    node_state: N,
}

impl<R> Reader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            id_state: (),
            node_state: (),
        }
    }
}

impl<R, I> Reader<R, I> {
    pub fn with_id_state(inner: R, id_state: I) -> Self {
        Self {
            inner,
            id_state,
            node_state: (),
        }
    }
}

impl<R, I, N> Reader<R, I, N> {
    pub fn with_id_and_node_state(inner: R, id_state: I, node_state: N) -> Self {
        Self {
            inner,
            id_state,
            node_state,
        }
    }
}

macro_rules! impl_read_num {
    ($($type:ident),+) => {
        $(
            pub fn $type(&mut self) -> Result<$type> {
                let mut buf = [0; size_of::<$type>()];
                self.inner
                    .read_exact(&mut buf)
                    .map_err(|err| Error(format!("{err}")))?;
                Ok($type::from_le_bytes(buf))
            }
        )+
    };
}

impl<R, I, N> Reader<R, I, N>
where
    R: Read,
{
    pub fn take(&mut self, limit: u64) -> Take<&mut R> {
        self.inner.borrow_mut().take(limit)
    }

    pub fn bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; n];
        self.inner
            .read_exact(&mut buf)
            .map_err(|err| Error(format!("{err}")))?;
        Ok(buf)
    }

    pub fn bytes_array<const S: usize>(&mut self) -> Result<[u8; S]> {
        let mut buf = [0; S];
        self.inner
            .read_exact(&mut buf)
            .map_err(|err| Error(format!("{err}")))?;
        Ok(buf)
    }

    impl_read_num!(u8, u16, u32, u64, i16, f32);

    pub fn bool(&mut self) -> Result<bool> {
        match self.u32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error(String::from("expected boolean"))),
        }
    }

    pub fn bool_u8(&mut self) -> Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error(String::from("expected boolean"))),
        }
    }

    pub fn packed_index(&mut self, max: u32) -> Result<u32> {
        if max <= u8::MAX as u32 {
            self.u8().map(|index| index as u32)
        } else if max <= u16::MAX as u32 {
            self.u16().map(|index| index as u32)
        } else {
            self.u32()
        }
    }

    pub fn string(&mut self) -> Result<String> {
        let len = self.u32()?;
        let bytes = self.bytes(len as usize)?;
        let string = String::from_utf8(bytes).map_err(|err| Error(format!("{err}")))?;
        Ok(string)
    }

    pub fn repeat<T, F>(&mut self, n: usize, mut read_fn: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        let mut vec = Vec::with_capacity(n);

        for _ in 0..n {
            vec.push(read_fn(self)?);
        }

        Ok(vec)
    }

    pub fn list<T, F>(&mut self, read_fn: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        let len = self.u32()?;
        self.repeat(len as usize, read_fn)
    }

    pub fn vec2f32(&mut self) -> Result<[f32; 2]> {
        let x = self.f32()?;
        let y = self.f32()?;

        Ok([x, y])
    }

    pub fn vec3u8(&mut self) -> Result<Vec3<u8>> {
        let x = self.u8()?;
        let y = self.u8()?;
        let z = self.u8()?;

        Ok(Vec3 { x, y, z })
    }

    pub fn vec3u32(&mut self) -> Result<Vec3<u32>> {
        let x = self.u32()?;
        let y = self.u32()?;
        let z = self.u32()?;

        Ok(Vec3 { x, y, z })
    }

    pub fn vec3f32(&mut self) -> Result<Vec3<f32>> {
        let x = self.f32()?;
        let y = self.f32()?;
        let z = self.f32()?;

        Ok(Vec3 { x, y, z })
    }

    pub fn optional_internal_file_ref(&mut self) -> Result<Option<InternalFileRef>> {
        match self.optional_file_ref()? {
            Some(file_ref) => file_ref
                .internal()
                .ok_or(Error(String::from("expected internal file ref")))
                .map(Some),
            None => Ok(None),
        }
    }

    pub fn optional_external_file_ref(&mut self) -> Result<Option<ExternalFileRef>> {
        match self.optional_file_ref()? {
            Some(file_ref) => file_ref
                .external()
                .ok_or(Error(String::from("expected external file ref")))
                .map(Some),
            None => Ok(None),
        }
    }

    pub fn optional_file_ref(&mut self) -> Result<Option<FileRef>> {
        if self.u8()? != 3 {
            return Err(Error(String::from("unsupported file ref version")));
        }

        let hash = self.bytes_array()?;
        let path = self.string()?;
        let locator_url = self.string()?;

        if hash.iter().all(|&byte| byte == 0) && path.is_empty() && locator_url.is_empty() {
            Ok(None)
        } else if hash[0] == 2 && hash[1..].iter().all(|&byte| byte == 0) && locator_url.is_empty()
        {
            Ok(Some(FileRef::Internal(InternalFileRef {
                path: path.into(),
            })))
        } else {
            Ok(Some(FileRef::External(ExternalFileRef {
                hash,
                path: path.into(),
                locator_url,
            })))
        }
    }

    pub fn chunk_id(&mut self, chunk_id: u32) -> Result<()> {
        let value = self.u32()?;

        if value != chunk_id {
            return Err(Error(format!(
                "expected chunk {chunk_id:08X}, got chunk {value:08X}"
            )));
        }

        Ok(())
    }

    pub fn skippable_chunk_id(&mut self, chunk_id: u32) -> Result<u32> {
        self.chunk_id(chunk_id)?;

        if self.bytes(4)? != b"PIKS" {
            return Err(Error(format!("expected skippable chunk {chunk_id:08X}")));
        }

        self.u32()
    }

    pub fn class_id(&mut self, class_id: u32) -> Result<()> {
        let value = self.u32()?;

        if value != class_id {
            return Err(Error(format!(
                "expected class {class_id:08X}, got class {value:08X}"
            )));
        }

        Ok(())
    }

    pub fn flat_node<T, F>(&mut self, class_id: u32, mut read_fn: F) -> Result<T>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        self.class_id(class_id)?;
        let node = read_fn(self)?;
        Ok(node)
    }

    pub fn node_end(&mut self) -> Result<()> {
        if self.u32()? != 0xFACADE01 {
            return Err(Error(String::from("expected end of node")));
        }

        Ok(())
    }
}

impl<R, I, N> Reader<R, I, N>
where
    R: Seek,
{
    pub fn skip(&mut self, n: u64) -> Result<()> {
        self.inner
            .seek(SeekFrom::Current(n as i64))
            .map_err(|err| Error(format!("{err}")))?;
        Ok(())
    }
}

impl<R, I, N> Reader<R, I, N>
where
    R: Read + Seek,
{
    #[allow(unused)]
    pub fn peek_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let bytes = self.bytes(n)?;
        self.inner
            .seek(SeekFrom::Current(-(n as i64)))
            .map_err(|err| Error(format!("{err}")))?;
        Ok(bytes)
    }

    pub fn peek_u32(&mut self) -> Result<u32> {
        let bytes = self.u32()?;
        self.inner
            .seek(SeekFrom::Current(-4))
            .map_err(|err| Error(format!("{err}")))?;
        Ok(bytes)
    }

    pub fn optional_chunk<F>(&mut self, chunk_id: u32, mut read_fn: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        if self.u32()? != chunk_id {
            self.inner
                .seek(SeekFrom::Current(-4))
                .map_err(|err| Error(format!("{err}")))?;
            return Ok(());
        }

        read_fn(self)
    }

    pub fn optional_skippable_chunk<F>(&mut self, chunk_id: u32, mut read_fn: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        if self.u32()? != chunk_id {
            self.inner
                .seek(SeekFrom::Current(-4))
                .map_err(|err| Error(format!("{err}")))?;
            return Ok(());
        }

        self.u32()?;
        self.u32()?;

        read_fn(self)
    }

    pub fn skip_chunk(&mut self, chunk_id: u32) -> Result<()> {
        let size = self.skippable_chunk_id(chunk_id)?;
        self.skip(size as u64)
    }

    pub fn skip_optional_chunk(&mut self, chunk_id: u32) -> Result<()> {
        let value = self.u32()?;

        if value != chunk_id {
            self.inner
                .seek(SeekFrom::Current(-4))
                .map_err(|err| Error(format!("{err}")))?;
            return Ok(());
        }

        if self.bytes(4)? != b"PIKS" {
            return Err(Error(format!("expected skippable chunk {chunk_id:08X}")));
        }

        let size = self.u32()?;
        self.skip(size as u64)?;
        Ok(())
    }

    pub fn optional_flat_node<T, F>(&mut self, class_id: u32, mut read_fn: F) -> Result<Option<T>>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        if self.u32()? == 0xFFFFFFFF {
            return Ok(None);
        }

        self.inner
            .seek(SeekFrom::Current(-4))
            .map_err(|err| Error(format!("{err}")))?;

        self.class_id(class_id)?;
        let node = read_fn(self)?;
        Ok(Some(node))
    }
}

impl<R, I, N> Reader<R, I, N>
where
    R: Read,
    I: BorrowMut<IdState>,
{
    pub fn id(&mut self) -> Result<Id> {
        match self.optional_id()? {
            Some(id) => Ok(id),
            None => Err(Error(String::from("expected id, got null"))),
        }
    }

    pub fn optional_id(&mut self) -> Result<Option<Id>> {
        if !self.id_state.borrow().seen_id {
            let version = self.u32()?;

            if version != 3 {
                return Err(Error(String::from("unsupported id version")));
            }

            self.id_state.borrow_mut().seen_id = true;
        }

        match self.u32()? {
            0xFFFFFFFF => Ok(None),
            0x40000000 => {
                let id = Id::new(self.string()?);
                self.id_state.borrow_mut().ids.push(Id::clone(&id));
                Ok(Some(id))
            }
            index if index & 0xFFFFF000 == 0x40000000 => {
                let id = self
                    .id_state
                    .borrow()
                    .ids
                    .get((index & 0x00000FFF) as usize - 1)
                    .ok_or_else(|| {
                        Error(format!(
                            "invalid id index {}",
                            (index & 0x00000FFF) as usize - 1
                        ))
                    })?;

                Ok(Some(Id::clone(id)))
            }
            0x00000001 => Ok(Some(Id::empty())), // what is this
            _ => Err(Error(String::from("expected id"))),
        }
    }
}

impl<R, I, N> Reader<R, I, N>
where
    R: Read,
    N: BorrowMut<NodeState>,
{
    pub fn node<T, F>(&mut self, class_id: u32, read_fn: F) -> Result<&T>
    where
        T: 'static,
        F: FnMut(&mut Self) -> Result<T>,
    {
        match self.optional_node(class_id, read_fn)? {
            Some(node) => Ok(node),
            None => Err(Error(String::from("expected node, got null"))),
        }
    }

    pub fn node_owned<T, F>(&mut self, class_id: u32, read_fn: F) -> Result<T>
    where
        T: 'static + Clone,
        F: FnMut(&mut Self) -> Result<T>,
    {
        Ok(self.node(class_id, read_fn)?.clone())
    }

    pub fn optional_node<T, F>(&mut self, class_id: u32, mut read_fn: F) -> Result<Option<&T>>
    where
        T: 'static,
        F: FnMut(&mut Self) -> Result<T>,
    {
        self.any_optional_node(|r, id| {
            if id != class_id {
                return Err(Error(format!(
                    "expected class {class_id:08X}, got class {id:08X}"
                )));
            }

            read_fn(r)
        })
    }

    pub fn optional_node_owned<T, F>(&mut self, class_id: u32, read_fn: F) -> Result<Option<T>>
    where
        T: 'static + Clone,
        F: FnMut(&mut Self) -> Result<T>,
    {
        self.optional_node(class_id, read_fn)
            .map(|optional_node| optional_node.cloned())
    }

    pub fn any_node<T, F>(&mut self, read_fn: F) -> Result<&T>
    where
        T: 'static,
        F: FnMut(&mut Self, u32) -> Result<T>,
    {
        match self.any_optional_node(read_fn)? {
            Some(node) => Ok(node),
            None => Err(Error(String::from("expected got, found null"))),
        }
    }

    pub fn any_node_owned<T, F>(&mut self, read_fn: F) -> Result<T>
    where
        T: 'static + Clone,
        F: FnMut(&mut Self, u32) -> Result<T>,
    {
        self.any_node(read_fn).map(|node| node.clone())
    }

    pub fn any_optional_node<T, F>(&mut self, mut read_fn: F) -> Result<Option<&T>>
    where
        T: 'static,
        F: FnMut(&mut Self, u32) -> Result<T>,
    {
        let index = self.u32()?;

        if index == 0xFFFFFFFF {
            return Ok(None);
        }

        let index = index as usize - 1;

        if index < self.node_state.borrow().nodes.len() {
            if self.node_state.borrow().nodes.get(index).unwrap().is_some() {
                let node_ref: &T = self
                    .node_state
                    .borrow()
                    .nodes
                    .get(index)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .downcast_ref()
                    .unwrap();

                Ok(Some(node_ref))
            } else {
                let class_id = self.u32()?;
                let node = read_fn(self, class_id)?;

                *self.node_state.borrow_mut().nodes.get_mut(index).unwrap() = Some(Box::new(node));

                let node_ref = self
                    .node_state
                    .borrow()
                    .nodes
                    .get(index)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .downcast_ref()
                    .unwrap();

                Ok(Some(node_ref))
            }
        } else {
            Err(Error(String::from("invalid node index")))
        }
    }

    pub fn any_optional_node_owned<T, F>(&mut self, read_fn: F) -> Result<Option<T>>
    where
        T: 'static + Clone,
        F: FnMut(&mut Self, u32) -> Result<T>,
    {
        self.any_optional_node(read_fn).map(|node| node.cloned())
    }
}
