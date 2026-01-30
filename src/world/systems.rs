use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_rapier3d::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::main_menu::WorldSettings;
use crate::player::settings_menu::Settings;
use crate::world::components::{
    Chunk, ChunkPosition, DespawnChunk, NeedsMeshUpdate, SunLight, VoxelType, CHUNK_SIZE,
};
use crate::world::VoxelWorld;

#[derive(Component)]
#[allow(dead_code)]
pub struct Block;

#[derive(Resource)]
pub struct BlockAssets {
    pub mesh: Handle<Mesh>,
    pub grass_top_material: Handle<StandardMaterial>,
    pub grass_side_material: Handle<StandardMaterial>,
    pub dirt_material: Handle<StandardMaterial>,
    pub stone_material: Handle<StandardMaterial>,
    pub coal_ore_material: Handle<StandardMaterial>,
    pub iron_ore_material: Handle<StandardMaterial>,
    pub gold_ore_material: Handle<StandardMaterial>,
    pub diamond_ore_material: Handle<StandardMaterial>,
    pub bedrock_material: Handle<StandardMaterial>,
    pub tall_grass_material: Handle<StandardMaterial>,
    pub wheat_material: Handle<StandardMaterial>,
    pub destroy_stages: [Handle<StandardMaterial>; 10],
}

#[derive(Resource, Default)]
pub struct InitialChunkMeshing(pub bool);

const MAX_CHUNKS_PER_FRAME: usize = usize::MAX;
const MAX_MESH_UPDATES_PER_FRAME: usize = usize::MAX;
const WORLD_MIN_Y: i32 = -32;
const WORLD_MAX_Y: i32 = 96;

pub fn spawn_chunks_around_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut voxel_world: ResMut<VoxelWorld>,
    player_query: Query<&Transform, With<crate::player::components::Player>>,
    settings: Res<Settings>,
    initial_meshing: Res<InitialChunkMeshing>,
    world_settings: Res<WorldSettings>,
) {
    let player_transform = match player_query.iter().next() {
        Some(t) => t,
        None => return,
    };

    let player_chunk_pos = VoxelWorld::world_to_chunk_pos(player_transform.translation);
    let view_distance = settings.render_distance;
    let (min_chunk_y, max_chunk_y) = world_chunk_y_range();

    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;

    if initial_meshing.0 {
        return;
    }

    let perlin = Perlin::new(world_settings.seed as u32);

    let mut spawned = 0;
    for y in min_chunk_y..=max_chunk_y {
        for x in -view_distance..=view_distance {
            for z in -view_distance..=view_distance {
                let chunk_key = IVec3::new(player_chunk_pos.x + x, y, player_chunk_pos.z + z);

                if let std::collections::hash_map::Entry::Vacant(e) =
                    voxel_world.chunks.entry(chunk_key)
                {
                    let chunk_data =
                        generate_chunk(chunk_key, base_height, amplitude, frequency, &perlin);

                    let entity = commands
                        .spawn((
                            chunk_data,
                            ChunkPosition(chunk_key),
                            Transform::from_translation(chunk_key.as_vec3() * CHUNK_SIZE as f32),
                            GlobalTransform::default(),
                            RigidBody::Fixed,
                            Friction::coefficient(0.0),
                            Visibility::Visible,
                            crate::world::components::InGameEntity,
                        ))
                        .id();
                    e.insert(entity);

                    // Hash-based cow spawning for localized herds
                    let cow_spawn_chance = {
                        let mut has_cow = false;
                        // Check the current chunk and its immediate neighbors (1 chunk radius)
                        for dx in -1..=1 {
                            for dz in -1..=1 {
                                let nx = chunk_key.x + dx;
                                let nz = chunk_key.z + dz;
                                // Deterministic hash for "Cow Source" chunks (0.4% chance)
                                let hash = (nx as i64 * 734287 ^ nz as i64 * 1237).abs();
                                if hash % 1000 < 4 {
                                    has_cow = true;
                                    break;
                                }
                            }
                            if has_cow {
                                break;
                            }
                        }
                        has_cow
                    };

                    if cow_spawn_chance {
                        let mut rng_cow = rand::thread_rng();
                        // 20% chance to spawn 1-2 cows in any chunk within a cow cluster area
                        if rand::Rng::gen_bool(&mut rng_cow, 0.20) {
                            for _ in 0..rand::Rng::gen_range(&mut rng_cow, 1..3) {
                                let ox = rand::Rng::gen_range(&mut rng_cow, 0.0..CHUNK_SIZE as f32);

                                let oz = rand::Rng::gen_range(&mut rng_cow, 0.0..CHUNK_SIZE as f32);
                                let world_x = chunk_key.x as f32 * CHUNK_SIZE as f32 + ox;
                                let world_z = chunk_key.z as f32 * CHUNK_SIZE as f32 + oz;

                                let noise_val = perlin.get([
                                    world_x as f64 * frequency as f64,
                                    world_z as f64 * frequency as f64,
                                ]);
                                let height = (base_height + noise_val as f32 * amplitude).round();

                                crate::mob::systems::spawn_mob_typed(
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                    Vec3::new(world_x, height + 1.0, world_z),
                                    crate::mob::components::MobType::Cow,
                                );
                            }
                        }
                    }

                    let neighbors = [
                        IVec3::new(1, 0, 0),
                        IVec3::new(-1, 0, 0),
                        IVec3::new(0, 0, 1),
                        IVec3::new(0, 0, -1),
                        IVec3::new(0, 1, 0),
                        IVec3::new(0, -1, 0),
                    ];
                    for offset in neighbors {
                        if let Some(neighbor_entity) = voxel_world.chunks.get(&(chunk_key + offset))
                        {
                            commands.entity(*neighbor_entity).insert(NeedsMeshUpdate);
                        }
                    }

                    spawned += 1;
                    if spawned == MAX_CHUNKS_PER_FRAME {
                        return;
                    }
                }
            }
        }
    }
}

