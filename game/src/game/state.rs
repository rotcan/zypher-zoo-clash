use bevy_web3::plugin::{EthContract};
use bevy::{
    prelude::*,
};
use bevy_web3::types::{H160};
use bevy_web3::plugin::tokens::Uint;
use crate::web3::{PlayerState,MatchState,PlayerCardData,CardProp,GameContractIxType,EnvCard};
use std::collections::{HashMap,BTreeMap};
use crate::error::GameError;
use super::{Player};

#[derive(Debug,Clone,Default)]
pub struct PlayerScreenData{
    pub current_hands: BTreeMap<usize,CardProp>,
    pub active_card: Option<CardProp>,
    pub position: Option<usize>,
}

 

#[derive(Resource)]
pub struct Game{
    pub account: Option<String>,
    pub account_bytes: Option<H160>,
    pub players_data: Vec<PlayerGameData>,
    pub match_index: Uint,
    //pub other_players_data: Vec<PlayerGameData>,
    pub card_contract: EthContract,
    pub game_contract: EthContract,
    pub chain: u64,
    pub match_state: Option<MatchState>,
    pub selected_cards: Vec<Uint>,
    pub pkc : Vec<Uint>,
    pub init_reveal_key: bool,
    pub screen_data: HashMap<H160,PlayerScreenData>,
    pub current_revealed_hand_index: u64,
    pub player_index: Option<u64>,
    pub player_count: u64,
    pub current_round: Option<u64>,
    pub current_env_card: Option<usize>,
    pub env_cards: Vec<EnvCard>,
    pub match_finished: bool,
}

impl Game{

    

    pub fn get_current_env_index(&self)->Option<usize>{
        if let Some(ref match_state) = self.match_state{
            if let Some(ref card) = match_state.get_current_env_card() {
                return Some(card.as_usize())
            }else{
                return None;
            };
            
        };
        None
    }
    pub fn get_current_env_card(&self)->Option<EnvCard>{
        self.get_current_env_index().as_ref().map(|m| self.env_cards[*m].clone())
        // if let Some(ref match_state) = self.match_state{
        //     if let Some(ref card) = match_state.get_current_env_card() {
        //         return Some(self.env_cards[card.as_usize()].clone())
        //     }else{
        //         return None;
        //     };
            
        // };
        // None
    }
}

#[derive(Debug,Clone,Default)]
pub struct PlayerGameData{
    pub account: Option<String>,
    pub account_bytes: Option<H160>,
    pub all_cards: PlayerCardData,
    pub player_state: PlayerState,
}
 
impl PlayerGameData{
    pub fn new(account: String, account_bytes: H160, all_card_index: Vec<Uint>)->Self{
        let mut all_cards=PlayerCardData::default();
        all_cards.parse_all_cards(all_card_index);
        let mut player_state=PlayerState::default();
        player_state.player = account_bytes.clone();
        PlayerGameData{
            account: Some(account),
            account_bytes: Some(account_bytes),
            all_cards,
            player_state,
        }
    }
}
// #[derive(Component)]
// pub struct ContractCall(Task<CommandQueue>);

#[derive(Bundle)]
pub struct ContractResult{
    result: Text,
}
 


#[derive(Debug,PartialEq,Eq,Clone,Hash,Default,States)]
#[repr(i32)]
pub enum GameStatus{
    #[default]
    ConnectWallet, //Both
    InitPlayer, //Both
    CreateNewMatch, //Creator
    SetCreatorDeck, //Creator
    WaitingForPlayers, //Both
    JoinMatchPreSelect, //Other
    JoinMatch,  //Other
    WaitingForJointKey, //Other
    SetJointKeyPreSet, //Creator
    SetJointKey, //Creator,
    MaskAndShuffleEnvDeck, //Creator,
    ShuffleEnvDeck,//Other
    WaitingForPlayerToShuffleEnvDeck, //Both
    ShuffleYourDeck, //Both
    WaitingForPlayerToShuffleCards, //Both
    ShuffleOthersDeck, //Both
    RevealEnvCard, //Both
    RevealOtherPlayerCards, //Both
    WaitingForRevealCards, //Both
    PlayerAction, //Boths
    PlayCardOnDeck, //Both
    PopupShow, //Game
    PopupYes, //Game
    PopupNo, //Game
    PopupHide, //Game
    TxnWait, //Game
    PlayerOriginalCards, //Game
    EnvOriginalCards, //Game
    WaitingForOtherPlayerToPlayCard, //Both
    PlayerActionEnv, //Boths
    PlayerActionCard, //Boths
    Finished, //Both
}

