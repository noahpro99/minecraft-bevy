use bevy::prelude::*;

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
}

#[derive(Component)]
pub struct Chunk {
    pub voxels: [[[VoxelType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

#[derive(Component, Copy, Clone, Debug)]
pub struct ChunkPosition(pub IVec3);

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct DropItem {
    pub voxel_type: VoxelType,
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
