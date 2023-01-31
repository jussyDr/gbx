use crate::error::WriteResult;
use crate::FileRef;
use indexmap::{indexset, IndexSet};
use std::borrow::BorrowMut;
use std::io::Write;

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

pub struct NodeState {
    num_nodes: u32,
}

impl NodeState {
    pub fn new() -> Self {
        Self { num_nodes: 0 }
    }

    pub fn num_nodes(&self) -> u32 {
        self.num_nodes
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
            pub fn $type(&mut self, val: $type) -> WriteResult {
                self.bytes(&val.to_le_bytes())
            }
        )+
    };
}

impl<W, I, N> Writer<W, I, N>
where
    W: Write,
{
    pub fn bytes(&mut self, bytes: &[u8]) -> WriteResult {
        self.inner.write_all(bytes).map_err(Into::into)
    }

    impl_write_num!(u8, u16, u32, u64, f32);

    pub fn string(&mut self, string: &str) -> WriteResult {
        self.u32(string.len() as u32)?;
        self.bytes(string.as_bytes())
    }

    pub fn file_ref(&mut self, file_ref: Option<FileRef>) -> WriteResult {
        self.u8(3)?;

        match file_ref {
            None => {
                self.bytes(&[0; 32])?;
                self.u32(0)?;
                self.u32(0)?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

impl<W, I, N> Writer<W, I, N>
where
    W: Write,
    I: BorrowMut<IdState>,
{
    pub fn id(&mut self, id: Option<&str>) -> WriteResult {
        if !self.id_state.borrow().seen_id {
            self.u32(3)?;
            self.id_state.borrow_mut().seen_id = true;
        }

        match id {
            Some(id) => {
                if let Some(index) = self.id_state.borrow().ids.get_index_of(id) {
                    self.u32(0x40000000 | (index as u32 + 1))
                } else {
                    self.id_state.borrow_mut().ids.insert(id.to_owned());
                    self.u32(0x40000000)?;
                    self.string(id)
                }
            }
            None => self.u32(0xFFFFFFFF),
        }
    }

    pub fn skippable_chunk<F>(&mut self, chunk_id: u32, write_fn: F) -> WriteResult
    where
        F: Fn(Writer<&mut Vec<u8>, &mut IdState, &mut N>) -> WriteResult,
    {
        let mut chunk = vec![];
        {
            let w = Writer::with_id_and_node_state(
                &mut chunk,
                self.id_state.borrow_mut(),
                self.node_state.borrow_mut(),
            );

            write_fn(w)?;
        }

        self.u32(chunk_id)?;
        self.bytes(b"PIKS")?;
        self.u32(chunk.len() as u32)?;
        self.bytes(&chunk)
    }
}

impl<W, I, N> Writer<W, I, N>
where
    W: Write,
    N: BorrowMut<NodeState>,
{
    pub fn node<F>(&mut self, class_id: u32, write_fn: F) -> WriteResult
    where
        F: Fn(&mut Self) -> WriteResult,
    {
        self.node_state.borrow_mut().num_nodes += 1;
        self.u32(self.node_state.borrow().num_nodes)?;
        self.u32(class_id)?;
        write_fn(self)?;
        self.u32(0xFACADE01)
    }
}
