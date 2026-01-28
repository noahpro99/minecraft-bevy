use crate::world::components::CHUNK_SIZE;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub chunks: HashMap<IVec3, Entity>,
}

impl VoxelWorld {
    pub fn world_to_chunk_pos(world_pos: Vec3) -> IVec3 {
        (world_pos / CHUNK_SIZE as f32).floor().as_ivec3()
    }

    pub fn world_to_voxel_pos(world_pos: Vec3) -> IVec3 {
        world_pos.floor().as_ivec3()
    }

    pub fn voxel_to_local_pos(voxel_pos: IVec3) -> IVec3 {
        let mut local_pos = voxel_pos % CHUNK_SIZE as i32;
        if local_pos.x < 0 {
            local_pos.x += CHUNK_SIZE as i32;
        }
        if local_pos.y < 0 {
            local_pos.y += CHUNK_SIZE as i32;
        }
        if local_pos.z < 0 {
            local_pos.z += CHUNK_SIZE as i32;
        }
        local_pos
    }
}
