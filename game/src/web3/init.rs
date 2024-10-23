use bevy::prelude::*;
use crate::{
    style::{HOVERED_BUTTON, NORMAL_BUTTON},
};
use bevy_web3::{
    plugin::{WalletChannel,TransactionResult,get_address_from_string,tokens::{ Tokenizable},
    tokens::Uint},
    error::Error as Web3Error};
use crate::{Game,
    game::{GameStatus,process_ui_query,UiElement,BeforeGameElements,PopupDrawEvent,PopupData,hide_popup,RevealData,RequestExtraData,
     MenuData, UiElementComponent, CardComponent,AsyncHttpResource,RevealRequestData,AsyncHttpResponse,}
};
use super::{WalletState,ContractState,GameContractIxType, ContractType,GameActions,PlayerAction,player_action,
    CallContractParam,ActionType,Web3Actions,CardContractViewActionType,CallContractEvent,
    send_txn_init_player,generate_key,send_txn_create_match,send_txn_set_creator_deck,
    get_player_index_by_address,ComputeChannelResource,are_other_players_card_revealed_by_current_player
    ,send_txn_join_match,DelegateTxnSendEvent,TxnDataRequestTimer,DelegateTxnResponseType,
    InGameContractEvent,GameContractViewActionType,send_txn_mask_and_shuffle_env_deck,send_txn_shuffle_env_deck,send_txn_shuffle_self_deck,PopupResult,
    Point,Web3ViewEvents,init_prover_key,public_compress,refresh_joint_key,DECK_SIZE,ComputeResultResponse,
    send_txn_joint_key,convert_vec_string_to_u256,u256_vec_to_string,get_player_index_not_by_address,show_next_env_card,reveal_other_player_cards,play_card,
    send_txn_shuffle_others_deck,};
use std::fmt;
use crate::error::GameError;
use bevy_pkv::PkvStore;
use bevy_text_edit::{ TextEditable};
use bevy_web3::types::U256;
use std::collections::HashMap;
use bevy_toast::{ToastEvent,ToastData};


#[derive(Debug,PartialEq)]
pub enum ViewType{

}

#[derive(Component,Debug)]
pub struct MenuButton{
    pub target: ActionType,
    pub layer: usize,
}

impl MenuButton{
    pub fn new(target: ActionType, layer: usize )->Self{
        MenuButton{
            target,
            layer
        }
    }
}

impl fmt::Display for ActionType{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        write!(f,"{:?}",self)
    }
}
 


