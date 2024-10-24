use bevy::{a11y::{
    accesskit::{NodeBuilder, Role},
    AccessibilityNode,},
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*
};
use crate::web3::{Web3Actions,MenuButton,ActionType,GameContractIxType,GameActions,CardProp,EnvCard,
    EnvCardGeography,EnvCardEnemy,MatchStateEnum,
    get_player_addresses,get_player1_steps,get_player2_steps};
use crate::style::NORMAL_BUTTON;
use bevy_text_edit::{ TextEditFocus,TextEditable};
use super::{GameStatus,CardImages,FACEDOWN_KEY,FACEUP_KEY,ENVUP_KEY,S_BULL,S_HORSE,A_GORILLA,A_DEER,B_GIRAFFE,B_RABBIT,C_CAT,C_DOG,D_FROG,D_SHEEP,
    FINISH_KEY,TRACK,update_winner_card,};
use crate::MAIN_LAYER;
//use bevy_ecs::system::EntityCommands;
// use crate::style::BORDER_COLOR;
use bevy_web3::types::U256;
use bevy::render::view::visibility::RenderLayers;
// use bevy_pkv::PkvStore;
// use crate::GameState;
use crate::game::Game;
//use bevy::window::PrimaryWindow;
//use bevy_toast::{ToastEvent,ToastData};

pub const CARD_WIDTH: f32=150.;
pub const CARD_HEIGHT: f32=183.;
pub const ANIMAL_IMG_WIDTH: f32=80.;
pub const CARD_CONTAINER_WIDTH: f32=140.;
pub const INVISIBLE_COLOR: Color = Color::srgba(0.4,0.0,0.4,0.);

#[derive(Resource)]
pub struct MenuData{
    pub main_entity: Option<Entity>,
    pub popup_entity: Option<Entity>,
    pub overlay_entity: Option<Entity>,
    pub active_layer: usize,
}

#[derive(Eq,PartialEq,Debug,Clone)]
#[repr(i32)]
pub enum CardRarity{
    RarityS,
    RarityA,
    RarityB,
    RarityC,
    RarityD,
}

impl CardRarity{
    pub fn from_value(val: i32)->CardRarity{
        match val{
            0=>Self::RarityS,
            1=>Self::RarityA,
            2=>Self::RarityB,
            3=>Self::RarityC,
            4=>Self::RarityD,
            _=>panic!("Value not present for card elements"),
        }
    }
}

#[derive(Component, Default)]
pub struct ScrollingList {
    position: f32,
    id: usize,
}

impl ScrollingList{
    pub fn new(id: usize)->ScrollingList{
        Self{
            position: 0.,
            id
        }
    }
}


#[derive(Eq,PartialEq,Clone,Debug)]
pub enum CardFace{
    Up,
    Down
}

impl CardFace{
    pub fn get_image_key(&self)->&str{
        match &self{
            Self::Down=>FACEDOWN_KEY,
            Self::Up=>FACEUP_KEY,
        }
    }
}

#[derive(Eq,PartialEq,Clone,Debug)]
#[repr(u8)]
pub enum Player{
    Player1=0,
    Player2=1
}

#[derive(Eq,PartialEq,Clone,Debug)]
#[repr(u8)]
pub enum DeckType{
    Env,
    Player1,
    Player2,
}


#[derive(Eq,PartialEq,Clone,Debug)]
pub enum CardComponentType{
    OriginalCards,
    EnvCard,
    PlayerCard(Player,usize),
    InActiveCard(DeckType,usize),
}

#[derive(Eq,PartialEq,Clone,Debug)]
pub enum DeckCardType{
    PlayerCard(CardProp),
    EnvCard(EnvCard),
}

impl DeckCardType{
    pub fn get_image_key(&self)->Option<&str>{
        match &self{
            Self::PlayerCard(card_prop)=>card_prop.get_image_key(),
            Self::EnvCard(env_card)=>env_card.get_image_key(),
        }
    }
}

#[derive(Component,Debug)]
pub struct CardComponent{
    pub index: usize,
    pub onchain_index: Option<U256>,
    pub val: Option<DeckCardType>,
    pub selected : bool,
    pub is_selectable: bool,
    pub face: CardFace, 
    pub card_type: CardComponentType,
}
impl CardComponent{
    pub fn new(index: usize,onchain_index: Option<U256>,  val: Option<DeckCardType>,is_selectable:bool,face: CardFace,card_type: CardComponentType,)->Self{
        CardComponent{
            index,
            onchain_index,
            val,
            selected: false,
            is_selectable,
            face,
            card_type,
        }
    }
}

impl Default for CardComponent{
    fn default()->Self{
        CardComponent{
            index: 0,
            onchain_index: None,
            val: None,
            selected: false,
            is_selectable: false,
            face: CardFace::Down,
            card_type: CardComponentType::OriginalCards,
        }
    }
}
 
#[derive(Eq,PartialEq,Debug,Clone)]
pub enum BeforeGameElements{
    CardList(usize),
    Cards(usize,usize),
    Status
}

#[derive(Eq,PartialEq,Debug,Clone)]
pub enum InGameElements{
    Status,
    Track,
    PlayerOriginalCards(usize),
    PlayerLabel,
    EnvDeck,
    PlayerHandCards,
    PlayerActiveCard{player: Player},
    WinnerCard,
}

#[derive(Component)]
pub struct TrackElementComponent{
    pub track: usize,
    pub step: usize,
}

#[derive(Eq,PartialEq,Debug,Clone)]
pub enum PostGameElements{

}

#[derive(Eq,PartialEq,Debug,Clone)]
pub enum UiElement{
    BeforeGame(BeforeGameElements),
    InGame(InGameElements),
    PostGame(PostGameElements)
}

#[derive(Component)]
pub struct UiElementComponent{
    element_type: UiElement,
    status: Option<GameStatus>,
}

impl UiElementComponent{
    pub fn new(element_type: UiElement, status: Option<GameStatus>)->Self{
        UiElementComponent{
            element_type,
            status
        }
    }

    pub fn update_status(&mut self, status: Option<GameStatus>){
        self.status=status;
    }
}


impl Default for MenuData{
    fn default()->Self{
        MenuData{
            main_entity: None,
            popup_entity: None,
            overlay_entity: None,
            active_layer: MAIN_LAYER,
            //button_entities: HashMap::new(),
            // current_entity: None,
            // current_entity_name: None,
            // last_entity_name: None,
        }
    }
}

impl MenuData{

    pub fn init(&mut self, commands: &mut Commands){
        self.main_entity=Some(fullscreen_entity(commands,FlexDirection::Row,None,None,Some(AlignItems::FlexStart),MAIN_LAYER));
        self.popup_entity=None;
        self.overlay_entity=None;
        self.active_layer=MAIN_LAYER;
    }
  
 
}

#[derive(Clone,Debug)]
pub struct OutlineArgs{
    pub width: Val,
    pub offset: Val,
    pub color: Color
}

impl Default for OutlineArgs{
    fn default()->Self{
        OutlineArgs{
            width: Val::Px(1.),
            offset: Val::Px(1.),
            color: INVISIBLE_COLOR,
        }
    }
}



#[derive(Clone,Debug)]
pub struct StyleArgs{
    pub width: Val,
    pub height: Val,
    pub max_width:Val,
    pub max_height: Val,
    pub direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub overflow: Overflow,
    pub position_type: PositionType,
    pub margin: UiRect,
    pub padding: UiRect,
    pub border:UiRect,
    pub left: Val,
    pub top: Val,
    pub bottom: Val,
    pub right: Val,
    pub transform: Transform,
    pub border_color: BorderColor,
    pub background_color: BackgroundColor,
    pub image_key: String,
    pub layer: usize,
    pub outline: Option<OutlineArgs>,
    pub font_size: f32,
    pub color: Color,

}

impl Default for StyleArgs{
    fn default()->StyleArgs{
        Self{
            width: Val::Auto,
            height: Val::Auto,
            max_width: Val::Auto,
            max_height: Val::Auto,
            direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            overflow: Overflow::visible(),
            position_type: PositionType::Relative,
            margin: UiRect{
                top: Val::Px(0.),
                bottom: Val::Px(0.),
                left: Val::Px(0.),        
                right: Val::Px(0.),
            },
            padding: UiRect{
                top: Val::Px(0.),
                bottom: Val::Px(0.),
                left: Val::Px(0.),        
                right: Val::Px(0.),
            },
            border: UiRect{
                top: Val::Px(0.),
                bottom: Val::Px(0.),
                left: Val::Px(0.),        
                right: Val::Px(0.),
            },
            left: Val::Px(0.),
            top: Val::Px(0.),
            bottom: Val::Px(0.),
            right: Val::Px(0.),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            border_color: BorderColor(Color::BLACK),
            background_color:BackgroundColor(Color::NONE),
            image_key: FACEDOWN_KEY.to_owned(),
            layer: MAIN_LAYER,
            outline: None,
            font_size:25.0,
            color:Color::srgb(0.9, 0.9, 0.9),
        }
    }
}

impl StyleArgs{
    pub fn card_sprite()->StyleArgs{
        Self{
            margin: UiRect::top(Val::Px(2.)),
            border: UiRect::all(Val::Px(2.)),
            width: Val::Px(CARD_WIDTH),
            height: Val::Px(CARD_HEIGHT),
            outline: Some(OutlineArgs::default()),
            ..default()
        }
    }

