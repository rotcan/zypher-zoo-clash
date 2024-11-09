use bevy::{prelude::*};
use bevy_web3::{types::{Address,U256}, plugin::EthContract};
use bevy_web3::plugin::tokens::{Uint,Token};
use crate::error::GameError;
use crate::game::{GAME_CARD_CONTRACT_ADDRESS,GAME_CONTRACT_ADDRESS,E1,E2,E3,G1,G2,G3,G4,G5,G6,RequestExtraData,RevealData};
use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap,HashMap};
use std::time::Duration;
use std::hash::{Hash,Hasher};

pub const VRF_MIN_BALANCE: u128=100000000000000u128;
pub const MIN_CARD_COUNT: u128 =20;
pub const DECK_SIZE: u64=20;
pub const WINNING_SCORE: u8 = 12;

#[derive(Clone,Eq,PartialEq,Debug,Hash,Default,States)]
pub enum WalletState{
    #[default]
    Disconnected,
    Connecting,
    Connected,
    
}

#[derive(Clone,Debug,PartialEq,Eq,Default,States)]
pub enum ContractState{
    #[default]
    None,
    Waiting,
    SendTransaction,
    
    TransactionResult{estimated_wait_time: Option<u32>},
    
}

impl Hash for ContractState{
    fn hash<H: Hasher>(&self, state: &mut H){
        match &self{
            Self::None=>{state.write_u8(0);},
            Self::Waiting=>{state.write_u8(1);},
            Self::SendTransaction=>{state.write_u8(2);},
            Self::TransactionResult{..}=>{
                state.write_u8(3);
            }
            
        };
    }
}


#[derive(Debug)]
pub enum CallContractParam{
    
    Data(Vec<u8>)
}

#[derive(Event,Debug)]
pub struct CallContractEvent {
    pub contract:EthContract,
    pub method: Web3ViewEvents,
    pub params: CallContractParam,
}

#[derive(Event,Debug)]
pub enum InGameContractEvent{
    JoinMatch,
    UpdateMatchIndex,
    UpdatePKC,
    UpdatePKCPopup,
    MaskAndShuffleEnvDeckPopup,
    MaskAndShuffleEnvDeck,
    ShuffleEnvDeckPopup,
    ShuffleYourDeckPopup,
    ShuffleOthersDeckPopup,
    ShuffleEnvDeck,
    ShuffleYourDeck,
    ShuffleOthersDeck,
    RevealMaskedCard{masked_card: HashMap<usize,Vec<String>>,extra_data: RequestExtraData,}
}

#[derive(Event,Debug)]
pub enum DelegateTxnSendEvent{
    LoadMatch,
    JoinMatch,
    SetJointKey,
    SetJointKeyPopup,
    MaskAndShuffleEnvDeckPopup,
    MaskAndShuffleEnvDeck,
    ShuffleEnvDeckPopup,
    ShuffleYourDeckPopup,
    ShuffleOthersDeckPopup,
    ShuffleEnvDeck,
    ShuffleYourDeck,
    ShuffleOthersDeck,
    RevealCard{ request_extra_data: RequestExtraData,
        reveal_map: HashMap<usize,RevealData>}
}


#[derive(Clone,Eq,PartialEq,Debug,Hash)]
#[repr(i32)]
pub enum ContractType{
    CardContract,
    GameContract
}

impl TryFrom<&str> for ContractType{
    type Error=GameError;

    fn try_from(val: &str)->Result<Self,Self::Error>{
        let updated_address= if val.starts_with("0x") {
            val.to_owned()
        }else{
            let address=format!("0x{}",val);
            address
        };
        
        match updated_address.as_str() {
            GAME_CARD_CONTRACT_ADDRESS => Ok(Self::CardContract),
            GAME_CONTRACT_ADDRESS => Ok(Self::GameContract),
            _=>Err(GameError::EnumError("Contract does not exist for this address".to_owned()))
        }
    }
}

