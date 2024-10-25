use bevy::prelude::*;

use crate::{MAIN_LAYER,OVERLAY_LAYER,POPUP_LAYER};
use super::{create_node_bundle,create_styled_node_bundle,create_text_bundle,add_button,add_editable_button,DeckCardType,Player,
    CardComponentType,CardFace,DeckType,OutlineArgs,
    GameStatus,MenuData,CardImages,StyleArgs,Game,CardComponent,add_active_card,CARD_HEIGHT,add_text,UiElementComponent,UiElement,InGameElements,
};
use crate::web3::{GameActions,PopupResult,ActionType,get_player_index_by_address,Web3Actions,GameContractIxType,};

#[derive(Debug)]
pub struct PopupData{
    pub msg: String,
    pub popup_type: i32,
    pub action_yes: Option<PopupResult>,
    pub action_no: Option<PopupResult>,
    
}

#[derive(Event,Debug)]
pub enum PopupDrawEvent {
    ShowPopup(PopupData),
    HidePopup,
    ShowMatchEndPopup,
}

#[derive(Event,Debug)]
pub struct PopupResponseEvent{
    pub popup_type: i32,
    pub response: PopupResult,
}

pub fn delete_overlay(menu_data: &mut MenuData, commands: &mut  Commands,){
    if let Some(overlay) = menu_data.overlay_entity.take(){
        if let Some(entity) = menu_data.main_entity {
            commands.entity(entity).remove_children(&[overlay]);
        }
        if let Some(mut overlay_commands) = commands.get_entity(overlay) {
            overlay_commands.despawn_descendants();
            overlay_commands.despawn();
        }
       
        // commands.entity(overlay).despawn_descendants();
        // commands.entity(overlay).despawn();
        menu_data.active_layer= MAIN_LAYER;
    }
}

pub fn delete_popup(menu_data:  &mut MenuData, commands: &mut  Commands,){
    if let Some(existing_popup) = menu_data.popup_entity.take() { 
        if let Some(entity) = menu_data.main_entity {
            commands.entity(entity).remove_children(&[existing_popup]);
        }
        if let Some(mut popup_commands) = commands.get_entity(existing_popup) {
            popup_commands.despawn_descendants();
            popup_commands.despawn();
        }
       
        
        menu_data.active_layer= OVERLAY_LAYER;
    }
}


pub fn create_overlay(menu_data: &mut MenuData, commands: &mut  Commands,){
    delete_overlay(menu_data,commands);

    let parent = commands.spawn(create_node_bundle(100., 100., FlexDirection::Column,
        JustifyContent::Center,Some(AlignItems::Center), Overflow::visible(),Some(Color::srgba(0.0,0.0,0.0,0.8)),
        Some(PositionType::Absolute),None,None, MAIN_LAYER)).id();
    //menu_data.overlay_entity=Some(parent);
    if let Some(entity) = menu_data.main_entity {
        commands.entity(entity).add_child(parent);
    }
    menu_data.overlay_entity=Some(parent);
    menu_data.active_layer= OVERLAY_LAYER;
}

pub fn update_winner_card( game: &Game, commands: &mut Commands,card_images: &CardImages, parent_entity: Entity){
    commands.entity(parent_entity).despawn_descendants();
    
    info!("update_winner_card");
    if let Some(address_bytes) = game.account_bytes {
        if let Some(index) = get_player_index_by_address(&game, &address_bytes)
        {
            if let Some(ref match_state) = game.match_state{
                let current_player_data=&game.players_data[index];
                if match_state.winner == current_player_data.player_state.player {
                    if let Some((_key,value)) = current_player_data.all_cards.all_card_props.last_key_value() {
                        //info!("update_winner_card value={:?}",value);
                        let key_idx=0;
                        let card_component=CardComponent{
                            index:key_idx,
                            onchain_index: Some(value.onchain_index),
                            face: CardFace::Up,
                            val: Some(DeckCardType::PlayerCard(value.clone())),
                            card_type: CardComponentType::PlayerCard(Player::Player1,key_idx),
                            is_selectable: true,
                            ..default()
                        };
                        let new_card=add_active_card( commands, &card_images,card_component,POPUP_LAYER, DeckType::Player1);
                        commands.entity(parent_entity).add_child(new_card);
                    }
                    
                }
            }
            
            
        }
    }
    
}

