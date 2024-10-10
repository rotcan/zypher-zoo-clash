use thiserror::Error;
use bevy_web3::error::Error as Web3Error;
use zshuffle::error::ShuffleError;
#[derive(Error,Debug)]
pub enum GameError{
    #[error("type mismatch: {0}")]
    EnumError(String),
    #[error("Web3 Error: {0}")]
    Web3Error(#[from] Web3Error),
    #[error("Send Txn Error: {0}")]
    SendTxnError(String),
    #[error("Compute Error: {0}")]
    ComputeError(String),
    #[error("Data Parse Error : {0}")]
    DataParseError(String),
    #[error("Shuffle Error : {0}")]
    ShuffleError(#[from] ShuffleError),
    #[error("Recv Empty error : {0}")]
    RecvError(#[from] async_channel::TryRecvError),

}
