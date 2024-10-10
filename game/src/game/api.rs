use bevy::{prelude::*,tasks::{TaskPool,IoTaskPool}};
use async_channel::{unbounded,Receiver,Sender};
use serde::{Deserialize,Serialize};
use std::collections::HashMap;

#[derive(Resource,Debug)]
pub struct AsyncHttpResource{
    http_tx:Sender<AsyncHttpCmd>,
    http_rx:Receiver<AsyncHttpCmd>,
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct RevealRequestData{
    pub secret_key: String,
    pub masked_card:HashMap<usize,Vec<String>>,
    
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct RevealData {
    pub card: (String, String),
    pub snark_proof: Vec<String>,
    
}
#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct RevealResponseData{
    pub data: HashMap<usize,RevealData>,
}

#[derive(Debug)]
pub struct AsyncHttpResponse{
    pub data: RevealResponseData,
    pub extra_data: RequestExtraData,
}

#[derive(Debug,Clone)]
pub enum RequestExtraData{
    RevealEnvCard{request_type: i32,secret_key: String},
    RevealOtherPlayersCard{request_type: i32, player_index: u64,reveal_count: u64, secret_key: String},
    PlayCardOnDeck{request_type: i32, hand_index: u64,secret_key: String}
}

type AsyncHttpResult<T> = Result<T, String>;

pub fn init_http_resource()->AsyncHttpResource{
    let (http_tx,http_rx)=unbounded();
    AsyncHttpResource{
        http_tx,http_rx
    }
}

#[derive(Debug,Clone)]
pub enum AsyncHttpCmd{
    RevealCmd{
        data: Option<RevealResponseData>,
        err: Option<String>,
        extra_data: RequestExtraData,
        // secret_key: String,
        // request_type: i32,
    }
}

impl AsyncHttpResource{
    pub fn send_http_call(&self,url: String,request_data: RevealRequestData,request_extra_data: RequestExtraData,){
        let tx=self.http_tx.clone();
        IoTaskPool::get_or_init(TaskPool::new)
        .spawn(async move{
            let client= reqwest::Client::new();
            info!("request_data = {:?}",request_data);
            let response = client.post(url.as_str()).json(
                &request_data
            )
            //.header("x-auth","xyz")
            .send().await;
            match response{
                Ok(response)=>{
                    let response_data = response.json::<RevealResponseData>().await;//serde_json::from_str(&.as_str()).unwrap();
                    match response_data{
                        Ok(data)=>{
//                            info!("response_data={:?}",data);
                            let _ = tx.send(AsyncHttpCmd::RevealCmd{data: Some(data),err:None,extra_data: request_extra_data}).await;
                        },
                        Err(err)=>{
                            //error!("error in response={:?}",err);
                            let _ = tx.send(AsyncHttpCmd::RevealCmd{data: None,err:Some(err.to_string()),extra_data: request_extra_data}).await;
                        }
                    }
                    
                },
                Err(err)=>{
                    error!("response error={:?}",err);
                    let _ = tx.send(AsyncHttpCmd::RevealCmd{data: None,err:Some(err.to_string()),extra_data: request_extra_data}).await;
                } 
            }
            
        }).detach();
    }

    pub fn recv_http_response(&self,)->AsyncHttpResult<AsyncHttpResponse>{
        match self.http_rx.try_recv(){
            Ok(AsyncHttpCmd::RevealCmd{data,err,extra_data})=>{
                if let Some(data)= data{
                    Ok(AsyncHttpResponse{data,extra_data})
                }else{
                    if let Some(err) = err {
                        Err(err)
                    }else{
                        Err("data empty".to_owned())
                    }
                    
                }
            },
            Err(err)=>{
                match err{
                    async_channel::TryRecvError::Empty=>{Err("empty".into())},
                    _=>{Err("closed".into())}
                }
            }
        }
    }
}