    pub fn button_style()->StyleArgs{
        Self{
            border_color: BorderColor(Color::BLACK),
            background_color: NORMAL_BUTTON.into(),
            margin: UiRect::all(Val::Px(4.0)),
            padding:UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(2.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }
    }
}

pub fn create_node_bundle(width: f32, height: f32,direction: FlexDirection,
    justify: JustifyContent, align_items: Option<AlignItems>, overflow: Overflow,background_color: Option<Color>,
    position_type: Option<PositionType>,margin: Option<f32>, padding: Option<f32>,
     layer: usize)-> impl Bundle{
        let color = if let Some(color) = background_color {
            color
        }else{
            Color::NONE
        };

        let position = if let Some(position) = position_type {
            position
        }else{
            PositionType::Relative
        };

        let align_items= align_items.unwrap_or(AlignItems::FlexStart);
        let margin =margin.unwrap_or(0.0);
        let padding =padding.unwrap_or(0.0);

    (
        NodeBundle {
            style: Style {
                width: Val::Percent(width),
                height: Val::Percent(height),
                align_items: align_items,
                justify_content: justify,
                flex_direction: direction,
                overflow: overflow,
                position_type:position,
                padding: UiRect{
                    top: Val::Px(padding),
                    bottom: Val::Px(padding),
                    left: Val::Px(padding),        
                    right: Val::Px(padding),
                },
                margin:  UiRect{
                    top: Val::Px(margin),
                    bottom: Val::Px(margin),
                    left: Val::Px(margin),        
                    right: Val::Px(margin),
                },
                ..default()
            },
            background_color: BackgroundColor(color),
            ..default()
        },Outline{
            width: Val::Px(1.),
            offset: Val::Px(1.),
            color: INVISIBLE_COLOR,
        },
        RenderLayers::layer(layer)
    )
}

pub fn scrollable_node_bundle(style: StyleArgs,accessibility_node: AccessibilityNode,scrolling_list: ScrollingList)->impl Bundle{
    
    // (
    //     node_bundle,
    //     
    // )
    if let Some(outline) = style.outline{
        (
            NodeBundle {
                style: Style {
                    width: style.width,
                    height: style.height,
                    align_items: style.align_items,
                    justify_content: style.justify_content,
                    flex_direction: style.direction,
                    overflow: style.overflow,
                    position_type:style.position_type,
                    padding: style.padding,
                    margin:  style.margin,
                    ..default()
                },
                background_color: style.background_color,
                ..default()
            },Outline{
                width: outline.width,
                offset: outline.offset,
                color: outline.color,
            },
            RenderLayers::layer(style.layer),
            scrolling_list,
            accessibility_node,
        )
    }else{
        (
            NodeBundle {
                style: Style {
                    width: style.width,
                    height: style.height,
                    align_items: style.align_items,
                    justify_content: style.justify_content,
                    flex_direction: style.direction,
                    overflow: style.overflow,
                    position_type:style.position_type,
                    padding: style.padding,
                    margin:  style.margin,
                    ..default()
                },
                background_color: style.background_color,
                ..default()
            },
            Outline{
                ..default()
            },
            RenderLayers::layer(style.layer),
            scrolling_list,
            accessibility_node,
        )
    }
}

pub fn create_styled_node_bundle(style: StyleArgs)-> impl Bundle{
    
    if let Some(outline) = style.outline{
        (
            NodeBundle {
                style: Style {
                    width: style.width,
                    height: style.height,
                    align_items: style.align_items,
                    justify_content: style.justify_content,
                    flex_direction: style.direction,
                    overflow: style.overflow,
                    position_type:style.position_type,
                    padding: style.padding,
                    margin:  style.margin,
                    ..default()
                },
                background_color: style.background_color,
                ..default()
            },Outline{
                width: outline.width,
                offset: outline.offset,
                color: outline.color,
            },
            RenderLayers::layer(style.layer)
        )
    }else{
        (
            NodeBundle {
                style: Style {
                    width: style.width,
                    height: style.height,
                    align_items: style.align_items,
                    justify_content: style.justify_content,
                    flex_direction: style.direction,
                    overflow: style.overflow,
                    position_type:style.position_type,
                    padding: style.padding,
                    margin:  style.margin,
                    ..default()
                },
                background_color: style.background_color,
                ..default()
            },
            Outline{
                ..default()
            },
            RenderLayers::layer(style.layer)
        )
    }

    
}

fn fullscreen_entity(commands: &mut Commands,flex_direction: FlexDirection,color : Option<Color>,position_type: Option<PositionType>,
    align_items: Option<AlignItems>,
     layer :usize )->Entity{
    commands
    .spawn(create_node_bundle(100.,100.,flex_direction,JustifyContent::FlexStart,align_items,
        Overflow::visible(),color,position_type,None,None,layer)).id()
}

fn lazy_check_main_entity(menu_data: &mut MenuData, commands: &mut Commands,){
    if menu_data.main_entity.is_none(){
        menu_data.init(commands);
    };
}

pub fn create_area_bundle( width_pct: f32, height_pct: f32, direction: FlexDirection,
     justify: JustifyContent,align_items:Option<AlignItems>,
     overflow: Overflow,position_type: Option<PositionType>,margin: Option<f32>,padding: Option<f32>, layer: usize)-> impl Bundle{
        create_node_bundle(width_pct,height_pct,direction,justify,align_items,
            overflow,None,position_type,margin,padding,layer)
        
    
}
 
fn create_button_bundle(
//    width_pct: f32, height_pct: f32, border: f32,margin:Option<f32>,padding:Option<f32>, layer: usize
style_args: StyleArgs,
)->impl Bundle{
    // let margin = margin.unwrap_or(0.0);
    // let padding = padding.unwrap_or(0.0);
    (
        ButtonBundle {
            style: Style {
                // width: Val::Px(400.0),
                // height: Val::Px(65.0),
                width: style_args.width,
                height: style_args.height,
                border: style_args.border,
                // horizontally center child text
                justify_content: style_args.justify_content,
                // vertically center child text
                align_items: style_args.align_items,
                margin:style_args.margin,
                padding: style_args.padding,
                ..default()
            },
            border_color: style_args.border_color,
            background_color: style_args.background_color,
            ..default()
        },
        RenderLayers::layer(style_args.layer)
    )
}


pub fn create_scrollable_sprite_bundle(  card_images: &CardImages,style : StyleArgs, 
    accessibility_node: AccessibilityNode)->impl Bundle{
    // let back_image= match card_component.face{
    //     CardFace::Down=>{card_images.cards.get(FACEDOWN_KEY).clone().unwrap()},
    //     CardFace::Up=>{card_images.cards.get(FACEUP_KEY).clone().unwrap()},
    // };
    // info!("deck_type.get_image_key()={:?} ",card_images.cards.get(&style.image_key));
        
    (
        ButtonBundle {
            style: Style {
                width: style.width,
                height: style.height,
                //border: UiRect::all(Val::Px(border)),
                // horizontally center child text
                justify_content: style.justify_content,
                // vertically center child text
                align_items: style.align_items,
               
                margin: style.margin,
                border: style.border,
                position_type: style.position_type,
                left: style.left,
                right: style.right,
                top: style.top,
                bottom: style.bottom,
                padding: style.padding,
                ..default()
            },
            // transform: Transform::from_scale(Vec3::splat(1.0)),
            // image: UiImage::new(back_image.clone()),
            // border_color: BorderColor(Color::BLACK),
            // background_color: NORMAL_BUTTON.into(),
            transform: style.transform,
            image: UiImage::new(card_images.cards.get(&style.image_key).clone().unwrap().clone()),
            border_color: style.border_color,
            background_color: style.background_color,

            ..default()
        },
        RenderLayers::layer(style.layer),
        accessibility_node
    )
}

pub fn create_sprite_bundle(  card_images: &CardImages,style : StyleArgs)->impl Bundle{
    // let back_image= match card_component.face{
    //     CardFace::Down=>{card_images.cards.get(FACEDOWN_KEY).clone().unwrap()},
    //     CardFace::Up=>{card_images.cards.get(FACEUP_KEY).clone().unwrap()},
    // };
    // info!("deck_type.get_image_key()={:?} ",card_images.cards.get(&style.image_key));
        
    (
        ButtonBundle {
            style: Style {
                width: style.width,
                height: style.height,
                //border: UiRect::all(Val::Px(border)),
                // horizontally center child text
                justify_content: style.justify_content,
                // vertically center child text
                align_items: style.align_items,
               
                margin: style.margin,
                border: style.border,
                position_type: style.position_type,
                left: style.left,
                right: style.right,
                top: style.top,
                bottom: style.bottom,
                padding: style.padding,
                ..default()
            },
            // transform: Transform::from_scale(Vec3::splat(1.0)),
            // image: UiImage::new(back_image.clone()),
            // border_color: BorderColor(Color::BLACK),
            // background_color: NORMAL_BUTTON.into(),
            transform: style.transform,
            image: UiImage::new(card_images.cards.get(&style.image_key).clone().unwrap().clone()),
            border_color: style.border_color,
            background_color: style.background_color,

            ..default()
        },
        RenderLayers::layer(style.layer)
    )
}
// fn create_editable_text_bundle( _allowed_chars: Vec<String>,)->impl Bundle{
//     (
//         TextEditable{
//             filter_in: vec!["0-9".to_owned()," ".to_owned()], // Only allow number and space
//             filter_out: vec!["5".into()],                // Ignore number 5
//             max_length:5,
//             blink:true,
//         }, // Mark text is editable
//         TextEditFocus,
//         Interaction::None,       // Mark entity is interactable
//         TextBundle::from_section(
//             "",
//             TextStyle {
//                 font_size: 30.,
//                 ..default()
//             },
//         ),
//         RenderLayers::layer(MAIN_LAYER)
//     )
// }

pub fn create_text_bundle(title: &str, style_args: StyleArgs)->impl Bundle{
    (
        TextBundle::from_section(
            title,
            TextStyle {
                font_size: style_args.font_size,
                color: style_args.color,
                ..default()
            },
        ).with_style(Style{
            top: style_args.top,
            left: style_args.left,
            position_type: style_args.position_type,
            justify_content: style_args.justify_content,
            padding: style_args.padding,
            margin: style_args.margin,
            ..default()
        }),
        RenderLayers::layer(style_args.layer)
    )
}



fn add_cards_area_entity(commands: &mut Commands,player: usize, row_count: usize, 
    style_args: StyleArgs,)->Entity{
    let layer=MAIN_LAYER;

    let parent_entity=commands.spawn(create_styled_node_bundle(style_args)).id();
   
    for id in 0..row_count {
        //let mut row_entity = 
        let row_entity = commands.spawn(
            
                scrollable_node_bundle(
                    StyleArgs{
                        //width: Val::Percent(100.),
                        //height: Val::Px(CARD_HEIGHT),
                        width: Val::Percent(100.),
                        direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        layer: layer,
                        ..StyleArgs::card_sprite()
                    }, AccessibilityNode(NodeBuilder::new(Role::List)),ScrollingList::new(id))
        ).id();
        // let text=commands.spawn(create_text_bundle(format!("{}{}","+",id).as_str(),StyleArgs{layer,
        //     position_type: PositionType::Absolute,
        //     top:Val::Px(20.),left: Val::Px(20.)
        //     ,..default()})).id();
        
        commands.entity(row_entity).insert(
            UiElementComponent::new(
                UiElement::BeforeGame(BeforeGameElements::Cards(player,id)),
                None)

        );
        // commands.entity(row_entity).add_child(text);
        commands.entity(parent_entity).add_child(row_entity);
    }
    commands.entity(parent_entity).insert(
        UiElementComponent::new(UiElement::BeforeGame(BeforeGameElements::CardList(player)),None));
    //commands.entity(parent).add_child(col_entity);
    // commands.entity(parent_entity).add_child(col_entity);
    parent_entity  
}

fn add_actions_area_entity(commands: &mut Commands,  width_pct: f32, height_pct: f32, direction:FlexDirection,
    justify:JustifyContent, align_items:Option<AlignItems>, overflow: Overflow, position_type: Option<PositionType>,
    margin: Option<f32>,padding: Option<f32>, layer: usize)->Entity{
    let child =  commands.spawn(
        create_area_bundle(width_pct,height_pct,direction,justify,align_items, overflow,position_type,
            margin,padding, layer)).id();
    child
   
}

fn get_card_sprite_text(commands: &mut Commands,card_prop: &CardComponent,layer: usize,style_args: StyleArgs)->Entity{
    let text=if card_prop.face == CardFace::Up{
        if let Some(_onchain_index) = card_prop.onchain_index {
            //onchain_index.to_string()
            "".to_owned()
        }else{
            "".to_owned()
        }
    }else{
        "".to_owned()
    };
    let button_text=commands.spawn(create_text_bundle(text.as_str(),StyleArgs{layer,..style_args})).id();
    button_text
}

fn add_scrollable_card_sprite(commands: &mut Commands,style: StyleArgs,
    card_prop: CardComponent,card_images: &CardImages,layer: usize)->Entity{
    let parent = commands.spawn(create_scrollable_sprite_bundle(
        &card_images,
        StyleArgs{
            image_key:card_prop.face.get_image_key().to_owned(),
            width: Val::Px(CARD_WIDTH),
            height: Val::Px(CARD_HEIGHT),
            direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            //overflow: Overflow::clip(),
            layer: layer,
            ..style
        },AccessibilityNode(NodeBuilder::new(Role::ListItem)))).id();
    let text_entity=get_card_sprite_text(commands,&card_prop,layer,StyleArgs{
        position_type: PositionType::Absolute, top: Val::Px(50.0),
        height: Val::Px(20.),
        ..default()});
    let mut card_data = if card_prop.face == CardFace::Up{
        Some(create_player_card(commands,card_images,&card_prop,layer))
    }else{
        None
    };
    commands.entity(text_entity).insert(card_prop);
    if let Some(card_data) = card_data.take(){
        commands.entity(parent).add_child(card_data);
        let clickable_entity = commands.spawn(create_button_bundle(StyleArgs{width: Val::Px(CARD_WIDTH),
            height: Val::Px(CARD_HEIGHT),position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Px(0.)),padding:UiRect::all(Val::Px(0.)),
            //border_color: BorderColor(Color::srgb(0.5, 0.6, 0.7)),
            background_color:BackgroundColor(Color::NONE),
            left: Val::Px(0.),right: Val::Px(0.), layer,..StyleArgs::button_style()})).id();

        commands.entity(clickable_entity).add_child(text_entity);
        commands.entity(parent).add_child(clickable_entity);
    }else{
        commands.entity(parent).add_child(text_entity);
    }
    
