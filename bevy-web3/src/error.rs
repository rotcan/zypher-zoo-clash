use derive_more::{Display, From};
use serde_json::Error as SerdeError;
use std::io::Error as IoError;


pub type Web3Result<T = ()> = std::result::Result<T, Error>;

/// Transport-depended error.
#[derive(Display, Debug, Clone, PartialEq)]
pub enum TransportError {
    /// Transport-specific error code.
    #[display(fmt = "code {}", _0)]
    Code(u16),
    /// Arbitrary, developer-readable description of the occurred error.
    #[display(fmt = "{}", _0)]
    Message(String),
}

/// Errors which can occur when attempting to generate resource uri.
#[derive(Debug, Display, From)]
pub enum Error {
    /// server is unreachable
    #[display(fmt = "Server is unreachable")]
    Unreachable,
    /// decoder error
    #[display(fmt = "Decoder error: {}", _0)]
    Decoder(String),
    /// invalid response
    #[display(fmt = "Got invalid response: {}", _0)]
    #[from(ignore)]
    InvalidResponse(String),
    /// transport error
    #[display(fmt = "Transport error: {}" _0)]
    #[from(ignore)]
    Transport(TransportError),
    /// rpc error
    // #[display(fmt = "RPC error: {:?}", _0)]
    // Rpc(RPCError),
    /// io error
    #[display(fmt = "IO error: {}", _0)]
    Io(IoError),
    /// recovery error
    // #[display(fmt = "Recovery error: {}", _0)]
    // Recovery(crate::signing::RecoveryError),
    /// web3 internal error
    #[display(fmt = "Internal Web3 error")]
    Internal,
    #[display(fmt = "Eth error: {}",_0)]
    #[from(ignore)]
    Eth(String),
    #[display(fmt = "EthAbi error: {}",_0)]
    #[from(ignore)]
    EthAbi(String),

    #[display(fmt="Web3 Error: {}",_0)]
    #[from(ignore)]
    Web3Error(String),

    #[display(fmt="Hex Error: {}",_0)]
    #[from(ignore)]
    HexError(String),

    #[display(fmt="Channel Closed")]    
    #[from(ignore)]
    ChannelEmpty,

    #[display(fmt="Channel Error: {}",_0)]
    #[from(ignore)]
    ChannelClosed(String)

}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;
        match *self {
            Unreachable | Decoder(_) | InvalidResponse(_) | Transport { .. } | Internal | Eth (_) | EthAbi(_)
            | Web3Error(_) | HexError(_) | ChannelEmpty | ChannelClosed(_)=> None,
            // Rpc(ref e) => Some(e),
            Io(ref e) => Some(e),
            // Recovery(ref e) => Some(e),
        }
    }
}

impl From<web3::contract::Error> for Error{
    fn from(err: web3::contract::Error)->Self{
        Error::Eth(format!("{:?}",err))
    }
}

impl From<web3::ethabi::Error> for Error{
    fn from(err: web3::ethabi::Error)->Self{
        Error::EthAbi(format!("{:?}",err))
    }
}

impl From<web3::Error> for Error{
    fn from(err: web3::Error)->Self{
        Error::Web3Error(format!("{:?}",err))
    }
}

impl From<hex::FromHexError> for Error{
    fn from(err: hex::FromHexError)->Self{
        Error::HexError(format!("{:?}",err))
    }
}

impl From<SerdeError> for Error {
    fn from(err: SerdeError) -> Self {
        Error::Decoder(format!("{:?}", err))
    }
}

// impl From<async_channel::TryRecvError> for Error{
//     fn from(err: async_channel::TryRecvError)->Self{
//         match err{
//             async_channel::TryRecvError::Empty => Error::ChannelEmpty,
//             _=>Error::ChannelClosed(format!("{:?}", err))
//         }
//     }
// }

impl Clone for Error {
    fn clone(&self) -> Self {
        use self::Error::*;
        match self {
            Unreachable => Unreachable,
            Decoder(s) => Decoder(s.clone()),
            InvalidResponse(s) => InvalidResponse(s.clone()),
            Transport(s) => Transport(s.clone()),
            // Rpc(e) => Rpc(e.clone()),
            Io(e) => Io(IoError::from(e.kind())),
            Eth(e)=>Eth(e.clone()),
            EthAbi(e)=>EthAbi(e.clone()),
            Web3Error(e)=>Web3Error(e.clone()),
            HexError(e)=>HexError(e.clone()),
            ChannelEmpty=>ChannelEmpty,
            ChannelClosed(e)=>ChannelClosed(e.clone()),
            // Recovery(e) => Recovery(e.clone()),
            Internal => Internal,
        }
    }
}