//Bevy System
pub fn direct_user_interaction(
    wallet: Res<WalletChannel>,
    game: Res<Game>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    //mut text_query: Query<&mut Text>,
    button_type_query: Query<&MenuButton>,
    mut next_wallet_state: ResMut<NextState<WalletState>>,
    mut next_contract_state: ResMut<NextState<ContractState>>,
    mut pkv: ResMut<PkvStore>,
    mut menu_data: ResMut<MenuData>,
    mut match_events_writer: EventWriter<InGameContractEvent>,
    card_query: Query<&CardComponent>,
    mut popup_draw_event: EventWriter<PopupDrawEvent>,
    mut commands: Commands,
    
) {
    let player_key=game.account_bytes.as_ref().map(|m| generate_key(m, &mut pkv));
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        // let _text = text_query.get_mut(children[0]).unwrap();
        match button_type_query.get(children[0])
        {
            Ok(button_type)=>
            {
                if button_type.layer == menu_data.active_layer {
                    match *interaction {
                        Interaction::Pressed => 
                        {
                            //text.sections[0].value = "Press".to_string();
                            // *color = PRESSED_BUTTON.into();
                            // border_color.0 = Color::srgb(1.0,0.0,0.0);
                
                            match button_type.target
                            {
                                ActionType::Web3Actions(ref web3_action)=>
                                {
                                    match web3_action{
                                        Web3Actions::ConnectWallet=>{
                                            wallet.connect();
                                            next_wallet_state.set(WalletState::Connecting);
                                        },
                                        Web3Actions::GameContractAction(ix_type)=>{
                                            match ix_type{
                                                GameContractIxType::InitPlayer=>{
                                                    if let Some(ref from) = game.account {
                                                        //let game_address=game.game_contract.contract.address();
                                                        info!("from_address={:?}",hex::encode(from));
                                                        //from: String,vrf_balance: Option<Uint>, ix_type: GameContractIxType
                                                        match send_txn_init_player(&wallet,game.game_contract.clone(),
                                                        from.clone(),None,ix_type){
                                                            Ok(_)=>{
                                                                next_contract_state.set(ContractState::SendTransaction)
                                                            },
                                                            Err(e)=>error!("{e:?}"),
                                                        };
                
                                                    }
                                                },
                                                GameContractIxType::CreateNewMatch=>{
                                                    if let Some(ref from) = game.account {
                                                        //let bytes=game.account_bytes.clone().unwrap();
                                                        let player_key_value=player_key.clone().unwrap();
                                                        let x=player_key_value.pkxy.0.trim_start_matches("0x");
                                                        info!("x={:?}",x);
                                                        let x= Uint::from_str_radix(x,16).unwrap();
                                                        let y=player_key_value.pkxy.1.trim_start_matches("0x");
                                                        let y=  Uint::from_str_radix(y,16).unwrap();
                                                        
                                                        match send_txn_create_match(&wallet,game.game_contract.clone(),
                                                        from.clone(),ix_type, 2 ,
                                                        Point{
                                                            x,
                                                            y
                                                        },None ){
                                                            Ok(_)=>{
                                                                next_contract_state.set(ContractState::SendTransaction)
                                                            },
                                                            Err(e)=>error!("{e:?}"),
                                                        };
                                                    }
                                                },
                                                GameContractIxType::SetCreatorDeck=>{
                                                    if let Some(ref from) = game.account {
                                                        let bytes=game.account_bytes.clone().unwrap();
                                                        
                                                        let selected_cards = card_query.iter().filter(|f| f.selected == true && f.onchain_index.is_some()).map(|m| m.onchain_index.clone().unwrap()).collect::<Vec<U256>>();
                                                        
                                                        if selected_cards.len() ==  DECK_SIZE as usize{
                                                            if let Some(ref match_state) = game.match_state{
                                                                //creator
                                                                if &bytes == &match_state.creator {
                                                                    if let Some(index) = get_player_index_by_address(&game, &bytes) {
                                                                        let current_player_data=&game.players_data[index];
                                                                        if current_player_data.player_state.original_cards.len() == 0 {
                                                                            match send_txn_set_creator_deck(&wallet,game.game_contract.clone(),
                                                                            from.clone(),ix_type, game.match_index, selected_cards.clone() ){
                                                                                Ok(_)=>{
                                                                                    next_contract_state.set(ContractState::SendTransaction)
                                                                                },
                                                                                Err(e)=>error!("{e:?}"),
                                                                            };
                                                                        }else{
                                                                            warn!("167")
                                                                        }
                                                                    }else{
                                                                        warn!("170")
                                                                    }
                                                                }
                                                                
                                                            }
                                                        }else{
                                                            //Show popup
                                                            popup_draw_event.send(
                                                                PopupDrawEvent::ShowPopup(PopupData {
                                                                    msg: format!("Need to select {:?} cards, but selected {:?} cards",DECK_SIZE, selected_cards.len()),
                                                                    popup_type: GameStatus::SetCreatorDeck as i32,
                                                                    action_yes: Some(PopupResult::Yes(GameStatus::PopupHide as i32)),
                                                                    action_no : None,
                                                                })
                                                            );
                                                        }
                                                        
                                                    }
                                                },
                                                GameContractIxType::JoinMatchPreSelect=>{
                                                    if game.match_index != U256::zero() {   
                                                        match_events_writer.send(InGameContractEvent::JoinMatch);
                                                    }else{
                                                        match_events_writer.send(InGameContractEvent::UpdateMatchIndex);
                                                    }
                                                    
                                                },
                                                GameContractIxType::SetJointKeyPreSet=>{
                                                    popup_draw_event.send(
                                                        PopupDrawEvent::ShowPopup(PopupData {
                                                            msg: format!("Waiting to init joint key "),
                                                            popup_type: GameStatus::SetJointKeyPreSet as i32,
                                                            action_yes: None,
                                                            action_no : None,
                                                        })
                                                    );
                                                    match_events_writer.send(InGameContractEvent::UpdatePKCPopup);
                                                },
                                                GameContractIxType::JoinMatch |
                                                GameContractIxType::SetJointKey=>{
                                                },
                                                GameContractIxType::MaskAndShuffleEnvDeck =>{
                                                    popup_draw_event.send(
                                                        PopupDrawEvent::ShowPopup(PopupData {
                                                            msg: format!("Waiting to mask and shuffle env deck"),
                                                            popup_type: GameStatus::MaskAndShuffleEnvDeck as i32,
                                                            action_yes: None,
                                                            action_no : None,
                                                        })
                                                    );
                                                    match_events_writer.send(InGameContractEvent::MaskAndShuffleEnvDeckPopup);
                                                    
                                                },
                                                GameContractIxType::ShuffleEnvDeck =>{
                                                    popup_draw_event.send(
                                                        PopupDrawEvent::ShowPopup(PopupData {
                                                            msg: format!("Waiting to shuffle env deck"),
                                                            popup_type: GameStatus::ShuffleEnvDeck as i32,
                                                            action_yes: None,
                                                            action_no : None,
                                                        })
                                                    );
                                                    match_events_writer.send(InGameContractEvent::ShuffleEnvDeckPopup);
                                                    
                                                },
                                                GameContractIxType::ShuffleYourDeck=>{
                                                    popup_draw_event.send(
                                                        PopupDrawEvent::ShowPopup(PopupData {
                                                            msg: format!("Waiting to shuffle deck"),
                                                            popup_type: GameStatus::ShuffleYourDeck as i32,
                                                            action_yes: None,
                                                            action_no : None,
                                                        })
                                                    );
                                                    match_events_writer.send(InGameContractEvent::ShuffleYourDeckPopup);

                                                    
                                                },
                                                GameContractIxType::ShuffleOthersDeck=>{
                                                    popup_draw_event.send(
                                                        PopupDrawEvent::ShowPopup(PopupData {
                                                            msg: format!("Waiting to shuffle deck"),
                                                            popup_type: GameStatus::ShuffleOthersDeck as i32,
                                                            action_yes: None,
                                                            action_no : None,
                                                        })
                                                    );
                                                    match_events_writer.send(InGameContractEvent::ShuffleOthersDeckPopup);
                                                    
                                                },
                                                GameContractIxType::RevealEnvCard=>{
                                                    //Do api call
                                                    if let Some(ref player_key) = player_key
                                                    {
                                                        if let Some(ref match_state) = game.match_state
                                                        {
                                                            let bytes=game.account_bytes.clone().unwrap();
                                                            if let Some(index) = get_player_index_by_address(&game, &bytes) {
                                                                let current_player_data=&game.players_data[index];
                                                                if match_state.is_env_revealed_by_player(current_player_data.player_state.player_index) == false
                                                                {
                                                                    let env_state=&match_state.env_deck;
                                                                    let env_reveal_index=env_state.env_reveal_index;
                                                                    let cards=env_state.cards.clone();
                                                                    let target= &cards[env_reveal_index as usize];
                                                                    let secret_key=player_key.sk.clone();
                                                                    match_events_writer.send(InGameContractEvent::RevealMaskedCard{
                                                                        masked_card: HashMap::from([(0,u256_vec_to_string(target.clone()))]),
                                                                        extra_data: RequestExtraData::RevealEnvCard{request_type: ix_type.clone() as i32,
                                                                            secret_key,},
                                                                        });
                                                                };
                                                            }
                                                            
                                                        }
                                                        
                                                    }
                                                    
                                                },
                                                GameContractIxType::RevealOtherPlayerCards=>{
                                                    if let Some(ref _from) = game.account {
                                                        let bytes=game.account_bytes.clone().unwrap();
                                                        if let Some(ref player_key) = player_key
                                                        {
                                                            if let Some(ref _match_state) = game.match_state
                                                            {
                                                                if let Some(_) = get_player_index_by_address(&game, &bytes) {
                                                                    //let current_player_data=&game.players_data[index];

                                                                    //let other_index_iter=get_player_index_not_by_address(&game,&bytes);
                                                                    let other_index_iter=are_other_players_card_revealed_by_current_player(&game,&bytes);
                                                                    
                                                                    other_index_iter.iter().for_each(|other_index| {
                                                                        let player_data= &game.players_data[other_index.clone()];
                                                                        let already_revealed = player_data.player_state.player_reveal_count;
                                                                        let cards=&player_data.player_state.player_deck;
                                                                        let reveal_count = if already_revealed == 0 {
                                                                            3
                                                                        }else {
                                                                            1
                                                                        };
                                                                        let mut card_map: HashMap<usize, Vec<String>>= HashMap::new();
                                                                        for i in 0..reveal_count{
                                                                            let idx=already_revealed as usize + i;
                                                                            card_map.insert(idx,u256_vec_to_string(cards[idx].clone()));
                                                                        }
                                                                        let secret_key=player_key.sk.clone();
                                                                        match_events_writer.send(InGameContractEvent::RevealMaskedCard{
                                                                            masked_card: card_map,
                                                                            extra_data: RequestExtraData::RevealOtherPlayersCard{
                                                                                request_type: ix_type.clone() as i32,
                                                                                secret_key,player_index: player_data.player_state.player_index,
                                                                                reveal_count: reveal_count as u64,  
                                                                            },
                                                                        });
                                                                            
                                                                        
                                                                    });
                                                                }
                                                                
                                                            }
                                                            
                                                        }
                                                    }
                                                    
                                                },
                                                GameContractIxType::PlayerActionEnv=>{
                                                    if let Some(ref from) = game.account {
                                                        match player_action(&wallet,game.game_contract.clone(),
                                                            from.clone(),ix_type, game.match_index, PlayerAction::ShowEnvCard ){
                                                                Ok(_)=>{
                                                                    next_contract_state.set(ContractState::SendTransaction)
                                                                },
                                                                Err(e)=>error!("{e:?}"),
                                                            };
                                                    }
                                                },
                                                GameContractIxType::PlayerActionCard=>{
                                                    if let Some(ref from) = game.account {
                                                        match player_action(&wallet,game.game_contract.clone(),
                                                            from.clone(),ix_type, game.match_index, PlayerAction::ShowPlayerCard ){
                                                                Ok(_)=>{
                                                                    next_contract_state.set(ContractState::SendTransaction)
                                                                },
                                                                Err(e)=>error!("{e:?}"),
                                                            };
                                                    }
                                                },
                                                GameContractIxType::PlayCardOnDeck=>{
                                                    if let Some(ref _from) = game.account {
                                                        let bytes=game.account_bytes.clone().unwrap();
                                                        if let Some(ref player_key) = player_key
                                                        {
                                                            let selected_cards = card_query.iter().filter(|f| f.selected == true && f.onchain_index.is_some()).map(|m| m.index).collect::<Vec<usize>>();

                                                            if selected_cards.len() != 1{
                                                                popup_draw_event.send(
                                                                    PopupDrawEvent::ShowPopup(PopupData {
                                                                        msg: format!("Need to select {:?} card, but selected {:?} card",1, selected_cards.len()),
                                                                        popup_type: GameStatus::SetCreatorDeck as i32,
                                                                        action_yes: Some(PopupResult::Yes(GameStatus::PopupHide as i32)),
                                                                        action_no : None,
                                                                    })
                                                                );
                                                            }else{
                                                                if let Some(index) = get_player_index_by_address(&game, &bytes) {
                                                                    let current_player_data=&game.players_data[index];
                                                                    //Todo! check if already played card 
                                                                    let hand_index=*selected_cards.first().unwrap();
                                                                    let cards=&current_player_data.player_state.player_deck;
                                                                    let mut card_map: HashMap<usize, Vec<String>>= HashMap::new();
                                                                    let card_index=current_player_data.player_state.player_hand.iter().position(|p| p==&(hand_index as u64)).unwrap();
                                                                    let card_in_hard=card_index;
                                                                    //info!("hand_index={} card_in_hard={:?}",hand_index,card_in_hard);
                                                                    card_map.insert(hand_index,u256_vec_to_string(cards[hand_index as usize].clone()));
                                                                    let secret_key=player_key.sk.clone();
                                                                    match_events_writer.send(InGameContractEvent::RevealMaskedCard{
                                                                        masked_card: card_map,
                                                                        extra_data: RequestExtraData::PlayCardOnDeck{
                                                                            request_type: ix_type.clone() as i32,
                                                                            secret_key, hand_index:  card_in_hard as u64 
                                                                        },
                                                                    });
                                                                }
                                                            }
                                                        }
                                                         
                                                    }
                                                }
                                            }
                                            
                                        },
                                        _=>{},
                                    }
                                },
                                ActionType::GameActions(ref game_actions)=>
                                {
                                    match game_actions{
                                        GameActions::PopupActions(popup_result)=>{
                                            match popup_result{
                                                PopupResult::Yes(val)=> {
                                                    match GameStatus::from_i32(*val){
                                                        GameStatus::PopupHide=>{
                                                            hide_popup(&mut menu_data, &mut commands);
                                                            
                                                        },
                                                        _=>{},
                                                    }
                                                },
                                                PopupResult::No(val)=> {
                                                    match GameStatus::from_i32(*val){
                                                        GameStatus::PopupHide=>{
                                                            hide_popup(&mut menu_data, &mut commands);
                                                            
                                                           
                                                        },
                                                        _=>{},
                                                    }
                                                },
                                            }
                                        },
                                        GameActions::ShowOriginalCards(_player)=>{
                                            //Todo!: Show player cards popup
                                        }
                                    }
                                        
                                }
                            }
                        },
                        Interaction::Hovered => {
                            //text.sections[0].value = "Hover".to_string();
                            *color = HOVERED_BUTTON.into();
                            border_color.0 = Color::WHITE;
                        }
                        Interaction::None => {
                            //text.sections[0].value = "Button".to_string();
                            *color = NORMAL_BUTTON.into();
                            border_color.0 = Color::BLACK;
                        }
                        
                    
                    }
                }
            },
            Err(_)=>{},
        
        
        };
    }
}

