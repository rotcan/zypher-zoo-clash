use web3::contract::{Contract,tokens::{Detokenize}
//    tokens::{Detokenize,Tokenizable},Error as Web3Error
};
use web3::api::{Eth,Namespace};
use crate::error::{Error,Web3Result};
use web3::types::{Address,H160,CallRequest};
use web3::transports::http::Http;
use async_channel::{unbounded, Receiver, Sender};
use web3::transports::eip_1193;
use bevy::{prelude::*,tasks::{TaskPool,IoTaskPool}};
//pub type EthContract = web3::contract::Contract<Http>;
pub mod tokens{
    pub use web3::contract::tokens::*;
    pub use web3::ethabi::*;
}


use crate::contract::tokens::{Token};
use std::fmt::Debug;
//use itertools::Itertools;

type HttpContract = web3::contract::Contract<Http>;
#[derive(Debug,Clone)]
pub struct EthContract{
    pub contract: HttpContract,
}

impl EthContract{

    pub fn load_contract(rpc_url: String, address: String, json: &[u8])->Web3Result<EthContract>{
        let address=if address.starts_with("0x") {
            address.strip_prefix("0x").unwrap()
        }else{
            &address
        };
        let address_bytes=Address::from_slice(&hex::decode(&address)?);
        let transport= Http::new(&rpc_url)?;
        let eth_api=Eth::new(transport);
        Ok(Self{
            contract: Contract::from_json(eth_api,address_bytes,json)?
        })
    }

    pub fn encode_data(&self, func: &str, params: Vec<Token>)->Web3Result<Vec<u8>>
    //where P : Tokenize
    {
        let func= self.contract.abi().function(func)?;
        //info!("func input params = {:?}",func.inputs);
        let tokens =params;
        //info!("tokens={:?}",&tokens);
        //info!("encode data params = {:?}",func.inputs.iter().map(|p| p.kind.clone()).collect::<Vec<_>>());
        let data = func.encode_input(&tokens)?;
        Ok(data)
    }
}


pub enum ContractChannelCmd {
    ContractResult{
        //result: Option<Box<dyn Detokenize + Send + 'static>>,
        result: Option<Token>,
        contract_address: H160,
        method_name: String, 
        error: Option<String>,
        params:Option<Vec<u8>>, },
        
}
 
#[derive(Resource,Debug)]
pub struct ContractChannelResource{
    contract_tx: Sender<ContractChannelCmd>,
    contract_rx: Receiver<ContractChannelCmd>,
}

pub fn init_contract_channel()->ContractChannelResource{
    let (contract_tx,contract_rx)=unbounded();

    ContractChannelResource{
        contract_tx,
        contract_rx,
    }
}

#[derive(Debug,Clone)]
pub struct RecvContractResponse{
    //pub result: Box<dyn Detokenize + Send + 'static>,
    pub result: Token,
    pub contract_address: H160,
    pub method_name: String,
    pub params:Option<Vec<u8>>,
} 

impl ContractChannelResource{
   
   
    pub fn call_contract_with_data(&self,contract: EthContract, method_name: String, data:  Vec<u8>)
    //where T: Detokenize + Tokenizable+ Send + 'static
    {
        let tx=self.contract_tx.clone();
        let address=contract.contract.address().clone();
        // let contract_clone=contract.clone();
        IoTaskPool::get_or_init(TaskPool::new)
            .spawn(async move {
                
                let provider = eip_1193::Provider::default().unwrap().unwrap();
                let transport = eip_1193::Eip1193::new(provider);
                let web3 = web3::Web3::new(transport);

                let mut call = CallRequest::default();
                call.to = Some(address);
                call.data = Some(data.clone().into());
                let bytes = web3.eth().call(call, None).await.unwrap();
                let result = contract.contract.abi().function(&method_name).unwrap().decode_output(&bytes.0);
                match result{
                    Ok(result)=>{
                        let output= if result.len() ==1{
                            Detokenize::from_tokens(result)
                        }else{
                            //Ok(output.into_tokens())
                            Detokenize::from_tokens(vec![Token::Tuple(result)])
                        };
                        // let res=match ret_type{
                        //     ContractCallReturnType::Vec => ContractReturnType::Vec(result),
                        //     ContractCallReturnType::Tuple=>ContractReturnType::Tuple(result),
                        // } ;
                        let output=output.unwrap();
                        let _ = tx.send(ContractChannelCmd::ContractResult{
                            //result:Some(ret_type.parse_data(result)),
                            result: Some(output),
                            contract_address: address,
                            method_name: method_name,
                            error:None,
                            params: Some(data)
                        }).await;
                    },
                    Err(err)=>{
                        let _ = tx.send(ContractChannelCmd::ContractResult{
                            result:None,
                            contract_address: address,
                            method_name: method_name,
                            error:Some(err.to_string()),
                            params: Some(data)
                        }).await;
                    }
                }
                
                
                // let _ = tx.send(ContractChannelCmd::GetCardsResult{
                //     result:"test".to_owned()
                // }).await;
            }).detach();
    }

    pub fn recv_contract(&self)->Web3Result<RecvContractResponse>{
        match self.contract_rx.try_recv(){
            Ok(ContractChannelCmd::ContractResult{result,error,contract_address,method_name,params})=>{
                if let Some(result) = result  {
                    Ok(RecvContractResponse{
                        result,
                        contract_address,
                        method_name
                        ,params
                    })
                }else {
                    if let Some(error)= error {
                        Err(Error::Web3Error(error))
                    }else{
                        Err(Error::Web3Error("unknown".to_owned()))
                    }
                }
            },
            Err(err)=>Err(err.into())
        }
    }
}

pub fn get_address_from_string(address: &str)->Web3Result<Address>{
    // info!("get_address_from_string={:?}",hex::decode(address)?);
    Ok(Address::from_slice(&hex::decode(address)?))
}