pub fn despawn_far_chunks(
    mut commands: Commands,
    mut voxel_world: ResMut<VoxelWorld>,
    player_query: Query<&Transform, With<crate::player::components::Player>>,
    settings: Res<Settings>,
) {
    let player_transform = match player_query.single() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let player_chunk_pos = VoxelWorld::world_to_chunk_pos(player_transform.translation);
    let view_distance = settings.render_distance;
    let (min_chunk_y, max_chunk_y) = world_chunk_y_range();

    let mut to_remove = Vec::new();

    for (chunk_pos, entity) in voxel_world.chunks.iter() {
        let delta_x = chunk_pos.x - player_chunk_pos.x;
        let delta_z = chunk_pos.z - player_chunk_pos.z;
        if delta_x.abs() > view_distance
            || delta_z.abs() > view_distance
            || chunk_pos.y < min_chunk_y
            || chunk_pos.y > max_chunk_y
        {
            commands.entity(*entity).insert(DespawnChunk);
            to_remove.push(*chunk_pos);
        }
    }

    for chunk_pos in to_remove {
        voxel_world.chunks.remove(&chunk_pos);
    }
}

pub fn apply_chunk_despawns(
    mut commands: Commands,
    chunks: Query<(Entity, Option<&Children>), With<DespawnChunk>>,
) {
    for (entity, children) in chunks.iter() {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(entity).despawn();
    }
}

pub fn reset_voxel_world(mut voxel_world: ResMut<VoxelWorld>) {
    voxel_world.chunks.clear();
}

