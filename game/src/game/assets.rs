use bevy::prelude::*;
use std::collections::HashMap;

pub const FACEDOWN_KEY: &str="card-down";
pub const FACEUP_KEY: &str="card-up";
pub const ENVUP_KEY: &str="env-up";
pub const FINISH_KEY: &str="finish";
pub const TRACK: &str="track-block";
//ANIMALS
pub const S_BULL: &str="s_bull";
pub const S_HORSE: &str="s_horse";
pub const A_GORILLA: &str="a_gorilla";
pub const A_DEER: &str="a_deer";
pub const B_GIRAFFE: &str="b_giraffe";
pub const B_RABBIT: &str="b_rabbit";
pub const C_CAT: &str="c_cat";
pub const C_DOG: &str="c_dog";
pub const D_FROG: &str="d_frog";
pub const D_SHEEP: &str="d_sheep";
//ENEMY
pub const E1: &str="e1";
pub const E2: &str="e2";
pub const E3: &str="e3";
//GEOGRAPHY
pub const G1: &str="g1";
pub const G2: &str="g2";
pub const G3: &str="g3";
pub const G4: &str="g4";
pub const G5: &str="g5";
pub const G6: &str="g6";

#[derive(Resource,Default)]
pub struct CardImages{
    pub cards: HashMap<String,Handle<Image>>,
    pub animals: HashMap<String,Handle<Image>>,
}

pub fn load_sprites(
    mut card_images: ResMut<CardImages>,
    asset_server: Res<AssetServer>
) {
    card_images.cards.insert(FACEDOWN_KEY.to_owned(),asset_server.load("cards/card-back.png"));
    card_images.cards.insert(FACEUP_KEY.to_owned(),asset_server.load("cards/card-front-8.png"));
    card_images.cards.insert(ENVUP_KEY.to_owned(),asset_server.load("cards/env-front.png"));
    card_images.cards.insert(FINISH_KEY.to_owned(),asset_server.load("cards/finish-icon-2.png"));
    card_images.cards.insert(TRACK.to_owned(),asset_server.load("cards/track-block.png"));
    //Animals
    card_images.cards.insert(S_BULL.to_owned(),asset_server.load("cards/s_bull.png"));
    card_images.cards.insert(S_HORSE.to_owned(),asset_server.load("cards/s_horse.png"));
    card_images.cards.insert(A_DEER.to_owned(),asset_server.load("cards/a_deer.png"));
    card_images.cards.insert(A_GORILLA.to_owned(),asset_server.load("cards/a_gorilla.png"));
    card_images.cards.insert(B_GIRAFFE.to_owned(),asset_server.load("cards/b_giraffe.png"));
    card_images.cards.insert(B_RABBIT.to_owned(),asset_server.load("cards/b_rabbit.png"));
    card_images.cards.insert(C_CAT.to_owned(),asset_server.load("cards/c_cat.png"));
    card_images.cards.insert(C_DOG.to_owned(),asset_server.load("cards/c_dog.png"));
    card_images.cards.insert(D_SHEEP.to_owned(),asset_server.load("cards/d_sheep.png"));
    card_images.cards.insert(D_FROG.to_owned(),asset_server.load("cards/d_frog.png"));
    //ENEMY
    card_images.cards.insert(E1.to_owned(),asset_server.load("cards/lion.png"));
    card_images.cards.insert(E2.to_owned(),asset_server.load("cards/wolf.png"));
    card_images.cards.insert(E3.to_owned(),asset_server.load("cards/bear.png"));
    //GEOGRAPHY
    card_images.cards.insert(G1.to_owned(),asset_server.load("cards/farm.png"));
    card_images.cards.insert(G2.to_owned(),asset_server.load("cards/forest.png"));
    card_images.cards.insert(G3.to_owned(),asset_server.load("cards/mountain.png"));
    card_images.cards.insert(G4.to_owned(),asset_server.load("cards/desert.png"));
    card_images.cards.insert(G5.to_owned(),asset_server.load("cards/river.png"));
    card_images.cards.insert(G6.to_owned(),asset_server.load("cards/swamp.png"));
    
}