    parent
}

fn add_card_sprite(commands: &mut Commands,style: StyleArgs,
card_prop: CardComponent,card_images: &CardImages,layer: usize,image_key: String)->Entity{
    //let button= commands.spawn(create_button_bundle(width,height,2.0)).id();
    // let parent = commands.spawn(create_styled_node_bundle(StyleArgs{
    //     width: Val::Px(CARD_WIDTH), height: Val::Px(CARD_HEIGHT),
    //      layer, ..style})).id();
    let parent=commands.spawn(create_sprite_bundle(card_images,StyleArgs{image_key,
        //position_type:PositionType::Absolute,
        width: Val::Px(CARD_WIDTH), height: Val::Px(CARD_HEIGHT),
        //width: Val::Px(CARD_WIDTH), height: Val::Px(CARD_HEIGHT),
        //left: Val::Px(0.),top: Val::Px(0.0),
        layer, ..style})).id();
    let text_entity=get_card_sprite_text(commands,&card_prop,layer,StyleArgs{
        position_type: PositionType::Absolute, top: Val::Px(50.0),
        height: Val::Px(20.),
        ..default()});
    let mut card_data = if card_prop.face == CardFace::Up{
        Some(create_player_card(commands,card_images,&card_prop,layer))
    }else{
        None
    };
    commands.entity(text_entity).insert(card_prop);
    if let Some(card_data) = card_data.take(){
        commands.entity(parent).add_child(card_data);
        let clickable_entity = commands.spawn(create_button_bundle(StyleArgs{width: Val::Px(CARD_WIDTH),
            height: Val::Px(CARD_HEIGHT),position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Px(0.)),padding:UiRect::all(Val::Px(0.)),
            //border_color: BorderColor(Color::srgb(0.5, 0.6, 0.7)),
            background_color:BackgroundColor(Color::NONE),
            left: Val::Px(0.),right: Val::Px(0.), layer,..StyleArgs::button_style()})).id();

        commands.entity(clickable_entity).add_child(text_entity);
        commands.entity(parent).add_child(clickable_entity);
    }else{
        commands.entity(parent).add_child(text_entity);
    }
    // let text_parent = commands.spawn(create_styled_node_bundle(StyleArgs{
    //     position_type: PositionType::Absolute, top: Val::Px(50.0),
    //     height: Val::Px(20.),
    //     ..default()})).id();
    // commands.entity(text_parent).add_child(text_entity);
    
    // commands.entity(parent).add_child(image);
    parent
}
 

pub fn add_text(commands: &mut Commands,text: &str,style_args: StyleArgs,)->Entity{
    // let parent=commands.spawn(create_node_bundle(100.,100.,FlexDirection::Row,JustifyContent::Center,
    //     Some(AlignItems::Center),
    //     Overflow::visible(),None,None,None,None,layer)).id(); 
    let layer=style_args.layer;
    let parent= commands.spawn(create_styled_node_bundle(style_args)).id();
    let text_entity=commands.spawn(create_text_bundle(text,StyleArgs{layer,..default()})).id();
    commands.entity(parent).add_child(text_entity);
    parent
}

pub fn add_button(commands: &mut Commands,game_status: GameStatus,action_type: ActionType,
//    width: f32,height: f32,margin: Option<f32>, padding: Option<f32>, layer: usize
style_args: StyleArgs,
)->Entity{
    let layer=style_args.layer;
    let parent=commands.spawn(
        create_styled_node_bundle(StyleArgs{width:Val::Percent(100.),height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        direction: FlexDirection::Row,
    layer,..default()})
        // create_node_bundle(100.,100.,FlexDirection::Row,JustifyContent::Center,
        //     Some(AlignItems::Center),
        //     Overflow::visible(),None,None,None,None,layer)
        ).id(); 
            //width,height,2.0,margin,padding,layer.clone()
    let button= commands.spawn(create_button_bundle(style_args)).id();
    let button_text=commands.spawn(create_text_bundle(game_status.value().as_str(),StyleArgs{layer,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()})).id();
    commands.entity(button_text).insert(MenuButton::new(action_type,layer));
    commands.entity(button).add_child(button_text);
    commands.entity(parent).add_child(button);
    parent
}