#[allow(clippy::too_many_arguments)]
pub fn update_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_assets: Res<BlockAssets>,
    children_query: Query<&Children>,
    voxel_world: Res<VoxelWorld>,
    chunk_lookup: Query<&Chunk>,
    mut initial_meshing: ResMut<InitialChunkMeshing>,
    query: Query<
        (Entity, &Chunk, &ChunkPosition, Option<&Mesh3d>),
        (
            Or<(Added<Chunk>, With<NeedsMeshUpdate>)>,
            Without<DespawnChunk>,
        ),
    >,
) {
    let limit = if initial_meshing.0 {
        usize::MAX
    } else {
        MAX_MESH_UPDATES_PER_FRAME
    };
    for (processed, (entity, chunk, chunk_pos, existing_mesh)) in query.iter().enumerate() {
        if processed >= limit {
            break;
        }
        // log::info!("Updating mesh for chunk entity: {:?}", entity);

        #[derive(Default)]
        struct MeshBuffers {
            positions: Vec<[f32; 3]>,
            normals: Vec<[f32; 3]>,
            uvs: Vec<[f32; 2]>,
            indices: Vec<u32>,
        }

        impl MeshBuffers {
            fn add_face(&mut self, vertices: [[f32; 3]; 4], _normal: [f32; 3]) {
                let start_idx = self.positions.len() as u32;
                for v in vertices {
                    self.positions.push(v);
                }
                self.normals
                    .extend_from_slice(&[_normal, _normal, _normal, _normal]);
                self.uvs
                    .extend_from_slice(&[[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]]);
                self.indices.extend_from_slice(&[
                    start_idx,
                    start_idx + 1,
                    start_idx + 2,
                    start_idx,
                    start_idx + 2,
                    start_idx + 3,
                ]);
            }

            fn is_empty(&self) -> bool {
                self.positions.is_empty()
            }

            fn into_mesh(self) -> Mesh {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
                mesh.insert_indices(Indices::U32(self.indices));
                mesh
            }
        }

        let mut combined = MeshBuffers::default();
        let mut grass_top = MeshBuffers::default();
        let mut grass_side = MeshBuffers::default();
        let mut dirt = MeshBuffers::default();
        let mut stone = MeshBuffers::default();
        let mut coal_ore = MeshBuffers::default();
        let mut iron_ore = MeshBuffers::default();
        let mut gold_ore = MeshBuffers::default();
        let mut diamond_ore = MeshBuffers::default();
        let mut bedrock = MeshBuffers::default();
        let mut tall_grass = MeshBuffers::default();
        let mut tall_grass_collision = MeshBuffers::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = IVec3::new(x as i32, y as i32, z as i32);
                    let voxel = chunk.get_voxel(pos);
                    if voxel == VoxelType::Air {
                        continue;
                    }

                    if voxel == VoxelType::TallGrass {
                        let visual_faces = [
                            [
                                [pos.x as f32, pos.y as f32, pos.z as f32],
                                [pos.x as f32 + 1.0, pos.y as f32, pos.z as f32 + 1.0],
                                [pos.x as f32 + 1.0, pos.y as f32 + 1.0, pos.z as f32 + 1.0],
                                [pos.x as f32, pos.y as f32 + 1.0, pos.z as f32],
                            ],
                            [
                                [pos.x as f32 + 1.0, pos.y as f32, pos.z as f32 + 1.0],
                                [pos.x as f32, pos.y as f32, pos.z as f32],
                                [pos.x as f32, pos.y as f32 + 1.0, pos.z as f32],
                                [pos.x as f32 + 1.0, pos.y as f32 + 1.0, pos.z as f32 + 1.0],
                            ],
                            [
                                [pos.x as f32 + 1.0, pos.y as f32, pos.z as f32],
                                [pos.x as f32, pos.y as f32, pos.z as f32 + 1.0],
                                [pos.x as f32, pos.y as f32 + 1.0, pos.z as f32 + 1.0],
                                [pos.x as f32 + 1.0, pos.y as f32 + 1.0, pos.z as f32],
                            ],
                            [
                                [pos.x as f32, pos.y as f32, pos.z as f32 + 1.0],
                                [pos.x as f32 + 1.0, pos.y as f32, pos.z as f32],
                                [pos.x as f32 + 1.0, pos.y as f32 + 1.0, pos.z as f32],
                                [pos.x as f32, pos.y as f32 + 1.0, pos.z as f32 + 1.0],
                            ],
                        ];
                        for face in visual_faces {
                            tall_grass.add_face(face, [0.0, 0.0, 0.0]);
                        }

                        // Add full cube for collision (Sensor)
                        let cube_faces = [
                            (
                                [0.0, 1.0, 0.0],
                                [
                                    [0.0, 1.0, 0.0],
                                    [0.0, 1.0, 1.0],
                                    [1.0, 1.0, 1.0],
                                    [1.0, 1.0, 0.0],
                                ],
                            ),
                            (
                                [0.0, -1.0, 0.0],
                                [
                                    [0.0, 0.0, 0.0],
                                    [1.0, 0.0, 0.0],
                                    [1.0, 0.0, 1.0],
                                    [0.0, 0.0, 1.0],
                                ],
                            ),
                            (
                                [1.0, 0.0, 0.0],
                                [
                                    [1.0, 0.0, 1.0],
                                    [1.0, 0.0, 0.0],
                                    [1.0, 1.0, 0.0],
                                    [1.0, 1.0, 1.0],
                                ],
                            ),
                            (
                                [-1.0, 0.0, 0.0],
                                [
                                    [0.0, 0.0, 0.0],
                                    [0.0, 0.0, 1.0],
                                    [0.0, 1.0, 1.0],
                                    [0.0, 1.0, 0.0],
                                ],
                            ),
                            (
                                [0.0, 0.0, 1.0],
                                [
                                    [0.0, 0.0, 1.0],
                                    [1.0, 0.0, 1.0],
                                    [1.0, 1.0, 1.0],
                                    [0.0, 1.0, 1.0],
                                ],
                            ),
                            (
                                [0.0, 0.0, -1.0],
                                [
                                    [1.0, 0.0, 0.0],
                                    [0.0, 0.0, 0.0],
                                    [0.0, 1.0, 0.0],
                                    [1.0, 1.0, 0.0],
                                ],
                            ),
                        ];
                        for (normal, vertices) in cube_faces {
                            let face = [
                                [
                                    pos.x as f32 + vertices[0][0],
                                    pos.y as f32 + vertices[0][1],
                                    pos.z as f32 + vertices[0][2],
                                ],
                                [
                                    pos.x as f32 + vertices[1][0],
                                    pos.y as f32 + vertices[1][1],
                                    pos.z as f32 + vertices[1][2],
                                ],
                                [
                                    pos.x as f32 + vertices[2][0],
                                    pos.y as f32 + vertices[2][1],
                                    pos.z as f32 + vertices[2][2],
                                ],
                                [
                                    pos.x as f32 + vertices[3][0],
                                    pos.y as f32 + vertices[3][1],
                                    pos.z as f32 + vertices[3][2],
                                ],
                            ];
                            tall_grass_collision.add_face(face, normal);
                        }
                        continue;
                    }

                    let faces = [
                        (
                            IVec3::new(0, 1, 0),
                            [0.0, 1.0, 0.0],
                            [
                                [0.0, 1.0, 0.0],
                                [0.0, 1.0, 1.0],
                                [1.0, 1.0, 1.0],
                                [1.0, 1.0, 0.0],
                            ],
                        ),
                        (
                            IVec3::new(0, -1, 0),
                            [0.0, -1.0, 0.0],
                            [
                                [0.0, 0.0, 1.0],
                                [0.0, 0.0, 0.0],
                                [1.0, 0.0, 0.0],
                                [1.0, 0.0, 1.0],
                            ],
                        ),
                        (
                            IVec3::new(1, 0, 0),
                            [1.0, 0.0, 0.0],
                            [
                                [1.0, 0.0, 1.0],
                                [1.0, 0.0, 0.0],
                                [1.0, 1.0, 0.0],
                                [1.0, 1.0, 1.0],
                            ],
                        ),
                        (
                            IVec3::new(-1, 0, 0),
                            [-1.0, 0.0, 0.0],
                            [
                                [0.0, 0.0, 0.0],
                                [0.0, 0.0, 1.0],
                                [0.0, 1.0, 1.0],
                                [0.0, 1.0, 0.0],
                            ],
                        ),
                        (
                            IVec3::new(0, 0, 1),
                            [0.0, 0.0, 1.0],
                            [
                                [0.0, 0.0, 1.0],
                                [1.0, 0.0, 1.0],
                                [1.0, 1.0, 1.0],
                                [0.0, 1.0, 1.0],
                            ],
                        ),
                        (
                            IVec3::new(0, 0, -1),
                            [0.0, 0.0, -1.0],
                            [
                                [1.0, 0.0, 0.0],
                                [0.0, 0.0, 0.0],
                                [0.0, 1.0, 0.0],
                                [1.0, 1.0, 0.0],
                            ],
                        ),
                    ];

                    for (offset, normal, vertices) in faces {
                        let neighbor_pos = pos + offset;
                        let neighbor_voxel = if neighbor_pos.x >= 0
                            && neighbor_pos.x < CHUNK_SIZE as i32
                            && neighbor_pos.y >= 0
                            && neighbor_pos.y < CHUNK_SIZE as i32
                            && neighbor_pos.z >= 0
                            && neighbor_pos.z < CHUNK_SIZE as i32
                        {
                            chunk.get_voxel(neighbor_pos)
                        } else {
                            let world_voxel_pos = chunk_pos.0 * CHUNK_SIZE as i32 + neighbor_pos;
                            let neighbor_chunk_pos =
                                VoxelWorld::world_to_chunk_pos(world_voxel_pos.as_vec3());
                            let neighbor_local_pos =
                                VoxelWorld::voxel_to_local_pos(world_voxel_pos);
                            voxel_world
                                .chunks
                                .get(&neighbor_chunk_pos)
                                .and_then(|entity| chunk_lookup.get(*entity).ok())
                                .map(|neighbor_chunk| neighbor_chunk.get_voxel(neighbor_local_pos))
                                .unwrap_or(VoxelType::Air)
                        };

                        if neighbor_voxel == VoxelType::Air
                            || neighbor_voxel == VoxelType::TallGrass
                        {
                            let face = [
                                [
                                    pos.x as f32 + vertices[0][0],
                                    pos.y as f32 + vertices[0][1],
                                    pos.z as f32 + vertices[0][2],
                                ],
                                [
                                    pos.x as f32 + vertices[1][0],
                                    pos.y as f32 + vertices[1][1],
                                    pos.z as f32 + vertices[1][2],
                                ],
                                [
                                    pos.x as f32 + vertices[2][0],
                                    pos.y as f32 + vertices[2][1],
                                    pos.z as f32 + vertices[2][2],
                                ],
                                [
                                    pos.x as f32 + vertices[3][0],
                                    pos.y as f32 + vertices[3][1],
                                    pos.z as f32 + vertices[3][2],
                                ],
                            ];

                            combined.add_face(face, normal);
                            if voxel == VoxelType::Grass {
                                if normal[1] > 0.5 {
                                    grass_top.add_face(face, normal);
                                } else if normal[1] < -0.5 {
                                    dirt.add_face(face, normal);
                                } else {
                                    grass_side.add_face(face, normal);
                                }
                            } else {
                                let buffers = match voxel {
                                    VoxelType::Dirt => Some(&mut dirt),
                                    VoxelType::Stone => Some(&mut stone),
                                    VoxelType::CoalOre => Some(&mut coal_ore),
                                    VoxelType::IronOre => Some(&mut iron_ore),
                                    VoxelType::GoldOre => Some(&mut gold_ore),
                                    VoxelType::DiamondOre => Some(&mut diamond_ore),
                                    VoxelType::Bedrock => Some(&mut bedrock),
                                    VoxelType::Grass | VoxelType::Air | VoxelType::TallGrass => {
                                        None
                                    }
                                };

                                if let Some(buffers) = buffers {
                                    buffers.add_face(face, normal);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut child_commands) = commands.get_entity(child) {
                    child_commands.despawn();
                }
            }
        }
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<Mesh3d>();
            entity_commands.remove::<MeshMaterial3d<StandardMaterial>>();
        } else {
            continue;
        }

        if combined.is_empty() && tall_grass_collision.is_empty() {
            if let Some(mesh) = existing_mesh {
                meshes.remove(mesh.0.id());
            }
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<Collider>();
                entity_commands.remove::<NeedsMeshUpdate>();
            }
            continue;
        }

        let collider_mesh = combined.into_mesh();
        let collider = Collider::from_bevy_mesh(
            &collider_mesh,
            &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
        )
        .unwrap();
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert((collider, Visibility::Visible));

            if !tall_grass_collision.is_empty() {
                let sensor_mesh = tall_grass_collision.into_mesh();
                let sensor_collider = Collider::from_bevy_mesh(
                    &sensor_mesh,
                    &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
                )
                .unwrap();
                entity_commands.with_children(|parent| {
                    parent.spawn((
                        sensor_collider,
                        Sensor,
                        Transform::default(),
                        GlobalTransform::default(),
                    ));
                });
            }
        } else {
            continue;
        }

        let grass_top_mesh = if grass_top.is_empty() {
            None
        } else {
            Some(grass_top.into_mesh())
        };
        let grass_side_mesh = if grass_side.is_empty() {
            None
        } else {
            Some(grass_side.into_mesh())
        };
        let dirt_mesh = if dirt.is_empty() {
            None
        } else {
            Some(dirt.into_mesh())
        };
        let stone_mesh = if stone.is_empty() {
            None
        } else {
            Some(stone.into_mesh())
        };
        let coal_ore_mesh = if coal_ore.is_empty() {
            None
        } else {
            Some(coal_ore.into_mesh())
        };
        let iron_ore_mesh = if iron_ore.is_empty() {
            None
        } else {
            Some(iron_ore.into_mesh())
        };
        let gold_ore_mesh = if gold_ore.is_empty() {
            None
        } else {
            Some(gold_ore.into_mesh())
        };
        let diamond_ore_mesh = if diamond_ore.is_empty() {
            None
        } else {
            Some(diamond_ore.into_mesh())
        };
        let bedrock_mesh = if bedrock.is_empty() {
            None
        } else {
            Some(bedrock.into_mesh())
        };
        let tall_grass_mesh = if tall_grass.is_empty() {
            None
        } else {
            Some(tall_grass.into_mesh())
        };

        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.with_children(|parent| {
                if let Some(mesh) = grass_top_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.grass_top_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = grass_side_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.grass_side_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = dirt_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.dirt_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = stone_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.stone_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = coal_ore_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.coal_ore_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = iron_ore_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.iron_ore_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = gold_ore_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.gold_ore_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = diamond_ore_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.diamond_ore_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = bedrock_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.bedrock_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
                if let Some(mesh) = tall_grass_mesh {
                    let handle = meshes.add(mesh);
                    parent.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(block_assets.tall_grass_material.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ));
                }
            });

            entity_commands.remove::<NeedsMeshUpdate>();
        }
    }

    if initial_meshing.0 && query.is_empty() {
        initial_meshing.0 = false;
    }
}

