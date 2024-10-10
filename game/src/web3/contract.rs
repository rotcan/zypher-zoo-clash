use bevy::prelude::*;
use bevy_web3::{
    contract::{ContractChannelResource,RecvContractResponse,get_address_from_string,
        
        tokens::{ Token,Tokenizable,Uint},EthContract},
    error::Error as Web3Error,
};
use bevy_web3::types::{Address,U256};
use crate::game::{Game,GAME_CARD_CONTRACT_ADDRESS,GAME_CONTRACT_ADDRESS,PlayerGameData,GameStatus,PlayerScreenData,
    CardComponent,CardImages,PopupDrawEvent,PopupData,hide_popup,CardFace,CardComponentType,RequestExtraData,
    process_ui_query,UiElementComponent,RevealData,DeckCardType,Player,UiUpdateEvent,
    UiElement,BeforeGameElements,MenuData,InGameElements,};
use super::{ContractState,CardContractViewActionType,GameContractViewActionType,PlayerState,
    MIN_CARD_COUNT,Web3ViewEvents,MatchState,PopupResult,CardProp,EnvCard,vec_to_reveals,
    MatchStateEnum,parse_token_to_vec_vec_uint,extract_vec,
    parse_token_to_vec_uint,generate_key, vec_to_masked_card,
    u256_to_string,
    ContractRequestTimer,PlayerAction,unmask_card};
use bevy_pkv::PkvStore;
use crate::GameState;
use std::collections::HashMap;
//use zshuffle::utils::MaskedCard;

