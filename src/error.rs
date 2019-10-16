use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Format error: {}", _0)]
    Fmt(#[cause] std::fmt::Error),
    #[fail(display = "Io error: {}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    NetAddrParse(#[cause] std::net::AddrParseError),
    #[fail(display = "Json error: {}", _0)]
    Json(#[cause] serde_json::Error),
    #[fail(display = "Hex convert error: {}", _0)]
    Hex(#[cause] hex::FromHexError),
    #[fail(display = "Rlp decode error: {}", _0)]
    RlpDecode(#[cause] rlp::DecoderError),
    #[fail(display = "Reqwest error: {}", _0)]
    Reqwest(reqwest::Error),
    #[fail(display = "Rpc internal error: {}", _0)]
    Web3Rpc(#[cause] web3::Error),
    #[fail(display = "EtherScanApi get Ethereum tx error")]
    EtherScanCannotGetTx,
    #[fail(display = "Non-existent Ethereum transaction")]
    NonExistentEthTx,
    #[fail(display = "Invalid Ethereum transaction signature")]
    InvalidEthTxSignature,
    #[fail(display = "Invalid Ethereum transaction data field")]
    EthTxInvalidData,
    #[fail(display = "You are NOT the DOT owner or You Have received the SDOT")]
    NoSdot,
    #[fail(display = "Mapping SDOT timeout")]
    MappingTimeout,
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Error::Fmt(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(err: std::net::AddrParseError) -> Self {
        Error::NetAddrParse(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Self {
        Error::Hex(err)
    }
}

impl From<rlp::DecoderError> for Error {
    fn from(err: rlp::DecoderError) -> Self {
        Error::RlpDecode(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<web3::Error> for Error {
    fn from(err: web3::Error) -> Self {
        Error::Web3Rpc(err)
    }
}

const ERROR: i64 = 10000;

pub fn rpc_error<S: Into<String>>(code: i64, msg: S) -> jsonrpc_core::Error {
    jsonrpc_core::Error {
        code: jsonrpc_core::ErrorCode::ServerError(code),
        message: msg.into(),
        data: None,
    }
}

#[rustfmt::skip]
impl From<Error> for jsonrpc_core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::EtherScanCannotGetTx => rpc_error(ERROR, "EtherScanApi get Ethereum tx error"),
            Error::NonExistentEthTx => rpc_error(ERROR + 1, "Non-existent Ethereum transaction"),
            Error::InvalidEthTxSignature => rpc_error(ERROR + 2, "Invalid Ethereum transaction signature"),
            Error::EthTxInvalidData => rpc_error(ERROR + 3, "Invalid Ethereum transaction data field"),
            Error::NoSdot => rpc_error(ERROR + 4, "You are NOT the DOT owner or You Have received the SDOT"),
            Error::MappingTimeout => rpc_error(ERROR + 5, "Mapping SDOT timeout"),
            _ => jsonrpc_core::Error::internal_error(),
        }
    }
}