//Bevy System
pub fn delegate_txn_call(
    wallet: Res<WalletChannel>,
    game: Res<Game>,
    mut delegate_txn_event: EventReader<DelegateTxnSendEvent>,
    mut next_contract_state: ResMut<NextState<ContractState>>,
    mut contract_events_writer: EventWriter<CallContractEvent>,
    mut in_game_events: EventWriter<InGameContractEvent>,
    mut pkv: ResMut<PkvStore>,
    mut popup_draw_event: EventWriter<PopupDrawEvent>,
    card_query: Query<&CardComponent>,
){
    let player_key=game.account_bytes.as_ref().map(|m| generate_key(m, &mut pkv));
    for event in delegate_txn_event.read(){
        match event{
            DelegateTxnSendEvent::LoadMatch=>{
                
                let data = game.game_contract.encode_input(GameContractViewActionType::GetMatch.get_view_method().as_str(),
                &[game.match_index.into_token()]).unwrap();
                contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                    method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetMatch), 
                    //params: CallContractParam::Single(game.match_index.into_token())
                    params: CallContractParam::Data(data),
                });
            },
            DelegateTxnSendEvent::JoinMatch=>{
                if let Some(ref from) = game.account {
                    let bytes=game.account_bytes.clone().unwrap();
                    let selected_cards = card_query.iter().filter(|f| f.selected == true && f.onchain_index.is_some()).map(|m| m.onchain_index.clone().unwrap()).collect::<Vec<U256>>();
                    if selected_cards.len() == DECK_SIZE as usize {
                        if let Some(ref match_state) = game.match_state{
                            if &bytes != &match_state.creator {
                                info!("game.selected_cards={:?}",selected_cards);
                                if let Some(index) = get_player_index_by_address(&game, &bytes) {
                                    info!("index={:?}",index);
                                    let current_player_data=&game.players_data[index];
                                    let player_key_value=player_key.clone().unwrap();
                                    let x=player_key_value.pkxy.0.trim_start_matches("0x");
                                    info!("x={:?}",x);
                                    let x= Uint::from_str_radix(x,16).unwrap();
                                    let y=player_key_value.pkxy.1.trim_start_matches("0x");
                                    let y=  Uint::from_str_radix(y,16).unwrap();
                                    
                                    if current_player_data.player_state.original_cards.len() == 0 {
                                        match send_txn_join_match(&wallet,game.game_contract.clone(),
                                        from.clone(),&GameContractIxType::JoinMatch, game.match_index, 
                                        Point{
                                            x,
                                            y
                                        },
                                        selected_cards.clone() ){
                                            Ok(_)=>{
                                                next_contract_state.set(ContractState::SendTransaction)
                                            },
                                            Err(e)=>error!("{e:?}"),
                                        };
                                    }
                                }
                            }
                        }
                    }else{
                        popup_draw_event.send(
                            PopupDrawEvent::ShowPopup(PopupData {
                                msg: format!("Select {} cards , only {} cards selected ",DECK_SIZE, selected_cards.len()),
                                popup_type: GameStatus::JoinMatchPreSelect as i32,
                                action_yes: Some(PopupResult::Yes(GameStatus::PopupHide as i32)),
                                action_no : None,
                            })
                        );
                    }
                }            
            },
            DelegateTxnSendEvent::SetJointKeyPopup => {
                in_game_events.send(InGameContractEvent::UpdatePKC);
            },
            DelegateTxnSendEvent::MaskAndShuffleEnvDeckPopup=>{
                in_game_events.send(InGameContractEvent::MaskAndShuffleEnvDeck);
            },
            DelegateTxnSendEvent::ShuffleEnvDeckPopup=>{
                in_game_events.send(InGameContractEvent::ShuffleEnvDeck);
            },
            DelegateTxnSendEvent::ShuffleYourDeckPopup=>{
                in_game_events.send(InGameContractEvent::ShuffleYourDeck);
            },
            DelegateTxnSendEvent::ShuffleOthersDeckPopup=>{
                in_game_events.send(InGameContractEvent::ShuffleOthersDeck);
            },
            DelegateTxnSendEvent::MaskAndShuffleEnvDeck=>{
                if let Some(ref from) = game.account {
                    let bytes=game.account_bytes.clone().unwrap();
                                
                    if let Some(ref match_state) = game.match_state{
                            //creator
                            if &bytes == &match_state.creator {
                                match send_txn_mask_and_shuffle_env_deck(&wallet,game.game_contract.clone(),
                                from.clone(),&GameContractIxType::MaskAndShuffleEnvDeck, game.match_index, match_state.game_key.clone(), DECK_SIZE as i32){
                                    Ok(_)=>{
                                        next_contract_state.set(ContractState::SendTransaction)
                                    },
                                    Err(e)=>error!("{e:?}"),
                                };
                                
                            } 
                    }
                }
            },
            DelegateTxnSendEvent::ShuffleEnvDeck=>{
                if let Some(ref from) = game.account {
                    let bytes=game.account_bytes.clone().unwrap();
                                
                    if let Some(ref match_state) = game.match_state{
                        if let Some(index) = get_player_index_by_address(&game, &bytes) {
                            let current_player_data=&game.players_data[index];
                            if current_player_data.player_state.done == false{
                                match send_txn_shuffle_env_deck(&wallet,game.game_contract.clone(),
                                from.clone(),&GameContractIxType::ShuffleEnvDeck, game.match_index, match_state.game_key.clone(),match_state.env_deck.cards.clone()){
                                    Ok(_)=>{
                                        next_contract_state.set(ContractState::SendTransaction)
                                    },
                                    Err(e)=>error!("{e:?}"),
                                };
                            }
                            
                        }
                    }
                }
            },
            DelegateTxnSendEvent::ShuffleYourDeck=>{
                if let Some(ref from) = game.account 
                {
                    let bytes=game.account_bytes.clone().unwrap();
                                
                    if let Some(ref match_state) = game.match_state{
                        if let Some(index) = get_player_index_by_address(&game, &bytes) {
                            let current_player_data=&game.players_data[index];
                            if current_player_data.player_state.done == false{
                                match send_txn_shuffle_self_deck(&wallet,game.game_contract.clone(),
                                from.clone(),&GameContractIxType::ShuffleYourDeck, game.match_index, match_state.game_key.clone(), DECK_SIZE as i32){
                                    Ok(_)=>{
                                        next_contract_state.set(ContractState::SendTransaction)
                                    },
                                    Err(e)=>error!("{e:?}"),
                                };
                            }
                            
                        }
                    }
                }
            },
            DelegateTxnSendEvent::ShuffleOthersDeck=>{
                if let Some(ref from) = game.account 
                {
                    let bytes=game.account_bytes.clone().unwrap();
                                
                    if let Some(ref match_state) = game.match_state{
                        if let Some(index) = get_player_index_by_address(&game, &bytes) {
                            let current_player_data=&game.players_data[index];
                            if current_player_data.player_state.done == false{
                                //Todo! pass other users address and deck
                                let other_index_iter=get_player_index_not_by_address(&game,&bytes);
                                other_index_iter.iter().for_each(|other_index| {
                                        let player_data= &game.players_data[other_index.clone()];
                                        info!("shuffle others deck index={:?} other_index={:?}",index,other_index);
                                        match send_txn_shuffle_others_deck(&wallet,game.game_contract.clone(),
                                        from.clone(),&GameContractIxType::ShuffleOthersDeck, game.match_index, match_state.game_key.clone(),
                                        player_data.player_state.player.clone(),player_data.player_state.player_deck.clone() ){
                                            Ok(_)=>{
                                                next_contract_state.set(ContractState::SendTransaction)
                                            },
                                            Err(e)=>error!("{e:?}"),
                                        };
                                    }
                                );
                                
                            }
                            
                        }
                    }
                }
            },
            DelegateTxnSendEvent::SetJointKey=>{
                if let Some(ref from) = game.account {
                    let bytes=game.account_bytes.clone().unwrap();
                    if let Some(ref match_state) = game.match_state{
                        if &bytes == &match_state.creator {
                            match send_txn_joint_key(&wallet,game.game_contract.clone(),
                            from.clone(),&GameContractIxType::SetJointKey, game.match_index, 
                            game.pkc.clone() ){
                                Ok(_)=>{
                                    next_contract_state.set(ContractState::SendTransaction)
                                },
                                Err(e)=>error!("{e:?}"),
                            };
                        }
                    }
                }
            },
            DelegateTxnSendEvent::RevealCard{reveal_map,request_extra_data}=>{
                if let Some(ref from) = game.account {
                    let bytes=game.account_bytes.clone().unwrap();
                                
                    if let Some(_) = get_player_index_by_address(&game, &bytes) {
                        // let current_player_data=&game.players_data[index];
                        //if current_player_data.player_state.done == false{
                        match request_extra_data{
                            RequestExtraData::RevealEnvCard{request_type,..}=>{
                                let values = reveal_map.clone().into_values().collect::<Vec<RevealData>>();
                                let value=values.first().unwrap();
                                match show_next_env_card(&wallet,game.game_contract.clone(),from.clone(),
                                &GameContractIxType::from_i32(*request_type),
                                    game.match_index, 
                                    value.card.clone(),value.snark_proof.clone()){
                                        Ok(_)=>{
                                            next_contract_state.set(ContractState::SendTransaction)
                                        },
                                        Err(e)=>error!("{e:?}"),
                                    };
                            },
                            RequestExtraData::RevealOtherPlayersCard{player_index, reveal_count,request_type,..}=>{
                                let mut keys=reveal_map.clone().into_keys().collect::<Vec<usize>>();
                                keys.sort();
                                let mut cards : Vec<(String,String)>=vec![];
                                let mut proofs: Vec<Vec<String>> = vec![];
                                for key in keys.iter(){
                                    if let Some(value) = reveal_map.get(key) {
                                        cards.push(value.card.clone());
                                        proofs.push(value.snark_proof.clone());
                                    };
                                }
                                
                                match reveal_other_player_cards(&wallet,game.game_contract.clone(),from.clone(),
                                &GameContractIxType::from_i32(*request_type),
                                    game.match_index, *reveal_count,*player_index,
                                    cards,proofs){
                                        Ok(_)=>{
                                            next_contract_state.set(ContractState::SendTransaction)
                                        },
                                        Err(e)=>error!("{e:?}"),
                                    };
                            },
                            RequestExtraData::PlayCardOnDeck{hand_index,request_type,..}=>{
                                let values = reveal_map.clone().into_values().collect::<Vec<RevealData>>();
                                let value=values.first().unwrap();
                                info!("play card on deck hand_index={:?}",hand_index);
                                match play_card(&wallet,game.game_contract.clone(),from.clone(),
                                &GameContractIxType::from_i32(*request_type),
                                    game.match_index,*hand_index,value.card.clone(),value.snark_proof.clone()){
                                        Ok(_)=>{
                                            next_contract_state.set(ContractState::SendTransaction)
                                        },
                                        Err(e)=>error!("{e:?}"),
                                    }
                            }
                        }
                        // let request = GameContractIxType::from_i32(*request_type);
                        // match request{
                        //     GameContractIxType::RevealEnvCard=>{
                                
                        //     },
                        //     GameContractIxType::RevealOtherPlayerCards=>{
                                
                        //     },
                        //     _=>{},
                        // }
                            
                        //}
                    }
                }
            }
        }
        
    }
}