#[derive(Debug)]
pub enum CallContractParam{
    // Zero(Token),
    // Single(Token),
    // Multiple(Vec<u8>)
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


// #[derive(Event)]
// pub struct ContractResponse{
//     result: RecvContractResponse,
// }

// pub fn call_contract(contract_channel: Res<ContractChannelResource>,
// game: Res<Game>,
// mut next_state: ResMut<NextState<ContractState>>
// ){
//     // info!("call_contract");
//     contract_channel.call_contract(game.card_contract.clone());
//     next_state.set(ContractState::SendViewCall)
// }

pub fn do_contract_call(
    contract_channel: Res<ContractChannelResource>,
    // mut next_state: ResMut<NextState<ContractState>>,
    mut contract_call_events: EventReader<CallContractEvent>){
    for event in contract_call_events.read(){
        
        match &event.method {
            Web3ViewEvents::CardContractViewActionType(val)  =>{
                // next_state.set(ContractState::SendViewCall);
                match &event.params{
                  
                     
                    CallContractParam::Data(data)=>{
                        contract_channel.call_contract_with_data (event.contract.clone(),
                        val.get_view_method(),data.clone(),
                        
                    );
                    },
                }
                
            },
            Web3ViewEvents::GameContractViewActionType(val) =>{
            //     // next_state.set(ContractState::SendViewCall);
            
                match &event.params{
            
                    CallContractParam::Data(data)=>{
                        contract_channel.call_contract_with_data (event.contract.clone(),
                        val.get_view_method(),data.clone(),
                        
                    );
                    },  
                }
            }
        }
        //info!("event : {:?} , contract : {:?}",event.method,event.contract.contract.address());
    }
}
 
//Bevy system
pub fn recv_contract_response(contract_channel: ResMut<ContractChannelResource>,
    mut next_state: ResMut<NextState<ContractState>>,
    //state: Res<State<ContractState>>,
    mut game: ResMut<Game>,
    mut contract_events_writer: EventWriter<CallContractEvent>,
    mut commands: Commands,
    // mut menu_data: ResMut<MenuData>,
    mut q: Query<(Entity, &mut ContractRequestTimer)>,
    mut delegate_txn_event: EventWriter<DelegateTxnSendEvent>,
    mut match_events_writer: EventWriter<InGameContractEvent>,
    mut pkv: ResMut<PkvStore>,
    mut ui_query: Query<(Entity, &mut UiElementComponent)>,
    //asset_server : Res<AssetServer>
    card_images: Res<CardImages>,
    mut popup_draw_event: EventWriter<PopupDrawEvent>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_game_status: ResMut<NextState<GameStatus>>,
    mut ui_update_event: EventWriter<UiUpdateEvent>,

){
    // if let ContractState::SendViewCall = state.get(){
        match contract_channel.recv_contract() {
            Ok(ref res)=>{
                // info!("res={:?}",res);
                // next_state.set(ContractState::ViewCallResult);
                //info!("result={:?}",res.result);
                //info!("val={:?}",res);
                //let parsed_result: Vec<Token>=val.clone().to_vec();
                
                process_contract_response(&res,&mut contract_events_writer,&mut game,&mut commands,
                    //&mut menu_data,
                    &mut q,&mut delegate_txn_event,&mut match_events_writer, &mut pkv,&mut ui_query,
                &card_images,&mut popup_draw_event,&mut next_game_state,&mut next_game_status,&mut ui_update_event);
                
                
            },
            Err(Web3Error::ChannelEmpty)=>{},
            Err(Web3Error::Web3Error(msg))=>{
                error!("error={:?}",msg);
                next_state.set(ContractState::Waiting);
            },
            Err(err)=>{
                error!("unknown error : {:?}",err.to_string());
                next_state.set(ContractState::Waiting);
            }
        };
    // }
}

pub fn process_contract_response(res: &RecvContractResponse,
    contract_events_writer: &mut EventWriter<CallContractEvent>,
    game: &mut Game,
    commands: &mut Commands,
    // menu_data: &mut MenuData,
    q: &mut Query<(Entity, &mut ContractRequestTimer)>,
    delegate_txn_event: &mut EventWriter<DelegateTxnSendEvent>,
    match_events_writer: &mut EventWriter<InGameContractEvent>,
    pkv: &mut PkvStore,
    mut ui_query:&mut Query<(Entity, &mut UiElementComponent)>,
    //asset_server: &AssetServer,
    card_images: &CardImages,
    popup_draw_event: &mut EventWriter<PopupDrawEvent>,
    next_game_state: &mut NextState<GameState>,
    next_game_status:  &mut NextState<GameStatus>,
    ui_update_event: &mut EventWriter<UiUpdateEvent>,
    
    ){
        let contract_address=&res.contract_address;
        let method_name=&res.method_name;
        let params=&res.params;
        let address=format!("{}{}","0x",hex::encode(contract_address));
        // info!("process_contract_response address={:?} method_name={:?}",address,method_name);
        
    match address.as_str(){
        GAME_CARD_CONTRACT_ADDRESS=>{
            let method=CardContractViewActionType::from(method_name);
            match method{
                CardContractViewActionType::GetAllCards=>{
                    match &res.result{
                        Token::Array(all_cards)=>{
                            //Keep on doing call
                            if q.is_empty(){
                                commands.spawn_empty().insert(ContractRequestTimer::default());
                            }

                            
                            
                            if all_cards.len() >= MIN_CARD_COUNT as usize {
                            //if processed_cards < MIN_CARD_COUNT as usize {
                                params.as_ref().map(|m| {
                                    
                                    let data = game.card_contract.contract.abi()
                                        .function(CardContractViewActionType::GetAllCards.get_view_method().as_str()).unwrap()
                                        .decode_input(&m[4..]).unwrap(); 
                                    // let address=data[0]
                                    let account_bytes=data[0].clone().into_address().unwrap();
                                    let account = get_address_from_bytes(&account_bytes);

                                    // let processed_cards = if let Some(index) = get_player_index_by_address(&game, &account_bytes) {
                                    //     game.players_data[index].all_cards.all_card_props.len()
                                    // }else{
                                    //     0
                                    // };
                                    // if processed_cards == 0 {
                                        //Set all cards
                                        info!("process_contract_response data={:?}",data[0].clone());
                                        //let account_bytes=game.account_bytes.clone().unwrap();
                                        if let Some(index) = get_player_index_by_address(&game, &account_bytes) {
                                            game.players_data[index].all_cards.parse_all_cards(
                                                all_cards.clone().iter_mut().map(|m| m.clone().into_uint().unwrap()).collect::<Vec<U256>>());
                                        }else{
                                            let player_data=PlayerGameData::new(account.clone(), account_bytes.clone(),
                                            all_cards.clone().iter_mut().map(|m| m.clone().into_uint().unwrap()).collect::<Vec<U256>>());
                                            game.players_data.push(player_data);
                                        };
                                        //Get Card Props
                                        let data = game.card_contract.contract.abi()
                                        .function(CardContractViewActionType::GetPlayerCardProps.get_view_method().as_str()).unwrap()
                                        .encode_input(&[
                                            get_address_from_string(&account).unwrap().into_token()
                                        ]).unwrap(); 
                                        
                                        contract_events_writer.send(CallContractEvent{contract: game.card_contract.clone(), 
                                            method:Web3ViewEvents::CardContractViewActionType(CardContractViewActionType::GetPlayerCardProps), 
                                        //    params: CallContractParam::Data(get_address_from_string(&account).unwrap().into_token())
                                            params:CallContractParam::Data(data),
                                        });

                                        if game.match_finished == false{
                                            //Check And Get Current match of player
                                            let data = game.game_contract.contract.abi()
                                            .function(GameContractViewActionType::GetCurrentMatch.get_view_method().as_str()).unwrap()
                                            .encode_input(&[
                                                get_address_from_string(&account).unwrap().into_token()
                                            ]).unwrap(); 
                                            contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                                                method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetCurrentMatch), 
                                            //    params: CallContractParam::Data(get_address_from_string(&account).unwrap().into_token())
                                                params:CallContractParam::Data(data),
                                            });

                                            if let Some(ref match_state) = game.match_state{
                                                if match_state.player_count == 0 {
                                                    //update_status_area(&mut ui_query,commands,GameStatus::CreateNewMatch);
                                                    next_game_status.set(GameStatus::CreateNewMatch);
                                                }
                                            }else{
                                                //update_status_area(&mut ui_query,commands,GameStatus::CreateNewMatch);
                                                next_game_status.set(GameStatus::CreateNewMatch);
                                            };
                                        }
                                    // }
                                    
                                    
                                });
                                //Get All Cards
                                //game.account.as_ref().map(|account| {
                                // if let Some(ref account) = game.account
                                // {
                                    
                                // };
                                
                                
                                 
                                
                            }else{
                                
 
                            }
                            //let first_val= val.first().unwrap();
                            //info!("GetAllCards first val={:?} for {:?}",first_val,game.account);
                            
                            
                        },
                        _=>{},
                    }
        
                },
           
                CardContractViewActionType::GetPlayerCardProps=>{
                    match &res.result{
                        Token::Tuple(val)=>{
                            // info!("GetPlayerCardProps val={:?}",val);
                            //let address_bytes=game.account_bytes.clone().unwrap();
                            //Should be present
                            // if let Some(index)=get_player_index_by_address(&game, &address_bytes) {
                            params.as_ref().map(|m| {
                                
                                let data = game.card_contract.contract.abi()
                                    .function(CardContractViewActionType::GetPlayerCardProps.get_view_method().as_str()).unwrap()
                                    .decode_input(&m[4..]).unwrap(); 

                                let address_bytes=data[0].clone().into_address().unwrap();
                                // let account = get_address_from_bytes(&address_bytes);
                                let is_selectable= if address_bytes == game.account_bytes.clone().unwrap() {
                                    true
                                }else{
                                    false
                                };
                                if let Some(card_props) = extract_vec(val[1].clone()).take() {
                                    if let Some(index)=get_player_index_by_address(&game, &address_bytes) {
                                        let old_count = game.players_data[index].all_cards.all_card_props.keys().len();
                                        game.players_data[index].all_cards.parse_all_card_props(
                                            parse_token_to_vec_uint(val[0].clone(),"Get Player card index field issue").unwrap(),
                                            parse_token_to_vec_vec_uint(Token::Array(card_props),"Get Player card props field issue").unwrap()
                                        );

                                        let new_count = game.players_data[index].all_cards.all_card_props.keys().len();
                                        if game.match_finished == true{
                                            if new_count > MIN_CARD_COUNT as usize {
                                                //
                                                ui_update_event.send(UiUpdateEvent::UpdateWinningCard);
                                            }else{
                                                //info!("No new card added . Total cards = {:?}",new_count);
                                                // ui_update_event.send(UiUpdateEvent::UpdateWinningCard);
                                            }
                                        }
                                        let mut counter=0;
                                        for (card_index,card) in game.players_data[index].all_cards.all_card_props.iter(){
                                            //info!("card = {:?}",card);
                                            process_ui_query(&mut ui_query,commands,
                                                UiElement::BeforeGame(BeforeGameElements::Cards(index.clone(),
                                                counter/5
                                            )),
                                            None,Some(CardComponent::new(counter, Some(card_index.clone()),Some(DeckCardType::PlayerCard(card.clone())),is_selectable,CardFace::Up,CardComponentType::OriginalCards)),
                                            Some(&card_images),&game);
                                            counter+=1;
                                        }
                                    }
                                }
                                
                                //Todo! remove this layer
                                // if game.players_data[index].player_state.original_cards.len() == 0 {
                                //     game.selected_cards=game.players_data[index].all_cards
                                //                     .all_cards.clone().iter().take(20).map(|m| U256::from(m)).collect::<Vec<U256>>();
                                // }
                            // };
                            });
                        },
                        _=>{},
                    }
                },
                _=>{},
            }
        },
        GAME_CONTRACT_ADDRESS=>{
            let method=GameContractViewActionType::from(method_name);
            // info!("{method:?} res.result={:?}",res.result);
            match method{
                GameContractViewActionType::GetCurrentMatch=>{
                    match &res.result{
                        Token::Uint(val)=>{
                            info!("GetCurrentMatch val={:?} ",val);
                            //If match is there get match data
                            let match_index=val;
                            if match_index>&Uint::zero() {
                                game.match_index=match_index.clone();
                                 
                                if q.is_empty(){
                                    commands.spawn_empty().insert(ContractRequestTimer::default());
                                }
 
                               
                            }
                        },
                        _=>{},
                    }
                },
                GameContractViewActionType::GetMatch=>{
                    //info!("Get match");
                    match &res.result{
                        Token::Tuple(val)=>{
                            if val[1].clone().into_uint().unwrap().as_u64()>0 {
                                let address_bytes=game.account_bytes.clone().unwrap();
                                if let Some(index)=get_player_index_by_address(&game, &address_bytes) {
                                    let current_player_data=&game.players_data[index];
                                    if q.is_empty() && current_player_data.player_state.original_cards.len() > 0{
                                        commands.spawn_empty().insert(ContractRequestTimer::default());
                                    }
                                
                                }
                                
                                //Should be present
                                // info!("GetMatch val={:?}",val);
                                let match_state=MatchState::from(val.clone());
                                // let bytes=game.account_bytes.clone().unwrap();
                                // info!("Parsed Data= {:?}",match_state);
                                //if player_count > 0 && is_finished < 1, match state =0
                                game.player_count=match_state.player_count;
                                
                                
                                if game.match_state.is_none(){
                                    if *&match_state.player_count>0 && &match_state.state == &MatchStateEnum::None && *&match_state.is_finished<1{
                                        //Join Match
                                        if &match_state.creator != &address_bytes {
                                            delegate_txn_event.send(DelegateTxnSendEvent::JoinMatch);
                                        }
                                    }
                                }
                                game.match_state=Some(match_state);
                                
                                //process_state_update(game,pkv,match_events_writer,next_game_state,next_game_status);
                            }else{
                                //Show popup
                                popup_draw_event.send(
                                    PopupDrawEvent::ShowPopup(PopupData {
                                        msg: "Match does not exist".to_owned(),
                                        popup_type: GameStatus::JoinMatchPreSelect as i32,
                                        action_yes: Some(PopupResult::Yes(GameStatus::PopupHide as i32)),
                                        action_no : None,
                                    })
                                );
                            }
                            
                        },
                        _=>{},
                    }
                },
                GameContractViewActionType::GetPlayerData |
                GameContractViewActionType::GetPlayerDataByIndex=>{
                    match &res.result{
                        Token::Tuple(val)=>{
                            //Todo! insert other players in game players_data
                            // info!("GetPlayerState val={:?}",val);
                            match PlayerState::try_from(val.clone()) {
                                Ok(contract_player_state)=>{
                                    // info!("Parsed Player Data = {contract_player_state:?}");
                                    update_game_with_player_data(game, &contract_player_state);

                                    //Get All cards once for other players
                                    if get_player_index_by_address(&game, &contract_player_state.player).is_none() {
                                        //Get cards
                                        let data = game.card_contract.contract.abi()
                                        .function(CardContractViewActionType::GetAllCards.get_view_method().as_str()).unwrap()
                                        .encode_input(&[
                                            contract_player_state.player.into_token()
                                        ]).unwrap(); 
                                        contract_events_writer.send(CallContractEvent{contract: game.card_contract.clone(), 
                                            method:Web3ViewEvents::CardContractViewActionType(CardContractViewActionType::GetAllCards), 
                                            
                                            params: CallContractParam::Data(data),
                                            });
                                    }

                                    process_state_update(game,pkv,match_events_writer,next_game_state,next_game_status,&contract_player_state.player,
                                    ui_update_event,popup_draw_event);
                                    
                                      
                                },
                                Err(err)=>{
                                    error!("Error in parsing player data {err:?}");
                                }
                            }
                        },
                        _=>{},
                    }
                },
                GameContractViewActionType::GetPkc=>{
                    match parse_token_to_vec_uint(res.result.clone(),"Pkc field parse error"){
                        Ok(v)=>{
                            game.pkc=v;
                        },
                        Err(e)=>{
                            error!("Error in getting pkc data {e:?}" );
                        }
                    }
                     
                },
                GameContractViewActionType::GetMatchEnvCards=>{
                    //info!("Get env result= {:?} for parms {:?} ",res.result,res.params);
                    match &res.result{
                        Token::Array(val)=>{
                            //info!("Get match env cards = {:?}",val);
                            if game.env_cards.len() == 0{
                                for card in val.into_iter(){
                                    match card{
                                        Token::Tuple(card)=>{
                                            game.env_cards.push(EnvCard::from(card[0].clone().into_uint().unwrap(),
                                            card[1].clone().into_uint().unwrap()));
                                        },
                                        _=>{},
                                    }
                                    
                                }
                            }
                            
                        },
                        _=>{},
                    }
                }
            }
        },
        _=>{},
    }
}

