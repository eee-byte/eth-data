use super::*;
use serde::{Deserialize, Serialize};
use std::ops;
use fixed_hash::*;
#[cfg(feature = "serialize")]
use impl_serde::impl_fixed_hash_serde;
use impl_rlp::impl_fixed_hash_rlp;

use parity_codec::{Decode, Encode};
use rlp::{Rlp, RlpStream, Encodable, DecoderError, Decodable};

/// Vector of bytes.
pub type Bytes = Vec<u8>;
/// Type for block number.
pub type BlockNumber = u64;
pub type Address = H160;

// 3 according to yellowpaper
const BLOOM_BITS: u32 = 3;
const BLOOM_SIZE: usize = 256;

construct_fixed_hash!{
	/// Bloom hash type with 256 bytes (2048 bits) size.
	pub struct Bloom(BLOOM_SIZE);
}
impl_fixed_hash_rlp!(Bloom, BLOOM_SIZE);

pub type Public = H512;
/// Helper structure, used for encoding blocks.
#[derive(Default, Clone)]
pub struct Block {
    /// Block header
    pub header: Header,
    /// Block transactions
    pub transactions: Vec<SignedTransaction>,
    /// Block uncles
    pub uncles: Vec<Header>
}

/// A `UnverifiedTransaction` with successfully recovered `sender`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SignedTransaction {
    transaction: UnverifiedTransaction,
    sender: Address,
    public: Option<Public>,
}


/// A block header.
///
/// Reflects the specific RLP fields of a block in the chain with additional room for the seal
/// which is non-specific.
///
/// Doesn't do all that much on its own.
#[derive(Debug, Clone, Eq)]
pub struct Header {
    /// Parent hash.
    parent_hash: H256,
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: BlockNumber,
    /// Block author.
    author: Address,

    /// Transactions root.
    transactions_root: H256,
    /// Block uncles hash.
    uncles_hash: H256,
    /// Block extra data.
    extra_data: Bytes,

    /// State root.
    state_root: H256,
    /// Block receipts root.
    receipts_root: H256,
    /// Block bloom.
    log_bloom: Bloom,
    /// Gas used for contracts execution.
    gas_used: U256,
    /// Block gas limit.
    gas_limit: U256,

    /// Block difficulty.
    difficulty: U256,
    /// Vector of post-RLP-encoded fields.
    seal: Vec<Bytes>,

    /// Memoized hash of that header and the seal.
    hash: Option<H256>,
}

impl PartialEq for Header {
    fn eq(&self, c: &Header) -> bool {
        if let (&Some(ref h1), &Some(ref h2)) = (&self.hash, &c.hash) {
            if h1 == h2 {
                return true
            }
        }

        self.parent_hash == c.parent_hash &&
            self.timestamp == c.timestamp &&
            self.number == c.number &&
            self.author == c.author &&
            self.transactions_root == c.transactions_root &&
            self.uncles_hash == c.uncles_hash &&
            self.extra_data == c.extra_data &&
            self.state_root == c.state_root &&
            self.receipts_root == c.receipts_root &&
            self.log_bloom == c.log_bloom &&
            self.gas_used == c.gas_used &&
            self.gas_limit == c.gas_limit &&
            self.difficulty == c.difficulty &&
            self.seal == c.seal
    }
}


impl Decodable for Header {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let mut blockheader = Header {
            parent_hash: r.val_at(0)?,
            uncles_hash: r.val_at(1)?,
            author: r.val_at(2)?,
            state_root: r.val_at(3)?,
            transactions_root: r.val_at(4)?,
            receipts_root: r.val_at(5)?,
            log_bloom: r.val_at(6)?,
            difficulty: r.val_at(7)?,
            number: r.val_at(8)?,
            gas_limit: r.val_at(9)?,
            gas_used: r.val_at(10)?,
            timestamp: r.val_at(11)?,
            extra_data: r.val_at(12)?,
            seal: vec![],
            hash: keccak(r.as_raw()).into(),
        };

        for i in 13..r.item_count()? {
            blockheader.seal.push(r.at(i)?.as_raw().to_vec())
        }

        Ok(blockheader)
    }
}