pub fn add_editable_button(commands: &mut Commands,initial_value: &str, width: f32, height: f32, game_status: GameStatus,action_type: ActionType, )->Entity{
     
    let layer= MAIN_LAYER;
    let text_entity = commands.spawn(create_node_bundle(width,height,FlexDirection::Row,JustifyContent::Center,
        Some(AlignItems::Center),
        Overflow::visible(),None,None,Some(2.0),Some(2.0),layer))
    .with_children(|parent|  {
        parent.spawn(create_node_bundle(50.,50.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::Center),
        Overflow::visible(),None, None,None,None, layer))
        .with_children(|parent| {
            // parent.spawn(create_editable_text_bundle(vec!["0-9".to_owned()," ".to_owned()]));
            parent.spawn(
                (
                    TextEditable{
                        filter_in: vec!["[0-9]".into(), " ".into()], // Only allow number and space
                        filter_out: vec!["5".into()],                // Ignore number 5
                        max_length:5,
                        blink:true,
                    }, // Mark text is editable
                    TextEditFocus,
                    Interaction::None,       // Mark entity is interactable
                    TextBundle::from_section(
                        initial_value,
                        TextStyle {
                            font_size: 30.,
                            ..default()
                        },
                    ),
                    RenderLayers::layer(layer),
                )
            );
        });
        
        parent
        .spawn(create_button_bundle(
            StyleArgs{width:Val::Percent(50.),height: Val::Percent(50.),layer,..StyleArgs::button_style()} 
            //50.0,50.0,2.0,None,None,layer
        ))
        .with_children(|parent| {
            parent.spawn(create_text_bundle(game_status.value().as_str(),StyleArgs{layer,..default()}))
            .insert(MenuButton::new(action_type,layer));
        });
    }).id();
        
    text_entity
}
 

pub fn create_add_text(menu_data: &mut MenuData, commands: &mut Commands,text_title: &str)->Option<Entity>{
    lazy_check_main_entity( menu_data,  commands);
    if let Some(main_entity)=menu_data.main_entity{
        if let Some(mut entity_commands) = commands.get_entity(main_entity) {
            let text_entity = entity_commands
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    text_title,
                    TextStyle {
                        font_size: 30.,
                        ..default()
                    },
                ));
             }).id();
             return Some(text_entity);
        }
    }
    None       

}

// pub fn create_input_text_with_button(menu_data: &mut MenuData, commands: &mut Commands,
//     button_title: &str,ix_type: GameContractIxType)->Option<Entity> {

//     lazy_check_main_entity( menu_data,  commands);

//     if let Some(main_entity)=menu_data.main_entity{
//         if let Some(mut entity_commands) = commands.get_entity(main_entity) {
//             let text_entity = entity_commands
//             .with_children(|parent|  {
//                 parent.spawn((
//                     NodeBundle {
//                         style: Style {
//                             width: Val::Percent(50.0),
//                             border: UiRect::all(Val::Px(5.0)),
//                             padding: UiRect::all(Val::Px(5.0)),
//                             ..default()
//                         },
//                         //border_color: BORDER_COLOR_ACTIVE.into(),
//                         //background_color: BACKGROUND_COLOR.into(),
//                         ..default()
//                     },
//                     //..default()
//                     // TextInputBundle::default().with_text_style(TextStyle {
//                     //     font_size: 40.,
//                     //     //color: TEXT_COLOR,
//                     //     ..default()
//                     // }),
                    

//                 ))
//                 .with_children(|parent| {
//                     parent.spawn((
//                         TextEditable{
//                             filter_in: vec!["[0-9]".into(), " ".into()], // Only allow number and space
//                             filter_out: vec!["5".into()],                // Ignore number 5
//                             max_length:5,
//                             blink:true,
//                         }, // Mark text is editable
//                         TextEditFocus,
//                         Interaction::None,       // Mark entity is interactable
//                         TextBundle::from_section(
//                             "",
//                             TextStyle {
//                                 font_size: 30.,
//                                 ..default()
//                             },
//                         )
//                     ));
//                 });
//                 // .insert(MenuButton{target: 
//                 //     ActionType::Web3Actions(Web3Actions::GameContractAction(ix_type))});
//                 parent
//                 .spawn(ButtonBundle {
//                     style: Style {
//                         width: Val::Percent(48.0),
//                         height: Val::Percent(100.0),
//                         border: UiRect::all(Val::Px(5.0)),
//                         // horizontally center child text
//                         justify_content: JustifyContent::Center,
//                         // vertically center child text
//                         align_items: AlignItems::Center,
//                         margin: UiRect {
//                             top: Val::Percent(5.),
//                             ..default()
//                         },
//                         ..default()
//                     },
//                     border_color: BorderColor(Color::BLACK),
//                     background_color: NORMAL_BUTTON.into(),
//                     ..default()
//                 })
//                 .with_children(|parent| {
//                     parent.spawn(TextBundle::from_section(
//                         button_title,
//                         TextStyle {
//                             font_size: 30.0,
//                             color: Color::srgb(0.9, 0.9, 0.9),
//                             ..default()
//                         },
//                     ))
//                     .insert(MenuButton{target: 
//                         ActionType::Web3Actions(Web3Actions::GameContractAction(ix_type))});
//                 });
//             }).id();
//             return Some(text_entity);
//         }
//     }
//     None
// }


// fn clear_entity_and_children(commands: &mut Commands, entity: Entity){
//     commands.entity(entity).despawn_recursive();
// }
                     
pub fn process_ui_query(ui_query: &mut Query<(Entity, &mut UiElementComponent)>,
commands: &mut Commands, 
ui_element: UiElement,
new_status: Option<GameStatus>,
mut card_prop: Option<CardComponent>,
//asset_server: &AssetServer
card_images: Option<&CardImages>,
game: &Game,
)
{
    for ( entity, mut element)  in ui_query{
        //update_status(entity, &mut element,commands,new_status.as_ref());
        if element.element_type == ui_element{
            match element.element_type {
                UiElement::BeforeGame(ref v) => match v {
                    BeforeGameElements::Status =>{
                        if let Some(ref new_status) = new_status{
                            let to_add = if let Some(ref status)= element.status{
                                if status == new_status {
                                    false
                                }else{
                                    true
                                }
                            }else{
                                true
                            };
                            if to_add == true
                            {
                                add_status_button(commands, entity,  &new_status,&game);
                                element.update_status(Some(new_status.clone()));
                            }
                        };
                    },
                    BeforeGameElements::Cards(_,_)=>{
                        if let Some( card_prop) = card_prop.take(){
                            if let Some(ref card_images)=card_images{
                                let  button= add_scrollable_card_sprite(commands,StyleArgs{
                                    position_type:PositionType::Relative,
                                    width: Val::Px(CARD_WIDTH),height: Val::Px(CARD_HEIGHT),
                                    ..StyleArgs::card_sprite()
                                },card_prop,card_images,MAIN_LAYER);
                                commands.entity(entity).add_child(button);
                            }
                        }
                        
                        
                    }
                    _=>{},
                },
                UiElement::InGame(ref v)=>match v{
                    InGameElements::Status=>{
                        if let Some(ref new_status) = new_status{
                            let to_add = if let Some(ref status)= element.status{
                                if status == new_status {
                                    false
                                }else{
                                    true
                                }
                            }else{
                                true
                            };
                            if to_add == true
                            {
                                add_status_button(commands, entity,  &new_status,&game);
                                element.update_status(Some(new_status.clone()));
                            }
                        };
                    },
                    InGameElements::EnvDeck=>{
                        if let Some(ref card_images)=card_images{
                            update_env_card(&game, commands,card_images,entity);
                        }
                    },
                    InGameElements::PlayerHandCards=>{
                        if let Some(ref card_images)=card_images{
                            update_player_hand_cards(&game, commands,card_images,entity)
                        }
                    },
                    InGameElements::PlayerActiveCard{player}=>{
                        if let Some(ref card_images)=card_images{
                            match player{
                                Player::Player1=>{update_player1_cards(&game, commands,card_images,entity)},
                                Player::Player2=>{update_player2_cards(&game, commands,card_images,entity)},
                                
                            }
                        }
                        
                    },
                    InGameElements::PlayerLabel=>{
                        update_player_labels(&game, commands,entity);
                    }
                    InGameElements::Track=>{
                        if let Some(ref card_images)=card_images{
                            update_track_entity(&game,commands,card_images,entity);
                        }
                    },
                    InGameElements::WinnerCard=>{
                        info!("winnercard");
                        if let Some(ref card_images)=card_images{
                            update_winner_card(&game,commands,card_images,entity);
                        }
                    },
                    _=>{},
                }
                _=>{}
            }
        }
    }
}