pub fn process_state_update(game: &mut Game, pkv: &mut PkvStore,
    match_events_writer: &mut EventWriter<InGameContractEvent>,
    next_game_state: &mut NextState<GameState>,
    next_game_status:  &mut NextState<GameStatus>,updated_player_address: &Address,
ui_update_event: &mut EventWriter<UiUpdateEvent>,
popup_draw_event: &mut EventWriter<PopupDrawEvent>,
// timer_query: &mut Query<(Entity, &mut ContractRequestTimer)>,
){
 
        if let Some(ref address_bytes)=game.account_bytes{

            // let player_address = format!("0x{:?}",hex::encode(address_bytes));
            
            let player_key=generate_key(&address_bytes, pkv);
            
            //Todo!
            //Update other players data on screen
            if let Some(ref match_state)=game.match_state{
                //All players
                
                if let Some(index)=get_player_index_by_address(&game, updated_player_address) {
                    let player_data = &game.players_data[index];
                    if game.screen_data.contains_key(&updated_player_address) == false {
                        game.screen_data.insert(updated_player_address.clone(), PlayerScreenData::default());
                    }
                    
                    let match_state_u64=match_state.state.clone() as u64;
                    if match_state_u64>=MatchStateEnum::SetPkc as u64{
                        if let Some(player_screen_data) = game.screen_data.get_mut(&updated_player_address){
                            if let Some(position) = player_screen_data.position {
                                if position != player_data.player_state.position.as_usize() {
                                    ui_update_event.send(UiUpdateEvent::UpdateTrack);
                                    ui_update_event.send(UiUpdateEvent::UpdatePlayerLabel);
                                }
                            }else{
                                ui_update_event.send(UiUpdateEvent::UpdateTrack);
                                ui_update_event.send(UiUpdateEvent::UpdatePlayerLabel);
                            }
                            player_screen_data.position=Some(player_data.player_state.position.as_usize());
                        }
                    }
                    if match_state_u64>=MatchStateEnum::RevealPlayersHand as u64
                    {
                        if let Some(player_screen_data) = game.screen_data.get_mut(&updated_player_address){
                            let before_hand_count= player_screen_data.current_hands.len();
                            
                            if updated_player_address==address_bytes{
                                //Hand cards
                                for hand in player_data.player_state.player_hand.iter() {
                                    let hand_idx=*hand as usize;
                                    if player_screen_data.current_hands.contains_key(&hand_idx) == false{
                                        let card=vec_to_masked_card(&player_data.player_state.player_deck[hand_idx]);
                                        let reveals=player_data.player_state.player_reveals[hand_idx].iter().map(|m| (u256_to_string(m[0]),
                                        u256_to_string(m[1]))).collect::<Vec<(String,String)>>();
                                        let unmasked_card=unmask_card(player_key.sk.clone(),card,reveals).unwrap() ;
                                        let original_card=player_data.player_state.original_cards[unmasked_card as usize];
                                        if let Some(card_prop)=player_data.all_cards.all_card_props.get(&original_card) 
                                        {
                                            player_screen_data.current_hands.insert(hand_idx,card_prop.clone());
                                        };
                                        
                                    }
                                }
                                //player board
                                if match_state_u64>=MatchStateEnum::PlayerPlayCard as u64{
                                    if let Some(card) = player_data.all_cards.all_card_props.get(&player_data.player_state.player_board) {
                                        if let Some(ref screen_card) = player_screen_data.active_card{
                                            if screen_card != card {
                                                ui_update_event.send(UiUpdateEvent::UpdateActiveCard{player: Player::Player1});
                                            }
                                        }else{
                                            ui_update_event.send(UiUpdateEvent::UpdateActiveCard{player: Player::Player1});
                                        }
                                        player_screen_data.active_card=Some(card.clone());
                                    }
                                    
                                }
                            }else{
                                //player board
                                if match_state_u64>=MatchStateEnum::PlayerPlayCard as u64{
                                    if let Some(card) = player_data.all_cards.all_card_props.get(&player_data.player_state.player_board) {
                                        if let Some(ref screen_card) = player_screen_data.active_card{
                                            if screen_card != card {
                                                ui_update_event.send(UiUpdateEvent::UpdateActiveCard{player: Player::Player2});
                                            }
                                        }else{
                                            ui_update_event.send(UiUpdateEvent::UpdateActiveCard{player: Player::Player2});
                                        }
                                        player_screen_data.active_card=Some(card.clone());
                                    }
                                    
                                }
                                // if player_data.player_state.player_board > Uint::zero() {
                                //     if player_screen_data.current_hands.contains_key(&0) == false{
                                //         if let Some(card_prop)=player_data.all_cards.all_card_props.get(&player_data.player_state.player_board) 
                                //         {
                                //             player_screen_data.current_hands.insert(0,card_prop.clone());
                                //         };
                                //     }
                                // }
                            }

                            //delete extra cards
                            let keys_to_delete=player_screen_data.current_hands.clone().into_keys()
                            .collect::<Vec<usize>>()
                            .into_iter()
                            .filter(|f| player_data.player_state.player_hand.contains(&(*f as u64))==false)
                            .collect::<Vec<usize>>();
    
                            for key in keys_to_delete.iter(){
                                player_screen_data.current_hands.remove_entry(key );
                            };
                            
                            if  player_screen_data.current_hands.len() != before_hand_count{
                                //Update ui
                                //Send event to update ui
                                if updated_player_address==address_bytes{
                                    ui_update_event.send(UiUpdateEvent::UpdateCards{cards: player_data.player_state.player_hand.clone()});
                                }
                            }

                            
                            
                        }
                        //Update Track
                        if let Some(round) = game.current_round {
                            if round != match_state.rounds {
                                //Update track
                                ui_update_event.send(UiUpdateEvent::UpdateTrack);
                            }
                        }else{
                            //Update track
                            ui_update_event.send(UiUpdateEvent::UpdateTrack);
                        }
                        game.current_round= Some(match_state.rounds);

                        //Update Env
                        let game_env_index=game.get_current_env_index();
                        game_env_index.as_ref().map(|m| {
                            if let Some(ref env) = game.current_env_card {
                                if env != m {
                                    ui_update_event.send(UiUpdateEvent::UpdateEnv);
                                }
                            }else{
                                ui_update_event.send(UiUpdateEvent::UpdateEnv);
                            }
                            game.current_env_card=Some(*m);
                        });

                        //Todo! Add play card code
                    }
                        
                        
                }

                //Current player
                if let Some(index)=get_player_index_by_address(&game, address_bytes) {
                    //game.players_data[index].player_state=contract_player_state;
                    let current_player_data=&game.players_data[index];
                    if current_player_data.player_state.original_cards.len() == 0 {
                        //Set creator deck
                        //info!("Set Creator deck! index={:?}",index);
                        
                        //update_status_area(&mut ui_query,commands,GameStatus::SetCreatorDeck);
                        if current_player_data.player_state.player != match_state.creator{
                            next_game_status.set(GameStatus::CreateNewMatch);
                        }else{
                            next_game_status.set(GameStatus::SetCreatorDeck);
                        }
                        
                        
                    }else{
                        //Already set 
                        //info!("match_state={:?}",match_state.state);
                        let match_state_u64=match_state.state.clone() as u64;
                        if match_state_u64>=MatchStateEnum::SetPkc as u64{
                            next_game_state.set(GameState::GameStart);
                        };
                        if match_state_u64 >= MatchStateEnum::ShuffleEnvDeck as u64 
                        && match_state_u64<MatchStateEnum::RevealEnvCard as u64{
                            if game.pkc.len() == 0 {
                                //load pkc
                                
                                popup_draw_event.send(
                                    PopupDrawEvent::ShowPopup(PopupData {
                                        msg: format!("Waiting to init joint key "),
                                        popup_type: GameStatus::SetJointKeyPreSet as i32,
                                        action_yes: None,
                                        action_no : None,
                                    })
                                );

                                match_events_writer.send(InGameContractEvent::UpdatePKCPopup);
                                //To stop call again
                                info!("UpdatePKC send");
                                game.pkc=vec![U256::zero()];
                            }
                        };
                        if match_state_u64>=MatchStateEnum::RevealEnvCard as u64{
                            if game.init_reveal_key ==false{
                                //init_reveal_key().unwrap();
                                game.init_reveal_key=true;
                            };  
                        };
                        match match_state.state {
                            MatchStateEnum::None=>{
                                if match_state.player_turn != match_state.player_count {
                                    next_game_status.set(GameStatus::WaitingForPlayers);
                                }
                            },
                            //Change to playing area in ui from this state
                            MatchStateEnum::SetPkc=>{
                                if &match_state.creator == address_bytes{
                                    //Set Joint key
                                    next_game_status.set(GameStatus::SetJointKeyPreSet);
                                    
                                }else{
                                    //Wait
                                    next_game_status.set(GameStatus::WaitingForJointKey);
                                }
                            },
                            MatchStateEnum::ShuffleEnvDeck=>{
                                //Pkc Set , shuffle
                                if &match_state.creator == address_bytes {
                                    
                                        if match_state.env_deck.is_card_init() == false{
                                            next_game_status.set(GameStatus::MaskAndShuffleEnvDeck);
                                        }else{
                                            next_game_status.set(GameStatus::WaitingForPlayerToShuffleEnvDeck);
                                        }
                                    
                                }else{
                                    if match_state.env_deck.is_card_init() == true{
                                        //Shuffle 
                                        next_game_status.set(GameStatus::ShuffleEnvDeck);
                                    }else{
                                        next_game_status.set(GameStatus::WaitingForPlayerToShuffleEnvDeck);
                                    }
                                }
                            },
                            MatchStateEnum::SubmitSelfDeck=>{
                                //Pkc Set , shuffle
                                
                                if current_player_data.player_state.done == false{
                                    next_game_status.set(GameStatus::ShuffleYourDeck);
                                }else{
                                    next_game_status.set(GameStatus::WaitingForPlayerToShuffleCards);
                                }
                                
                            
                            },
                            MatchStateEnum::ShuffleOpponentDeck=>{
                                
        
                            //Pkc Set , shuffle
                            //info!("current_player_data.player_state.done={:?} {:?}",current_player_data.player_state.done,current_player_data.player_state.player ); 
                                if current_player_data.player_state.done == false{
                                    next_game_status.set(GameStatus::ShuffleOthersDeck);
                                }else{
                                    next_game_status.set(GameStatus::WaitingForPlayerToShuffleCards);
                                }
                                
                            
                            },
                            MatchStateEnum::RevealEnvCard=>{
                                
                                //if current_player_data.player_state.done == false{
                                if match_state.is_env_revealed_by_player(current_player_data.player_state.player_index) == false{
                                    next_game_status.set(GameStatus::RevealEnvCard);
                                    
                                }else{
                                    next_game_status.set(GameStatus::WaitingForRevealCards);
                                    
                                }
                                
                                
                            },
                            MatchStateEnum::RevealPlayersHand=>{
                                
                                let others=are_other_players_card_revealed_by_current_player(&game, &address_bytes);
                                if others.len() > 0{
                                    // update_status_area(&mut ui_query,commands,GameStatus::RevealOtherPlayerCards);
                                    next_game_status.set(GameStatus::RevealOtherPlayerCards);
                                }else{
                                    //update_status_area(&mut ui_query,commands,GameStatus::WaitingForRevealCards);
                                    next_game_status.set(GameStatus::WaitingForRevealCards);
                                }
                                 
                                
                            },
                            MatchStateEnum::PlayerPlayCard=>{
                                //printHandCards
                                //Todo remove

                                // game.match_finished=true;
                                // // next_game_state.set(GameState::GameEnd);
                                // popup_draw_event.send(
                                //     PopupDrawEvent::ShowMatchEndPopup
                                // );
                                
                                //Current players turn
                                let player_action  = PlayerAction::from_value(match_state.player_turn_type as i32);
                                if current_player_data.player_state.player_index == match_state.player_turn {
                                    
                                    match player_action {
                                        PlayerAction::None=>{
                                            //Show player action
                                            next_game_status.set(GameStatus::PlayerAction);
                                        },
                                        PlayerAction::ShowEnvCard=>{
                                            //Show env card
                                            if match_state.is_env_revealed_by_player(current_player_data.player_state.player_index) == false{
                                                next_game_status.set(GameStatus::RevealEnvCard);
                                            }else{
                                                next_game_status.set(GameStatus::WaitingForRevealCards);
                                            }
                                        },
                                        PlayerAction::ShowPlayerCard=>{
                                            //Player player card
                                            if current_player_data.player_state.player_reveal_count != current_player_data.player_state.next_round_player_reveal_count {
                                                //First reveal then play card on deck
                                                next_game_status.set(GameStatus::WaitingForRevealCards);
                                            }else{
                                                next_game_status.set(GameStatus::PlayCardOnDeck);
                                            }
                                        },
                                    }
                                }else{
                                    match player_action {
                                        PlayerAction::None=>{
                                            //Show player action
                                            next_game_status.set(GameStatus::WaitingForOtherPlayerToPlayCard);
                                        },
                                        PlayerAction::ShowEnvCard=>{
                                            //Show env card
                                            if match_state.is_env_revealed_by_player(current_player_data.player_state.player_index) == false{
                                                next_game_status.set(GameStatus::RevealEnvCard);
                                            }else{
                                                next_game_status.set(GameStatus::WaitingForOtherPlayerToPlayCard);
                                            }
                                        },
                                        PlayerAction::ShowPlayerCard=>{
                                            //Player player card
                                            if are_other_players_card_need_to_be_revealed(&game,&address_bytes).len() > 0{
                                                //Reveal cards
                                                next_game_status.set(GameStatus::RevealOtherPlayerCards);
                                            }else{
                                                next_game_status.set(GameStatus::WaitingForOtherPlayerToPlayCard);
                                            }
                                            //First reveal then play card on deck
                                            
                                        },
                                    }
                                    //Wait for your turn
                                    
                                }
                                
                                //Play cards
                                //
                                
                            },
                            MatchStateEnum::Finished=>{
                                //Show popup , with create / join
                                //This will keep on going
                                //Do a loop for get all cards
                                
                                game.match_finished=true;
                                // next_game_state.set(GameState::GameEnd);
                                popup_draw_event.send(
                                    PopupDrawEvent::ShowMatchEndPopup
                                );
                            },
                        }
                        
                    }
                }
            }

            
            
        }
       
}