// impl TryFrom<H160> for ContractType{

// }

#[derive(Clone,Eq,PartialEq,Debug,Hash)]
#[repr(i32)]
pub enum CardContractIxType{
    None
}

#[derive(Clone,Eq,PartialEq,Debug,Hash,Copy)]
#[repr(i32)]
pub enum GameContractIxType{
    InitPlayer,
    CreateNewMatch,
    JoinMatchPreSelect,
    JoinMatch,
    SetCreatorDeck,
    SetJointKeyPreSet,
    SetJointKey,
    MaskAndShuffleEnvDeck,
    ShuffleEnvDeck,
    ShuffleYourDeck,
    ShuffleOthersDeck,
    RevealEnvCard,
    RevealOtherPlayerCards,
    PlayCardOnDeck,
    PlayerActionEnv,
    PlayerActionCard,
}

impl GameContractIxType{
    pub fn get_ix_method(&self)->String{
        match &self{
            Self::InitPlayer=>"initPlayer".to_owned(),
            Self::CreateNewMatch=>"createNewMatch".to_owned(),
            Self::JoinMatch=>"joinMatch".to_owned(),
            Self::SetCreatorDeck=>"setCreatorDeck".to_owned(),
            Self::JoinMatchPreSelect=>"".to_owned(),
            Self::SetJointKeyPreSet=>"".to_owned(),
            Self::SetJointKey=>"setJointKey".to_owned(),
            Self::MaskAndShuffleEnvDeck=>"maskEnvDeck".to_owned(),
            Self::ShuffleEnvDeck=>"shuffleEnvDeck".to_owned(),
            Self::ShuffleYourDeck=>"submitDeck".to_owned(),
            Self::ShuffleOthersDeck=>"shuffleOtherDeck".to_owned(),
            Self::RevealEnvCard=>"showNextEnvCard".to_owned(),
            Self::RevealOtherPlayerCards=>"showOpponentCards".to_owned(),
            Self::PlayCardOnDeck=>"playCardOnDeck".to_owned(),
            Self::PlayerActionEnv=>"playerAction".to_owned(),
            Self::PlayerActionCard=>"playerAction".to_owned(),
        }
    }

    pub fn from_i32(val: i32)->Self{
        match val{
            0=>Self::InitPlayer,
            1=>Self::CreateNewMatch,
            2=>Self::JoinMatchPreSelect,
            3=>Self::JoinMatch,
            4=>Self::SetCreatorDeck,
            5=>Self::SetJointKeyPreSet,
            6=>Self::SetJointKey,
            7=>Self::MaskAndShuffleEnvDeck,
            8=>Self::ShuffleEnvDeck,
            9=>Self::ShuffleYourDeck,
            10=>Self::ShuffleOthersDeck,
            11=>Self::RevealEnvCard,
            12=>Self::RevealOtherPlayerCards,
            13=>Self::PlayCardOnDeck,
            14=>Self::PlayerActionEnv,
            15=>Self::PlayerActionCard,
            _=>panic!("Failed to parse num")
        }
    }
}


#[derive(Clone,Eq,PartialEq,Debug,Hash)]
#[repr(i32)]
pub enum CardContractViewActionType{
    GetAllCards,
    GetCardProp,
    GetPlayerCardProps, 
}

impl CardContractViewActionType{
    pub fn get_view_method(&self)->String{
        match &self{
            Self::GetAllCards=>"getAllCards".to_owned(),
            Self::GetCardProp=>"getCardProp".to_owned(),
            Self::GetPlayerCardProps=>"getPlayerCardProps".to_owned(),
        }
    }

    pub fn from(val: &str)->Self{
        match val{
            "getAllCards"=>Self::GetAllCards,
            "getCardProp"=>Self::GetCardProp,
            "getPlayerCardProps"=>Self::GetPlayerCardProps,
            _=>panic!("No matching type"),
        }
    }
 
}