fn add_status_button(commands: &mut Commands,entity: Entity, new_status: &GameStatus,game: &Game,){
    commands.entity(entity).despawn_descendants();
    let layer= MAIN_LAYER;
    match new_status{
        GameStatus::InitPlayer | 
        GameStatus::SetCreatorDeck |
        GameStatus::JoinMatchPreSelect | 
        GameStatus::SetJointKeyPreSet |
        GameStatus::MaskAndShuffleEnvDeck |
        GameStatus::ShuffleEnvDeck |
        GameStatus::ShuffleYourDeck |
        GameStatus::ShuffleOthersDeck |
        GameStatus::RevealEnvCard |
        GameStatus::RevealOtherPlayerCards |
        GameStatus::PlayCardOnDeck =>
        {
            let button= add_button(commands,new_status.clone(),
                ActionType::Web3Actions(Web3Actions::GameContractAction(new_status.clone().try_into().unwrap())),
                StyleArgs{width:Val::Percent(80.),height: Val::Percent(40.),layer,..StyleArgs::button_style()} 
                //80.,40.0,Some(4.0),Some(4.0),MAIN_LAYER
            );
            //element.update_status(Some(new_status.clone()));
            commands.entity(entity).add_child(button);
        },
        GameStatus::PlayerAction=>{
            let area_entity=add_actions_area_entity(commands, 100.0,100.,FlexDirection::Row, JustifyContent::Center,
                Some(AlignItems::FlexStart), Overflow::visible(),None,None,None, MAIN_LAYER);
            
            let reveal_env= add_button(commands,GameStatus::PlayerActionEnv,
                ActionType::Web3Actions(Web3Actions::GameContractAction(GameContractIxType::PlayerActionEnv)),
               // 50.0,100.0,Some(4.0),Some(4.0),MAIN_LAYER
               StyleArgs{width:Val::Percent(50.),height: Val::Percent(80.),layer,..StyleArgs::button_style()} 
               );
            commands.entity(area_entity).add_child(reveal_env);
            let reveal_card= add_button(commands,GameStatus::PlayerActionCard,
                ActionType::Web3Actions(Web3Actions::GameContractAction(GameContractIxType::PlayerActionCard)),
                //50.0,100.0,Some(4.0),Some(4.0),MAIN_LAYER
                StyleArgs{width:Val::Percent(50.),height: Val::Percent(80.),layer,..StyleArgs::button_style()} 
                );
            commands.entity(area_entity).add_child(reveal_card);
            commands.entity(entity).add_child(area_entity);
        },
        GameStatus::CreateNewMatch=>
        {
            let button= add_button(commands,GameStatus::CreateNewMatch,
                ActionType::Web3Actions(Web3Actions::GameContractAction(GameContractIxType::CreateNewMatch)),
            //    80.0,40.0,Some(4.0),Some(4.0),MAIN_LAYER
            StyleArgs{width:Val::Percent(80.),height: Val::Percent(40.),layer,..StyleArgs::button_style()}  
            );
            commands.entity(entity).add_child(button);
            let join_button=add_editable_button(commands,format!("{}",game.match_index).as_str(),100.,50.,GameStatus::JoinMatchPreSelect,
                ActionType::Web3Actions(Web3Actions::GameContractAction(GameContractIxType::JoinMatchPreSelect)));
            commands.entity(entity).add_child(join_button);
        
            //element.update_status(Some(new_status.clone()));
            
        },
        GameStatus::WaitingForPlayers=>{
            let area_entity=add_actions_area_entity(commands, 100.0,100.,FlexDirection::Column, JustifyContent::Center,
                Some(AlignItems::FlexStart), Overflow::visible(),None,None,None, MAIN_LAYER);
            
            let text=add_text(commands,new_status.value().as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
                direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
            layer,..default()});
            commands.entity(area_entity).add_child(text);

            let button= add_button(commands,GameStatus::ShareMatchUrl,
                ActionType::GameActions(GameActions::CopyMatchUrl),
                StyleArgs{width:Val::Percent(80.),height: Val::Percent(40.),layer,..StyleArgs::button_style()} 
            );
            
            commands.entity(area_entity).add_child(button);
            commands.entity(entity).add_child(area_entity);
        },
        GameStatus::WaitingForPlayerToShuffleEnvDeck |
        GameStatus::WaitingForPlayerToShuffleCards |
        GameStatus::WaitingForJointKey |
        GameStatus::WaitingForRevealCards |
        GameStatus::WaitingForOtherPlayerToPlayCard |
        GameStatus::Finished => {
            info!("in game add status button new_status={:?}",new_status);
            let text=add_text(commands,new_status.value().as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
                direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
            layer,..default()});
            commands.entity(entity).add_child(text);
            //element.update_status(Some(new_status.clone()));
        }
        _=>{},
    }
}

fn update_cards(commands: &mut Commands, entity : Entity,
     card_images: &CardImages, card_components: Vec<CardComponent>, deck_type: DeckType  ){
    remove_card(commands,entity);
    for card_component in card_components.into_iter(){
        let new_card=add_active_card( commands, &card_images,card_component, MAIN_LAYER, deck_type.clone());
        commands.entity(entity).add_child(new_card);
    }
}

fn remove_card(commands: &mut Commands, entity : Entity, ){
    commands.entity(entity).despawn_descendants();
}
 
// fn update_status(entity: Entity, element: &mut UiElementComponent,commands: &mut Commands, new_status: Option<&GameStatus>){
    
// }

 
///Game UI Pages
//Systems
pub fn init_ui(mut commands: Commands,
    mut menu_data: ResMut<MenuData>,
) {
        let layer=MAIN_LAYER;
        lazy_check_main_entity( &mut  menu_data,  &mut commands);
        if let Some(main_entity) = menu_data.main_entity{
            //Cards
            let card_entity = add_cards_area_entity(&mut commands,0,4,
                StyleArgs{layer,
                    width: Val::Px(CARD_WIDTH*4.0),
                    height: Val::Percent(100.),//Val::Px((CARD_HEIGHT+3.0)*4.),
                    overflow: Overflow::clip(),
                            direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                    ..default()});
            commands.entity(main_entity).add_child(card_entity);
            //Actions Area
            let action_entity = add_actions_area_entity(&mut commands, 20.0,40.0,FlexDirection::Column, JustifyContent::Center,
                Some(AlignItems::FlexStart), Overflow::visible(),None,None,None, MAIN_LAYER);
            commands.entity(action_entity).insert(
                UiElementComponent::new(UiElement::BeforeGame(BeforeGameElements::Status),Some(GameStatus::ConnectWallet)));
            let connect_button= add_button(&mut commands,GameStatus::ConnectWallet,ActionType::Web3Actions(Web3Actions::ConnectWallet),
        //    100.0,50.0,Some(4.0),Some(4.0),MAIN_LAYER
            StyleArgs{width:Val::Percent(100.),height: Val::Percent(50.),layer,..StyleArgs::button_style()} 
            );
            commands.entity(action_entity).add_child(connect_button);
            
            commands.entity(main_entity).add_child(action_entity);
            //Other player Cards
            //let other_card_entity = add_cards_area_entity(&mut commands,1,5,40.0,90.0,FlexDirection::Column, JustifyContent::Start);
            let other_card_entity = add_cards_area_entity(&mut commands,1,4,
                StyleArgs{layer,
                    width: Val::Px(CARD_WIDTH*4.0),
                    height: Val::Px((CARD_HEIGHT*3.0)*4.),
                    overflow: Overflow::clip(),
                            direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                    ..default()});
            commands.entity(main_entity).add_child(other_card_entity);

           
        }
    
}