fn update_game_with_player_data(game: &mut Game,player_state: &PlayerState){
    // info!("player_state.player={:?}",player_state.player);
    // let address_bytes=Address::from_slice(&hex::decode(&player_state.player.clone().strip_prefix("0x").unwrap()).unwrap());
    if let Some(index) = get_player_index_by_address(&game, &player_state.player) {
        game.players_data[index].player_state = player_state.clone();
    } 
}
 
//Bevy system
pub fn handle_contract_state_change(contract_state: Res<State<ContractState>>,
    mut popup_draw_event: EventWriter<PopupDrawEvent>,
    mut menu_data: ResMut<MenuData>,
    mut commands: Commands){
        match contract_state.get(){
            ContractState::SendTransaction=>{
                //show popup
                popup_draw_event.send(
                    PopupDrawEvent::ShowPopup(PopupData {
                        msg: GameStatus::TxnWait.value(),
                        popup_type: GameStatus::TxnWait as i32,
                        action_yes: None,
                        action_no : None,
                    })
                );
            },
            ContractState::Waiting |
            ContractState::TransactionResult=>{
                //hide popup
                hide_popup(&mut menu_data, &mut commands);
            },
        }
}

//Bevy System
pub fn handle_game_status_change(game_status: Res<State<GameStatus>>,
    mut ui_query: Query<(Entity, &mut UiElementComponent)>,
    // mut menu_data: ResMut<MenuData>,
    mut commands: Commands,
    game: Res<Game>,
){
    update_status_area(&game, &mut ui_query,&mut commands,game_status.get().clone());
    
    
}