//Bevy System
pub fn wallet_account(
    //mut next_state: ResMut<NextState<GameState>>,
    mut game: ResMut<Game>,
    channel: Res<WalletChannel>,
    q: Query<(Entity, &mut TxnDataRequestTimer)>,
    mut commands: Commands,
    // mut menu_data: ResMut<MenuData>,
    wallet_state: Res<State<WalletState>>,
    mut next_wallet_state: ResMut<NextState<WalletState>>,
    contract_state: Res<State<ContractState>>,
    mut next_contract_state: ResMut<NextState<ContractState>>,
    mut contract_events_writer: EventWriter<CallContractEvent>,
    mut ui_query: Query<(Entity, &mut UiElementComponent)>,
    mut toast_event_writer: EventWriter<ToastEvent>,
) 
{
    //Todo!: For now Only handle disconnect to connect
    if let WalletState::Connecting = wallet_state.get(){
        match channel.recv_account() {
            Ok((account, network)) => {
                info!("account: {:?}, network: {}", account, network);
                // TODO
                game.account_bytes=Some(account.clone());
                game.account = Some(hex::encode(&account));
                game.chain = network.as_u64();
                next_wallet_state.set(WalletState::Connected);
                 
                let data = game.card_contract.encode_input(CardContractViewActionType::GetAllCards.get_view_method().as_str(),
                &[get_address_from_string(&hex::encode(account)).unwrap().into_token()]).unwrap();
                
                contract_events_writer.send(CallContractEvent{contract: game.card_contract.clone(), 
                method:Web3ViewEvents::CardContractViewActionType(CardContractViewActionType::GetAllCards), 
                params: CallContractParam::Data(data),
                });
                 
               
                process_ui_query(&mut ui_query,&mut commands, 
                    UiElement::BeforeGame(BeforeGameElements::Status),
                    Some(GameStatus::InitPlayer),None,None,&game);
                
                 //Test toast
                 toast_event_writer.send(ToastEvent::ShowToast{data: ToastData{content: "Loading...".to_owned(),timeout_secs: 15., 
                ..default()}});
 
                //Test shuffle
                // generate_key(&account,&mut pkv);
                // info!("get_proof_token {:?}",convert_string_to_u256("0x2da8e5abccfc535e01b1eef0f4ec18361b5f7309fa6e15ce0a5a670eadc7d55e".to_owned()));
            }
            Err(Web3Error::ChannelEmpty) => {

            },
            Err(_err) => {
                // info!("wallet_account={:?}",err.to_string())
            }
        };
    }

    //Check Transaction 
    if let ContractState::SendTransaction = contract_state.get(){
        match channel.recv_transaction(){
            Ok(TransactionResult{hash,ix_type,to,estimated_wait_time})=>{
                let contract_type = ContractType::try_from(hex::encode(&to).as_str()).unwrap();
                info!("hash={:?} contract_type={:?} ix_type={:?}",hash,contract_type,ix_type);
                let tm = estimated_wait_time.map(|m| m as u32);
                next_contract_state.set(ContractState::TransactionResult{estimated_wait_time: tm});
                //Query contract 
                match contract_type{
                    ContractType::CardContract=>{},
                    ContractType::GameContract=>{
                        if let Some(ix_type) = ix_type{
                            let game_contract_ix_type=GameContractIxType::from_i32(ix_type);
                            info!("GameContract ix_type={:?}",game_contract_ix_type);
                            match game_contract_ix_type{
                                GameContractIxType::InitPlayer=>{
                                    if q.is_empty(){
                                        commands.spawn_empty().insert(TxnDataRequestTimer::new(DelegateTxnResponseType::InitPlayer));
                                    }
                                },  
                                GameContractIxType::CreateNewMatch=>{   
                                    if q.is_empty(){
                                        commands.spawn_empty().insert(TxnDataRequestTimer::new(DelegateTxnResponseType::CreateNewMatch));
                                    }
                                },
                                GameContractIxType::JoinMatch | GameContractIxType::JoinMatchPreSelect=>{
                                    if q.is_empty(){
                                        commands.spawn_empty().insert(TxnDataRequestTimer::new(DelegateTxnResponseType::JoinMatch));
                                    }
                                },
                                GameContractIxType::SetCreatorDeck=>{
                                    if q.is_empty(){
                                        commands.spawn_empty().insert(TxnDataRequestTimer::new(DelegateTxnResponseType::SetCreatorDeck));
                                    }
                                },
                                GameContractIxType::SetJointKeyPreSet | GameContractIxType::SetJointKey=>{
                                    if q.is_empty(){
                                        commands.spawn_empty().insert(TxnDataRequestTimer::new(DelegateTxnResponseType::SetJointKey));
                                    }
                                },
                                GameContractIxType::MaskAndShuffleEnvDeck | GameContractIxType::ShuffleEnvDeck=>{

                                },
                                GameContractIxType::ShuffleYourDeck |
                                GameContractIxType::ShuffleOthersDeck |
                                GameContractIxType::RevealEnvCard |
                                GameContractIxType::RevealOtherPlayerCards |
                                GameContractIxType::PlayerActionEnv | 
                                GameContractIxType::PlayerActionCard | 
                                GameContractIxType::PlayCardOnDeck=>{

                                }
                            }
                        }
                    }
                }
            }
            Err(Web3Error::ChannelEmpty)=>{},
            Err(Web3Error::Web3Error(msg))=>{
                error!("error={:?}",msg);
                next_contract_state.set(ContractState::Waiting);
            },
            Err(err)=>{
                error!("unknown error : {:?}",err.to_string());
                next_contract_state.set(ContractState::Waiting);
            }
        }
    }
}

