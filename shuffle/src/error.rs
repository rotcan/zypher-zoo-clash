use thiserror::Error;
#[derive(Error,Debug)]
pub enum ShuffleError{
    #[error("Error in utils: {0}")]
    UtilError(String),
    #[error("Error in Wasm part : {0}")]
    WasmError(String),
    #[error("Hex error:{0}")]
    HexError(#[from] hex::FromHexError),
    #[error("UzkgeError Error : {0}")]
    UzkgeError(#[from] uzkge::errors::UzkgeError),
}