pub fn create_winner_popup(menu_data: &mut MenuData, commands: &mut  Commands,game: &Game,_card_images: &CardImages){
    let layer=POPUP_LAYER;
    let parent_entity= commands.spawn(create_styled_node_bundle(StyleArgs{ width: Val::Percent(100.),
    height: Val::Percent(100.),direction: FlexDirection::Column, justify_content: JustifyContent::Center,
    align_items: AlignItems::Center, position_type: PositionType::Absolute,
     layer , ..default()})).id();

    let modal_entity =   commands.spawn(create_styled_node_bundle(StyleArgs{ width: Val::Px(600.),
        height: Val::Px(500.),direction: FlexDirection::Column, justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, position_type: PositionType::Relative, background_color: BackgroundColor(Color::srgba(0.1,0.1,0.1,1.0)),
        outline: Some( OutlineArgs{
            width: Val::Px(1.),
            offset: Val::Px(1.),
            color: Color::srgb(1.,1.,1.),
        }),
         layer , ..default()})).id();
    
    let layout_entity = commands.spawn(create_styled_node_bundle(StyleArgs{ width: Val::Percent(100.),
        height: Val::Percent(100.),direction: FlexDirection::Column,justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, position_type: PositionType::Relative,  ..default()})).id();
    
    let layout_label_entity = commands.spawn(create_styled_node_bundle(StyleArgs{ width: Val::Percent(100.),
        height: Val::Px(40.),direction: FlexDirection::Column,justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(5.0)), margin: UiRect::all(Val::Px(5.0)),
        align_items: AlignItems::Center, position_type: PositionType::Relative,  ..default()})).id();
    let layout_top_entity = commands.spawn(create_styled_node_bundle(StyleArgs{ width: Val::Percent(100.),
        height: Val::Px(CARD_HEIGHT+20.),direction: FlexDirection::Column,justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, position_type: PositionType::Relative,  ..default()})).id();

    let layout_bottom_entity = commands.spawn(create_styled_node_bundle(StyleArgs{ width: Val::Percent(100.),
        height: Val::Px(250.),direction: FlexDirection::Row,justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, position_type: PositionType::Relative,  ..default()})).id();
    
    if let Some(ref match_state) = game.match_state{
        game.account_bytes.as_ref().map(|address_bytes| {
            //Add text
            if &match_state.winner== address_bytes{
                let text_entity=add_text(commands,format!("Congratulations! : {}",match_state.winner).as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
                    direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                   overflow: Overflow::visible(),
                layer,..default()});
                let text_entity_2=add_text(commands,format!("Your reward for last game!").as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
                    direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                   overflow: Overflow::visible(),
                layer,..default()});
                commands.entity(layout_label_entity).add_child(text_entity);
                commands.entity(layout_label_entity).add_child(text_entity_2);
                //Add winning card
                commands.entity(layout_top_entity).insert(
                    UiElementComponent::new(
                        UiElement::InGame(InGameElements::WinnerCard),
                        None));
                let text_entity_3=add_text(commands,format!("Waiting!").as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
                    direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    overflow: Overflow::visible(),
                layer,..default()});
                commands.entity(layout_top_entity).add_child(text_entity_3);
                // update_winner_card(&game,commands,&card_images, layout_top_entity);
            }else{
                let text_entity=add_text(commands,format!("You lost. Try again, {}",address_bytes).as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
                    direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    overflow: Overflow::visible(),
                layer,..default()});
                
                commands.entity(layout_label_entity).add_child(text_entity);
                //Todo: remove this
                 //Add winning card
                //  commands.entity(layout_top_entity).insert(
                //     UiElementComponent::new(
                //         UiElement::InGame(InGameElements::WinnerCard),
                //         None));
                // update_winner_card(&game,commands,&card_images, layout_top_entity);
            }
           
        });
    }
    
 
    //Add create new match / join match 
    let create_button= add_button(commands,GameStatus::CreateNewMatch,
        ActionType::Web3Actions(Web3Actions::GameContractAction(GameContractIxType::CreateNewMatch)),
    //    80.0,40.0,Some(4.0),Some(4.0),MAIN_LAYER
    StyleArgs{width:Val::Percent(50.),height: Val::Percent(25.),layer,..StyleArgs::button_style()}  
    );
    commands.entity(layout_bottom_entity).add_child(create_button);
    let join_button=add_editable_button(commands,"0",80.,50. ,GameStatus::JoinMatchPreSelect,
        ActionType::Web3Actions(Web3Actions::GameContractAction(GameContractIxType::JoinMatchPreSelect)));
    commands.entity(layout_bottom_entity).add_child(join_button);

    commands.entity(layout_entity).add_child(layout_label_entity);
    commands.entity(layout_entity).add_child(layout_top_entity);
    commands.entity(layout_entity).add_child(layout_bottom_entity);
    commands.entity(modal_entity).add_child(layout_entity);
    commands.entity(parent_entity).add_child(modal_entity);

    if let Some(entity) = menu_data.main_entity {
        commands.entity(entity).add_child(parent_entity);
    }
    menu_data.popup_entity=Some(parent_entity);
    menu_data.active_layer= layer;
}