pub fn update_game_time(
    time: Res<Time>,
    mut game_time: ResMut<crate::world::components::GameTime>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), With<SunLight>>,
    mut celestial_query: Query<
        (&mut Transform, &crate::world::components::CelestialBody),
        Without<SunLight>,
    >,
) {
    let dt = time.delta_secs();
    game_time.time += dt / game_time.day_length_seconds;
    if game_time.time >= 1.0 {
        game_time.time -= 1.0;
    }

    if let Ok((mut transform, mut light)) = sun_query.single_mut() {
        // Angle goes from 0 to 2*PI
        let _angle = game_time.time * std::f32::consts::TAU;

        // Rotate sun around X axis
        // We want 0.5 to be noon (sun overhead)
        // angle_adjusted = (time - 0.25) * 2*PI
        // at 0.5: angle = 0.25 * 2*PI = PI/2. sin(PI/2) = 1.0 (overhead)
        let angle_adjusted = (game_time.time - 0.25) * std::f32::consts::TAU;
        let sun_dir = Vec3::new(0.0, angle_adjusted.sin(), angle_adjusted.cos());
        let distance = 100.0;

        // Update sun position
        let sun_pos = sun_dir * distance;
        transform.translation = sun_pos;
        transform.look_at(Vec3::ZERO, Vec3::Y);

        let sin_angle = angle_adjusted.sin();

        // 0.0 to 1.0 based on height above horizon
        let intensity = sin_angle.max(0.0);

        // Day: 8000, Night: 100
        light.illuminance = 100.0 + intensity * 7900.0;
    }

    // Update celestial bodies (sun and moon visuals)
    for (mut transform, celestial) in celestial_query.iter_mut() {
        let angle_adjusted = (game_time.time - 0.25) * std::f32::consts::TAU;
        let distance = 90.0; // Distance from player

        let sun_dir = Vec3::new(0.0, angle_adjusted.sin(), angle_adjusted.cos());

        if celestial.is_moon {
            // Moon is opposite to sun (180 degrees offset)
            let moon_pos = -sun_dir * distance;
            transform.translation = moon_pos;
            transform.look_at(Vec3::ZERO, Vec3::Y);
        } else {
            let sun_pos = sun_dir * distance;
            transform.translation = sun_pos;
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut voxel_world: ResMut<VoxelWorld>,
    settings: Res<Settings>,
    world_settings: Res<WorldSettings>,
) {
    commands.insert_resource(InitialChunkMeshing(true));
    let dirt_texture = asset_server.load_with_settings(
        "textures/block/dirt.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let grass_top_texture = asset_server.load_with_settings(
        "textures/block/grass_block_top.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let grass_side_texture = asset_server.load_with_settings(
        "textures/block/grass_block_side.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let stone_texture = asset_server.load_with_settings(
        "textures/block/stone.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let coal_ore_texture = asset_server.load_with_settings(
        "textures/block/coal_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let iron_ore_texture = asset_server.load_with_settings(
        "textures/block/iron_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let gold_ore_texture = asset_server.load_with_settings(
        "textures/block/gold_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let diamond_ore_texture = asset_server.load_with_settings(
        "textures/block/diamond_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let bedrock_texture = asset_server.load_with_settings(
        "textures/block/bedrock.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let tall_grass_texture = asset_server.load_with_settings(
        "textures/block/short_grass.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let wheat_texture = asset_server.load_with_settings(
        "textures/item/wheat.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );

    let mut destroy_stages = Vec::new();
    for i in 0..10 {
        let texture = asset_server.load_with_settings(
            format!("textures/block/destroy_stage_{}.png", i),
            |settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            },
        );
        destroy_stages.push(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }));
    }

    let mesh_handle = meshes.add(Cuboid::from_size(Vec3::ONE));
    let grass_top_material = materials.add(StandardMaterial {
        base_color_texture: Some(grass_top_texture),
        base_color: Color::srgb(0.5, 0.8, 0.4),
        ..default()
    });
    let grass_side_material = materials.add(StandardMaterial {
        base_color_texture: Some(grass_side_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let dirt_material = materials.add(StandardMaterial {
        base_color_texture: Some(dirt_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let stone_material = materials.add(StandardMaterial {
        base_color_texture: Some(stone_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let coal_ore_material = materials.add(StandardMaterial {
        base_color_texture: Some(coal_ore_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let iron_ore_material = materials.add(StandardMaterial {
        base_color_texture: Some(iron_ore_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let gold_ore_material = materials.add(StandardMaterial {
        base_color_texture: Some(gold_ore_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let diamond_ore_material = materials.add(StandardMaterial {
        base_color_texture: Some(diamond_ore_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let bedrock_material = materials.add(StandardMaterial {
        base_color_texture: Some(bedrock_texture),
        base_color: Color::WHITE,
        ..default()
    });
    let tall_grass_material = materials.add(StandardMaterial {
        base_color_texture: Some(tall_grass_texture),
        base_color: Color::srgb(0.5, 0.8, 0.4),
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        ..default()
    });
    let wheat_material = materials.add(StandardMaterial {
        base_color_texture: Some(wheat_texture),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        ..default()
    });

    commands.insert_resource(BlockAssets {
        mesh: mesh_handle.clone(),
        grass_top_material,
        grass_side_material,
        dirt_material,
        stone_material,
        coal_ore_material,
        iron_ore_material,
        gold_ore_material,
        diamond_ore_material,
        bedrock_material,
        tall_grass_material,
        wheat_material,
        destroy_stages: destroy_stages.try_into().unwrap(),
    });

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 8000.0,
            ..default()
        },
        Transform::from_xyz(80.0, 120.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight,
        crate::world::components::InGameEntity,
    ));

    // Spawn sun
    let sun_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_settings(
            "textures/environment/celestial/sun.png",
            |settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            },
        )),
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(8.0, 8.0))),
        MeshMaterial3d(sun_material),
        Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
        crate::world::components::CelestialBody { is_moon: false },
        crate::world::components::InGameEntity,
    ));

    // Spawn moon
    let moon_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_settings(
            "textures/environment/celestial/moon/full_moon.png",
            |settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            },
        )),
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(8.0, 8.0))),
        MeshMaterial3d(moon_material),
        Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
        crate::world::components::CelestialBody { is_moon: true },
        crate::world::components::InGameEntity,
    ));

    let view_distance = settings.render_distance;
    let (min_chunk_y, max_chunk_y) = world_chunk_y_range();
    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;

    let perlin = Perlin::new(world_settings.seed as u32);

    for y in min_chunk_y..=max_chunk_y {
        for x in -view_distance..=view_distance {
            for z in -view_distance..=view_distance {
                let chunk_key = IVec3::new(x, y, z);
                let chunk_data =
                    generate_chunk(chunk_key, base_height, amplitude, frequency, &perlin);
                let entity = commands
                    .spawn((
                        chunk_data,
                        ChunkPosition(chunk_key),
                        Transform::from_translation(chunk_key.as_vec3() * CHUNK_SIZE as f32),
                        GlobalTransform::default(),
                        RigidBody::Fixed,
                        Friction::coefficient(0.0),
                        Visibility::Visible,
                        crate::world::components::InGameEntity,
                    ))
                    .id();
                voxel_world.chunks.insert(chunk_key, entity);

                // Check for cows in initial chunks
                let mut rng = rand::thread_rng();
                let seed = world_settings.seed as f32;

                // Sample the center of the chunk for the hotspot
                let sample_x = chunk_key.x as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0;
                let sample_z = chunk_key.z as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0;

                // Use a much smaller frequency (0.005) for 100-300 block spacing
                let cow_freq = 0.005;
                let hotspot = ((sample_x * cow_freq + seed % 1000.0).sin()
                    + (sample_z * cow_freq + seed % 1000.0).cos())
                    > 0.95; // Increased threshold for rarity

                if hotspot && y == max_chunk_y {
                    // Only spawn in top chunks
                    // Spawn a herd of cows
                    for _ in 0..rand::Rng::gen_range(&mut rng, 2..6) {
                        // Reduced herd size
                        let ox = rand::Rng::gen_range(&mut rng, 0.0..CHUNK_SIZE as f32);
                        let oz = rand::Rng::gen_range(&mut rng, 0.0..CHUNK_SIZE as f32);
                        let world_x = chunk_key.x as f32 * CHUNK_SIZE as f32 + ox;
                        let world_z = chunk_key.z as f32 * CHUNK_SIZE as f32 + oz;

                        // Estimate height (could use generate_chunk but let's keep it simple for now)
                        let noise_val = perlin.get([
                            world_x as f64 * frequency as f64,
                            world_z as f64 * frequency as f64,
                        ]);
                        let height = (base_height + noise_val as f32 * amplitude).round();

                        crate::mob::systems::spawn_mob_typed(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Vec3::new(world_x, height + 1.0, world_z),
                            crate::mob::components::MobType::Cow,
                        );
                    }
                }
            }
        }
    }
}

fn generate_chunk(
    chunk_key: IVec3,
    base_height: f32,
    amplitude: f32,
    frequency: f32,
    perlin: &Perlin,
) -> Chunk {
    let mut chunk_data = Chunk::empty();
    let chunk_world_y = chunk_key.y * CHUNK_SIZE as i32;

    for vx in 0..CHUNK_SIZE {
        for vz in 0..CHUNK_SIZE {
            let world_vx = chunk_key.x * CHUNK_SIZE as i32 + vx as i32;
            let world_vz = chunk_key.z * CHUNK_SIZE as i32 + vz as i32;

            let noise_val = perlin.get([
                world_vx as f64 * frequency as f64,
                world_vz as f64 * frequency as f64,
            ]);
            let mut height = (base_height + noise_val as f32 * amplitude).round() as i32;
            if height < WORLD_MIN_Y + 1 {
                height = WORLD_MIN_Y + 1;
            }
            if height > WORLD_MAX_Y - 1 {
                height = WORLD_MAX_Y - 1;
            }

            for vy in 0..CHUNK_SIZE {
                let world_vy = chunk_world_y + vy as i32;
                if world_vy > WORLD_MAX_Y {
                    continue;
                }

                if world_vy == WORLD_MIN_Y {
                    chunk_data.set_voxel(
                        IVec3::new(vx as i32, vy as i32, vz as i32),
                        VoxelType::Bedrock,
                    );
                    continue;
                }
                if world_vy < WORLD_MIN_Y {
                    chunk_data.set_voxel(
                        IVec3::new(vx as i32, vy as i32, vz as i32),
                        VoxelType::Stone,
                    );
                    continue;
                }

                if world_vy <= height {
                    let voxel = if world_vy == height {
                        VoxelType::Grass
                    } else {
                        select_stone_variant(world_vx, world_vy, world_vz)
                    };

                    chunk_data.set_voxel(IVec3::new(vx as i32, vy as i32, vz as i32), voxel);

                    if world_vy == height && world_vy < WORLD_MAX_Y {
                        let hash = (world_vx as i64 * 734287 + world_vz as i64 * 1237).abs();
                        if hash % 100 < 5 {
                            chunk_data.set_voxel(
                                IVec3::new(vx as i32, vy as i32 + 1, vz as i32),
                                VoxelType::TallGrass,
                            );
                        }
                    }
                }
            }
        }
    }

    chunk_data
}

fn select_stone_variant(x: i32, y: i32, z: i32) -> VoxelType {
    let hash = (x as i64 * 734287 + y as i64 * 912931 + z as i64 * 1237).abs();
    let roll = (hash % 100) as i32;

    if y < 10 && roll < 2 {
        VoxelType::DiamondOre
    } else if y < 20 && roll < 4 {
        VoxelType::GoldOre
    } else if y < 40 && roll < 7 {
        VoxelType::IronOre
    } else if roll < 12 {
        VoxelType::CoalOre
    } else {
        VoxelType::Stone
    }
}

fn world_chunk_y_range() -> (i32, i32) {
    let min = (WORLD_MIN_Y as f32 / CHUNK_SIZE as f32).floor() as i32;
    let max = (WORLD_MAX_Y as f32 / CHUNK_SIZE as f32).floor() as i32;
    (min, max)
}
