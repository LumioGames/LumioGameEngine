//! Generated ContractRuntime artifact. Do not hand-edit.
//! Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27.

#![forbid(unsafe_code)]

mod sha256;
pub use sha256::{sha256, sha256_hex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hash256(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainBreak { Truncated, Mismatch }

pub fn hash_chain_append(prev: &Hash256, payload: &[u8]) -> Hash256 {
    let mut buf = Vec::with_capacity(32 + payload.len());
    buf.extend_from_slice(&prev.0);
    buf.extend_from_slice(payload);
    Hash256(sha256(&buf))
}

pub fn hash_chain_verify(prev: &Hash256, payload: &[u8], expected: &Hash256) -> Result<(), ChainBreak> {
    let got = hash_chain_append(prev, payload);
    if got.0 == expected.0 { Ok(()) } else { Err(ChainBreak::Mismatch) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferFull;

pub struct BoundedBuffer { inner: Vec<u8>, cap: usize }
impl BoundedBuffer {
    pub fn new(cap: usize) -> Self { Self { inner: Vec::new(), cap } }
    pub fn push(&mut self, byte: u8) -> Result<(), BufferFull> {
        if self.inner.len() >= self.cap { return Err(BufferFull); }
        self.inner.push(byte); Ok(())
    }
    pub fn as_slice(&self) -> &[u8] { &self.inner }
}

pub fn canonical_object_pairs(pairs: &mut [(String, String)]) -> String {
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push('"');
        out.push_str(k);
        out.push_str("\":");
        out.push_str(v);
    }
    out.push('}');
    out
}