pub fn create_popup_entity( menu_data: &mut MenuData, commands: &mut  Commands,
    width: f32, height: f32,msg: String,  
    action_yes: Option<PopupResult>,action_no: Option<PopupResult>, ){
        delete_popup(menu_data,commands);

    let layer=POPUP_LAYER;
    let parent= commands.spawn(create_node_bundle(100.0, 100.0, FlexDirection::Column,
        JustifyContent::Center,Some(AlignItems::Center),Overflow::visible(),None,Some(PositionType::Absolute),
        None,None,layer)).id();
    
    let popup_screen = commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(width),
    height: Val::Percent(height),direction: FlexDirection::Column, justify_content: JustifyContent::Center,
    align_items: AlignItems::Center, position_type: PositionType::Relative, background_color: BackgroundColor(Color::srgba(0.1,0.1,0.1,1.0)),
    outline: Some( OutlineArgs{
        width: Val::Px(1.),
        offset: Val::Px(1.),
        color: Color::srgb(1.,1.,1.)
    }),layer,..default() }
        // width, height, FlexDirection::Column,
        // JustifyContent::Center, Some(AlignItems::Center),Overflow::visible(),Some(Color::srgba(0.0,0.0,0.0,1.0)),Some(PositionType::Relative),None,None,layer
        )).id();
    commands.entity(parent).add_child(popup_screen);
    
    let title_node_entity = commands.spawn(create_node_bundle(100.0, 50.0, FlexDirection::Row,
        JustifyContent::Center, Some(AlignItems::Center),Overflow::visible(),None,
        Some(PositionType::Relative),Some(4.0),Some(4.0),layer)).id();
    commands.entity(popup_screen).add_child(title_node_entity);

    let title_entity=commands.spawn(create_text_bundle(&msg, StyleArgs{layer,..default()})).id();
    commands.entity(title_node_entity).add_child(title_entity);
    
    let button_node_entity = commands.spawn(create_node_bundle(100.0, 50.0, FlexDirection::Row,
        JustifyContent::Center, Some(AlignItems::Center),Overflow::visible(),None,
        Some(PositionType::Relative),Some(4.0),Some(4.0),layer)).id();
    commands.entity(popup_screen).add_child(button_node_entity);
 
    action_yes.as_ref().map(|m|{
        let button = add_button(commands,GameStatus::PopupYes,
            ActionType::GameActions(GameActions::PopupActions(m.clone())),
        //    20.0,20.0,Some(4.0),Some(4.0),POPUP_LAYER
        StyleArgs{width:Val::Percent(20.),height: Val::Percent(20.),layer,..StyleArgs::button_style()} 
        );
        commands.entity(button_node_entity).add_child(button);
    });

    action_no.as_ref().map(|m| {
        let button = add_button(commands,GameStatus::PopupNo,
            ActionType::GameActions(GameActions::PopupActions(m.clone())),
            //20.0,20.0,Some(4.0),Some(4.0),POPUP_LAYER
            StyleArgs{width:Val::Percent(20.),height: Val::Percent(20.),layer,..StyleArgs::button_style()} 
        );
        commands.entity(button_node_entity).add_child(button);
    });
     
    //menu_data.popup_entity=Some(parent);
    if let Some(entity) = menu_data.main_entity {
        commands.entity(entity).add_child(parent);
    }
    menu_data.popup_entity=Some(parent);
    menu_data.active_layer= layer;
}