#[derive(Clone,Eq,PartialEq,Debug,Hash)]
#[repr(i32)]
pub enum GameContractViewActionType{
    GetCurrentMatch,
    GetMatch,
    GetPlayerData,
    GetPkc,
    GetPlayerDataByIndex,
    GetMatchEnvCards,
}


impl GameContractViewActionType{
    pub fn get_view_method(&self)->String{
        match &self{
            Self::GetCurrentMatch=>"currentMatch".to_owned(),
            Self::GetMatch=>"matches".to_owned(),
            Self::GetPlayerData=>"getPlayerData".to_owned(),
            Self::GetPlayerDataByIndex=>"getPlayerDataByIndex".to_owned(),
            Self::GetPkc => "getPKC".to_owned(),
            Self::GetMatchEnvCards=>"getMatchEnvCards".to_owned(),
        }
    }

    pub fn from(val: &str)->Self{
        match val{
            "getPlayerData"=>Self::GetPlayerData,
            "matches"=>Self::GetMatch,
            "currentMatch"=>Self::GetCurrentMatch,
            "getPKC" => Self::GetPkc ,
            "getPlayerDataByIndex" =>Self::GetPlayerDataByIndex,
            "getMatchEnvCards"  =>Self::GetMatchEnvCards,
            _=>panic!("No matching type"),
        }
    }
 
}


#[derive(Debug,Eq,PartialEq,Clone)]
#[repr(i32)]
pub enum PopupResult{
    Yes(i32),
    No(i32)
}

#[derive(Debug,PartialEq)]
pub enum ActionType{
    Web3Actions(Web3Actions),
    GameActions(GameActions)
}


#[derive(Debug,PartialEq,Eq)]
pub enum Web3Actions{
    ConnectWallet,
    CardContractAction(CardContractIxType),
    GameContractAction(GameContractIxType),

}

#[derive(Debug,PartialEq,Eq)]
pub enum GameActions{
    PopupActions(PopupResult),
    ShowOriginalCards(usize),
    CopyMatchUrl,
}

#[derive(Debug,PartialEq)]
pub enum Web3ViewEvents{
    CardContractViewActionType(CardContractViewActionType),
    GameContractViewActionType(GameContractViewActionType),
}

//Game State
#[derive(Debug,Serialize,Deserialize,Default,Clone,Hash)]
pub struct Point {
    pub x: Uint,
    pub y: Uint
}

impl TryFrom<Token> for Point{
    type Error=GameError;
    fn try_from(val : Token)->Result<Self,Self::Error>{
        let val=val.into_tuple().ok_or(GameError::DataParseError("Point token should be tuple".to_owned()))?;
        if val.len() == 2 {
            Ok(Point{
                x: val[0].clone().into_uint().ok_or(GameError::DataParseError("Point x value missing".to_owned()))?,
                y: val[1].clone().into_uint().ok_or(GameError::DataParseError("Point y value missing".to_owned()))?,
            })
        }else{
            Err(GameError::DataParseError("Array size should be 2 for Point".to_owned()))
        }
    }
}

#[derive(Debug,PartialEq,Eq,Clone,Serialize,Deserialize,Hash)]
#[repr(u64)]
pub enum MatchStateEnum{
    None,
    //InitEnvDeck,
    SetPkc,
    ShuffleEnvDeck,
    SubmitSelfDeck,
    ShuffleOpponentDeck,
    RevealEnvCard,
    RevealPlayersHand,
    PlayerPlayCard,
    Finished,
}

impl Default for MatchStateEnum{
    fn default()->Self{
        MatchStateEnum::None
    }
}

