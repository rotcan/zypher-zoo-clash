pub mod web3;
pub mod style;
pub mod game;
pub mod error;

use bevy::{prelude::*};
use bevy::window::PresentMode;
use bevy_web3::{plugin::WalletPlugin,contract::{init_contract_channel}};
use crate::game::{Game,load_sprites,CardImages,MenuData,init_ui,start_ui,running_ui,GameStatus, PopupDrawEvent,
    card_click_interaction,draw_popup, PopupResponseEvent,init_http_resource,UiUpdateEvent,mouse_scroll};
use crate::web3::{recv_contract_response,do_contract_call,WalletState,ContractState,handle_game_status_change,
    direct_user_interaction,wallet_account,CallContractEvent,InGameContractEvent,
    DelegateTxnSendEvent,delegate_txn_call,handle_contract_state_change,update_ui_event_handler,
    delegate_txn_response_call,init_compute_resource,spawn_async_task,recv_async_task,
    request_contract_data,request_contract_data_if_true,read_editable_text,
    await_http_request,send_http_request,
    };
use bevy::render::view::visibility::RenderLayers;
use std::fmt::Debug;
use bevy_pkv::PkvStore;
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_text_edit::{TextEditPluginNoState,listen_changing_focus};

pub const MAIN_LAYER: usize = 1;
pub const OVERLAY_LAYER: usize=2;
pub const POPUP_LAYER: usize=3;

#[derive(Clone,Eq,PartialEq,Debug,Hash,Default,States)]
pub enum GameState{
    #[default]
    Init,
    GameStart,
    GameRunning,
    GameEnd,
    //WalletConnect,
}

pub fn start(){
   
    App::new()
        //File path is relative to project root
        .add_plugins(DefaultPlugins
            .set(AssetPlugin{ file_path: "game/assets".to_owned(),..Default::default()})
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }))
        //.add_plugins((MinimalPlugins, PanicHandlerPlugin))
        //.add_plugins(TextInputPlugin)
        .add_plugins(TextEditPluginNoState)
        //.add_plugins(EguiPlugin)
        .add_plugins(WalletPlugin)
        .insert_resource(PkvStore::new("Rotcan", "AnimalRaceGame"))
        .init_resource::<MenuData>()
        .init_state::<GameState>()
        .init_state::<GameStatus>()
        .init_state::<WalletState>()
        .init_state::<ContractState>()
        .add_event::<CallContractEvent>()
        .add_event::<InGameContractEvent>()
        .add_event::<UiUpdateEvent>()
        .add_event::<DelegateTxnSendEvent>()
        .add_event::<PopupDrawEvent>()
        .add_event::<PopupResponseEvent>()
        .add_systems(Startup,setup_2d_cameras)
        //.add_systems(Startup,connect_wallet)
        .insert_resource(CardImages::default())
        .insert_resource(Game::init())
        .insert_resource(init_contract_channel()) 
        .insert_resource(init_compute_resource())
        .insert_resource(init_http_resource())
        .add_systems(Startup,load_sprites)
        .add_systems(OnEnter(GameState::Init),
            init_ui,
        )
        .add_systems(OnEnter(GameState::GameStart),
            start_ui,
        )
        .add_systems(OnEnter(GameState::GameRunning),
            running_ui,
        )
        // .add_systems(OnEnter(GameState::GameEnd),
        //     running_ui,
        // )   
        .add_systems(
            Update,
            (
                direct_user_interaction,
                delegate_txn_call,
                wallet_account,
                delegate_txn_response_call,
                card_click_interaction,
                draw_popup,
                update_ui_event_handler,
                mouse_scroll
            )
        )
        .add_systems(Update,handle_contract_state_change.run_if(state_changed::<ContractState>))
        .add_systems(
            Update,
            (
                do_contract_call,
                recv_contract_response,
                
            )
        )
        .add_systems(
            Update,read_editable_text.after(listen_changing_focus)
        )
        .add_systems(Update, request_contract_data.run_if(request_contract_data_if_true))
        .add_systems(Update, handle_game_status_change.run_if(state_changed::<GameStatus>))
        //.add_systems(Update, text_listener.after(TextInputSystem))
        .add_systems(Update,(spawn_async_task,recv_async_task,send_http_request,await_http_request))
        .run();
}
 

fn setup_2d_cameras(mut commands: Commands) {
     
 
    commands.spawn(
        (
        Camera2dBundle{
            camera: Camera{
                order:1,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[MAIN_LAYER,OVERLAY_LAYER,POPUP_LAYER])
        //Camera2dBundle::default()
        )
    );
    // commands.spawn(
    //     (
    //     Camera2dBundle{
    //         camera: Camera{
    //             order:2,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     OVERLAY_LAYER    
    //     )
    // );
    // commands.spawn(
    //     (
    //     Camera2dBundle{
    //         camera: Camera{
    //             order:3,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     POPUP_LAYER    
    //     )
    // );
}