pub fn get_player_index_by_address<'a>(game: &'a Game, address_bytes : &'a Address)->Option<usize>{
    let val = game.players_data.iter()
        .enumerate()
        .filter(|&(_i,m)| &m.player_state.player==address_bytes)
        .map(|(i,_)| i.clone())
        .collect::<Vec<usize>>();
    if let Some(v) = val.first(){
        return Some(*v);
    };
    None
}

pub fn get_player_data_by_address<'a>(game: &'a Game, address_bytes: &'a Address)->Option<&'a PlayerGameData>{
    if let Some(index)=get_player_index_by_address(&game,&address_bytes){
        return Some(&game.players_data[index]);
    };
    None
}

pub fn get_player_addresses<'a>(game: &'a Game)->Vec<Address>{
    let mut addresses:Vec<Address> = vec![];
    for player_data in game.players_data.iter(){
        if let Some(ref address_bytes)=player_data.account_bytes{
            addresses.push(address_bytes.clone());
        }
    }
    addresses
}
 
pub fn get_player_index_not_by_address<'a>(game: &'a Game, address_bytes: &'a Address)->Vec<usize>{
    let val = game.players_data.iter()
        .enumerate()
        .filter(|&(_i,m)| &m.player_state.player!=address_bytes)
        .map(|(i,_)| i.clone())
        .collect::<Vec<usize>>();
    val
}