impl TryFrom<u64> for MatchStateEnum{
    type Error=GameError;
    fn try_from(val: u64)->Result<Self,Self::Error>{
        match val{
            0=>Ok(Self::None),
            1=>Ok(Self::SetPkc),
            2=>Ok(Self::ShuffleEnvDeck),
            3=>Ok(Self::SubmitSelfDeck),
            4=>Ok(Self::ShuffleOpponentDeck),
            5=>Ok(Self::RevealEnvCard),
            6=>Ok(Self::RevealPlayersHand),
            7=>Ok(Self::PlayerPlayCard),
            8=>Ok(Self::Finished),
            _=>Err(GameError::EnumError("Failed to convert u64 to MatchStateEnum".to_owned()))
        }
    }
}

#[derive(Debug,Serialize,Deserialize,Default,Hash)]
pub struct MatchState{
    pub state: MatchStateEnum,
    //PlayerMatchData[] players;
    // pub players:PlayerState,
    pub player_count: u64,
    pub game_key:Point,
    // pub pkc: Vec<Uint>,
    pub player_turn: u64,
    pub player_turn_type: u64,
    pub reveal_env: u64,
    // pub is_finished: u64,
    pub turn_start: u64,
    pub rounds: u64,
    pub winner: Address,
    pub env_deck: EnvDeck, 
    pub creator: Address,
    pub winners_card: Uint,
}

fn extract_from_tuple(v: Token)->Vec<Token>{
    match v{
        Token::Tuple(v)=>v,
        Token::Array(v)=>v,
        _=>{vec![]}
    }
}

pub fn extract_vec(v: Token)->Option<Vec<Token>>{
    match v{
        Token::Tuple(v) | Token::Array(v) | Token::FixedArray(v)=>Some(v),
        _=>{None}
    }
}

fn get_uint_from_token(v: Token)->Uint{
    match v{
        Token::Uint(v)=>v,
        _=>{0.into()}
    }
}

fn get_address_from_token(v:Token)->Address{
    match v{
        Token::Address(v)=>v,
        _=>{panic!("Should be address")}
    }
}
impl From<Vec<Token>> for MatchState{
    fn from(item : Vec<Token>)->Self{
        //let item=val[0];
        let game_key=extract_from_tuple(item[2].clone());
        let inc=item.len()-12;
        // let pkc= if let Token::Uint(v)= item[3]{
        //     vec![]
        // }else{
        //     info!("Check PKC!");
        //     vec![]
        // };
        MatchState{
            state: MatchStateEnum::try_from(get_uint_from_token(item[0].clone()).as_u64()).unwrap(),
            player_count:  get_uint_from_token(item[1].clone()).as_u64(),
            game_key: Point{
                x: get_uint_from_token(game_key[0].clone()),
                y: get_uint_from_token(game_key[1].clone())
            },
            //pkc,
            player_turn:  get_uint_from_token(item[3+inc].clone()).as_u64(),
            player_turn_type:  get_uint_from_token(item[4+inc].clone()).as_u64(),
            reveal_env:  get_uint_from_token(item[5+inc].clone()).as_u64(),
            //is_finished:  get_uint_from_token(item[6+inc].clone()).as_u64(),
            turn_start:  get_uint_from_token(item[6+inc].clone()).as_u64(),
            rounds:  get_uint_from_token(item[7+inc].clone()).as_u64(),
            winner:  get_address_from_token(item[8+inc].clone()),
            env_deck: EnvDeck::from(extract_from_tuple(item[9+inc].clone())).into(),
            creator: get_address_from_token(item[10+inc].clone()),
            winners_card: get_uint_from_token(item[11+inc].clone()),
        }
    }
}
#[derive(Debug,Serialize,Deserialize,Default,Hash)]
pub struct EnvDeck{
    pub cards: Vec<Vec<Uint>>,
    pub env_reveals: Vec<Vec<Vec<Uint>>>,
    pub env_reveal_index: u64,
    pub env_board: Uint,
    pub shuffle_count: Uint,
    pub player_reveal_proof_index: Uint,
    
}

impl MatchState{
    pub fn is_env_revealed_by_player(&self, player_index: u64)->bool{
        if extract_uint_value(self.env_deck.shuffle_count,player_index as u32,1) == 1{
            true
        }else{
            false
        }
    }

