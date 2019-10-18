use super::*;
use serde::{Deserialize, Serialize};
use std::ops;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ///create creates new contract
    Create,
    ///calls contract at given address
    /// in the case of a transfer, this is receiver's address.
    Call(H160),
}

impl Default for Action {
    fn default() -> Action {
        Action::Create
    }
}

impl rlp::Encodable for Action {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match *self {
            Action::Create => s.append_internal(&""),
            Action::Call(ref addr) => s.append_internal(addr),
        };
    }
}

impl rlp::Decodable for Action {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.is_empty() {
            Ok(Action::Create)
        } else {
            Ok(Action::Call(rlp.as_val()?))
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    ///Nonce
    pub nonce: U256,
    ///Gas price
    pub gas_price: U256,
    ///Gas paid up front for transaction execution
    pub gas: U256,
    ///Action, can be either call or contract create
    pub action: Action,
    ///Transferred value
    pub value: U256,
    ///Transaction data
    pub data: Bytes,
}

impl Transaction {
    pub fn rlp_append_usigned_transaction(&self, s: &mut rlp::RlpStream, chain_id: Option<u64>) {
        s.begin_list(if chain_id.is_none() { 6 } else { 9 });
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas);
        s.append(&self.action);
        s.append(&self.value);
        s.append(&self.data);
        if let Some(n) = chain_id {
            s.append(&n);
            s.append(&0u8);
            s.append(&0u8);
        }
    }

    pub fn raw_msg(&self, chain_id: Option<u64>) -> Vec<u8> {
        let mut stream = rlp::RlpStream::new();
        self.rlp_append_usigned_transaction(&mut stream, chain_id);
        stream.out()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnverifiedTransaction {
    pub unsigned: Transaction,
    pub v: u64,
    pub r: U256,
    pub s: U256,
    pub hash: H256,
}

impl ops::Deref for UnverifiedTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.unsigned
    }
}

impl rlp::Encodable for UnverifiedTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.rlp_append_sealed_transaction(s)
    }
}

impl rlp::Decodable for UnverifiedTransaction {
    fn decode(d: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        if d.item_count()? != 9 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }
        let hash = keccak(d.as_raw());
        Ok(UnverifiedTransaction {
            unsigned: Transaction {
                nonce: d.val_at(0)?,
                gas_price: d.val_at(1)?,
                gas: d.val_at(2)?,
                action: d.val_at(3)?,
                value: d.val_at(4)?,
                data: d.val_at(5)?,
            },
            v: d.val_at(6)?,
            r: d.val_at(7)?,
            s: d.val_at(8)?,
            hash,
        })
    }
}

impl UnverifiedTransaction {
    ///check is signature is empty
    pub fn is_unsigned(&self) -> bool {
        self.r.is_zero() && self.s.is_zero()
    }

    fn rlp_append_sealed_transaction(&self, s: &mut rlp::RlpStream) {
        s.begin_list(9);
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas);
        s.append(&self.action);
        s.append(&self.value);
        s.append(&self.data);
        s.append(&self.v);
        s.append(&self.r);
        s.append(&self.s);
    }

    ///returns standardized 'v' value (0, 1 or 4 (invalid))
    pub fn standard_v(&self) -> u8 {
        check_replay_protection(self.v)
    }

    ///the chain ID, or 'None' if this is a gloable transaction
    pub fn chain_id(&self) -> Option<u64> {
        match self.v {
            v if self.is_unsigned() => Some(v),
            v if v >= 35 => Some((v - 35) / 2),
            _ => None,
        }
    }

    pub fn raw_msg(&self) -> Vec<u8> {
        self.unsigned.raw_msg(self.chain_id())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct FullTransaction {
    #[serde(rename = "blockHash")]
    pub block_hash: H256,
    #[serde(rename = "blockNumber")]
    pub block_number: U256,
    pub from: H160,
    pub to: H160,
    pub gas: U256,
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
    pub hash: H256,
    pub nonce: U256,
    pub raw: Option<Bytes>,
    pub input: Bytes,
    pub r: U256,
    pub s: U256,
    pub v: U64,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U128,
    pub value: U256,
}

impl From<FullTransaction> for UnverifiedTransaction {
    fn from(tx: FullTransaction) -> UnverifiedTransaction {
        UnverifiedTransaction {
            unsigned: Transaction {
                nonce: tx.nonce,
                gas_price: tx.gas_price,
                gas: tx.gas,
                action: Action::Call(tx.to),
                value: tx.value,
                data: tx.input,
            },
            v: tx.v.as_u64(),
            r: tx.r,
            s: tx.s,
            hash: tx.hash,
        }
    }
}

fn check_replay_protection(v: u64) -> u8 {
    match v {
        v if v == 27 => 0,
        v if v == 28 => 1,
        v if v >= 35 => ((v - 1) % 2) as u8,
        _ => 4,
    }
}

fn keccak<T: AsRef<[u8]>>(s: T) -> H256 {
    let result = tiny_keccak::keccak256(s.as_ref());
    H256::from(result)
}
