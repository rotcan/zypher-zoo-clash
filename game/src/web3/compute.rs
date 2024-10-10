use bevy::prelude::*;
use super::{Point,process_pkc};
use crate::error::GameError;
use async_channel::{unbounded, Receiver, Sender};
use bevy::tasks::{AsyncComputeTaskPool};

#[derive(Debug)]
pub enum ComputeChannelCmd {
    ComputeResult{
        //result: Option<Box<dyn Detokenize + Send + 'static>>,
        result: Option<Vec<String>>,
        error: Option<GameError> 
    },
}
 
#[derive(Resource,Debug)]
pub struct ComputeChannelResource{
    compute_tx: Sender<ComputeChannelCmd>,
    compute_rx: Receiver<ComputeChannelCmd>,
}

#[derive(Debug,Clone)]
pub struct ComputeResultResponse{
    pub result: Option<Vec<String>>,
}


pub fn init_compute_resource()->ComputeChannelResource{
    let (compute_tx,compute_rx)=unbounded();

    ComputeChannelResource{
        compute_tx,
        compute_rx,
    }
}

impl  ComputeChannelResource{

    pub fn process_pkc_task(&self, point: Point){
        let tx=self.compute_tx.clone();
        let point = point.clone();
        AsyncComputeTaskPool::get()
        .spawn(
            async move {
            let result = process_pkc(point);
            match result{
                Ok(v)=>{
                    let _ = tx.send(ComputeChannelCmd::ComputeResult{
                        //result:Some(ret_type.parse_data(result)),
                        result: Some(v),
                        error:None,
                    }).await;
                },
                Err(err)=>{
                    let _ = tx.send(ComputeChannelCmd::ComputeResult{
                        //result:Some(ret_type.parse_data(result)),
                        result: None,
                        error:Some(err),
                    }).await;
                }
            }
            
        }).detach();
        //
    }

    pub fn recv_compute(&self)->Result<ComputeResultResponse,GameError>{
        match self.compute_rx.try_recv(){
            Ok(ComputeChannelCmd::ComputeResult{result,error})=>{
                if let Some(result) = result  {
                    Ok(ComputeResultResponse{
                        result: Some(result),
                    })
                }else {
                    if let Some(error)= error {
                        Err(GameError::ComputeError(error.to_string()))
                    }else{
                        Err(GameError::ComputeError("unknown".to_owned()))
                    }
                }
            },
            Err(err)=>Err(err.into())
        }
    }
}