    pub fn get_current_env_card(&self)->Option<Uint>{
        let state=&self.state ;
        let state=state.clone() as u64;
        if state > MatchStateEnum::RevealEnvCard as u64 {
            return Some(self.env_deck.env_board);
        };
        None
    }
}

impl From<Vec<Token>> for EnvDeck{
    fn from(val : Vec<Token>)->Self{
        let reveals=extract_from_tuple(val[1].clone());
        let reveals=reveals.iter()
        .map(|m| 
                extract_from_tuple(m.clone()) //Array to vec<Token>
                .iter()
                .map(|m2| 
                    extract_from_tuple(m2.clone()) //Array/Tuple to vec<Token>
                    .iter().
                    map(|m3| get_uint_from_token(m3.clone())) //Token to Uint
                    .collect::<Vec<Uint>>()
                ).collect::<Vec<Vec<Uint>>>()) 
                .collect::<Vec<Vec<Vec<Uint>>>>();
        // info!("env deck data = {:?}",val[3]);
        EnvDeck{
            cards: parse_token_to_vec_vec_uint(val[0].clone(),"Env card parsing error").unwrap(),
            env_reveals: reveals,
            env_reveal_index:  get_uint_from_token(val[2].clone()).as_u64(),
            env_board: get_uint_from_token(val[3].clone()),
            shuffle_count: get_uint_from_token(val[4].clone()),
            player_reveal_proof_index: get_uint_from_token(val[5].clone()),
        }
    }
}

impl EnvDeck{
    pub fn is_card_init(&self)->bool{
        if self.cards.len() >0 && self.cards[0].len()>0 && self.cards[0][0] != U256::zero() {
            return true;
        }
        false
    }
}
// #[derive(Debug,Serialize,Deserialize)]
// pub struct DiscardDeck{
//     pub cards: Vec<Vec<Uint>>,
//     pub card_index: Uint,
// }

#[derive(Debug,Serialize,Deserialize,Default,Clone,Hash)]
pub struct PlayerState{
    pub original_cards: Vec<Uint>,
    pub player: Address,
    pub done: bool,
    pub public_key: Point,
    pub player_index: u64,
    pub player_deck: Vec<Vec<Uint>>,
    pub player_reveals: Vec<Vec<Vec<Uint>>>,
    pub player_reveal_count: u64,
    pub player_reveal_proof_index: Uint,
    pub player_hand: Vec<u64>,
    pub player_hand_index: Uint,
    pub player_board: Uint,
    pub position: Uint,
    pub next_round_player_reveal_count: u64,
}

impl PlayerState{
    pub fn is_other_player_cards_revealed_by_player(&self,player_reveal_proof_index: u64)->bool{
        if extract_uint_value(player_reveal_proof_index.into(),self.player_index as u32,1) == 1{
            true
        }else{
            false
        }
    }

    pub fn are_cards_to_be_revealed(&self)->bool{
        if self.player_reveal_proof_index !=self.next_round_player_reveal_count.into() {
            true
        }else{
            false
        }
    }
}