pub fn draw_popup(
    mut popup_draw_event: EventReader<PopupDrawEvent>,
    mut commands: Commands,
    mut menu_data: ResMut<MenuData>,
    game: Res<Game>,
    card_images: Res<CardImages>,
){
    let menu_data=&mut menu_data;
    let commands=&mut commands;
    for event in popup_draw_event.read(){
        match event{
            PopupDrawEvent::ShowPopup(data)=>{
                //let status = GameStatus::from_i32(event.popup_type);
                info!("Draw popup");
                create_overlay(menu_data,commands);
                create_popup_entity( menu_data,commands, 40.0,40.0,data.msg.clone(), 
                data.action_yes.clone(),data.action_no.clone(),);
            },
            PopupDrawEvent::HidePopup=>{
                hide_popup(menu_data,commands);
            },
            PopupDrawEvent::ShowMatchEndPopup=>{
                info!("Draw match end popup");
                create_overlay(menu_data,commands);
                create_winner_popup( menu_data,commands,&game,&card_images);
            }
        }
        
    }
}

pub fn hide_popup(menu_data: &mut MenuData, commands: &mut Commands,){
    delete_popup(menu_data, commands);
    delete_overlay( menu_data, commands);
}
//Bevy system
// pub fn popup_action(
//     mut popup_response: EventWriter<PopupResponseEvent>,
//     mut interaction_query: Query<
//         (
//             &Interaction,
//             &mut BackgroundColor,
//             &mut BorderColor,
//             &Children,
//         ),
//         (Changed<Interaction>, With<Button>),
//     >,
//     button_type_query: Query<&MenuButton>,
//     mut menu_data: ResMut<MenuData>,
//     mut commands: Commands,

// ){
//     let active_layer=menu_data.active_layer;
//     for (interaction, mut back_color, mut border_color, children) in &mut interaction_query{
//         match button_type_query.get(children[0]) {
//             Ok(button_type)=>{
//                 //info!("layer {:?} {:?}",button_type.layer,menu_data.active_layer );
//                 if button_type.layer == active_layer {
//                     match *interaction{
//                         Interaction::Pressed => {
//                             // *back_color = PRESSED_BUTTON.into();
//                             // border_color.0 = Color::srgb(1.0,0.0,0.0);

//                             match button_type.target {
//                                 ActionType::Web3Actions(ref web3_action)=>{},
//                                 ActionType::GameActions(ref game_actions)=>{
                                    
//                                     match game_actions{
//                                         GameActions::PopupActions(popup_result)=>{
//                                             match popup_result{
//                                                 PopupResult::Yes(val)=> {
//                                                     match GameStatus::from_i32(*val){
//                                                         GameStatus::PopupHide=>{
//                                                             hide_popup(&mut menu_data, &mut commands);
                                                            
//                                                         },
//                                                         _=>{},
//                                                     }
//                                                 },
//                                                 PopupResult::No(val)=> {
//                                                     match GameStatus::from_i32(*val){
//                                                         GameStatus::PopupHide=>{
//                                                             hide_popup(&mut menu_data, &mut commands);
                                                            
                                                           
//                                                         },
//                                                         _=>{},
//                                                     }
//                                                 },
//                                             }
                                            
//                                         },
//                                     }
                                    
//                                 },
//                             }
//                         },
//                         Interaction::Hovered => {
//                             *back_color = HOVERED_BUTTON.into();
//                             border_color.0 = Color::WHITE;
//                         },
//                         Interaction::None => {
//                             *back_color = NORMAL_BUTTON.into();
//                             border_color.0 = Color::BLACK;
//                         },
//                     }
//                 }
//             },
//             Err(_)=>{},
//         }
//     }
//     // popup_response.send(PopupResponseEvent{popup_type : event.popup_type,
//     //     response: None})
// }
 
// pub rarity: u64,
// pub animal : u64,
// pub shield: u64,
// pub health: u64,
// pub weakness: Vec<u64>,
// pub favored_geographies: Vec<u64>,
// pub steps: u64,
// pub onchain_index: Uint,
