use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use resources::{ChunkLoadFrameCounter, VoxelWorld};
use systems::{
    apply_chunk_despawns, despawn_far_chunks, reset_voxel_world, setup_world,
    spawn_chunks_around_player, update_chunk_mesh, update_game_time,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<crate::world::components::GameTime>()
            .init_resource::<ChunkLoadFrameCounter>()
            .add_systems(
                OnEnter(crate::main_menu::AppState::InGame),
                (reset_voxel_world, setup_world).chain(),
            )
            .add_systems(
                Update,
                (
                    spawn_chunks_around_player,
                    despawn_far_chunks,
                    apply_chunk_despawns,
                    update_chunk_mesh,
                    update_game_time,
                )
                    .run_if(in_state(crate::main_menu::AppState::InGame)),
            );
    }
}
