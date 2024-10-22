pub mod systems;
mod toast;
use bevy::prelude::*;

#[derive(Debug,Event)]
pub enum ToastEvent{
    ShowToast{data: ToastData},
    HideToast,
}

#[derive(Debug)]
pub struct ToastData{
    pub content: String,
    pub font: Option<Handle<Font>>,
    pub font_size: f32,
    pub font_color: Color,
    pub border: UiRect,
    pub border_color: Color,
    pub padding: UiRect,
    pub margin: UiRect,
    pub text_alignment: JustifyText,
    pub background_color: Color,
    pub z_index: ZIndex,
    pub timeout_secs: f32,
    pub layer: usize,
}

impl Default for ToastData{
    fn default()->Self{
        ToastData{
            content: "".to_owned(),
            font: None,
            font_size: 20.,
            font_color: Color::WHITE,
            border: UiRect::all(Val::Px(0.)),
            border_color: Color::BLACK,
            padding: UiRect::all(Val::Px(4.)),
            margin: UiRect::all(Val::Px(4.)),
            text_alignment: JustifyText::Center,
            background_color: Color::BLACK,
            z_index: ZIndex::Global(0),
            timeout_secs: 5.,
            layer: 1,

        }
    }
}
pub struct ToastPlugin;

#[derive(Debug, Component)]
pub struct ToastExpirationTime {
    pub expiration_time: f64,
}

#[derive(Debug, Component)]
pub struct Toast;



impl Plugin for ToastPlugin{
    fn build(&self, app: &mut App){
        app.add_event::<ToastEvent>()
            .add_systems(
                Update,
                (   
                    systems::handle_toast_events,
                    systems::remove_expired_toasts
                )
            );
    }
}