use bevy::prelude::*;
use bevy_web3::types::{H160};
use bevy_web3::plugin::{WalletChannel,get_address_from_string,EthContract,tokens::{Uint,Tokenizable,Token}};
use super::{Point,VRF_MIN_BALANCE,GameContractIxType,init_masked_cards,shuffle_cards,public_compress,convert_string_to_u256,
    convert_vec_string_to_u256,WINNING_SCORE,
    verify_shuffled_cards,u256_to_string,PlayerAction};
use crate::error::GameError;
use zshuffle::utils::{MaskedCard};

type SendResult<T>=Result<T,GameError>;



// pub fn init_masked_cards(joint_key: String, num: i32)->ShuffleResult<Vec<MaskedCardWithProof>>{
//     Ok(wasm::init_masked_cards(joint_key,num)?)
// }

// pub fn shuffle_cards(joint_key: String, deck: Vec<MaskedCard>)->ShuffleResult<ShuffledCardsWithProof>{
//     Ok(wasm::shuffle_cards(joint_key,deck)?)
// }
fn get_masked_card_token(data: Vec<MaskedCard>)->Token{
    Token::Array(
        data.into_iter().map(
        |m| Token::FixedArray(vec![
            Token::Uint(convert_string_to_u256(m.0.clone())),
            Token::Uint(convert_string_to_u256(m.1.clone())),
            Token::Uint(convert_string_to_u256(m.2.clone())),
            Token::Uint(convert_string_to_u256(m.3.clone())),
            ])
        ).collect::<Vec<Token>>()
    )
}

pub fn vec_to_masked_card(data : &Vec<Uint>)->MaskedCard{
    MaskedCard(u256_to_string(data[0].clone()), 
        u256_to_string(data[1].clone()), 
        u256_to_string(data[2].clone()), 
        u256_to_string(data[3].clone()), 
    )
}

pub fn vec_to_masked_cards(data: &Vec<Vec<Uint>>)->Vec<MaskedCard>{
    data.into_iter().map(
        |m| vec_to_masked_card(m)
    ).collect::<Vec<MaskedCard>>()
}

pub fn vec_to_reveals(data:&Vec<Vec<Vec<Uint>>>, hand_index: usize)->Vec<(String,String)>{
    data[hand_index].iter().map(|m| (u256_to_string(m[0]),u256_to_string(m[1]))).collect::<Vec<(String,String)>>()
}

 
pub fn token_to_masked_cards(data: Token)->Vec<MaskedCard>{
    data.into_array().unwrap().into_iter().map(
        |m| vec_to_masked_card(&m.into_fixed_array().unwrap().into_iter().map(|m2| m2.into_uint().unwrap()).collect::<Vec<Uint>>())
    ).collect::<Vec<MaskedCard>>()
}

pub fn debug_masked_cards_token(data: Token){
    let debug_data: Vec<String> = data.into_array().unwrap().into_iter().map(
        |m| m.into_fixed_array().unwrap().into_iter().map(|m2| u256_to_string(m2.into_uint().unwrap())).collect::<Vec<String>>()
    ).collect::<Vec<Vec<String>>>().into_iter().flatten().collect::<Vec<String>>();
    for item in debug_data.iter(){
        info!("0x{}",item);
    };
}

pub fn get_proof_token(data: String)->Token{
    let bytes= hex::decode(&data.trim_start_matches("0x")).unwrap();
    Token::Bytes(bytes)
}

pub fn proof_token_to_string(data: Token)->String{
    format!("0x{}",hex::encode(data.into_bytes().unwrap()))
}
fn send_txn(ix_type: &GameContractIxType, data: Vec<Token>,from_address:H160,game_address: H160,
    value: Option<Uint>,contract: &EthContract,wallet: &Res<WalletChannel>,estimated_wait_time: Option<f32>,)->SendResult<()>{
        match contract.encode_input(ix_type.get_ix_method().as_str(),&data){
            Ok(data)=>{
                wallet.send(from_address,game_address,data, 
                    Some(ix_type.clone() as i32), 
                value,estimated_wait_time); //
                Ok(())
            },
            Err(e)=>{
                //error!("Error e:{:?}",e.to_string())
                Err(GameError::SendTxnError(format!("Error e:{:?}",e.to_string())))
            }
        }
    }

    
