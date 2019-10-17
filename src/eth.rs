use parity_codec::Encode;
use serde::Deserialize;
use substrate_primitives::crypto::Ss58Codec;
use substrate_primitives::ed25519::{Pair, Public};

use chainx_primitives::AccountId;
use xsdot::{EcdsaSignature, EthereumAddress};

use crate::error::{Error, Result};
use crate::types::{FullTransaction, UnverifiedTransaction, H160, H256};

pub static ETHERSCAN_API: &str = "http://api-cn.etherscan.com/api?module=proxy&action=eth_getTransactionByHash&apikey=WEPDGZ6U6GQ2RD4AGZNUV7CJ25C44KQQAJ&txhash=";

#[derive(Deserialize, Debug)]
struct EtherScanApiResult {
    id: u64,
    jsonrpc: String,
    result: Option<FullTransaction>,
}

#[derive(Clone)]
pub struct EtherScanApi(reqwest::Client);

impl EtherScanApi {
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }

    pub fn get_tx_by_hash(&self, hash: H256) -> Result<FullTransaction> {
        match self.get_tx_by_hash_impl(hash) {
            Ok(transaction) => match transaction {
                Some(transaction) => Ok(transaction),
                None => {
                    warn!("Non-existent Ethereum transaction");
                    Err(Error::NonExistentEthTx.into())
                }
            },
            Err(err) => {
                error!("EtherScanApi get Ethereum tx error: {:?}", err);
                Err(Error::EtherScanCannotGetTx.into())
            }
        }
    }

    pub fn get_tx_by_hash_impl(&self, hash: H256) -> Result<Option<FullTransaction>> {
        let url = format!("{}{:?}", ETHERSCAN_API, hash);
        let result = self.0.get(&url).send()?.json::<EtherScanApiResult>()?;
        Ok(result.result)
    }
}

pub fn check_tx(tx: FullTransaction) -> Result<AccountId> {
    let (from, raw, data, r, s, v) = parse_tx(tx);
    info!(
        "Transaction content: raw[0x{}], data [0x{}], r [0x{}], s [0x{}], v [{}]",
        hex::encode(&raw),
        hex::encode(&data),
        hex::encode(&r),
        hex::encode(&s),
        v
    );

    let signature = EcdsaSignature(r, s, v as i8);
    check_tx_signature(&signature, &raw, &data, from)?;

    let who = check_tx_data(&data)?;

    Ok(who)
}

fn parse_tx(tx: FullTransaction) -> (H160, Vec<u8>, Vec<u8>, [u8; 32], [u8; 32], u8) {
    let from = tx.from;
    let unsigned_tx: UnverifiedTransaction = tx.clone().into();
    let raw = unsigned_tx.raw_msg();
    let data = unsigned_tx.data.0.clone();
    let mut r = [0u8; 32];
    let mut s = [0u8;32];
    unsigned_tx.r.to_big_endian(&mut r[..]);
    unsigned_tx.s.to_big_endian(&mut s[..]);
    let standard_v = unsigned_tx.standard_v();
    (from, raw, data, r, s, standard_v)
}

fn check_tx_signature(
    signature: &EcdsaSignature,
    raw: &[u8],
    data: &[u8],
    from: H160,
) -> Result<()> {
    let eth_addr = eth_recover(&signature, &raw);
    if !contains(&raw, &data) || eth_addr.is_none() || eth_addr != Some(from.to_fixed_bytes()) {
        return Err(Error::InvalidEthTxSignature.into());
    }
    Ok(())
}


fn check_tx_data(data: &[u8]) -> Result<AccountId> {
    let data_channel = split_tx_data(&data);
    if data_channel.len() != 1 && data_channel.len() != 2{
        return Err(Error::EthTxInvalidData.into());
    }
    let public = std::str::from_utf8(&data_channel[0]).unwrap();
    info!("Public: {:?}, Channnel: {:?}", public, data_channel.get(1));
    let who = match Public::from_ss58check(public) {
        Ok(public) => AccountId::from_slice(public.as_slice()),
        Err(_) => return Err(Error::EthTxInvalidData.into()),
    };
    info!("AccountId '{:?}' ss58 check result successfully", who);
    Ok(who)
}

fn eth_recover(s: &EcdsaSignature, sign_data: &[u8]) -> Option<EthereumAddress> {
    use tiny_keccak::keccak256;
    let msg = keccak256(sign_data);
    let mut res = EthereumAddress::default();
    res.copy_from_slice(&keccak256(&ecdsa_recover(s, &msg)?[..])[12..]);
    Some(res)
}

fn ecdsa_recover(sig: &EcdsaSignature, msg: &[u8; 32]) -> Option<[u8; 64]> {
    let msg = secp256k1::Message::parse(msg);
    let signature = secp256k1::Signature::parse_slice(&(sig.0, sig.1).encode()).ok()?;
    let recovery_id = if sig.2 > 26 { sig.2 - 27 } else { sig.2 };
    let recovery_id = secp256k1::RecoveryId::parse(recovery_id as u8).ok()?;
    let pub_key = secp256k1::recover(&msg, &signature, &recovery_id).ok()?;
    let mut res = [0u8; 64];
    res.copy_from_slice(&pub_key.serialize()[1..65]);
    Some(res)
}


fn contains(seq: &[u8], sub_seq: &[u8]) -> bool {
    seq.windows(sub_seq.len()).any(|window| window == sub_seq)
}


fn split_tx_data(data: &[u8]) -> Vec<Vec<u8>> {
    data.split(|x| *x == b'@').map(|d| d.to_vec()).collect()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use super::*;

    #[test]
    fn test_etherscan_api() {
        let result = EtherScanApi::new().get_tx_by_hash(H256::from(&hex!(
            "148d4cde95c0f7064679037147619f6e418587119f27e61f17e02e009576b041"
        )));
        println!("result: {:?}", result);
    }

}