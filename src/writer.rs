use crate::error::WriteResult;
use crate::FileRef;
use indexmap::{indexset, IndexSet};
use std::io::Write;

#[derive(Clone)]
pub struct IdState {
    seen_id: bool,
    ids: IndexSet<String>,
}

impl IdState {
    pub fn new() -> Self {
        Self {
            seen_id: false,
            ids: indexset! {},
        }
    }
}

#[derive(Clone)]
pub struct NodeState {
    num_nodes: u32,
}

impl NodeState {
    pub fn new() -> Self {
        Self { num_nodes: 0 }
    }
}

pub struct Writer<W, I = (), N = ()> {
    inner: W,
    id_state: I,
    node_state: N,
}

impl<W> Writer<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            id_state: (),
            node_state: (),
        }
    }
}

impl<W, I> Writer<W, I> {
    pub fn with_id_state(inner: W, id_state: I) -> Self {
        Self {
            inner,
            id_state,
            node_state: (),
        }
    }
}

impl<W, I, N> Writer<W, I, N> {
    pub fn with_id_and_node_state(inner: W, id_state: I, node_state: N) -> Self {
        Self {
            inner,
            id_state,
            node_state,
        }
    }
}

macro_rules! impl_write_num {
    ($($type:ident),+) => {
        $(
            pub fn $type(&mut self, value: $type) -> WriteResult {
                self.bytes(&value.to_le_bytes())
            }
        )+
    };
}

impl<W, I, N> Writer<W, I, N>
where
    W: Write,
{
    pub fn bytes(&mut self, bytes: &[u8]) -> WriteResult {
        self.inner.write_all(bytes)?;
        Ok(())
    }

    impl_write_num!(u8, u16, u32, u64, f32);

    pub fn bool(&mut self, value: bool) -> WriteResult {
        if value {
            self.u32(1)
        } else {
            self.u32(0)
        }
    }

    pub fn string(&mut self, string: &str) -> WriteResult {
        self.u32(string.len() as u32)?;
        self.bytes(string.as_bytes())
    }

    pub fn null_file_ref(&mut self) -> WriteResult {
        self.u8(3)?;
        self.bytes(&[0; 32])?;
        self.string("")?;
        self.string("")
    }

    pub fn file_ref(&mut self, file_ref: &FileRef) -> WriteResult {
        self.u8(3)?;

        match file_ref {
            FileRef::Internal { path } => {
                self.u8(2)?;
                self.bytes(&[0; 31])?;
                self.string(path)?;
                self.string("")
            }
            FileRef::External {
                hash,
                path,
                locator_url,
            } => {
                self.bytes(hash)?;
                self.string(path)?;
                self.string(locator_url)
            }
        }
    }

    pub fn chunk_id(&mut self, chunk_id: u32) -> WriteResult {
        self.u32(chunk_id)
    }

    pub fn null_node(&mut self) -> WriteResult {
        self.u32(0xFFFFFFFF)
    }

    pub fn flat_node<F>(&mut self, class_id: u32, write_fn: F) -> WriteResult
    where
        F: Fn(&mut Self) -> WriteResult,
    {
        self.u32(class_id)?;
        write_fn(self)?;
        self.node_end()
    }

    pub fn node_end(&mut self) -> WriteResult {
        self.u32(0xFACADE01)
    }
}

impl<W, I, N> Writer<W, I, N>
where
    W: Write,
    I: Clone,
    N: Clone,
{
    pub fn skippable_chunk<F>(&mut self, chunk_id: u32, write_fn: F) -> WriteResult
    where
        F: Fn(&mut Writer<&mut Vec<u8>, I, N>) -> WriteResult,
    {
        let mut buf = vec![];
        let mut w = Writer::with_id_and_node_state(
            &mut buf,
            self.id_state.clone(),
            self.node_state.clone(),
        );
        write_fn(&mut w)?;
        self.id_state = w.id_state;
        self.node_state = w.node_state;
        self.u32(chunk_id)?;
        self.bytes(b"PIKS")?;
        self.u32(buf.len() as u32)?;
        self.bytes(&buf)
    }
}

impl<W, N> Writer<W, IdState, N>
where
    W: Write,
{
    fn id_version(&mut self) -> WriteResult {
        if !self.id_state.seen_id {
            self.u32(3)?;
            self.id_state.seen_id = true;
        }

        Ok(())
    }

    pub fn null_id(&mut self) -> WriteResult {
        self.id_version()?;
        self.u32(0xFFFFFFFF)
    }

    pub fn id(&mut self, id: &str) -> WriteResult {
        self.id_version()?;

        if let Some(index) = self.id_state.ids.get_index_of(id) {
            self.u32(0x40000000 | (index as u32 + 1))
        } else {
            self.id_state.ids.insert(id.to_owned());
            self.u32(0x40000000)?;
            self.string(id)
        }
    }
}

impl<W, I> Writer<W, I, NodeState> {
    pub fn num_nodes(&self) -> u32 {
        self.node_state.num_nodes
    }
}

impl<W, I> Writer<W, I, NodeState>
where
    W: Write,
{
    pub fn node<F>(&mut self, class_id: u32, write_fn: F) -> WriteResult
    where
        F: Fn(&mut Self) -> WriteResult,
    {
        self.node_state.num_nodes += 1;
        self.u32(self.node_state.num_nodes)?;
        self.flat_node(class_id, write_fn)
    }
}
