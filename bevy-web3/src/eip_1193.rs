use std::{cell::RefCell,rc::Rc};
use wasm_bindgen::{prelude::*};
use crate::error::{self,Error};
use jsonrpc_core::{
    error::{Error as RPCError, ErrorCode as RPCErrorCode},
    types::request::{Call, MethodCall},
};
use serde_wasm_bindgen::Serializer;
use futures::{channel::mpsc, future::LocalBoxFuture, Stream};
use serde::{
    de::{value::StringDeserializer, DeserializeOwned, IntoDeserializer},
    Deserialize, Serialize,
};

#[derive(Clone, Debug)]
pub struct Eip1193 {
    provider: Rc<RefCell<Provider>>,
}

#[wasm_bindgen]
#[rustfmt::skip]
extern "C" {
    #[derive(Clone, Debug)]
    pub type Provider;
}


impl Provider {
    /// Get the provider at `window.ethereum`.
    pub fn default() -> Result<Option<Self>, JsValue> {
        get_provider_js()
    }

    fn parse_response(resp: Result<JsValue, JsValue>) -> error::Web3Result<serde_json::value::Value> {
        // Fix #544
        #[derive(Debug, Deserialize)]
        pub struct RPCErrorExtra {
            /// Code
            pub code: RPCErrorCode,
            /// Message
            pub message: String,
            /// Optional data
            pub data: Option<serde_json::value::Value>,
            /// Optional stack
            pub stack: Option<serde_json::value::Value>,
        }

        impl Into<RPCError> for RPCErrorExtra {
            fn into(self) -> RPCError {
                RPCError {
                    code: self.code,
                    message: self.message,
                    data: self.data,
                }
            }
        }

        let parsed_value = resp
            .map(deserialize_from_js)
            .map_err(deserialize_from_js::<RPCErrorExtra>);
        match parsed_value {
            Ok(Ok(res)) => Ok(res),
            Err(Ok(err)) => Err(Error::Rpc(err.into())),
            err => Err(Error::InvalidResponse(format!("{:?}", err))),
        }
    }

    async fn request_wrapped(&self, args: RequestArguments) -> error::Web3Result<serde_json::value::Value> {
        let response = self.request(args).await;
        Self::parse_response(response)
    }
}


#[wasm_bindgen(inline_js = "export function get_provider_js() {return window.ethereum}")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn get_provider_js() -> Result<Option<Provider>, JsValue>;
}