//Bevy System
pub fn delegate_txn_response_call(
    game: Res<Game>,
    mut commands: Commands,
    mut contract_events_writer: EventWriter<CallContractEvent>,
    time: Res<Time>,
    mut q: Query<(Entity, &mut TxnDataRequestTimer)>,
){
    //let player_key=game.account_bytes.as_ref().map(|m| generate_key(m, &mut pkv));
    for (entity, mut timer) in &mut q{
        timer.timer.tick(time.delta());
        if timer.timer.finished(){
            if let Some(ref from) = game.account {
                match timer.req_type{
                    DelegateTxnResponseType::InitPlayer=>{
                        // let data = game.game_contract.contract.abi()
                        // .function(CardContractViewActionType::GetAllCards.get_view_method().as_str()).unwrap()
                        // .encode_input(&[
                        //     get_address_from_string(&from).unwrap().into_token()
                        // ]).unwrap(); 
                        // contract_events_writer.send(CallContractEvent{contract: game.card_contract.clone(), 
                        //     method:Web3ViewEvents::CardContractViewActionType(CardContractViewActionType::GetAllCards), 
                        //     //params: CallContractParam::Single(get_address_from_string(&from).unwrap().into_token())
                        //     params: CallContractParam::Data(data),
                        //     });
                        
                    },
                     
                    DelegateTxnResponseType::CreateNewMatch |
                    DelegateTxnResponseType::JoinMatch=>{
                        // let data = game.game_contract.contract.abi()
                        // .function(GameContractViewActionType::GetCurrentMatch.get_view_method().as_str()).unwrap()
                        // .encode_input(&[
                        //     get_address_from_string(&from).unwrap().into_token()
                        // ]).unwrap(); 
                        let data = game.game_contract.encode_input(GameContractViewActionType::GetCurrentMatch.get_view_method().as_str(),
                        &[get_address_from_string(&from).unwrap().into_token()]).unwrap();
                        info!("CreateNewMatch or joinmatch delegate data={:?}",data);
                        contract_events_writer.send(CallContractEvent{contract: game.game_contract.clone(), 
                            method:Web3ViewEvents::GameContractViewActionType(GameContractViewActionType::GetCurrentMatch), 
                            //params: CallContractParam::Single(get_address_from_string(&from).unwrap().into_token())
                            params: CallContractParam::Data(data),
                            });
                        
                    },
                    _=>{},
                }
                commands.entity(entity).remove::<TxnDataRequestTimer>();
            }
        }
    }
     
}

