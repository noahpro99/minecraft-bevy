use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::log;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_rapier3d::prelude::*;

use crate::player::settings_menu::Settings;
use crate::world::components::{
    Chunk, ChunkPosition, DespawnChunk, NeedsMeshUpdate, SunLight, VoxelType, CHUNK_SIZE,
};
use crate::world::VoxelWorld;

#[derive(Component)]
pub struct Block;

#[derive(Resource)]
pub struct BlockAssets {
    pub mesh: Handle<Mesh>,
    pub grass_material: Handle<StandardMaterial>,
    pub dirt_material: Handle<StandardMaterial>,
    pub stone_material: Handle<StandardMaterial>,
    pub glowstone_material: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct InitialChunkMeshing(pub bool);

const MAX_CHUNKS_PER_FRAME: usize = 4;
const MAX_MESH_UPDATES_PER_FRAME: usize = 2;

pub fn spawn_chunks_around_player(
    mut commands: Commands,
    mut voxel_world: ResMut<VoxelWorld>,
    player_query: Query<&Transform, With<crate::player::components::Player>>,
    settings: Res<Settings>,
    initial_meshing: Res<InitialChunkMeshing>,
) {
    let player_transform = match player_query.iter().next() {
        Some(t) => t,
        None => return,
    };

    let player_chunk_pos = VoxelWorld::world_to_chunk_pos(player_transform.translation);
    let view_distance = settings.render_distance;
    let vertical_distance = 2;

    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;

    if initial_meshing.0 {
        return;
    }

    let mut spawned = 0;
    for y in -vertical_distance..=vertical_distance {
        for x in -view_distance..=view_distance {
            for z in -view_distance..=view_distance {
                let chunk_pos = player_chunk_pos + IVec3::new(x, y, z);
                let chunk_key = chunk_pos;

                if !voxel_world.chunks.contains_key(&chunk_key) {
                    let chunk_data = generate_chunk(chunk_key, base_height, amplitude, frequency);

                    let entity = commands
                        .spawn((
                            chunk_data,
                            ChunkPosition(chunk_key),
                            Transform::from_translation(chunk_key.as_vec3() * CHUNK_SIZE as f32),
                            GlobalTransform::default(),
                            RigidBody::Fixed,
                            Friction::coefficient(0.0),
                            Visibility::Visible,
                        ))
                        .id();
                    voxel_world.chunks.insert(chunk_key, entity);

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
                    if spawned >= MAX_CHUNKS_PER_FRAME {
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
    let vertical_distance = 2;

    let mut to_remove = Vec::new();

    for (chunk_pos, entity) in voxel_world.chunks.iter() {
        let delta = *chunk_pos - player_chunk_pos;
        if delta.x.abs() > view_distance
            || delta.z.abs() > view_distance
            || delta.y.abs() > vertical_distance
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
    let mut processed = 0;
    for (entity, chunk, chunk_pos, existing_mesh) in query.iter() {
        if processed >= limit {
            break;
        }
        processed += 1;
        log::info!("Updating mesh for chunk entity: {:?}", entity);

        #[derive(Default)]
        struct MeshBuffers {
            positions: Vec<[f32; 3]>,
            normals: Vec<[f32; 3]>,
            uvs: Vec<[f32; 2]>,
            indices: Vec<u32>,
        }

        impl MeshBuffers {
            fn add_face(&mut self, vertices: [[f32; 3]; 4], normal: [f32; 3]) {
                let start_idx = self.positions.len() as u32;
                for v in vertices {
                    self.positions.push(v);
                    self.normals.push(normal);
                }
                self.uvs
                    .extend_from_slice(&[[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
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
        let mut grass = MeshBuffers::default();
        let mut dirt = MeshBuffers::default();
        let mut stone = MeshBuffers::default();
        let mut glowstone = MeshBuffers::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = IVec3::new(x as i32, y as i32, z as i32);
                    let voxel = chunk.get_voxel(pos);
                    if voxel == VoxelType::Air {
                        continue;
                    }

                    let mut target_buffers = match voxel {
                        VoxelType::Grass => Some(&mut grass),
                        VoxelType::Dirt => Some(&mut dirt),
                        VoxelType::Stone => Some(&mut stone),
                        VoxelType::Glowstone => Some(&mut glowstone),
                        VoxelType::Air => None,
                    };

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

                        if neighbor_voxel == VoxelType::Air {
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
                            if let Some(buffers) = target_buffers.as_deref_mut() {
                                buffers.add_face(face, normal);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(entity).remove::<Mesh3d>();
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();

        if combined.is_empty() {
            if let Some(mesh) = existing_mesh {
                meshes.remove(mesh.0.id());
            }
            commands.entity(entity).remove::<Collider>();
            commands.entity(entity).remove::<NeedsMeshUpdate>();
            continue;
        }

        let collider_mesh = combined.into_mesh();
        let collider = Collider::from_bevy_mesh(
            &collider_mesh,
            &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
        )
        .unwrap();
        commands
            .entity(entity)
            .insert((collider, Visibility::Visible));

        let grass_mesh = if grass.is_empty() {
            None
        } else {
            Some(grass.into_mesh())
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
        let glowstone_mesh = if glowstone.is_empty() {
            None
        } else {
            Some(glowstone.into_mesh())
        };

        commands.entity(entity).with_children(|parent| {
            if let Some(mesh) = grass_mesh {
                let handle = meshes.add(mesh);
                parent.spawn((
                    Mesh3d(handle),
                    MeshMaterial3d(block_assets.grass_material.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
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
                ));
            }
            if let Some(mesh) = glowstone_mesh {
                let handle = meshes.add(mesh);
                parent.spawn((
                    Mesh3d(handle),
                    MeshMaterial3d(block_assets.glowstone_material.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                ));
            }
        });

        commands.entity(entity).remove::<NeedsMeshUpdate>();
    }

    if initial_meshing.0 && query.is_empty() {
        initial_meshing.0 = false;
    }
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut voxel_world: ResMut<VoxelWorld>,
    settings: Res<Settings>,
) {
    commands.insert_resource(InitialChunkMeshing(true));
    let dirt_texture = asset_server.load_with_settings(
        "textures/dirt.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let grass_texture = asset_server.load_with_settings(
        "textures/grass_block_top.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let stone_texture = asset_server.load_with_settings(
        "textures/stone.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let glowstone_texture = asset_server.load_with_settings(
        "textures/glowstone.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let mesh_handle = meshes.add(Cuboid::from_size(Vec3::ONE));
    let grass_material = materials.add(StandardMaterial {
        base_color_texture: Some(grass_texture),
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
    let glowstone_material = materials.add(StandardMaterial {
        base_color_texture: Some(glowstone_texture),
        base_color: Color::WHITE,
        emissive: Color::srgb(4.0, 4.0, 3.0).into(),
        ..default()
    });

    commands.insert_resource(BlockAssets {
        mesh: mesh_handle.clone(),
        grass_material,
        dirt_material,
        stone_material,
        glowstone_material,
    });

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 30000.0,
            ..default()
        },
        Transform::from_xyz(80.0, 120.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight,
    ));

    let view_distance = settings.render_distance;
    let vertical_distance = 2;
    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;

    for y in -vertical_distance..=vertical_distance {
        for x in -view_distance..=view_distance {
            for z in -view_distance..=view_distance {
                let chunk_key = IVec3::new(x, y, z);
                let chunk_data = generate_chunk(chunk_key, base_height, amplitude, frequency);
                let entity = commands
                    .spawn((
                        chunk_data,
                        ChunkPosition(chunk_key),
                        Transform::from_translation(chunk_key.as_vec3() * CHUNK_SIZE as f32),
                        GlobalTransform::default(),
                        RigidBody::Fixed,
                        Friction::coefficient(0.0),
                        Visibility::Visible,
                    ))
                    .id();
                voxel_world.chunks.insert(chunk_key, entity);
            }
        }
    }
}

fn generate_chunk(chunk_key: IVec3, base_height: f32, amplitude: f32, frequency: f32) -> Chunk {
    let mut chunk_data = Chunk::empty();
    let chunk_world_y = chunk_key.y * CHUNK_SIZE as i32;

    for vx in 0..CHUNK_SIZE {
        for vz in 0..CHUNK_SIZE {
            let world_vx = chunk_key.x * CHUNK_SIZE as i32 + vx as i32;
            let world_vz = chunk_key.z * CHUNK_SIZE as i32 + vz as i32;

            let wave = (world_vx as f32 * frequency).sin()
                + (world_vz as f32 * frequency).cos()
                + (world_vx as f32 * frequency * 0.5).sin() * 0.5
                + (world_vz as f32 * frequency * 0.5).cos() * 0.5;
            let mut height = (base_height + wave * amplitude * 0.5).round() as i32;
            if height < 1 {
                height = 1;
            }

            for vy in 0..CHUNK_SIZE {
                let world_vy = chunk_world_y + vy as i32;
                if world_vy <= height {
                    let voxel = if world_vy == height {
                        VoxelType::Grass
                    } else if glowstone_at(world_vx, world_vy, world_vz) {
                        VoxelType::Glowstone
                    } else {
                        VoxelType::Stone
                    };

                    chunk_data.set_voxel(IVec3::new(vx as i32, vy as i32, vz as i32), voxel);
                }
            }
        }
    }

    chunk_data
}

fn glowstone_at(x: i32, y: i32, z: i32) -> bool {
    let hash = (x as i64 * 734287 + y as i64 * 912931 + z as i64 * 1237).abs();
    (hash % 100) < 3
}
