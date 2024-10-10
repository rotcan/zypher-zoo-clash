use zshuffle::keygen::{Keypair as CoreKeypair};
use zshuffle::utils::{MaskedCardWithProof,MaskedCard,ShuffledCardsWithProof,RevealedCardWithSnarkProof,RevealedCardWithProof,
    default_prng,point_to_uncompress,scalar_to_hex,point_to_hex,uncompress_to_point,};
use zshuffle::{wasm,};
use serde::{Serialize,Deserialize};
use bevy_web3::types::H160;
use bevy_pkv::PkvStore;
use super::{Point};
use bevy::prelude::*;
use crate::error::GameError;
//use bevy::prelude::*;

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct Keypair {
    /// 0xHex (U256)
    pub sk: String,
    /// 0xHex (U256)
    pub pk: String,
    /// public key uncompress x, y
    pub pkxy: (String, String),
}

type ShuffleResult<T> = Result<T, GameError>;
pub fn generate_key(address: &H160, pkv: &mut PkvStore)->Keypair{
    let key=format!("keypair_{}",hex::encode(address));
    if let Ok(keypair) = pkv.get::<Keypair>(key.as_str()) {
//        info!("Saved Shuffle key = {:?}",keypair);
        keypair
    } else {
        let mut prng = default_prng();
        let keypair = CoreKeypair::generate(&mut prng);
        let pkxy = point_to_uncompress(&keypair.public, true);

        let ret = Keypair {
            sk: scalar_to_hex(&keypair.secret, true),
            pk: point_to_hex(&keypair.public, true),
            pkxy,
        };
        pkv.set(key.as_str(), &ret).expect("failed to store keypair");
        ret
        
    }

    
}

pub fn public_compress(point : &Point)->ShuffleResult<String>{
    let mut x_bytes: [u8;32] = [0u8;32];
    let mut y_bytes: [u8;32] = [0u8;32];
    point.x.to_big_endian(&mut x_bytes);
    point.y.to_big_endian(&mut y_bytes);
    let x = hex::encode(x_bytes);
    let y=  hex::encode(y_bytes);
    // info!("point={:?} {:?}",x,y);
    
    let pk = uncompress_to_point(&x, &y)?;
    Ok(point_to_hex(&pk, true))
}

pub fn init_prover_key(num: i32)->ShuffleResult<()> {
    Ok(wasm::init_prover_key(num)?)
}

pub fn refresh_joint_key(game_key: String, num : i32 )-> ShuffleResult<Vec<String>>{
    Ok(wasm::refresh_joint_key(game_key, num)?)
}

pub fn init_masked_cards(joint_key: String, num: i32)->ShuffleResult<Vec<MaskedCardWithProof>>{
    Ok(wasm::init_masked_cards(joint_key,num)?)
}

pub fn shuffle_cards(joint_key: String, deck: Vec<MaskedCard>)->ShuffleResult<ShuffledCardsWithProof>{
    Ok(wasm::shuffle_cards(joint_key,deck)?)
}

pub fn verify_shuffled_cards(first_deck: Vec<MaskedCard>, second_deck: Vec<MaskedCard>, proof: String)->ShuffleResult<bool>{
    Ok(wasm::verify_shuffled_cards(first_deck,second_deck,proof)?)
}

pub fn init_reveal_key()->ShuffleResult<()>{
    info!("init_reveal_key");
    Ok(wasm::init_reveal_key())
}

pub fn reveal_card_with_snark(sk: String, card: MaskedCard) -> ShuffleResult<RevealedCardWithSnarkProof> {
    Ok(wasm::reveal_card_with_snark(sk,card)?)
}

pub fn reveal_card(sk: String, card: MaskedCard)->ShuffleResult<RevealedCardWithProof>{
    Ok(wasm::reveal_card(sk,card)?)
}

pub fn unmask_card(sk: String, card: MaskedCard, reveals:  Vec<(String, String)>)->ShuffleResult<i32>{
    Ok(wasm::unmask_card(sk,card,reveals)?)
}
