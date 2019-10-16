#![allow(unused)]

use parity_codec::Decode;
use serde::{Deserialize, Serialize};

use chainx_primitives::{Balance, Index};

#[derive(Clone,Serialize,Deserialize)]
pub struct DecodeWrapper(substrate_primitives::storage::StorageData);

impl DecodeWrapper {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(substrate_primitives::storage::StorageData(bytes))
    }
}