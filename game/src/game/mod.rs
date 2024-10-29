use bevy_web3::plugin::{EthContract};
use bevy_web3::types::{ U256};
use std::collections::HashMap;

mod state;
mod assets;
mod ui;
mod api;
mod popup;

pub use self::state::*;
pub use self::assets::*;
pub use self::ui::*;
pub use self::api::*;
pub use self::popup::*;
 
pub const GAME_CARD_CONTRACT_ADDRESS: &str= "0xd239440f2de52f0468eaf8d85b793ec82fe400bb";
pub const GAME_CONTRACT_ADDRESS: &str= "0xe7e06d2b8cefa83b1cd56f42d722b15d960c3056";
 

impl Game{
    pub fn init()->Game{
        let rpc_url="https://opbnb-testnet-rpc.bnbchain.org".to_owned();
        let game_card_contract_address = GAME_CARD_CONTRACT_ADDRESS.to_owned();
        let game_address=GAME_CONTRACT_ADDRESS.to_owned();
        Game{
            card_contract: EthContract::load_contract(rpc_url.clone(),game_card_contract_address,
                include_bytes!("./res/game_card.json")).unwrap(),
            game_contract: EthContract::load_contract(rpc_url.clone(),game_address,
                include_bytes!("./res/game.json")).unwrap(),
            account: None,
            match_index: U256::zero(),
            chain:0,
            account_bytes: None,
            players_data: vec![],
            match_state: None,
            selected_cards: vec![],
            pkc: vec![],
            init_reveal_key: false,
            screen_data: HashMap::new(),
            current_revealed_hand_index: 0,
            player_index: None,
            player_count:1,
            env_cards:vec![],
            current_round:None,
            current_env_card:None,
            match_finished: false,
        }
    }

    pub fn test_contract(){

    }
}