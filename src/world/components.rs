use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 16;

#[derive(Component)]
pub struct NeedsMeshUpdate;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum VoxelType {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Bedrock,
    TallGrass,
}

#[derive(Component)]
pub struct InGameEntity;

#[derive(Component)]
pub struct Chunk {
    pub voxels: [[[VoxelType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

#[derive(Component, Copy, Clone, Debug)]
pub struct ChunkPosition(pub IVec3);

#[derive(Resource)]
pub struct GameTime {
    pub time: f32, // 0.0 to 1.0 (0.5 is noon, 0.0/1.0 is midnight)
    pub day_length_seconds: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            time: 0.5,
            day_length_seconds: 600.0, // 10 minutes
        }
    }
}

#[derive(Component)]
pub struct SunLight;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum ItemType {
    #[default]
    None,
    GrassBlock,
    Dirt,
    Stone,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Wheat,
}

#[derive(Component)]
pub struct DropItem {
    pub item_type: ItemType,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct DespawnChunk;

impl Chunk {
    pub fn empty() -> Self {
        Self {
            voxels: [[[VoxelType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }

    pub fn get_voxel(&self, pos: IVec3) -> VoxelType {
        if pos.x < 0
            || pos.x >= CHUNK_SIZE as i32
            || pos.y < 0
            || pos.y >= CHUNK_SIZE as i32
            || pos.z < 0
            || pos.z >= CHUNK_SIZE as i32
        {
            return VoxelType::Air;
        }
        self.voxels[pos.x as usize][pos.y as usize][pos.z as usize]
    }

    pub fn set_voxel(&mut self, pos: IVec3, voxel: VoxelType) {
        if pos.x >= 0
            && pos.x < CHUNK_SIZE as i32
            && pos.y >= 0
            && pos.y < CHUNK_SIZE as i32
            && pos.z >= 0
            && pos.z < CHUNK_SIZE as i32
        {
            self.voxels[pos.x as usize][pos.y as usize][pos.z as usize] = voxel;
        }
    }
}