//Bevy System
pub fn read_editable_text(
    texts: Query<&Text, With<TextEditable>>,
    mut in_game_events: EventReader<InGameContractEvent>,
    mut contract_events_writer: EventWriter<DelegateTxnSendEvent>,
    mut game: ResMut<Game>,
) {
    for event in in_game_events.read(){
        match event{
            InGameContractEvent::UpdateMatchIndex=>{
                if let Ok(text) =texts.get_single() {
                    if let Ok(val)=U256::from_str_radix(&text.sections[0].value,10){
                        game.match_index= val;
//                        info!("read_editable_text game.match_index {:?}",game.match_index);
                        contract_events_writer.send(DelegateTxnSendEvent::LoadMatch);
                    }
                }
            },
            _=>{},
        }
    }
    
    
}

pub fn process_pkc(game_key : Point)->Result<Vec<String>,GameError>{
    let key = public_compress(&game_key).unwrap();
    init_prover_key(DECK_SIZE as i32).unwrap();
    let pkc = refresh_joint_key(key, DECK_SIZE as i32);
    pkc

}


pub fn spawn_async_task(
    //mut task_pool: AsyncTaskPool<Vec<String>>,
    mut in_game_events: EventReader<InGameContractEvent>,
    mut delegate_events_writer: EventWriter<DelegateTxnSendEvent>,
    game: Res<Game>,
    compute_channel: Res<ComputeChannelResource>,
    mut popup_draw_event: EventWriter<PopupDrawEvent>,
    //mut commands: Commands,
    //mut menu_data: ResMut<MenuData>,
    //http_channel: Res<AsyncHttpResource>,
){
    for event in in_game_events.read(){
            match event{
                InGameContractEvent::JoinMatch=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::JoinMatch);
                },
                InGameContractEvent::UpdatePKCPopup=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::SetJointKeyPopup);
                },
                InGameContractEvent::MaskAndShuffleEnvDeckPopup=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::MaskAndShuffleEnvDeckPopup);
                },
                InGameContractEvent::MaskAndShuffleEnvDeck=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::MaskAndShuffleEnvDeck);
                },
                InGameContractEvent::ShuffleEnvDeckPopup=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::ShuffleEnvDeckPopup);
                },
                InGameContractEvent::ShuffleEnvDeck=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::ShuffleEnvDeck);
                },
                InGameContractEvent::ShuffleYourDeckPopup=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::ShuffleYourDeckPopup);
                },
                InGameContractEvent::ShuffleYourDeck=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::ShuffleYourDeck);
                },
                InGameContractEvent::ShuffleOthersDeckPopup=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::ShuffleOthersDeckPopup);
                },
                InGameContractEvent::ShuffleOthersDeck=>{
                    delegate_events_writer.send(DelegateTxnSendEvent::ShuffleOthersDeck);
                },
                InGameContractEvent::UpdatePKC=>{
                    if let Some(_) = game.account {
                        if let Some(ref match_state) = game.match_state{
                            if game.pkc.len() <24  {
                                info!("process_pkc_task");
                                //hide_popup(&mut menu_data, &mut commands);
                                compute_channel.process_pkc_task(match_state.game_key.clone());
                            }else{
                                popup_draw_event.send(
                                    PopupDrawEvent::ShowPopup(PopupData {
                                        msg: format!("Initializing game key, please wait few sec"),
                                        popup_type: GameStatus::SetJointKeyPreSet as i32,
                                        action_yes: None,
                                        action_no : None,
                                    })
                                );
                                delegate_events_writer.send(DelegateTxnSendEvent::SetJointKey);

                            }
                            
                        }
                    }
                },
                _=>{},
            
            }

    }
}