impl TryFrom<Vec<Token>> for PlayerState{
    type Error=GameError;
    fn try_from(val : Vec<Token>)->Result<Self,Self::Error>{
        Ok(PlayerState{
            original_cards: parse_token_to_vec_uint(val[0].clone(), "original cards value missing")?,
            player: val[1].clone().into_address().ok_or(GameError::DataParseError("Address Value Missing".to_owned()))?,
            done: val[2].clone().into_bool().ok_or(GameError::DataParseError("Done Value Missing".to_owned()))?,
            public_key: Point::try_from(val[3].clone())?,
            player_index: val[4].clone().into_uint().ok_or(GameError::DataParseError("player_index Value Missing".to_owned()))?.as_u64(),
            player_deck: parse_token_to_vec_vec_uint(val[5].clone(), "player deck value missing")?,
            player_reveals:  parse_token_to_vec_vec_vec_uint(val[6].clone(),"player_reveals value missing")?,
            player_reveal_count: val[7].clone().into_uint().ok_or(GameError::DataParseError("player_reveal_count Value Missing".to_owned()))?.as_u64(),
            player_reveal_proof_index: val[8].clone().into_uint().ok_or(GameError::DataParseError("player_reveal_proof_index Value Missing".to_owned()))?,
            player_hand: parse_token_to_vec_uint(val[9].clone(),"player hand value missing")?
                .into_iter().map(|m| m.as_u64()).collect::<Vec<u64>>(),
            player_hand_index: val[10].clone().into_uint().ok_or(GameError::DataParseError("player_hand_index Value Missing".to_owned()))?,
            player_board: val[11].clone().into_uint().ok_or(GameError::DataParseError("player_board Value Missing".to_owned()))?,
            position: val[12].clone().into_uint().ok_or(GameError::DataParseError("position Value Missing".to_owned()))?,
            next_round_player_reveal_count: val[13].clone().into_uint().ok_or(GameError::DataParseError("next_round_player_reveal_count Value Missing".to_owned()))?.as_u64(),
        })
    }
}

impl PlayerState{
    
}

// impl PlayerState{
//     pub fn update_from(&mut self, player_state: &PlayerState){
//         self.original_cards=player_state.original_cards.clone();
//         self.player=player_state.player.clone();
//         self.done=player_state.done.clone();

//     }
// }

pub fn parse_token_to_vec_vec_uint(val: Token,field_name: &str )->Result<Vec<Vec<Uint>>,GameError>{
    let v2= extract_vec(val.clone()).ok_or(GameError::DataParseError(field_name.to_owned()))?; //Vec<Token>
    let mut v2_uint_array: Vec<Vec<Uint>>=vec![];
    for u in v2.iter(){
        let v3_uint_array: Vec<Uint> = parse_token_to_vec_uint(u.clone(),field_name)?;
        v2_uint_array.push(v3_uint_array);
    }
    Ok(v2_uint_array)
}

pub fn parse_token_to_vec_uint(val: Token,field_name: &str )->Result<Vec<Uint>,GameError>{
    let mut v1_uint_array: Vec<Uint> = vec![];
    let v1= extract_vec(val.clone()).ok_or(GameError::DataParseError(field_name.to_owned()))?; //Vec<Token>
    for w in v1.iter(){
        v1_uint_array.push(w.clone().into_uint().ok_or(GameError::DataParseError(field_name.to_owned()))?);
    }
    Ok(v1_uint_array)
}

fn parse_token_to_vec_vec_vec_uint(val: Token,field_name: &str)->Result<Vec<Vec<Vec<Uint>>>,GameError>{
    let val = val.into_array().ok_or(GameError::DataParseError(field_name.to_owned()))?; //Vec<Token>
    let mut final_val: Vec<Vec<Vec<Uint>>> = vec![];
    for v in val.iter(){
        let v2= v.clone().into_array().ok_or(GameError::DataParseError(field_name.to_owned()))?; //Vec<Token>
        let mut v2_uint_array: Vec<Vec<Uint>>=vec![];
        for u in v2.iter(){
            let v3_uint_array: Vec<Uint> = parse_token_to_vec_uint(u.clone(),field_name)?;
            v2_uint_array.push(v3_uint_array);
        }

        final_val.push(v2_uint_array);
    };
    Ok(final_val)
    
}

pub fn u256_to_string(v: U256)->String{
    let mut bytes: [u8;32] = [0u8;32];
    v.to_big_endian(&mut bytes);
    hex::encode(bytes)
}


pub fn u256_vec_to_string(v: Vec<U256>)->Vec<String>{
    v.into_iter().map(|m| u256_to_string(m)).collect::<Vec<String>>()
}

