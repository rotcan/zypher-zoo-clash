use bevy::prelude::*;
use crate::{ToastExpirationTime,ToastEvent,Toast,toast::generate_toast};

pub fn remove_expired_toasts(mut commands: Commands, time: Res<Time>,
    toast_query: Query<(Entity, &ToastExpirationTime)>){
    let current_time=time.elapsed_seconds_f64();
    for (entity, expiration_time) in toast_query.iter() {
        if expiration_time.expiration_time < current_time {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn handle_toast_events(mut commands: Commands, mut toast_events: EventReader<ToastEvent>,time: Res<Time>,
    toast_query: Query<(Entity, &Toast)>){
    for event in toast_events.read(){
        match event{
            ToastEvent::ShowToast{data}=>{generate_toast(&mut commands, &time, &data)},
            ToastEvent::HideToast=>{
                for (entity, _) in toast_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
        
    }
}