pub fn send_txn_init_player(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String,vrf_balance: Option<Uint>, ix_type: &GameContractIxType)->SendResult<()>{
    //let game_address=game.game_contract.address();
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    // info!("from_address={:?}",from_address);
    let value:Option<Uint>=if let Some(vrf_balance) = vrf_balance{
        if vrf_balance>VRF_MIN_BALANCE.into(){
            Some(0.into())
        }else{
            Some(VRF_MIN_BALANCE.into())
        }
    }else{
        Some(VRF_MIN_BALANCE.into())
    };
     //let data: Vec<Token>= vec![ from_address.into_token(),50u8.into_token()];
    let data: Vec<Token>= if value.is_some() {
        vec![1u64.into_token()]
    }else{
        vec![0u64.into_token()]
    };

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),value,&contract,wallet,Some(30.))
    
}

pub fn send_txn_create_match(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    player_count: u8,player_public_key: Point,vrf_balance: Option<Uint>,)->SendResult<()>
{

    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    // info!("from_address={:?}",from_address);
    let value:Option<Uint>=if let Some(vrf_balance) = vrf_balance{
        if vrf_balance>VRF_MIN_BALANCE.into(){
            Some(0.into())
        }else{
            Some((VRF_MIN_BALANCE*2).into())
        }
    }else{
        Some((VRF_MIN_BALANCE*2).into())
    };
    let topup: u64 = if let Some(ref value) = value{
        if value == &Uint::zero() {
            0
        }else{
            1
        }
    }else{
        1
    };
    let data: Vec<Token>=vec![Token::Tuple(vec![player_public_key.x.into_token(),
    player_public_key.y.into_token()]),player_count.into_token(),WINNING_SCORE.into_token(),topup.into_token()];


    send_txn(ix_type, data,from_address.clone(),game_address.clone(),value,&contract,wallet,Some(30.))
}

pub fn send_txn_set_creator_deck(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint, card_indices: Vec<Uint>)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    let data: Vec<Token>=vec![match_index.into_token(),
    Token::Array(card_indices.into_iter().map(|m| m.into_token()).collect::<Vec<Token>>())];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}


pub fn send_txn_join_match(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint,player_public_key: Point, card_indices: Vec<Uint>)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    let data: Vec<Token>=vec![match_index.into_token(),
    Token::Tuple(vec![player_public_key.x.into_token(),
    player_public_key.y.into_token()]),
    Token::Array(card_indices.into_iter().map(|m| m.into_token()).collect::<Vec<Token>>())];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}


pub fn send_txn_joint_key(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint, pkc: Vec<Uint>)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    let data: Vec<Token>=vec![match_index.into_token(),
    Token::FixedArray(pkc.into_iter().map(|m| m.into_token()).collect::<Vec<Token>>())];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}