pub fn convert_vec_string_string_to_u256(data: Vec<Vec<String>>)->Vec<Vec<U256>>{
    let mut final_data: Vec<Vec<U256>>=vec![];
    for d in data.iter(){
        final_data.push(convert_vec_string_to_u256(d.clone()))
    }
    final_data
}

pub fn convert_vec_string_to_u256(data: Vec<String>)->Vec<U256>{
    let mut final_data :Vec<U256> = vec![];
    for d in data{
        final_data.push(convert_string_to_u256(d.clone()))
    }
   final_data
}

pub fn convert_string_to_u256(data: String)->U256{
    let bytes=hex::decode(data.trim_start_matches("0x")).unwrap();
    U256::from_big_endian(&bytes)
}

#[derive(Debug,Serialize,Deserialize,Default,Clone)]
pub struct PlayerCardData{
    pub all_cards: Vec<Uint>,
    pub all_card_props: BTreeMap<Uint,CardProp>,
}

impl PlayerCardData{
    pub fn parse_all_cards(&mut self,all_cards: Vec<Uint>){
        self.all_cards=all_cards;
    }

    pub fn parse_all_card_props(&mut self, all_cards: Vec<Uint>, all_card_props: Vec<Vec<Uint>>){
        let all_card_props=all_card_props.iter().map(|m| CardProp::from(m[0].clone())).collect::<Vec<CardProp>>();
        for it in all_cards.iter().zip(all_card_props.iter()) {
            let (index,val)=it;
            let mut val_copy=val.clone();
            val_copy.set_onchain_index(index.clone());
            self.all_card_props.insert(index.clone(),val_copy);
        }
       
    }
}

#[derive(Debug,Eq,PartialEq,Clone)]
#[repr(u8)]
pub enum EnvCardGeography{
    G1,//Forest,
    G2,//Mountain,
    G3,//Swamp,
    G4,//Desert,
    G5,//River,
    G6//Grass
}

impl EnvCardGeography{
    pub fn from_u8(val : u8)->Self{
        match val{
            0=>EnvCardGeography::G1,
            1=>EnvCardGeography::G2,
            2=>EnvCardGeography::G3,
            3=>EnvCardGeography::G4,
            4=>EnvCardGeography::G5,
            5=>EnvCardGeography::G6,
            _=>panic!("Env geography enum mismatch!"),
        }
    }
}

#[derive(Debug,Eq,PartialEq,Clone)]
#[repr(u8)]
pub enum EnvCardEnemy{
    E1,//Tiger,
    E2,//Crocodile,
    E3//Bear
}


impl EnvCardEnemy{
    pub fn from_u8(val : u8)->Self{
        match val{
            0=>EnvCardEnemy::E1,
            1=>EnvCardEnemy::E2,
            2=>EnvCardEnemy::E3,
            _=>panic!("Env enemy enum mismatch!"),
        }
    }
}

#[derive(Debug,Eq,PartialEq,Clone,)]
pub enum EnvCard{
    Geography(EnvCardGeography),
    Enemy(EnvCardEnemy),
}

impl EnvCardGeography{
    pub fn get_image_key(&self)->&str{
        match &self{
            Self::G1=>G1,
            Self::G2=>G2,
            Self::G3=>G3,
            Self::G4=>G4,
            Self::G5=>G5,
            Self::G6=>G6,
        }
    }
}

impl EnvCardEnemy{
    pub fn get_image_key(&self)->&str{
        match &self{
            Self::E1=>E1,
            Self::E2=>E2,
            Self::E3=>E3,
        }
    }
}

impl EnvCard{
    pub fn from(env_type: Uint, env_value: Uint)->EnvCard{
        let env_type = env_type.as_u32();
        let env_value= env_value.as_u32() as u8;
        match env_type{
            0=>EnvCard::Geography(EnvCardGeography::from_u8(env_value)),
            1=>EnvCard::Enemy(EnvCardEnemy::from_u8(env_value)),
            _=>panic!("Env card enum mismatch!"),
        }

    }

