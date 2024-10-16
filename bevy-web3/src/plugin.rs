use async_channel::{unbounded, Receiver, Sender, TryRecvError};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, TaskPool},
};
use web3::contract::tokens::Detokenize;
use web3::{transports::eip_1193, types::TransactionRequest};
use crate::error::{Error,Web3Result};
pub use web3::types::{H160, H256, H520, U256,CallRequest,Address};
pub use web3::ethabi::{Uint,Contract as Abi};
pub mod tokens{
    pub use web3::contract::tokens::*;
    pub use web3::ethabi::*;
}
use crate::plugin::tokens::{Token};


#[derive(Debug,Clone)]
pub struct EthContract{
    pub address: H160,
    pub abi: Abi,
    pub rpc_url: String,
}


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
    TransactionResult{
        hash: Option<H256>,
        error: Option<String>,
        ix_type: Option<i32>,
        to: H160,
    },
    CallResult{
        //result: Option<Box<dyn Detokenize + Send + 'static>>,
        result: Option<Vec<u8>>,
        contract_address: H160,
        method_name: String, 
        error: Option<String>,
        params:Option<Vec<u8>>, },
}

 
#[derive(Debug,Clone)]
pub struct TransactionResult{
    pub hash: H256,
    pub ix_type: Option<i32>,
    pub to: H160,
}

#[derive(Debug,Clone)]
pub struct CallResponse{
    //pub result: Box<dyn Detokenize + Send + 'static>,
    pub result: Vec<u8>,
    pub contract_address: H160,
    pub method_name: String,
    pub params:Option<Vec<u8>>,
} 


#[derive(Resource)]
pub struct WalletChannel {
    account_tx: Sender<(H160, U256)>,
    account_rx: Receiver<(H160, U256)>,
    signature_tx: Sender<H520>,
    signature_rx: Receiver<H520>,
    transaction_tx: Sender<WalletCmd>,
    transaction_rx: Receiver<WalletCmd>,
    contract_tx: Sender<WalletCmd>,
    contract_rx: Receiver<WalletCmd>,
}

fn init_wallet(mut commands: Commands) {
    let (account_tx, account_rx) = unbounded();
    let (signature_tx, signature_rx) = unbounded();
    let (transaction_tx, transaction_rx) = unbounded();
    let (contract_tx,contract_rx)=unbounded();

    commands.insert_resource(WalletChannel {
        account_tx,
        account_rx,
        signature_tx,
        signature_rx,
        transaction_tx,
        transaction_rx,
        contract_tx,
        contract_rx,
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
                        let _ = tx.send(WalletCmd::TransactionResult{hash:Some(hash),error:None,ix_type,to}).await;
                    },
                    Err(err)=>{
                        let _ = tx.send(WalletCmd::TransactionResult{hash:None,error:Some(err.to_string()),ix_type,to}).await;
                    },
                };
                //let _ = tx.send(hash).await;
            })
            .detach();
    }


   
    pub fn call_contract_with_data(&self,contract: EthContract, method_name: String, data:  Vec<u8>)
    //where T: Detokenize + Tokenizable+ Send + 'static
    {
        let tx=self.contract_tx.clone();
        let address=contract.address().clone();
        // let contract_clone=contract.clone();
        IoTaskPool::get_or_init(TaskPool::new)
            .spawn(async move {
                
                let provider = eip_1193::Provider::default().unwrap().unwrap();
                let transport = eip_1193::Eip1193::new(provider);
                let web3 = web3::Web3::new(transport);

                let mut call = CallRequest::default();
                call.to = Some(address);
                call.data = Some(data.clone().into());
                match web3.eth().call(call, None).await{
                    Ok(bytes)=>{
                        let _ = tx.send(WalletCmd::CallResult{
                            //result:Some(ret_type.parse_data(result)),
                            result: Some(bytes.0),
                            contract_address: address,
                            method_name: method_name,
                            error:None,
                            params: Some(data)
                        }).await;
                    },
                    Err(err)=>{
                        let _ = tx.send(WalletCmd::CallResult{
                            result:None,
                            contract_address: address,
                            method_name: method_name,
                            error:Some(err.to_string()),
                            params: Some(data)
                        }).await;
                    }
                }
                
            }).detach();
    }


    pub fn recv_account(&self) -> Result<(H160, U256), Error> {
        Ok(self.account_rx.try_recv()?)
    }

    pub fn recv_signature(&self) -> Result<H520, Error> {
        Ok(self.signature_rx.try_recv()?)
    }

    pub fn recv_transaction(&self) -> Result<TransactionResult, Error> {
        match self.transaction_rx.try_recv(){
            Ok(WalletCmd::TransactionResult{hash,error,ix_type,to})=>{
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
            _=>{Err(Error::Web3Error("Response mismatch".to_owned()))}
        }
        //Ok(self.transaction_rx.try_recv()?)
    }

    pub fn recv_contract(&self)->Web3Result<CallResponse>{
        match self.contract_rx.try_recv(){
            Ok(WalletCmd::CallResult{result,error,contract_address,method_name,params})=>{
                if let Some(result) = result  {
                    Ok(CallResponse{
                        result,
                        contract_address,
                        method_name,
                        params
                    })
                }else {
                    if let Some(error)= error {
                        Err(Error::Web3Error(error))
                    }else{
                        Err(Error::Web3Error("unknown".to_owned()))
                    }
                }
            },
            Err(err)=>Err(err.into()),
            _=>{Err(Error::Web3Error("Response mismatch".to_owned()))}
        }
    }
}


impl EthContract{

    pub fn load_contract(rpc_url: String, address: String, json: &[u8])->Web3Result<EthContract>{
        let address=if address.starts_with("0x") {
            address.strip_prefix("0x").unwrap()
        }else{
            &address
        };
        let address_bytes=Address::from_slice(&hex::decode(&address)?);
        // let transport= Http::new(&rpc_url)?;
        // let eth_api=Eth::new(transport);
        Ok(Self{
            rpc_url,
            address: address_bytes,
            abi: Abi::load(json)?
        })
    }

    pub fn address(&self)->H160{
        self.address
    }

    pub fn encode_input(&self, func: &str, params: &[Token])->Web3Result<Vec<u8>>
    //where P : Tokenize
    {
        let func= self.abi.function(func)?;
        let tokens =params;
        let data = func.encode_input(tokens)?;
        Ok(data)
    }

    pub fn decode_input(&self, func: &str, bytes: &[u8])->Web3Result<Vec<Token>>
    //where P : Tokenize
    {
        let func= self.abi.function(func)?;
        //let tokens =params;
        let data = func.decode_input(bytes)?;
        Ok(data)
    }

    pub fn decode_output(&self, func: &str, bytes: &[u8])->Web3Result<Token>{
        let func= self.abi.function(func)?;
        let data= func.decode_output(bytes)?;
        let output= if data.len() ==1{
            Detokenize::from_tokens(data)?
        }else{
            //Ok(output.into_tokens())
            Detokenize::from_tokens(vec![Token::Tuple(data)])?
        };
        Ok(output)
    }
}


pub fn get_address_from_string(address: &str)->Web3Result<Address>{
    // info!("get_address_from_string={:?}",hex::decode(address)?);
    Ok(Address::from_slice(&hex::decode(address)?))
}

