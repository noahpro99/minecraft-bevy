use bevy::prelude::*;

pub mod components;
pub mod systems;

pub use components::*;
pub use systems::*;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MobSpawner>().add_systems(
            Update,
            (mob_spawner_system, mob_behavior_system, mob_movement_system)
                .run_if(in_state(crate::main_menu::AppState::InGame)),
        );
    }
}