//creator
pub fn send_txn_mask_and_shuffle_env_deck(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint,game_key: Point, num : i32)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    let joint_key_string=public_compress(&game_key).unwrap();
    // info!("joint_key_string={:?}",joint_key_string);
    let masked_cards=init_masked_cards(joint_key_string.clone(),num).unwrap();
    let masked_card_deck: Vec<MaskedCard>=masked_cards.iter().map(|m| m.card.clone() ).collect::<Vec<MaskedCard>>();
    let shuffle_cards=shuffle_cards(joint_key_string,masked_card_deck.clone()).unwrap();
    //masked_cards: Vec<MaskedCard>, shuffled_cards: Vec<MaskedCard>, proof: String
    let shuffle_cards_deck=shuffle_cards.cards;
    let proof= shuffle_cards.proof;
    // info!("send_txn_mask_and_shuffle_env_deck proof={:?}", proof);
    //let _= masked_card_deck.clone().iter().map(|m| info!("masked_card_deck={:?} ",m)).collect::<Vec<_>>();
    // let flatten_deck= shuffle_cards_deck.clone().iter().map(|m| vec![m.0.clone(),m.1.clone(),m.2.clone(),m.3.clone()])
    // .collect::<Vec<Vec<String>>>().into_iter().flatten().collect::<Vec<String>>();
    // info!("shuffle_cards_deck={:?} ",flatten_deck);
    // info!("masked cards={:?}",get_masked_card_token(masked_card_deck.clone()));
    // info!("verify shuffled cards={:?}",get_masked_card_token(shuffle_cards_deck.clone()));
    // info!("proof token={:?}",get_proof_token(proof.clone()));

    let masked_cards_token=get_masked_card_token(masked_card_deck.clone());
    let shuffled_cards_token=get_masked_card_token(shuffle_cards_deck);
    let proof_token=get_proof_token(proof.clone());
    info!("verify_shuffled_cards = {:?}",verify_shuffled_cards(
        token_to_masked_cards(masked_cards_token.clone() ),
        token_to_masked_cards(shuffled_cards_token.clone() ),
        proof_token_to_string(proof_token.clone())
            // shuffle_cards_deck.clone(),proof.clone()
        ));
    // info!("Masked cards");
    // debug_masked_cards_token(masked_cards_token.clone());
    // info!("Shuffled cards");
    // debug_masked_cards_token(shuffled_cards_token.clone());
    // info!("Proof");
    // info!("{}",proof_token_to_string(proof_token.clone()));

    let data: Vec<Token>=vec![match_index.into_token(),
    masked_cards_token,shuffled_cards_token,proof_token
    ];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}
//other
pub fn send_txn_shuffle_env_deck(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint,game_key:Point, current_deck: Vec<Vec<Uint>>,)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    let joint_key_string=public_compress(&game_key).unwrap();
    
    let current_deck=vec_to_masked_cards(&current_deck);
    let shuffle_cards=shuffle_cards(joint_key_string,current_deck.clone()).unwrap();
    //masked_cards: Vec<MaskedCard>, shuffled_cards: Vec<MaskedCard>, proof: String
    let shuffle_cards_deck=shuffle_cards.cards;
    let proof= shuffle_cards.proof;
    
    // matchIndex,  maskedCards, shuffledCards, proof
    let data: Vec<Token>=vec![match_index.into_token(),
    get_masked_card_token(current_deck.clone()),get_masked_card_token(shuffle_cards_deck),
    get_proof_token(proof.clone())
    ];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}

//both
pub fn send_txn_shuffle_self_deck(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint,game_key: Point, num : i32)->SendResult<()>{
        let game_address=contract.address();
        let from_address = get_address_from_string(from.as_str()).unwrap();
        let joint_key_string=public_compress(&game_key).unwrap();
        let masked_cards=init_masked_cards(joint_key_string.clone(),num).unwrap();
        let masked_card_deck: Vec<MaskedCard>=masked_cards.iter().map(|m| m.card.clone() ).collect::<Vec<MaskedCard>>();
        let shuffle_cards=shuffle_cards(joint_key_string,masked_card_deck.clone()).unwrap();
        //masked_cards: Vec<MaskedCard>, shuffled_cards: Vec<MaskedCard>, proof: String
        let shuffle_cards_deck=shuffle_cards.cards;
        let proof= shuffle_cards.proof;
        // info!("send_txn_mask_and_shuffle_env_deck proof={:?}", proof);
        let _= masked_card_deck.clone().iter().map(|m| info!("masked_card_deck={:?} ",m)).collect::<Vec<_>>();
        let _= shuffle_cards_deck.clone().iter().map(|m| info!("shuffle_cards_deck={:?} ",m)).collect::<Vec<_>>();
        // info!("shuffle_cards_deck={:?} ",shuffle_cards_deck);
        info!("verify_shuffled_cards = {:?}",verify_shuffled_cards(masked_card_deck.clone(),
                shuffle_cards_deck.clone(),proof.clone()));
        let data: Vec<Token>=vec![match_index.into_token(),
        get_masked_card_token(masked_card_deck.clone()),get_masked_card_token(shuffle_cards_deck),
        get_proof_token(proof.clone())
        ];
    
        send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)

}
//both
pub fn send_txn_shuffle_others_deck(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint,game_key: Point, player_address: H160, current_deck: Vec<Vec<Uint>>,)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    let joint_key_string=public_compress(&game_key).unwrap();
    
    let current_deck=vec_to_masked_cards(&current_deck);
    let shuffle_cards=shuffle_cards(joint_key_string,current_deck.clone()).unwrap();
    //masked_cards: Vec<MaskedCard>, shuffled_cards: Vec<MaskedCard>, proof: String
    let shuffle_cards_deck=shuffle_cards.cards;
    let proof= shuffle_cards.proof;
    
    // matchIndex,  maskedCards, shuffledCards, proof
    let data: Vec<Token>=vec![match_index.into_token(),player_address.into_token(),
    get_masked_card_token(current_deck.clone()),get_masked_card_token(shuffle_cards_deck),
    get_proof_token(proof.clone())
    ];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}

