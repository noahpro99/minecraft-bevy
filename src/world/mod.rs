use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use resources::VoxelWorld;
use systems::{
    apply_chunk_despawns, despawn_far_chunks, setup_world, spawn_chunks_around_player,
    update_chunk_mesh,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .add_systems(Startup, setup_world)
            .add_systems(
                Update,
                (
                    update_chunk_mesh,
                    spawn_chunks_around_player,
                    despawn_far_chunks,
                ),
            )
            .add_systems(PostUpdate, apply_chunk_despawns);
    }
}