    pub fn get_image_key(&self)->Option<&str>{
        match &self{
            Self::Geography(val)=>Some(val.get_image_key()),
            Self::Enemy(val)=>Some(val.get_image_key()),
        }
    }
}

#[derive(Debug,Serialize,Deserialize,Default,Clone,PartialEq,Eq,)]
pub struct CardProp{
    pub rarity: u64,
    pub animal : u64,
    pub shield: u64,
    pub health: u64,
    pub weakness: Vec<u64>,
    pub favored_geographies: Vec<u64>,
    pub steps: u64,
    pub onchain_index: Uint,
}

fn extract_uint_value(val: Uint, shift_right : u32,limit: u8)->u64{
    let val=val.as_u128();
    let new_val=if let Some(val) = val.checked_shr(shift_right){
        val as u8
    }else{
        0u8
    };
    (new_val & limit) as u64
}

impl CardProp{
    fn parse_vec(val: U256, shift_right : u32)->Vec<u64>{
        let val=val.as_u128();
        let new_val=if let Some(val) = val.checked_shr(shift_right){
            val as u8
        }else{
            0u8
        };
        let mut v=vec![];
        for i in 0..8 {
            v.push(((new_val >> i) & 1) as u64);
        }
        v
        
    }

    fn parse_u64(val: Uint, shift_right : u32,limit: u8)->u64{
        extract_uint_value(val,shift_right,limit)
    }

    pub fn set_onchain_index(&mut self, index: Uint){
        self.onchain_index=index;
    }
}

impl From<Uint> for CardProp{

    fn from(val : Uint)->Self{
       
        CardProp{
            rarity: Self::parse_u64(val,28,7),
            animal: Self::parse_u64(val,22, 63),
            shield: 0,
            health: Self::parse_u64(val,19,7),
            weakness: Self::parse_vec(val,11),
            favored_geographies: Self::parse_vec(val,3),
            steps: Self::parse_u64(val,0,7),
            onchain_index: Uint::zero(),
        }
    }
}


#[derive(Debug,Eq,PartialEq,Clone)]
pub enum DelegateTxnResponseType{
    InitPlayer,
    CreateNewMatch,
    JoinMatch,
    SetJointKey,
    SetCreatorDeck,
}


#[derive(Component)]
pub struct ContractRequestTimer {
    /// track when the bomb should explode (non-repeating timer)
    pub timer: Timer,

}

impl Default for ContractRequestTimer{
    fn default()->Self{
        let mut timer = Timer::from_seconds(2.0,TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(2.0));
        info!("ContractRequestTimer default");
        ContractRequestTimer{
            timer: timer
        }
    }
}
 

#[derive(Component)]
pub struct TxnDataRequestTimer {
    /// track when the bomb should explode (non-repeating timer)
    pub timer: Timer,
    pub req_type: DelegateTxnResponseType,
}

impl Default for TxnDataRequestTimer{
    fn default()->Self{
        TxnDataRequestTimer {
            timer: Timer::from_seconds(5.0,TimerMode::Once),
            req_type: DelegateTxnResponseType::InitPlayer,
        }
    }
}

impl TxnDataRequestTimer{
    pub fn new(req_type: DelegateTxnResponseType)->Self{
        TxnDataRequestTimer{
            timer: Timer::from_seconds(10.0,TimerMode::Once),
            req_type
        }
    }
}

#[derive(Clone,Eq,PartialEq,Debug,Hash)]
#[repr(i32)]
pub enum PlayerAction{
    None,
    ShowEnvCard,
    ShowPlayerCard
}

impl PlayerAction{
    pub fn from_value(val: i32)->PlayerAction{
        match val {
            0 => Self::None,
            1 => Self::ShowEnvCard,
            2 => Self::ShowPlayerCard,
            _ => panic!("player action not present for this value")
        }
    }
}