//both
pub fn show_next_env_card(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint, 
    card: (String,String), snark_proof: Vec<String>,
    )->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
   
    
    let card=vec![card.0,card.1].iter().map(|m| convert_string_to_u256(m.to_owned()).into_token()).collect::<Vec<Token>>();
    let reveal_proof=snark_proof.iter().map(|m| convert_string_to_u256(m.to_owned()).into_token()).collect::<Vec<Token>>();
    let data: Vec<Token> = vec![match_index.into_token(),Token::FixedArray(card),
        Token::FixedArray(reveal_proof)];

    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}

pub fn reveal_other_player_cards(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint,  reveal_count: u64, 
   player_index: u64,cards: Vec< (String,String)>, snark_proofs: Vec<Vec<String>>)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
 
    let cards = cards.into_iter().map(|m| convert_vec_string_to_u256(vec![m.0,m.1])).collect::<Vec<Vec<Uint>>>();
    let cards = cards.into_iter().map(|m| Token::FixedArray(vec![Token::Uint(m[0]),Token::Uint(m[1])])).collect::<Vec<Token>>();

    
    let reveal_proofs=snark_proofs.into_iter().map(|m| convert_vec_string_to_u256(m)).collect::<Vec<Vec<Uint>>>();
    let reveal_proofs=reveal_proofs.into_iter().map(|m| m.into_iter().map(|m2| Token::Uint(m2)).collect::<Vec<Token>>()).collect::<Vec<Vec<Token>>>();
    let reveal_proofs=reveal_proofs.into_iter().map(|m| Token::FixedArray(m)).collect::<Vec<Token>>();

    let data: Vec<Token> = vec![match_index.into_token(),player_index.into_token(),
    reveal_count.into_token(), 
        Token::Array(cards),Token::Array(reveal_proofs)];
    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}

pub fn player_action(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint, player_action: PlayerAction)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    
    let action_value = if player_action==PlayerAction::ShowEnvCard{
        1
    }else{
        0
    };
    let data: Vec<Token> = vec![match_index.into_token(),Token::Uint(action_value.into())];
    info!("player action data ={:?}",data);
    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}

pub fn play_card(wallet: &Res<WalletChannel>,
    contract: EthContract,from: String, ix_type: &GameContractIxType,
    match_index: Uint, hand_index: u64, 
    card: (String,String), snark_proof: Vec<String>,)->SendResult<()>
{
    let game_address=contract.address();
    let from_address = get_address_from_string(from.as_str()).unwrap();
    
    //playCardOnDeck(matchIndex, handIndex, revealToken, proof)
    let card=vec![card.0,card.1].iter().map(|m| convert_string_to_u256(m.to_owned()).into_token()).collect::<Vec<Token>>();
    let reveal_proof=snark_proof.iter().map(|m| convert_string_to_u256(m.to_owned()).into_token()).collect::<Vec<Token>>();
    
    let data: Vec<Token> = vec![match_index.into_token(), hand_index.into_token(),Token::FixedArray(card),
    Token::FixedArray(reveal_proof)];
    
    send_txn(ix_type, data,from_address.clone(),game_address.clone(),None,&contract,wallet,None)
}