//Bevy system
pub fn start_ui(mut commands: Commands,
    mut menu_data: ResMut<MenuData>,
    game_status: Res<State<GameStatus>>,
    game: Res<Game>,
    card_images: Res<CardImages>,
)
{
        lazy_check_main_entity( &mut  menu_data,  &mut commands);
        if let Some(main_entity) = menu_data.main_entity.take() {
            commands.entity(main_entity).despawn_recursive();
        }
        let layer=MAIN_LAYER;
        //Delete old
        let main_entity=fullscreen_entity(&mut commands,FlexDirection::Column,None,None,Some(AlignItems::FlexStart),layer);
        //Actions Area
        let row_entity_1= commands
        .spawn(create_node_bundle(100.,20.,FlexDirection::Row,JustifyContent::Start,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,layer)).id();
        let legends_entity=commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(25.), height: Val::Percent(100.),
            layer,..default()})).id();
        commands.entity(legends_entity).insert(
            UiElementComponent::new(UiElement::InGame(InGameElements::PlayerLabel),None));

        let legends_child_entity=add_player_legend(&mut commands,&game,layer);
        
        
        

        let action_entity = add_actions_area_entity(&mut commands, 50.0,100.0,FlexDirection::Column, JustifyContent::Center,
            Some(AlignItems::FlexStart), Overflow::visible(),None,None,None, layer);
        commands.entity(action_entity).insert(
            UiElementComponent::new(UiElement::InGame(InGameElements::Status),Some(game_status.get().clone())));

        commands.entity(row_entity_1).add_child(legends_entity);
        commands.entity(row_entity_1).add_child(action_entity);
        commands.entity(main_entity).add_child(row_entity_1);
        commands.entity(legends_entity).add_child(legends_child_entity);

        //Track
        let row_entity_2= commands
        .spawn(create_node_bundle(100.,20.,FlexDirection::Column,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        update_track_entity(&game,&mut commands,&card_images, row_entity_2);
        commands.entity(row_entity_2).insert(
            UiElementComponent::new(
                UiElement::InGame(InGameElements::Track),
                None));
        commands.entity(main_entity).add_child(row_entity_2);
        
        //Game Cards,  Env Cards , Game Cards
        let row_entity_3= commands
        .spawn(create_node_bundle(100.,30.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        let mut col_entity_3 : Vec<Entity>= vec![];
        for i in 0..3 {
            let flex_direction = if i==1 {
                FlexDirection::Column
            }else{
                FlexDirection::Row
            };
            col_entity_3.push(commands
                .spawn(create_node_bundle(33.,100.,flex_direction,JustifyContent::Center,Some(AlignItems::FlexStart),
                    Overflow::visible(),None,None,None,None,MAIN_LAYER)).id())
            
        }
        
        // let player1_button= add_button(&mut commands,GameStatus::PlayerOriginalCards,
        //         ActionType::GameActions(GameActions::ShowOriginalCards(0)),
        //         //80.,40.0,Some(4.0),Some(4.0),MAIN_LAYER
        //         StyleArgs{width:Val::Percent(80.),height: Val::Percent(40.),layer,..StyleArgs::button_style()} 
        // );
        // let player2_button= add_button(&mut commands,GameStatus::PlayerOriginalCards,
        //         ActionType::GameActions(GameActions::ShowOriginalCards(1)),
        //         // 80.,40.0,Some(4.0),Some(4.0),MAIN_LAYER
        //         StyleArgs{width:Val::Percent(80.),height: Val::Percent(40.),layer,..StyleArgs::button_style()} 
        //     );
        // let all_env_cards_button= add_button(&mut commands,GameStatus::EnvOriginalCards,
        //     ActionType::GameActions(GameActions::ShowOriginalCards(1)),
        //     //80.,15.0,Some(4.0),Some(4.0),MAIN_LAYER
        //     StyleArgs{width:Val::Percent(80.),height: Val::Percent(15.),layer,..StyleArgs::button_style()} 
        //     );
        let card_upper_block = commands
        .spawn(create_node_bundle(100.,20.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        let all_envs_text=add_text(&mut commands,"Env Deck",StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
            direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
        layer,..default()});
        commands.entity(card_upper_block).add_child(all_envs_text);
            
        let card_lower_block = commands
        .spawn(create_node_bundle(100.,80.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        
        let card_env_block= commands
        .spawn(create_node_bundle(30.,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        commands.entity(card_env_block).insert(UiElementComponent::new(UiElement::InGame(InGameElements::EnvDeck),None));
        
        let card_deck_parent=commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(70.), height: Val::Percent(100.),
        ..default()})).id();
        let card_deck=add_empty_card_deck(&mut commands,20,2.0,0.0,&card_images,DeckType::Env);
        // let active_card=   add_active_card(&mut commands,CardComponent{card_type: CardComponentType::EnvCard,
        //     ..default()} , &card_images,  DeckType::Env);
        commands.entity(card_deck_parent).add_child(card_deck);
        commands.entity(card_lower_block).add_child(card_deck_parent);
        commands.entity(card_lower_block).add_child(card_env_block);
        // commands.entity(card_env_block).add_child(active_card);
        
        let player1_text=add_text(&mut commands,"Player1 Deck",StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
            direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
        layer,..default()});
        let player2_text=add_text(&mut commands,"Player2 Deck",StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),
            direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
        layer,..default()});
        commands.entity(col_entity_3[0]).add_child(player1_text);
        commands.entity(col_entity_3[1]).add_child(card_upper_block);
        commands.entity(col_entity_3[1]).add_child(card_lower_block);
        commands.entity(col_entity_3[2]).add_child(player2_text);

        for e in col_entity_3.into_iter(){
            commands.entity(row_entity_3).add_child(e);
        };
        commands.entity(main_entity).add_child(row_entity_3);
        
        //Player Cards
        //50,20,30
        let row_entity_4= commands
        .spawn(create_node_bundle(100.,30.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        let row_entity_4_widths = vec![50.,20.,30.];
        let mut col_entity_4 : Vec<Entity>= vec![];
        for i in 0..3 {
            col_entity_4.push(commands
                .spawn(create_node_bundle(row_entity_4_widths[i],100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
                    Overflow::visible(),None,None,None,None,MAIN_LAYER)).id())
            
        }
        let col_entity_4_1_1=commands
        .spawn(create_node_bundle(CARD_CONTAINER_WIDTH,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        let inactive_card_deck=add_empty_card_deck(&mut commands,17,5.0,0.0,&card_images,DeckType::Player1);
        commands.entity(col_entity_4_1_1).add_child(inactive_card_deck);

        let player_hands_card_block=commands
            .spawn(create_node_bundle(CARD_CONTAINER_WIDTH*3.0,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        commands.entity(player_hands_card_block).insert(UiElementComponent::new(UiElement::InGame(InGameElements::PlayerHandCards),None));

        commands.entity(col_entity_4[0]).add_child(col_entity_4_1_1);
        commands.entity(col_entity_4[0]).add_child(player_hands_card_block);

        let player_active_card_block=commands
            .spawn(create_node_bundle(CARD_CONTAINER_WIDTH*1.0,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        commands.entity(player_active_card_block).insert(UiElementComponent::new(UiElement::InGame(InGameElements::PlayerActiveCard{player: Player::Player1}),None));
        commands.entity(col_entity_4[1]).add_child(player_active_card_block);

        let col_entity_4_3_1=commands
        .spawn(create_node_bundle(CARD_CONTAINER_WIDTH,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        let p2_active_card_block= commands
            .spawn(create_node_bundle(CARD_CONTAINER_WIDTH,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
                Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        commands.entity(p2_active_card_block).insert(UiElementComponent::new(UiElement::InGame(InGameElements::PlayerActiveCard{player: Player::Player2}),None));
        commands.entity(col_entity_4_3_1).add_child(p2_active_card_block);
        
        let col_entity_4_3_2=commands
            .spawn(create_node_bundle(CARD_CONTAINER_WIDTH,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
            Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
        let inactive_card_deck=add_empty_card_deck(&mut commands,19,-5.0,0.0,&card_images,DeckType::Player2);
        commands.entity(col_entity_4_3_2).add_child(inactive_card_deck);
    
        commands.entity(col_entity_4[2]).add_child(col_entity_4_3_1);
        commands.entity(col_entity_4[2]).add_child(col_entity_4_3_2);

        for e in col_entity_4.into_iter(){
            commands.entity(row_entity_4).add_child(e);
        };
        commands.entity(main_entity).add_child(row_entity_4);
        

        menu_data.main_entity=Some(main_entity);

        //Update actions area
        add_status_button(&mut commands, action_entity, &game_status.get(),&game);
        //Update env card
        update_env_card(&game,&mut commands, &card_images,card_env_block);
        //Update player cards
        update_player_hand_cards(&game, &mut commands,&card_images,player_hands_card_block);
        update_player1_cards(&game, &mut commands,&card_images,player_active_card_block);
        update_player2_cards(&game, &mut commands,&card_images,p2_active_card_block);
        
        
        
}


fn update_player_hand_cards(game: &Game, commands: &mut Commands,card_images: &CardImages,player_active_card_block: Entity){
    if let Some(ref address_bytes)=game.account_bytes{
        for (key,value) in game.screen_data.iter(){
            if key == address_bytes {
                //Current player
                let mut card_components: Vec<CardComponent> = vec![];
                //Todo! Sort keys
                for (hand,card) in value.current_hands.iter(){
                    let key_idx=*hand as usize;
                    card_components.push(CardComponent{
                        index:key_idx,
                        onchain_index: Some(card.onchain_index),
                        face: CardFace::Up,
                        val: Some(DeckCardType::PlayerCard(card.clone())),
                        card_type: CardComponentType::PlayerCard(Player::Player1,key_idx),
                        is_selectable: true,
                        ..default()
                    })
                    
                }
                update_cards(commands,player_active_card_block,&card_images, 
                    card_components,
                    DeckType::Player1)
            } 
        }
    }
}

fn update_player1_cards(game: &Game, commands: &mut Commands,card_images: &CardImages,player_active_card_block: Entity){
    if let Some(match_state) = &game.match_state {
        let state_u64 = match_state.state.clone() as u64;
        if state_u64 >=MatchStateEnum::PlayerPlayCard as u64 {
            if let Some(ref address_bytes)=game.account_bytes{
                for player_data in game.players_data.iter(){
                    if &player_data.player_state.player == address_bytes {
                        //info!("update_player1_cards player_board={:?} keys={:?}",player_data.player_state.player_board,player_data.all_cards.all_card_props.keys());
                        //info!("update_player1_cards player_reveals={:?} {:?}",player_data.player_state.player_reveals[0].len(),player_data.player_state.player_reveals[0][0].len());
                        let reveal_count = if player_data.player_state.player_reveals.len()>0 {
                            player_data.player_state.player_reveals[0][0].len()
                        }else{
                            0
                        };
                        if reveal_count == match_state.player_count as usize {
                            if let Some(card) = player_data.all_cards.all_card_props.get(&player_data.player_state.player_board) {
                                //info!("update_player1_cards card={:?}",card);
                                let key_idx=player_data.player_state.player_board.as_usize();
                                update_cards(commands,player_active_card_block,&card_images, 
                                    vec![CardComponent{
                                        index:key_idx,
                                        onchain_index: Some(card.onchain_index),
                                        face: CardFace::Up,
                                        val: Some(DeckCardType::PlayerCard(card.clone())),
                                        card_type: CardComponentType::PlayerCard(Player::Player1,key_idx),
                                        ..default()
                                    }],
                                    DeckType::Player1)
                                
                            }
                        }
                    }
                }
                
            }
        }
    }
}


fn update_player2_cards(game: &Game, commands: &mut Commands,card_images: &CardImages,player_active_card_block: Entity){
    if let Some(match_state) = &game.match_state {
        let state_u64 = match_state.state.clone() as u64;
        if state_u64 >=MatchStateEnum::PlayerPlayCard as u64 {
            if let Some(ref address_bytes)=game.account_bytes{
                for player_data in game.players_data.iter(){
                    if &player_data.player_state.player != address_bytes {
                        //info!("update_player2_cards player_board={:?} keys={:?}",player_data.player_state.player_board,player_data.all_cards.all_card_props.keys());
                        //info!("update_player2_cards player_reveals={:?}",player_data.player_state.player_reveals[0].len());
                        let reveal_count = if player_data.player_state.player_reveals.len()>0 {
                            player_data.player_state.player_reveals[0][0].len()
                        }else{
                            0
                        };
                        if reveal_count == match_state.player_count  as usize {
                            if let Some(card) = player_data.all_cards.all_card_props.get(&player_data.player_state.player_board) {
                                //info!("update_player2_cards card={:?}",card);
                                let key_idx=player_data.player_state.player_board.as_usize();
                                update_cards(commands,player_active_card_block,&card_images, 
                                    vec![CardComponent{
                                        index:key_idx,
                                        onchain_index: Some(card.onchain_index),
                                        face: CardFace::Up,
                                        val: Some(DeckCardType::PlayerCard(card.clone())),
                                        card_type: CardComponentType::PlayerCard(Player::Player2,key_idx),
                                        ..default()
                                    }],
                                    DeckType::Player2)
                                
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_env_card(game: &Game,commands: &mut Commands,card_images: &CardImages,parent: Entity){
    if let Some(ref env_card_index) = game.get_current_env_index(){
        update_cards(  commands,parent,&card_images, 
            vec![CardComponent{
                index:0,
                onchain_index: Some((*env_card_index).into()),
                face: CardFace::Up,
                val: Some(DeckCardType::EnvCard(game.env_cards[*env_card_index].clone())),
                card_type: CardComponentType::EnvCard,
                ..default()
            }],
            DeckType::Env)
    }
}

fn add_player_legend(commands: &mut Commands, game: &Game,layer: usize)->Entity{
    let player_addresses=get_player_addresses(&game);
    let legends_child_entity=commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(100.), height: Val::Percent(100.),
        direction: FlexDirection::Column, 
        layer,..default()})).id();
    let row_count = player_addresses.len()+1;
    for (index,addr) in player_addresses.iter().enumerate(){
        let text= format!("Player{} : {}",(index+1),addr);
        let legends_text_entity_parent=commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(100.), 
            height: Val::Percent(100./(row_count as f32)),justify_content : JustifyContent::Start, align_items: AlignItems::Start,
            layer,..default()})).id();
        let text_entity=add_text( commands,text.as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),font_size:20.,
            direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
        layer,..default()});
        commands.entity(legends_text_entity_parent).add_child(text_entity);
        commands.entity(legends_child_entity).add_child(legends_text_entity_parent);
    }
    game.match_state.as_ref().map(|match_state| {
        let text=format!("Rounds : {}",match_state.rounds);
        let legends_text_entity_parent=commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(100.), 
            height: Val::Percent(100./(row_count as f32)),justify_content : JustifyContent::Start, align_items: AlignItems::Start,
            layer,..default()})).id();
        let text_entity=add_text( commands,text.as_str(),StyleArgs{width: Val::Percent(100.),height: Val::Percent(100.),font_size:20.,
            direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::visible(),
        layer,..default()});
        commands.entity(legends_text_entity_parent).add_child(text_entity);
        commands.entity(legends_child_entity).add_child(legends_text_entity_parent);
        
    });
    
    legends_child_entity
}
fn add_track_entity(commands: &mut Commands,card_images: &CardImages, col_count: usize,player_position: Vec<usize>)->Entity{
    let layer = MAIN_LAYER;
    let track_entity = commands.spawn(create_area_bundle(100.0,100.0,
        FlexDirection::Column,JustifyContent::FlexStart,Some(AlignItems::Center), Overflow::clip_y(),None,None,None, layer)).id();

    for track_id in 0..player_position.len(){
        let row_entity = commands.spawn(create_area_bundle(100.0,50.0,
            FlexDirection::Row,JustifyContent::FlexStart,Some(AlignItems::Center), Overflow::clip_y(),None,None,None, layer)).id();

        for step_id in 0..col_count {
            //let mut row_entity = 
            let step_width=100.0/(col_count as f32);
            let col_entity = commands.spawn(
                create_area_bundle(step_width,100.0 ,FlexDirection::Row,JustifyContent::FlexStart,
                    Some(AlignItems::Center),Overflow::clip_x(),
                None,None,None, layer)
            ).id();
            //Add back image
            let back_image = commands.spawn(create_sprite_bundle(&card_images,StyleArgs{
                width: Val::Percent(100.),height: Val::Percent(100.),layer,image_key:TRACK.to_owned(),
                ..default()} )).id();
            commands.entity(col_entity).add_child(back_image);
            if step_id==col_count-1 {
                let finish_icon =  commands.spawn(create_sprite_bundle(&card_images,StyleArgs{image_key: FINISH_KEY.to_owned(), position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                        height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    left: Val::Px(0.),
                    top: Val::Px(0.),layer,..default()})).id();
                commands.entity(col_entity).add_child(finish_icon);
            }
            if step_id == player_position[track_id] {
                let text_parent =  commands.spawn(create_styled_node_bundle(
                    StyleArgs{width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        layer,
                         ..default()}
                        )).id();

                //let button_text=commands.spawn(create_text_bundle(format!("{}",step_id).as_str(),StyleArgs{layer,..default()})).id();
                let button_text=commands.spawn(create_text_bundle(format!("P{}",(track_id+1)).as_str(),StyleArgs{layer,..default()})).id();
                commands.entity(text_parent).add_child(button_text);
                commands.entity(col_entity).add_child(text_parent);
                
            }
            
            commands.entity(col_entity).insert(
                TrackElementComponent{
                    track: track_id,
                    step: step_id
                }
            );
            commands.entity(row_entity).add_child(col_entity);
        }
        
        commands.entity(track_entity).add_child(row_entity);
        //commands.entity(row_entity).insert(
        //    UiElementComponent::new(UiElement::BeforeGame(BeforeGameElements::CardList(player)),None));
        
    }
    //commands.entity(parent).add_child(col_entity);
    track_entity  
}

fn update_track_entity(game: &Game,commands: &mut Commands,card_images: &CardImages,parent_entity: Entity){
    commands.entity(parent_entity).despawn_descendants();
    // let player_count=game.match_state.as_ref().map(|m| m.player_count);
    // let player_count=player_count.unwrap_or(2) as usize;
    let steps=21;
    let mut player_steps: Vec<usize>=vec![];
    //Todo! handle for multiple players
    get_player1_steps(&game).as_ref().map(|m| player_steps.push(*m));
    get_player2_steps(&game).as_ref().map(|m| player_steps.push(*m));
    
    let track_entity = add_track_entity( commands, card_images,steps,player_steps);
    commands.entity(parent_entity).add_child(track_entity);
    
}

fn update_player_labels(game: &Game,commands: &mut Commands,parent_entity: Entity){
    commands.entity(parent_entity).despawn_descendants();
    let label_entity=add_player_legend(commands,&game, MAIN_LAYER);
    commands.entity(parent_entity).add_child(label_entity);
}

fn add_empty_card_deck(commands: &mut Commands,count: usize,space_x: f32,space_y:f32,card_images: &CardImages,deck_type: DeckType)->Entity{
    let parent = commands
    .spawn(create_node_bundle(100.,100.,FlexDirection::Row,JustifyContent::Center,Some(AlignItems::FlexStart),
        Overflow::visible(),None,None,None,None,MAIN_LAYER)).id();
    let mut x = 0.0;
    let mut top=0.0;
    for i in 0..count{
        let card_style=if space_x>0.{
            StyleArgs{
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                //top: Val::Px(top),
                outline: Some(OutlineArgs::default()),
                ..StyleArgs::card_sprite()
            }
        }else{
            StyleArgs{
                position_type: PositionType::Absolute,
                left: Val::Px(100.+x),
               // top: Val::Px(top),
                outline: Some(OutlineArgs::default()),
                ..StyleArgs::card_sprite()
            }
        };
        let card_prop=CardComponent::new((count - i).into(),None,None,false,CardFace::Down,CardComponentType::InActiveCard(deck_type.clone(),i));
        let image_key=card_prop.face.get_image_key().to_owned();
        let card_sprite=  add_card_sprite(commands, card_style,card_prop,card_images,MAIN_LAYER,image_key);
        x=x+space_x;
        top=top+space_y;
        commands.entity(parent).add_child(card_sprite);
    };
    
    parent
}

pub fn add_active_card(commands: &mut Commands,  card_images: &CardImages,card_component: CardComponent,layer: usize, deck_type: DeckType)->Entity{
    let image_key=if deck_type== DeckType::Env{
        ENVUP_KEY.to_owned()
    }else{
        card_component.face.get_image_key().to_owned()
    };
    let active_card=  add_card_sprite(commands, StyleArgs::card_sprite(),
    card_component,&card_images,layer,image_key);
    active_card
}

  
 


//Bevy system
pub fn card_click_interaction(
    mut interaction_query: Query<
    (
        &Interaction,
        &mut BorderColor,
        &Children,
    ),
    (Changed<Interaction>, With<Button>),
    >,
    mut card_query: Query<&mut CardComponent>,
    // q_windows: Query<&Window, With<PrimaryWindow>>,
){
    for (interaction, mut border_color, children) in &mut interaction_query {
        // for children_list in children.into_iter(){
            match card_query.get_mut(children[children.len()-1]) {
                Ok(mut card)=>{
                    match *interaction {
                        Interaction::Pressed => {
                            if card.is_selectable {
                                info!("card_click_interaction card = {:?}",card.index);
                                
                                card.selected=!card.selected;
                                if card.selected == true{
                                    border_color.0 = Color::srgba(1.0,0.0,0.0,1.0);
                                }else{
                                    border_color.0 = Color::srgba(0.0,0.0,0.0,1.0);
                                }
                            };
                        },
                        Interaction::Hovered=>{
                            // if let Some(position) = q_windows.single().cursor_position() {
                            //     info!("Cursor is inside the primary window, at {:?}", position);
                            // } else {
                            //     info!("Cursor is not in the game window.");
                            // }
                        },
                        _=>{},
                    }
                },
                _=>{
                    //info!("car")
                },
            }
        // }
        
    }
}
 

impl CardProp{
    pub fn get_image_key(&self)->Option<&str>{
        // let rarity= CardRarity::from_value(self.rarity as i32);
        let animal = self.animal;
        match animal{
            0=>Some(S_BULL),
            1=>Some(S_HORSE),
            2=>Some(A_DEER),
            3=>Some(A_GORILLA),
            4=>Some(B_GIRAFFE),
            5=>Some(B_RABBIT),
            6=>Some(C_CAT),
            7=>Some(C_DOG),
            8=>Some(D_FROG),
            9=>Some(D_SHEEP),
            _=>None,
        }
       
    }
}
 

pub fn create_player_card(commands: &mut Commands, card_images: &CardImages, card_prop: &CardComponent, layer: usize)->Entity{

    let parent = commands.spawn(create_styled_node_bundle(StyleArgs{height: Val::Px(CARD_HEIGHT),
        position_type: PositionType::Absolute,align_items: AlignItems::FlexStart,justify_content:JustifyContent::Default,
        width:Val::Px(CARD_WIDTH),layer,..default()})).id();

    if let Some(ref deck_type)= card_prop.val{
        let image_key=deck_type.get_image_key().clone().unwrap().to_owned();
        // info!("card_prop={:?} image_key={:?}",card_prop,image_key);
        
        match deck_type{
            DeckCardType::PlayerCard(card_prop)=>{
                let card_part = commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(100.), height: Val::Percent(50.),
                    overflow: Overflow::visible(), position_type: PositionType::Absolute, top:Val::Px(50.),left: Val::Px(0.), layer, ..default()})).id();
                //let image_key=card_prop.get_image_key().to_owned();
                let card_image_entity=commands.spawn(create_sprite_bundle(card_images,StyleArgs{image_key ,position_type: PositionType::Absolute,
                    top:Val::Px((CARD_WIDTH-ANIMAL_IMG_WIDTH)/2.0),left:Val::Px((CARD_WIDTH-ANIMAL_IMG_WIDTH)/2.0),
                    width: Val::Px(ANIMAL_IMG_WIDTH),height: Val::Px(ANIMAL_IMG_WIDTH), ..default()})).id();
                commands.entity(card_part).add_child(card_image_entity);
            
                // let step_part = commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(100.), height: Val::Percent(16.), overflow: Overflow::visible(),direction: FlexDirection::Row,position_type: PositionType::Absolute,top: Val::Px(CARD_HEIGHT/2.0), align_items: AlignItems::Center,
                //     left: Val::Px(0.), layer, ..default()})).id();
                // let geography_part = commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Px(25.), height: Val::Percent(100.),
                //     overflow: Overflow::visible(),direction: FlexDirection::Column, justify_content: JustifyContent::Start, position_type: PositionType::Absolute,top:Val::Percent(20.),left: Val::Px(10.), align_items: AlignItems::FlexStart,
                //      layer, ..default()})).id();
                
                let gs=&card_prop.favored_geographies;
                for (index,item) in gs.iter().enumerate() {
                    if index<6{
                        if item == &1 {
                            //Add
                            // info!("index = {:?} top={:?}",index,25.+(index as f32)*25.);
                            let geography=EnvCardGeography::from_u8(index as u8);
                            let image_entity=commands.spawn(create_sprite_bundle(card_images,StyleArgs{image_key:geography.get_image_key().to_owned() ,top: Val::Px(25.+(index as f32)*25.),position_type: PositionType::Absolute,
                                width: Val::Px(25.),height: Val::Px(25.),left: Val::Px(5.), ..StyleArgs::card_sprite()})).id();
                            commands.entity(parent).add_child(image_entity);
                        }
                    }
                }

                // let enemy_part = commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Px(25.), height: Val::Percent(100.),top:Val::Percent(20.),left: Val::Px(120.), justify_content: JustifyContent::Start, align_items: AlignItems::FlexStart,
                //     overflow: Overflow::visible(),direction: FlexDirection::Column, position_type: PositionType::Relative,layer, ..default()})).id();
                let es= &card_prop.weakness;
                //info!("gs ={:?}, es={:?}",gs,es);
                for (index,item) in es.iter().enumerate() {
                    if index < 3{
                        if item == &1 {
                            //Add
                            let enemy=EnvCardEnemy::from_u8(index as u8);
                            let image_entity=commands.spawn(create_sprite_bundle(card_images,StyleArgs{image_key:enemy.get_image_key().to_owned() ,top: Val::Px(25.+(index as f32)*25.),position_type: PositionType::Absolute,
                                width: Val::Px(25.),height: Val::Px(25.),left: Val::Px(115.), ..default()})).id();
                            commands.entity(parent).add_child(image_entity);
                        }
                    }
                }
                // let rarity_stat=create_stat(commands, &card_images,20.,20.,20.,20.,card_prop.rarity,layer,"");
                // let step_stat=create_stat(commands, &card_images,20.,20.,20.,40.,card_prop.steps,layer,"+");
                let text=commands.spawn(create_text_bundle(format!("{}{}","+",card_prop.steps).as_str(),StyleArgs{layer,
                    position_type: PositionType::Absolute,color: Color::srgb(0.2,0.2,0.2),
                    top:Val::Px(146.),left: Val::Percent(46.)
                    ,..default()})).id();

                // commands.entity(card_part).add_child(rarity_stat);
                // commands.entity(step_part).add_child(text);

                commands.entity(parent).add_child(card_part);
                commands.entity(parent).add_child(text);
                // commands.entity(parent).add_child(geography_part);
                // commands.entity(parent).add_child(enemy_part);
                
            },
            DeckCardType::EnvCard(_env_card)=>{
                let card_part = commands.spawn(create_styled_node_bundle(StyleArgs{width: Val::Percent(100.), height: Val::Percent(100.),
                    overflow: Overflow::visible(), position_type: PositionType::Absolute, top:Val::Px(0.),left: Val::Px(0.), layer, ..default()})).id();
                let card_image_entity=commands.spawn(create_sprite_bundle(card_images,StyleArgs{image_key,
                    position_type:PositionType::Absolute,top:Val::Px(37.),left:Val::Px(37.),
                    width: Val::Px(75.),
                    height: Val::Px(75.) ,..default()})).id();
                    commands.entity(card_part).add_child(card_image_entity);
                commands.entity(card_part).add_child(card_image_entity);
                commands.entity(parent).add_child(card_part);
            }
        }
        
    }
   
    parent

}

// fn create_stat(commands: &mut Commands, card_images: &CardImages,width: f32,height: f32, left: f32,top:f32, value: u64, layer: usize,prefix: &str,)->Entity{
//     let parent = commands.spawn(create_styled_node_bundle(StyleArgs{height: Val::Px(height),
//         width:Val::Px(width),layer,position_type : PositionType::Relative,left: Val::Px(left), top: Val::Px(top), ..default()})).id();
//     // let back_image=commands.spawn(create_sprite_bundle(card_images,StyleArgs{image_key:"".to_owned(),height: Val::Px(height),
//     //  width:Val::Px(width), layer, ..default()})).id();
//     let text=commands.spawn(create_text_bundle(format!("{}{}",prefix,value).as_str(),StyleArgs{layer,..default()})).id();
//     // commands.entity(back_image).add_child(text);
//     commands.entity(parent).add_child(text);
//     parent
// }



//Bevy system
pub fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            //Todo! Fix this 
            let items_width = 750.;
            let container_width = query_node.get(parent.get()).unwrap().size().x;

            let max_scroll = (items_width - container_width).max(0.);
            // info!("items_width={:?} container_width={:?} id={:?}",items_width,container_width,scrolling_list.id);
            let dx = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.x * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.x,
            };
            //info!("dx={:?}",dx);

            scrolling_list.position += dx;
            scrolling_list.position = scrolling_list.position.clamp( -max_scroll,0.);
            style.left = Val::Px(scrolling_list.position);
        }
    }
}