pub fn get_player_hand_cards<'a>(game: &'a Game, address_bytes: &'a Address,private_key: String)->Vec<CardProp>{
    let mut revealed_cards: Vec<CardProp>=vec![];
    if let Some(ref player_data)=get_player_data_by_address(&game,&address_bytes){
        let player_state=&player_data.player_state;
        let hands= &player_state.player_hand;
        let reveals=&player_state.player_reveals;
        let original_cards=&player_state.original_cards;
        for hand in hands.iter(){
            let hand_index=*hand as usize;
            let masked_card= vec_to_masked_card(&player_state.player_deck[hand_index]);
            let reveal=vec_to_reveals(&reveals, hand_index);
            let unmasked_card = unmask_card(private_key.clone(),masked_card.clone(),reveal).unwrap();
            let original_card=original_cards[unmasked_card as usize];
            if let Some(card_prop)= player_data.all_cards.all_card_props.get(&original_card){
                revealed_cards.push(card_prop.clone())
            }
            
        };
        
    };
    revealed_cards
}

pub fn get_player_board_card<'a>(game: &'a Game, address_bytes: &'a Address,)->Option<CardProp>{
    if let Some(ref player_data)=get_player_data_by_address(&game,&address_bytes){
        let player_state=&player_data.player_state;
        let card=player_state.player_board;
        if let Some(card_prop)= player_data.all_cards.all_card_props.get(&card){
            return Some(card_prop.clone());
        };
    };
    None
}