impl GameStatus{
    pub fn value(&self)->String{
        match &self{
            GameStatus::ConnectWallet => "Connect Wallet".to_owned(),
            GameStatus::InitPlayer => "Init Player".to_owned(),
            GameStatus::CreateNewMatch => "Create New Game".to_owned(),
            GameStatus::SetCreatorDeck => "Set Creator Deck".to_owned(),
            GameStatus::WaitingForPlayers => "Waiting For Others".to_owned(),
            GameStatus::JoinMatchPreSelect => "Join Match".to_owned(),
            GameStatus::JoinMatch => "Join Match".to_owned(),
            GameStatus::WaitingForJointKey => "Waiting for Joint Key".to_owned(),
            GameStatus::SetJointKeyPreSet => "Set Joint Key".to_owned(),
            GameStatus::SetJointKey => "Set Joint Key".to_owned(),
            GameStatus::MaskAndShuffleEnvDeck => "Mask and submit Env deck".to_owned(), //Creator,
            GameStatus::ShuffleEnvDeck => "Shuffle and submit Env deck".to_owned(),//Other
            GameStatus::WaitingForPlayerToShuffleEnvDeck => "Waiting for others to shuffle env deck".to_owned(), //Both
            GameStatus::ShuffleYourDeck => "Mask and shuffle cards".to_owned(), //Both
            GameStatus::WaitingForPlayerToShuffleCards => "Waiting for others to shuffle cards".to_owned(), //Both
            GameStatus::ShuffleOthersDeck => "Shuffle other players cards".to_owned(), //Both
            GameStatus::RevealEnvCard => "Reveal env card".to_owned(), //Both
            GameStatus::RevealOtherPlayerCards => "Reveal other player card".to_owned(), //Both,
            GameStatus::WaitingForRevealCards => "Waiting for reveal cards".to_owned(), //Both
            GameStatus::PlayCardOnDeck => "Play Card".to_owned(),
            GameStatus::PlayerAction => "Player Action".to_owned(),
            GameStatus::PopupShow=>"PopupShow".to_owned(),
            GameStatus::PopupYes => "Ok".to_owned(),
            GameStatus::PopupNo => "Cancel".to_owned(),
            GameStatus::PopupHide=>"PopupHide".to_owned(),
            GameStatus::TxnWait => "Waiting for txn to finish".to_owned(),
            GameStatus::PlayerOriginalCards=>format!("Player Cards"),
            GameStatus::EnvOriginalCards=>format!("Env Cards"),
            GameStatus::WaitingForOtherPlayerToPlayCard => "Waiting for other player action".to_owned(),
            GameStatus::PlayerActionEnv => "Reveal Env Action".to_owned(),
            GameStatus::PlayerActionCard => "Play Card Action".to_owned(),
            GameStatus::Finished=>"Game Finished!".to_owned(),
        }
    }

    pub fn from_i32(val : i32)->Self{
        match val{
            0 => Self::ConnectWallet, //Both
            1 => Self::InitPlayer, //Both
            2 => Self::CreateNewMatch, //Creator
            3 => Self::SetCreatorDeck, //Creator
            4 => Self::WaitingForPlayers, //Both
            5 => Self::JoinMatchPreSelect, //Other
            6 => Self::JoinMatch,  //Other
            7 => Self::WaitingForJointKey, //Other
            8 => Self::SetJointKeyPreSet, //Creator
            9 => Self::SetJointKey, //Creator,
            10 => Self::MaskAndShuffleEnvDeck, //Creator,
            11 => Self::ShuffleEnvDeck,//Other
            12 => Self::WaitingForPlayerToShuffleEnvDeck, //Both
            13 => Self::ShuffleYourDeck, //Both
            14 => Self::WaitingForPlayerToShuffleCards, //Both
            15 => Self::ShuffleOthersDeck, //Both
            16 => Self::RevealEnvCard, //Both
            17 => Self::RevealOtherPlayerCards, //Both
            18 => Self::WaitingForRevealCards, //Both
            19 => Self::PlayCardOnDeck, //Both
            20 => Self::PlayerAction, //Both
            21 => Self::PopupShow, //Game
            22 => Self::PopupYes,
            23 => Self::PopupNo,
            24 => Self::PopupHide,
            25=>Self::TxnWait,
            26=>Self::PlayerOriginalCards,
            27 => Self::EnvOriginalCards,
            28 => GameStatus::WaitingForOtherPlayerToPlayCard,
            29 => Self::PlayerActionEnv,
            30 => Self::PlayerActionCard,
            31 => Self::Finished,
            _=>{panic!("GameStatus not mapping to i32")}
        }
    }
}


impl TryFrom<GameStatus> for GameContractIxType{
    type Error=GameError;
    fn try_from(status: GameStatus)->Result<Self,Self::Error>{
        match status{
            GameStatus::InitPlayer=>Ok(GameContractIxType::InitPlayer),
            GameStatus::CreateNewMatch=>Ok(GameContractIxType::CreateNewMatch),
            GameStatus::SetCreatorDeck=>Ok(GameContractIxType::SetCreatorDeck),
            GameStatus::JoinMatchPreSelect=>Ok(GameContractIxType::JoinMatchPreSelect),
            GameStatus::JoinMatch=>Ok(GameContractIxType::JoinMatch),
            GameStatus::SetJointKeyPreSet=>Ok(GameContractIxType::SetJointKeyPreSet),
            GameStatus::SetJointKey=>Ok(GameContractIxType::SetJointKey),
            GameStatus::MaskAndShuffleEnvDeck=>Ok(GameContractIxType::MaskAndShuffleEnvDeck),
            GameStatus::ShuffleEnvDeck=>Ok(GameContractIxType::ShuffleEnvDeck),
            GameStatus::ShuffleYourDeck=>Ok(GameContractIxType::ShuffleYourDeck),
            GameStatus::ShuffleOthersDeck=>Ok(GameContractIxType::ShuffleOthersDeck),
            GameStatus::RevealEnvCard=>Ok(GameContractIxType::RevealEnvCard),
            GameStatus::RevealOtherPlayerCards=>Ok(GameContractIxType::RevealOtherPlayerCards),
            GameStatus::PlayerActionEnv=>Ok(GameContractIxType::PlayerActionEnv),
            GameStatus::PlayerActionCard=>Ok(GameContractIxType::PlayerActionCard),
            GameStatus::PlayCardOnDeck=>Ok(GameContractIxType::PlayCardOnDeck),
            _=>Err(GameError::EnumError("Cannot convert gameStatus to ixType".to_owned()))  
        }
        
    }
}


#[derive(Event,Debug)]
pub enum UiUpdateEvent{
    UpdateCards{ cards: Vec<u64>},
    UpdateActiveCard{player: Player},
    UpdateEnv,
    UpdateTrack,
    UpdatePlayerLabel,
    UpdateWinningCard,
}