pub fn recv_async_task(
    compute_channel: Res<ComputeChannelResource>,
    mut delegate_events_writer: EventWriter<DelegateTxnSendEvent>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut menu_data: ResMut<MenuData>,
){
    match compute_channel.recv_compute(){
        Ok(ComputeResultResponse{result})=>{
            //let contract_type = ContractType::try_from(hex::encode(&to).as_str()).unwrap();
            info!("recv_compute_task result={:?}",result);
            //Query contract 
            if game.pkc.len() == 0 {
                if let Some(result)=result{
                    let pkc=convert_vec_string_to_u256(result.clone());
                    // let big_endian_arr=result.iter()
                    //     .map(|m| hex::decode(m.strip_prefix("0x").unwrap()).unwrap())
                    //     .collect::<Vec<Vec<u8>>>();
                    // let pkc=big_endian_arr.into_iter().map(|m| U256::from_big_endian(&m)).collect::<Vec<U256>>();
                    game.pkc=pkc;
                    delegate_events_writer.send(DelegateTxnSendEvent::SetJointKey);
                }
            }else{
                hide_popup(&mut menu_data, &mut commands);
            }
            
        },
        
        Err(GameError::RecvError(_))=>{

        },
        Err(GameError::ComputeError(msg))=>{
            error!("recv_compute_task error={:?}",msg);
        },
        Err(err)=>{
            error!("recv_compute_task unknown error : {:?}",err.to_string());
            
        }
    }
}  