pub fn get_player_hidden_cards<'a>(game: &'a Game, address_bytes: &'a Address)->usize{
    let count = if let Some(ref player_data)=get_player_data_by_address(&game,&address_bytes){
        player_data.player_state.player_deck.len() - player_data.player_state.player_reveal_count as usize
    }else{
        0
    };
    count
}

pub fn are_other_players_card_revealed_by_current_player<'a>(game: &'a Game,  address_bytes: &'a Address)->Vec<usize>{
    let mut others : Vec<usize> = vec![];
    if let Some(index) = get_player_index_by_address(&game, &address_bytes){
        let current_player_data=&game.players_data[index];
        let current_player_state=&current_player_data.player_state;
        let other_index_iter=get_player_index_not_by_address(&game,&address_bytes);
        other_index_iter.iter().for_each(|other_index| {
            let player_data= &game.players_data[other_index.clone()];
            let player_state = &player_data.player_state;
            if current_player_state.is_other_player_cards_revealed_by_player(player_state.player_reveal_proof_index.as_u64()) ==false{
                others.push(*other_index);
            }
        });
    }
    others
}


pub fn are_other_players_card_need_to_be_revealed<'a>(game: &'a Game,  address_bytes: &'a Address)->Vec<usize>{
    let mut others : Vec<usize> = vec![];
    if let Some(index) = get_player_index_by_address(&game, &address_bytes){
        let current_player_data=&game.players_data[index];
        let current_player_state=&current_player_data.player_state;
        let other_index_iter=get_player_index_not_by_address(&game,&address_bytes);
        other_index_iter.iter().for_each(|other_index| {
            let player_data= &game.players_data[other_index.clone()];
            let player_state = &player_data.player_state;
            if player_state.are_cards_to_be_revealed() == true && current_player_state.is_other_player_cards_revealed_by_player(player_state.player_reveal_proof_index.as_u64()) ==false{
                others.push(*other_index);
            }
        });
    }
    others
}

pub fn get_player1_steps<'a>(game: &'a Game)->Option<usize>{
    
    for player_data in game.players_data.iter(){
        if let Some(ref address_bytes)=game.account_bytes{
            if &player_data.player_state.player == address_bytes {
                return Some(player_data.player_state.position.as_usize());
            }
        }
    };
    None
}

pub fn get_player2_steps<'a>(game: &'a Game)->Option<usize>{
    for player_data in game.players_data.iter(){
        if let Some(ref address_bytes)=game.account_bytes{
            if &player_data.player_state.player != address_bytes {
                return Some(player_data.player_state.position.as_usize());
            }
        }
    };
    None
}

pub fn request_contract_data_if_true(
    game: Res<Game>,
    q: Query<(Entity, &ContractRequestTimer)>,
)->bool{
     //game.match_index > U256::zero() && 
     q.is_empty() == false
}

