use async_channel::{unbounded, Receiver, Sender, TryRecvError};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, TaskPool},
};
use web3::{transports::eip_1193, types::TransactionRequest};
use crate::error::Error;
pub use web3::types::{H160, H256, H520, U256};
pub use web3::ethabi::Uint;

// #[derive(Debug)]
// pub enum RecvError {
//     Empty,
//     Closed,
// }

// impl fmt::Display for RecvError{
//     fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
//         write!(f,"{:?}",self)
//     }
// }


impl From<TryRecvError> for Error {
    fn from(e: TryRecvError) -> Error {
        match e {
            TryRecvError::Empty => Error::ChannelEmpty,
            TryRecvError::Closed => Error::ChannelClosed(e.to_string()),
        }
    }
}

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_wallet);
    }
}

enum WalletCmd{
    Transaction{
        hash: Option<H256>,
        error: Option<String>,
        ix_type: Option<i32>,
        to: H160,
    }
}

pub struct TransactionResult{
    pub hash: H256,
    pub ix_type: Option<i32>,
    pub to: H160,
}

#[derive(Resource)]
pub struct WalletChannel {
    account_tx: Sender<(H160, U256)>,
    account_rx: Receiver<(H160, U256)>,
    signature_tx: Sender<H520>,
    signature_rx: Receiver<H520>,
    transaction_tx: Sender<WalletCmd>,
    transaction_rx: Receiver<WalletCmd>,
}

fn init_wallet(mut commands: Commands) {
    let (account_tx, account_rx) = unbounded();
    let (signature_tx, signature_rx) = unbounded();
    let (transaction_tx, transaction_rx) = unbounded();

    commands.insert_resource(WalletChannel {
        account_tx,
        account_rx,
        signature_tx,
        signature_rx,
        transaction_tx,
        transaction_rx,
    });
}

impl WalletChannel {
    pub fn connect(&self) {
        let tx = self.account_tx.clone();
        IoTaskPool::get_or_init(TaskPool::new)
            .spawn(async move {
                let provider = eip_1193::Provider::default().unwrap().unwrap();
                let transport = eip_1193::Eip1193::new(provider);
                let web3 = web3::Web3::new(transport);

                let addrs = web3.eth().request_accounts().await.unwrap();
                let chain = web3.eth().chain_id().await.unwrap();

                if !addrs.is_empty() {
                    info!("addrs: {:?}", addrs);
                    let _ = tx.send((addrs[0], chain)).await;
                }
            })
            .detach();
    }

    pub fn sign(&self, account: H160, msg: String) {
        let tx = self.signature_tx.clone();
        IoTaskPool::get_or_init(TaskPool::new)
            .spawn(async move {
                let provider = eip_1193::Provider::default().unwrap().unwrap();
                let transport = eip_1193::Eip1193::new(provider);
                let web3 = web3::Web3::new(transport);

                let msg = web3::types::Bytes(msg.as_bytes().to_vec());
                let signature = web3.eth().sign(account, msg).await.unwrap();
                let _ = tx.send(signature).await;
            })
            .detach();
    }

    pub fn send(&self, from: H160, to: H160, data: Vec<u8>, ix_type: Option<i32>,value: Option<Uint>) {
        let tx = self.transaction_tx.clone();
        IoTaskPool::get_or_init(TaskPool::new)
            .spawn(async move {
                let provider = eip_1193::Provider::default().unwrap().unwrap();
                let transport = eip_1193::Eip1193::new(provider);
                let web3 = web3::Web3::new(transport);

                let mut txr = TransactionRequest::default();
                txr.from = from;
                txr.to = Some(to);
                txr.data = Some(data.into());
                txr.value=value;
                
                match web3.eth().send_transaction(txr).await {
                    Ok(hash)=>{
                        let _ = tx.send(WalletCmd::Transaction{hash:Some(hash),error:None,ix_type,to}).await;
                    },
                    Err(err)=>{
                        let _ = tx.send(WalletCmd::Transaction{hash:None,error:Some(err.to_string()),ix_type,to}).await;
                    },
                };
                //let _ = tx.send(hash).await;
            })
            .detach();
    }

    pub fn recv_account(&self) -> Result<(H160, U256), Error> {
        Ok(self.account_rx.try_recv()?)
    }

    pub fn recv_signature(&self) -> Result<H520, Error> {
        Ok(self.signature_rx.try_recv()?)
    }

    pub fn recv_transaction(&self) -> Result<TransactionResult, Error> {
        match self.transaction_rx.try_recv(){
            Ok(WalletCmd::Transaction{hash,error,ix_type,to})=>{
                if let Some(hash) = hash{
                    Ok(TransactionResult{hash,ix_type,to})
                }else{
                    if let Some(error) = error {
                        Err(Error::Web3Error(error.to_string()))
                    }else{
                        Err(Error::Web3Error("Unknown Error".to_owned()))
                    }
                }
            },
            Err(err)=>{
                //error!("Err {:?}",err.to_string());
                Err(err.into())
            },
        }
        //Ok(self.transaction_rx.try_recv()?)
    }
}