pub fn send_http_request(
    http_channel: Res<AsyncHttpResource>,
    mut in_game_events: EventReader<InGameContractEvent>,
    mut popup_draw_event: EventWriter<PopupDrawEvent>,
    // game: Res<Game>,
)
{
    for event in in_game_events.read()
    {
        match event{
            InGameContractEvent::RevealMaskedCard{masked_card,extra_data}=>{
                let (secret_key, request_type)= match &extra_data{
                    RequestExtraData::RevealEnvCard{secret_key, request_type,..}=>{
                        (secret_key.clone(), *request_type)
                    },
                    RequestExtraData::RevealOtherPlayersCard{secret_key, request_type,..}=>{
                        (secret_key.clone(), *request_type)
                    },
                    RequestExtraData::PlayCardOnDeck{secret_key, request_type,..}=>{
                        (secret_key.clone(), *request_type)
                    },
                };
                //https://zypher-test.onrender.com/reveal_with_snark
                http_channel.send_http_call("http://127.0.0.1:10000/reveal_with_snark".to_owned(),
                    RevealRequestData{
                        secret_key:secret_key.clone(),
                        masked_card:masked_card.clone(),
                        
                    },  extra_data.clone()
                );
                
                popup_draw_event.send(
                    PopupDrawEvent::ShowPopup(PopupData {
                        msg: format!("Waiting for txn"),
                        popup_type: request_type,
                        action_yes: None,
                        action_no : None,
                    })
                );
            },
            _=>{}
        }
    }
}


pub fn await_http_request(  http_channel: Res<AsyncHttpResource>,
    mut contract_events_writer: EventWriter<DelegateTxnSendEvent>,
    mut menu_data: ResMut<MenuData>,
    mut commands: Commands
)
{
    match http_channel.recv_http_response(){
        Ok(AsyncHttpResponse{data,extra_data})=>{
            info!("http req done, data= {:?}",data);
            contract_events_writer.send(DelegateTxnSendEvent::RevealCard{
                reveal_map: data.data,
                request_extra_data: extra_data});
             
            
        },
        Err(err)=>{if err!="empty" {
            hide_popup(&mut menu_data, &mut commands);
            error!("await_http_request error {:?}",err);
        }}
    };
}