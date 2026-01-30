use bevy::audio::AudioSource;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SoundAssets {
    pub break_grass: Handle<AudioSource>,
    pub break_stone: Handle<AudioSource>,
    pub place_block: Handle<AudioSource>,
    pub pickup_item: Handle<AudioSource>,
    pub step_grass: Handle<AudioSource>,
    pub step_stone: Handle<AudioSource>,
    pub step_dirt: Handle<AudioSource>,
    pub hit_grass: [Handle<AudioSource>; 4],
    pub hit_stone: [Handle<AudioSource>; 4],
}

pub fn load_sound_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut hit_grass: [Handle<AudioSource>; 4] = [
        Handle::default(),
        Handle::default(),
        Handle::default(),
        Handle::default(),
    ];
    for i in 0..4 {
        hit_grass[i] = asset_server.load(format!("sounds/step/grass{}.ogg", i + 1));
    }
    let mut hit_stone: [Handle<AudioSource>; 4] = [
        Handle::default(),
        Handle::default(),
        Handle::default(),
        Handle::default(),
    ];
    for i in 0..4 {
        hit_stone[i] = asset_server.load(format!("sounds/step/stone{}.ogg", i + 1));
    }

    commands.insert_resource(SoundAssets {
        break_grass: asset_server.load("sounds/dig/grass1.ogg"),
        break_stone: asset_server.load("sounds/dig/stone1.ogg"),
        place_block: asset_server.load("sounds/random/wood_click.ogg"),
        pickup_item: asset_server.load("sounds/random/pop.ogg"),
        step_grass: asset_server.load("sounds/block/moss/step1.ogg"),
        step_stone: asset_server.load("sounds/block/deepslate/step1.ogg"),
        step_dirt: asset_server.load("sounds/block/rooted_dirt/step1.ogg"),
        hit_grass,
        hit_stone,
    });
}