//Bevy system to run every x seconds
pub fn request_contract_data(
    mut contract_events_writer: EventWriter<CallContractEvent>,
    game: Res<Game>,
    mut q: Query<(Entity, &mut ContractRequestTimer)>,
    time: Res<Time>,
    game_state: Res<State<GameState>>,
){
    let match_index=game.match_index;
    if  q.is_empty() == false{
        
        let (_,mut val)=q.get_single_mut().unwrap();
        val.timer.tick(time.delta());
        //info!("val.timer={:?}",val.timer);
        if val.timer.finished() 
        {
            let gs=game_state.get();
            // info!("timer gs={:?}",gs);
            if game.match_finished == false
            {
                if match_index > U256::zero() 
                {
                    let data = game.game_contract.contract.abi()
                                    .function(GameContractViewActionType::GetMatch.get_view_method().as_str()).unwrap()
                                    .encode_input(&[
                                        match_index.clone().into_token()
                                    ]).unwrap(); 
                                
                    contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                        method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetMatch), 
                    //    params: CallContractParam::Single(match_index.into_token())}
                    params: CallContractParam::Data(data)}
                    );
                        
                    game.match_state.as_ref().map(|match_state|{
                        for i in 0..match_state.player_count {
                            let data = game.game_contract.contract.abi()
                                .function(GameContractViewActionType::GetPlayerDataByIndex.get_view_method().as_str()).unwrap()
                                .encode_input(&[
                                    Token::Uint(match_index.clone()),
                                    Token::Uint(i.clone().into())
                                ]).unwrap(); 
                            contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                            method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetPlayerDataByIndex), 
                            params: CallContractParam::Data(data)});
                        }
                    });

                    if game.env_cards.len() == 0{
                        //get env cards
                        let data = game.game_contract.contract.abi()
                        .function(GameContractViewActionType::GetMatchEnvCards.get_view_method().as_str()).unwrap()
                        .encode_input(&[
                            match_index.clone().into_token()
                        ]).unwrap(); 
                    
                        contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                        method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetMatchEnvCards), 
                        //    params: CallContractParam::Single(match_index.into_token())}
                        params: CallContractParam::Data(data)}
                        );
                    }

                
                
                
                }else
                {
                    //info!("1072");
                    game.account_bytes.as_ref().map(|account| {
                        //Get all cards
                        if let Some(index) = get_player_index_by_address(&game, account) {
                            let data = game.card_contract.contract.abi()
                            .function(CardContractViewActionType::GetAllCards.get_view_method().as_str()).unwrap()
                            .encode_input(&[
                                get_address_from_string(&hex::encode(account)).unwrap().into_token()
                            ]).unwrap(); 
                            
                            contract_events_writer.send(CallContractEvent{contract: game.card_contract.clone(), 
                            method:Web3ViewEvents::CardContractViewActionType(CardContractViewActionType::GetAllCards), 
                            params: CallContractParam::Data(data),
                            });
                        };

                        let account_str=game.account.clone().unwrap();
                        //Get current match
                        let data = game.game_contract.contract.abi()
                        .function(GameContractViewActionType::GetCurrentMatch.get_view_method().as_str()).unwrap()
                        .encode_input(&[
                            get_address_from_string(&account_str).unwrap().into_token()
                        ]).unwrap(); 
                        contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                            method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetCurrentMatch), 
                            params:CallContractParam::Data(data),
                        });
                    });
                
                }
            }else
            {
                game.account_bytes.as_ref().map(|account| {
                    if let Some(index) = get_player_index_by_address(&game, account) {
                        let data = game.card_contract.contract.abi()
                        .function(CardContractViewActionType::GetAllCards.get_view_method().as_str()).unwrap()
                        .encode_input(&[
                            get_address_from_string(&hex::encode(account)).unwrap().into_token()
                        ]).unwrap(); 
                        
                        contract_events_writer.send(CallContractEvent{contract: game.card_contract.clone(), 
                        method:Web3ViewEvents::CardContractViewActionType(CardContractViewActionType::GetAllCards), 
                        params: CallContractParam::Data(data),
                        });
                    };
                });
            }

            
            // game.account.as_ref().map(|account| {
                
               
                // let data = game.game_contract.contract.abi().function("getPlayerData").unwrap()
                // .encode_input(&[
                //     Token::Uint(match_index.clone()),
                //     get_address_from_string(&account).unwrap().into_token()
                // ]).unwrap();
                

                // contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                //     method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetPlayerData), 
                //     params: CallContractParam::Multiple(data)});
            // });
            //val.timer.reset();
        }
    }
}

fn update_status_area(game: &Game, ui_query: &mut Query<(Entity, &mut UiElementComponent)>,commands: &mut Commands, 
status: GameStatus){
    if let Some(ref match_state)= game.match_state{
        if (match_state.state.clone() as u64) < 1{
            process_ui_query( ui_query,commands,
                UiElement::BeforeGame(BeforeGameElements::Status),
                Some(status),None,None,&game);
        }else{
            process_ui_query( ui_query,commands,
                UiElement::InGame(InGameElements::Status),
                Some(status),None,None,&game);
        }
    }else{
        process_ui_query( ui_query,commands,
            UiElement::BeforeGame(BeforeGameElements::Status),
            Some(status),None,None,&game);
    }
    
}

//bevy system
pub fn update_ui_event_handler( mut ui_query: Query<(Entity, &mut UiElementComponent)>,mut commands: Commands,
mut ui_update_event: EventReader<UiUpdateEvent>,game: Res<Game>,card_images: Res<CardImages>,)
{
    //Add & remove
    for event in ui_update_event.read(){
        info!("update_ui_event_handler event = {:?}",event);
        match event{
            UiUpdateEvent::UpdateCards{..}=>{
                process_ui_query(  &mut ui_query,&mut commands,
                    UiElement::InGame(InGameElements::PlayerHandCards ),
                    None,None,Some(&card_images),&game);
            },
            UiUpdateEvent::UpdateActiveCard{player}=>{
                process_ui_query( &mut ui_query,&mut commands,
                    UiElement::InGame(InGameElements::PlayerActiveCard{player : player.clone()}),
                    None,None,Some(&card_images),&game);
                // if player== &Player::Player1 {
                    
                    
                // }else{
                //     process_ui_query(  &mut ui_query,&mut commands,
                //         UiElement::InGame(InGameElements::PlayerActiveCards{player : player.clone()}),
                //         None,None,None,&game);
                // }
            },
            UiUpdateEvent::UpdateTrack=>{
                process_ui_query(  &mut ui_query,&mut commands,
                    UiElement::InGame(InGameElements::Track),
                    None,None,Some(&card_images),&game);
            },
            UiUpdateEvent::UpdateEnv=>{
                process_ui_query(  &mut ui_query,&mut commands,
                    UiElement::InGame(InGameElements::EnvDeck),
                    None,None,Some(&card_images),&game);
            },
            UiUpdateEvent::UpdatePlayerLabel=>{
                process_ui_query(  &mut ui_query,&mut commands,
                    UiElement::InGame(InGameElements::PlayerLabel),
                    None,None,Some(&card_images),&game);
            },
            UiUpdateEvent::UpdateWinningCard=>{
                process_ui_query(  &mut ui_query,&mut commands,
                    UiElement::InGame(InGameElements::WinnerCard),
                    None,None,Some(&card_images),&game);
            }
        };
        
    }
}

fn get_address_from_bytes(data: &Address)->String{
    //info!("get_address_from_bytes data={:?}",hex::encode(&data).to_owned());
    hex::encode(&data).trim_start_matches('0').to_owned()
}