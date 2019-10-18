#![allow(unused)]

use parity_codec::Decode;
use serde::{Deserialize, Serialize};

use chainx_primitives::{Balance, Index};

#[derive(Clone, Serialize, Deserialize)]
pub struct DecodeWrapper(substrate_primitives::storage::StorageData);

impl DecodeWrapper {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(substrate_primitives::storage::StorageData(bytes))
    }
    pub fn into_inner(self) -> Vec<u8> {
        (self.0).0
    }
    pub fn nonce(self) -> Option<Index> {
        Decode::decode(&mut (self.0).0.as_slice())
    }
    pub fn balance(self) -> Option<Balance> {
        Decode::decode(&mut (self.0).0.as_slice())
    }
}
