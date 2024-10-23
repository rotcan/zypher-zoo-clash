use bevy::prelude::*;
use crate::{ToastExpirationTime,ToastData,Toast};
use bevy::render::view::visibility::RenderLayers;

pub fn generate_toast(commands: &mut Commands, time: &Time, event: &ToastData){
    //root bundle
    let root=commands.spawn(
        (
            NodeBundle{
                style:Style{
                    // display: Display::Flex,
                    // flex_direction: FlexDirection::Row,

                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width : Val::Percent(50.),
                    height: Val::Auto,
                position_type: PositionType::Absolute,
                    left: Val::Percent(25.),
                    right: Val::Px(0.),
                ..default()
                },
                z_index: event.z_index,
                ..default()
                
            },
            RenderLayers::layer(event.layer)
        )
    ).id();
    //add timeout to root
    commands.entity(root).insert((Toast,ToastExpirationTime{expiration_time: time.elapsed_seconds_f64()+ event.timeout_secs as f64}));
    
    let border_entity=commands.spawn(
        (
        NodeBundle{
            style: Style{
                width : Val::Auto,
                height: Val::Auto,
                border: event.border,
                padding: event.padding,
                margin:event.margin,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color:BackgroundColor(event.background_color),
            border_color: BorderColor(event.border_color),
            ..default()
        },RenderLayers::layer(event.layer))
    ).id();

    let text_entity=commands.spawn(
        (
        TextBundle::from_section(
            &event.content,
            TextStyle {
                font_size: event.font_size,
                color: event.font_color,
                ..default()
            },
        ).with_style(Style{
            padding: event.padding,
            margin: event.margin,
            ..default()
        })
        .with_text_justify(event.text_alignment),
        RenderLayers::layer(event.layer))
        ).id();
     

    commands.entity(border_entity).add_child(text_entity);
    commands.entity(root).add_child(border_entity);

}