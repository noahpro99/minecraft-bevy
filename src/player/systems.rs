use crate::mob::components::{Mob, MobBehavior, MobState, MobType};
use crate::player::components::{
    CameraController, CharacterController, DespawnMiningEffect, FootstepTimer, Health, Hunger,
    Inventory, MiningProgress, PickupDrops, Player,
};
use crate::player::resources::SoundAssets;
use crate::player::settings_menu::Settings;
use crate::world::components::{CHUNK_SIZE, Chunk, DropItem, ItemType, NeedsMeshUpdate, VoxelType};
use crate::world::resources::VoxelWorld;
use crate::world::systems::{BlockAssets, InitialChunkMeshing};
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_player(mut commands: Commands, world_settings: Res<crate::main_menu::WorldSettings>) {
    let inventory = world_settings.inventory.clone().unwrap_or_default();
    let spawn_pos = world_settings
        .player_position
        .unwrap_or_else(|| Vec3::new(0.0, spawn_height(), 0.0));

    let player_entity = commands
        .spawn((
            Player,
            CharacterController::default(),
            MiningProgress::default(),
            inventory,
            Health::default(),
            Hunger::default(),
            PickupDrops,
            Transform::from_translation(spawn_pos),
            GlobalTransform::default(),
        ))
        .insert((
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::cuboid(0.3, 0.9, 0.3),
            Friction::coefficient(0.0),
            LockedAxes::ROTATION_LOCKED,
            Ccd::enabled(),
            Velocity::default(),
            crate::world::components::InGameEntity,
        ))
        .insert(FootstepTimer::default())
        .insert(TransformInterpolation::default())
        .id();

    let camera_entity = commands
        .spawn((
            Camera3d::default(),
            Camera {
                clear_color: Color::srgb(0.5, 0.7, 1.0).into(), // Light blue sky
                order: 0,
                ..default()
            },
            IsDefaultUiCamera,
            CameraController { sensitivity: 0.1 },
            Transform::from_xyz(0.0, 0.75, 0.0),
            GlobalTransform::default(),
            Visibility::Visible,
            crate::world::components::InGameEntity,
        ))
        .id();

    commands.entity(player_entity).add_child(camera_entity);
}

fn spawn_height() -> f32 {
    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;
    let wave = (0.0_f32 * frequency).sin()
        + (0.0_f32 * frequency).cos()
        + (0.0_f32 * frequency * 0.5).sin() * 0.5
        + (0.0_f32 * frequency * 0.5).cos() * 0.5;
    let height = (base_height + wave * amplitude * 0.5).round();
    height + 3.0
}

pub fn spawn_player_when_ready(
    commands: Commands,
    meshing: Res<InitialChunkMeshing>,
    players: Query<Entity, With<Player>>,
    voxel_world: Res<VoxelWorld>,
    chunk_colliders: Query<(), With<Collider>>,
    world_settings: Res<crate::main_menu::WorldSettings>,
) {
    if meshing.0 {
        return;
    }
    if !players.is_empty() {
        return;
    }

    let spawn_pos = world_settings
        .player_position
        .unwrap_or_else(|| Vec3::new(0.0, spawn_height(), 0.0));

    let spawn_chunk_pos = VoxelWorld::world_to_chunk_pos(spawn_pos);
    let has_chunk = voxel_world
        .chunks
        .get(&spawn_chunk_pos)
        .map(|entity| chunk_colliders.contains(*entity))
        .unwrap_or(false);
    if !has_chunk {
        return;
    }

    spawn_player(commands, world_settings);
}



pub fn player_move(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    mut query: Query<(
        Entity,
        &Transform,
        &mut CharacterController,
        &mut Velocity,
        &mut Health,
    )>,
    rapier_context: ReadRapierContext,
    settings_menu: Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
    command_state: Res<crate::player::inventory_ui::CommandState>,
) {
    if command_state.open {
        return;
    }

    if let Ok(visibility) = settings_menu.single()
        && *visibility != Visibility::Hidden
    {
        return;
    }

    if let Ok((entity, transform, mut controller, mut velocity, mut health)) = query.single_mut() {
        let rapier_context = rapier_context.single().expect("No RapierContext found");
        let rotation = transform.rotation;
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;

        let mut direction = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += right;
        }

        direction.y = 0.0;
        let direction = direction.normalize_or_zero();

        let mut speed = controller.speed;
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            speed = 5.612;
        }
        let sneaking = keyboard_input.pressed(KeyCode::ControlLeft);
        if sneaking {
            speed = controller.speed / 3.0;
        }
        let dt = time.delta_secs();
        let target_velocity = direction * speed;
        let current_velocity = Vec2::new(velocity.linvel.x, velocity.linvel.z);
        let target_velocity_2d = Vec2::new(target_velocity.x, target_velocity.z);
        let accel = if direction.length_squared() > 0.0 {
            30.0
        } else {
            40.0
        };
        let delta = target_velocity_2d - current_velocity;
        let max_change = accel * dt;
        let change = if delta.length() > max_change {
            delta.normalize() * max_change
        } else {
            delta
        };

        let mut next_velocity = current_velocity + change;

        if sneaking && controller.is_grounded {
            let next_position =
                transform.translation + Vec3::new(next_velocity.x, 0.0, next_velocity.y) * dt;
            let ray_origin = next_position + Vec3::new(0.0, 0.05, 0.0);
            let ray_dir = -Vec3::Y;
            let max_toi = 1.1;
            let has_ground = rapier_context
                .cast_ray(
                    ray_origin,
                    ray_dir,
                    max_toi,
                    true,
                    QueryFilter::default().exclude_rigid_body(entity),
                )
                .is_some();
            if !has_ground {
                next_velocity = Vec2::ZERO;
            }
        }

        velocity.linvel.x = if next_velocity.x.abs() < 0.001 {
            0.0
        } else {
            next_velocity.x
        };
        velocity.linvel.z = if next_velocity.y.abs() < 0.001 {
            0.0
        } else {
            next_velocity.y
        };

        // Ground check using Rapier raycast from player's feet
        let was_grounded = controller.is_grounded;
        let ray_pos = transform.translation + Vec3::new(0.0, -0.9, 0.0); // Feet position (bottom of collider)
        let ray_dir = -Vec3::Y;
        let max_toi = 0.15; // Short distance just to check if there's a block right below

        controller.is_grounded = false;
        if let Some((_entity, toi)) = rapier_context.cast_ray(
            ray_pos,
            ray_dir,
            max_toi,
            true,
            QueryFilter::default().exclude_rigid_body(entity).exclude_sensors(),
        )
        {
            controller.is_grounded = true;
        }

        if was_grounded && !controller.is_grounded {
            controller.fall_start_y = transform.translation.y;
        }

        if !was_grounded && controller.is_grounded {
            let fall_distance = controller.fall_start_y - transform.translation.y;
            if fall_distance > 3.0 {
                let damage = (fall_distance - 3.0).floor() as i32;
                health.current = (health.current - damage).max(0);
            }
            controller.fall_start_y = transform.translation.y;
        }

        if controller.is_grounded && keyboard_input.pressed(KeyCode::Space) {
            velocity.linvel.y = controller.jump_force;
            controller.is_grounded = false;
        }

        if !controller.is_grounded {
            velocity.linvel.y *= 0.98;
        }
    }
}

pub fn update_hunger(
    time: Res<Time<Fixed>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Hunger, &Velocity, &mut Health), With<Player>>,
) {
    let Ok((mut hunger, velocity, mut health)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let moving = velocity.linvel.xz().length_squared() > 0.1;
    let sprinting = keyboard_input.pressed(KeyCode::ShiftLeft) && moving;
    let interval = if sprinting { 8.0 } else { 20.0 };

    if hunger.current > 0 {
        hunger.timer += dt;
        if hunger.timer >= interval {
            hunger.timer -= interval;
            hunger.current = (hunger.current - 1).max(0);
        }
        hunger.damage_timer = 0.0;
    } else {
        hunger.damage_timer += dt;
        if hunger.damage_timer >= 2.0 {
            hunger.damage_timer -= 2.0;
            health.current = (health.current - 1).max(0);
        }
    }
}

pub fn handle_player_death(
    mut query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Health,
            &mut Hunger,
            &mut CharacterController,
        ),
        With<Player>,
    >,
) {
    let Ok((mut transform, mut velocity, mut health, mut hunger, mut controller)) =
        query.single_mut()
    else {
        return;
    };

    if health.current > 0 {
        return;
    }

    health.current = health.max;
    hunger.current = hunger.max;
    hunger.timer = 0.0;
    hunger.damage_timer = 0.0;
    transform.translation = Vec3::new(0.0, spawn_height(), 0.0);
    velocity.linvel = Vec3::ZERO;
    controller.is_grounded = false;
    controller.is_grounded = false;
    controller.fall_start_y = transform.translation.y;
}

pub fn update_sprint_fov(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<crate::player::settings_menu::Settings>,
    mut cameras: Query<&mut Projection, With<Camera3d>>,
) {
    let sprinting = keyboard_input.pressed(KeyCode::ShiftLeft);
    let target_fov = settings.fov + if sprinting { 8.0 } else { 0.0 };
    let t = (time.delta_secs() * 10.0).clamp(0.0, 1.0);

    for mut projection in cameras.iter_mut() {
        if let Projection::Perspective(perspective) = &mut *projection {
            let current = perspective.fov.to_degrees();
            let next = current + (target_fov - current) * t;
            perspective.fov = next.to_radians();
        }
    }
}

pub fn player_inventory_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Inventory, With<Player>>,
) {
    if let Ok(mut inventory) = query.single_mut() {
        if keyboard_input.just_pressed(KeyCode::Digit1) {
            inventory.selected_slot = 0;
        }
        if keyboard_input.just_pressed(KeyCode::Digit2) {
            inventory.selected_slot = 1;
        }
        if keyboard_input.just_pressed(KeyCode::Digit3) {
            inventory.selected_slot = 2;
        }
        if keyboard_input.just_pressed(KeyCode::Digit4) {
            inventory.selected_slot = 3;
        }
        if keyboard_input.just_pressed(KeyCode::Digit5) {
            inventory.selected_slot = 4;
        }
        if keyboard_input.just_pressed(KeyCode::Digit6) {
            inventory.selected_slot = 5;
        }
        if keyboard_input.just_pressed(KeyCode::Digit7) {
            inventory.selected_slot = 6;
        }
        if keyboard_input.just_pressed(KeyCode::Digit8) {
            inventory.selected_slot = 7;
        }
        if keyboard_input.just_pressed(KeyCode::Digit9) {
            inventory.selected_slot = 8;
        }
        if keyboard_input.just_pressed(KeyCode::Digit0) {
            inventory.selected_slot = 9;
        }
    }
}

pub fn despawn_mining_effects(mut commands: Commands, query: Query<Entity, With<DespawnMiningEffect>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(SystemParam)]
pub struct InteractionParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub mouse_input: Res<'w, ButtonInput<MouseButton>>,
    pub camera_query: Query<'w, 's, (&'static GlobalTransform, &'static Camera)>,
    pub rapier_context: ReadRapierContext<'w, 's>,
    pub chunk_query: Query<'w, 's, &'static mut Chunk>,
    pub voxel_world: Res<'w, VoxelWorld>,
    pub block_assets: Res<'w, BlockAssets>,
    pub sound_assets: Res<'w, SoundAssets>,
    pub settings: Res<'w, Settings>,
    pub inventory_query: Query<'w, 's, &'static mut Inventory, With<Player>>,
    pub mining_query: Query<'w, 's, &'static mut MiningProgress, With<Player>>,
    pub player_query: Query<'w, 's, (Entity, &'static GlobalTransform), With<Player>>,
    pub settings_menu:
        Query<'w, 's, &'static Visibility, With<crate::player::settings_menu::SettingsMenu>>,
    pub mob_query: Query<
        'w,
        's,
        (
            &'static Mob,
            &'static mut MobState,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
}

pub fn player_interact(mut params: InteractionParams, time: Res<Time>) {
    let mut rng = rand::thread_rng();
    if let Ok(visibility) = params.settings_menu.single()
        && *visibility != Visibility::Hidden
    {
        return;
    }

    let right_click = params.mouse_input.just_pressed(MouseButton::Right);
    let left_click_pressed = params.mouse_input.pressed(MouseButton::Left);
    let _left_click_just_pressed = params.mouse_input.just_pressed(MouseButton::Left);

    let Ok(mut mining_progress) = params.mining_query.single_mut() else {
        return;
    };

    if !right_click && !left_click_pressed {
        mining_progress.target = None;
        mining_progress.progress = 0.0;
        return;
    }

    let rapier_context = params
        .rapier_context
        .single()
        .expect("No RapierContext found");
    if let Ok((player_entity, player_transform)) = params.player_query.single()
        && let Ok((camera_global_transform, _camera)) = params.camera_query.single()
    {
        // Ray comes from player's eye level (anchor), NOT camera position
        let anchor_y_offset = 0.87; // Eye level offset
        let ray_origin = player_transform.translation() + Vec3::new(0.0, anchor_y_offset, 0.0);
        let ray_direction = camera_global_transform.forward();

        let filter = QueryFilter::default().exclude_rigid_body(player_entity);

        // Offset ray origin slightly forward to avoid self-collision if filter fails
        let ray_origin = ray_origin + *ray_direction * 0.1;

        if let Some((target_entity, toi)) =
            rapier_context.cast_ray(ray_origin, *ray_direction, 4.0, true, filter)
        {
            // Skip hits that are too close (e.g. self)
            if toi < 0.1 {
                return;
            }

            let hit_point = ray_origin + *ray_direction * toi;

            // Target position based on interaction type

            // Left click: remove block (aim slightly inside the block)
            // Right click: add block (aim slightly outside the block)
            let (world_voxel_pos, target_entity) = if left_click_pressed {
                let p: Vec3 = hit_point + *ray_direction * 0.05;
                (p.floor().as_ivec3(), target_entity)
            } else {
                let p: Vec3 = hit_point - *ray_direction * 0.05;
                (p.floor().as_ivec3(), target_entity)
            };

            if left_click_pressed {
                if mining_progress.target != Some(world_voxel_pos) {
                    mining_progress.target = Some(world_voxel_pos);
                    mining_progress.progress = 0.0;
                }
            }

            if right_click && let Ok(mut inventory) = params.inventory_query.single_mut() {
                let selected_slot = inventory.selected_slot;
                let selected_item = inventory.slots[selected_slot].item_type;

                if selected_item == ItemType::Wheat
                    && let Ok((mob, mut state, material_handle)) =
                        params.mob_query.get_mut(target_entity)
                    && matches!(mob.mob_type, MobType::Cow)
                    && state.state != MobBehavior::Love
                {
                    state.state = MobBehavior::Love;
                    state.timer = 20.0;
                    if let Some(material) = params.materials.get_mut(material_handle) {
                        material.base_color = Color::srgb(1.0, 0.4, 0.4);
                        // Redder
                    }
                    // Consume wheat
                    let slot = &mut inventory.slots[selected_slot];
                    slot.count -= 1;
                    if slot.count == 0 {
                        slot.item_type = ItemType::None;
                    }
                    return; // Don't place block
                }
            }

            let world_pos: Vec3 = world_voxel_pos.as_vec3();
            let chunk_pos = VoxelWorld::world_to_chunk_pos(world_pos);
            let local_voxel_pos = VoxelWorld::voxel_to_local_pos(world_voxel_pos);

            // Important: World generation currently only spawns chunks at Y=0
            // We should use the calculated chunk_pos directly.
            if let Some(&chunk_entity) = params.voxel_world.chunks.get(&chunk_pos)
                && let Ok(mut chunk) = params.chunk_query.get_mut(chunk_entity)
            {
                if left_click_pressed {
                    let voxel = chunk.get_voxel(local_voxel_pos);
                    if voxel != VoxelType::Air && voxel != VoxelType::Bedrock {
                        // Calculate mining speed
                        let hardness = voxel.hardness();
                        let tool_speed = 1.0; // Hand speed for now
                        let multiplier = 1.5; // Correct tool multiplier is usually 1.5

                        // T = (Hardness * multiplier) / Speed
                        let time_to_break = (hardness * multiplier) / tool_speed;

                        if time_to_break <= 0.0 {
                            mining_progress.progress = 1.0;
                        } else {
                            mining_progress.progress += time.delta_secs() / time_to_break;
                        }

                        if mining_progress.progress >= 1.0 {
                            mining_progress.target = None;
                            mining_progress.progress = 0.0;
                            mining_progress.timer = 0.0;

                            chunk.set_voxel(local_voxel_pos, VoxelType::Air);
                            params.commands.entity(chunk_entity).insert(NeedsMeshUpdate);
                            mark_neighbor_chunks(
                                &mut params.commands,
                                &params.voxel_world,
                                chunk_pos,
                                local_voxel_pos,
                            );

                            let drop_item_type = if voxel == VoxelType::TallGrass {
                                use rand::Rng;
                                if rng.gen_bool(0.1) {
                                    ItemType::Wheat
                                } else {
                                    ItemType::None
                                }
                            } else {
                                match voxel {
                                    VoxelType::Grass => ItemType::GrassBlock,
                                    VoxelType::Dirt => ItemType::Dirt,
                                    VoxelType::Stone => ItemType::Stone,
                                    VoxelType::CoalOre => ItemType::CoalOre,
                                    VoxelType::IronOre => ItemType::IronOre,
                                    VoxelType::GoldOre => ItemType::GoldOre,
                                    VoxelType::DiamondOre => ItemType::DiamondOre,
                                    _ => ItemType::None,
                                }
                            };

                            if drop_item_type != ItemType::None {
                                spawn_drop_item(
                                    &mut params.commands,
                                    &params.block_assets,
                                    world_voxel_pos,
                                    drop_item_type,
                                );
                            }

                            if let Some(sound) = block_break_sound(voxel, &params.sound_assets) {
                                play_sound(
                                    &mut params.commands,
                                    sound,
                                    params.settings.master_volume,
                                );
                            }
                        } else {
                            // Play hit sound every 0.25s
                            mining_progress.timer += time.delta_secs();
                            if mining_progress.timer >= 0.25 {
                                mining_progress.timer -= 0.25;
                                use rand::Rng;
                                let index = rng.gen_range(0..4);
                                let sound = match voxel {
                                    VoxelType::Grass | VoxelType::Dirt | VoxelType::TallGrass => {
                                        Some(params.sound_assets.hit_grass[index].clone())
                                    }
                                    _ => Some(params.sound_assets.hit_stone[index].clone()),
                                };
                                if let Some(sound) = sound {
                                    play_sound(
                                        &mut params.commands,
                                        sound,
                                        params.settings.master_volume,
                                    );
                                }
                            }

                            // Render cracking texture
                            let stage = (mining_progress.progress * 10.0).floor() as usize;
                            let stage = stage.min(9);
                            let material = params.block_assets.destroy_stages[stage].clone();

                            params.commands.spawn((
                                Mesh3d(params.block_assets.mesh.clone()),
                                MeshMaterial3d(material),
                                Transform::from_translation(world_voxel_pos.as_vec3() + 0.5)
                                    .with_scale(Vec3::splat(1.001)),
                                crate::world::components::InGameEntity,
                                // Temporary entity that despawns next frame if not updated
                                DespawnMiningEffect,
                            ));
                        }
                    }
                } else if right_click && let Ok(mut inventory) = params.inventory_query.single_mut()
                {
                    let selected_slot = inventory.selected_slot;
                    let selected_item = inventory.slots[selected_slot].item_type;

                    let place_voxel = match selected_item {
                        ItemType::GrassBlock => VoxelType::Grass,
                        ItemType::Dirt => VoxelType::Dirt,
                        ItemType::Stone => VoxelType::Stone,
                        ItemType::CoalOre => VoxelType::CoalOre,
                        ItemType::IronOre => VoxelType::IronOre,
                        ItemType::GoldOre => VoxelType::GoldOre,
                        ItemType::DiamondOre => VoxelType::DiamondOre,
                        _ => VoxelType::Air,
                    };

                    if place_voxel != VoxelType::Air
                        && chunk.get_voxel(local_voxel_pos) == VoxelType::Air
                    {
                        // Prevent placing block inside player
                        if let Ok((_player_entity, player_global_transform)) =
                            params.player_query.single()
                        {
                            let voxel_center = world_voxel_pos.as_vec3() + Vec3::splat(0.5);
                            let player_translation: Vec3 = player_global_transform.translation();

                            // Check distance between voxel center and player center
                            // Player is roughly 1.8 units high, 0.8 units wide (capsule)
                            let player_center = player_translation;
                            let dist = player_center.distance(voxel_center);

                            if dist < 0.8 {
                                return;
                            }
                        }

                        chunk.set_voxel(local_voxel_pos, place_voxel);
                        params.commands.entity(chunk_entity).insert(NeedsMeshUpdate);
                        mark_neighbor_chunks(
                            &mut params.commands,
                            &params.voxel_world,
                            chunk_pos,
                            local_voxel_pos,
                        );
                        play_sound(
                            &mut params.commands,
                            params.sound_assets.place_block.clone(),
                            params.settings.master_volume,
                        );

                        let slot = &mut inventory.slots[selected_slot];
                        if slot.count > 0 {
                            slot.count -= 1;
                            if slot.count == 0 {
                                slot.item_type = ItemType::None;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn spawn_drop_item(
    commands: &mut Commands,
    block_assets: &BlockAssets,
    voxel_pos: IVec3,
    item_type: ItemType,
) {
    let material = match item_type {
        ItemType::GrassBlock => block_assets.grass_side_material.clone(),
        ItemType::Dirt => block_assets.dirt_material.clone(),
        ItemType::Stone => block_assets.stone_material.clone(),
        ItemType::CoalOre => block_assets.coal_ore_material.clone(),
        ItemType::IronOre => block_assets.iron_ore_material.clone(),
        ItemType::GoldOre => block_assets.gold_ore_material.clone(),
        ItemType::DiamondOre => block_assets.diamond_ore_material.clone(),
        ItemType::Wheat => block_assets.wheat_material.clone(),
        _ => block_assets.stone_material.clone(), // Fallback for tools/other items
    };

    let translation = voxel_pos.as_vec3() + Vec3::splat(0.5);
    commands.spawn((
        DropItem {
            item_type,
            velocity: Vec3::ZERO,
        },
        Mesh3d(block_assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(translation).with_scale(Vec3::splat(0.4)),
        GlobalTransform::default(),
        Visibility::Visible,
        crate::world::components::InGameEntity,
    ));
}

fn mark_neighbor_chunks(
    commands: &mut Commands,
    voxel_world: &VoxelWorld,
    chunk_pos: IVec3,
    local_pos: IVec3,
) {
    let mut neighbors = Vec::new();
    if local_pos.x == 0 {
        neighbors.push(IVec3::new(-1, 0, 0));
    } else if local_pos.x == CHUNK_SIZE as i32 - 1 {
        neighbors.push(IVec3::new(1, 0, 0));
    }
    if local_pos.y == 0 {
        neighbors.push(IVec3::new(0, -1, 0));
    } else if local_pos.y == CHUNK_SIZE as i32 - 1 {
        neighbors.push(IVec3::new(0, 1, 0));
    }
    if local_pos.z == 0 {
        neighbors.push(IVec3::new(0, 0, -1));
    } else if local_pos.z == CHUNK_SIZE as i32 - 1 {
        neighbors.push(IVec3::new(0, 0, 1));
    }

    for offset in neighbors {
        if let Some(entity) = voxel_world.chunks.get(&(chunk_pos + offset)) {
            commands.entity(*entity).insert(NeedsMeshUpdate);
        }
    }
}

pub fn update_drop_items(
    time: Res<Time>,
    rapier_context: ReadRapierContext,
    voxel_world: Res<VoxelWorld>,
    chunk_query: Query<&Chunk>,
    mut drops: Query<(&mut Transform, &mut DropItem)>,
) {
    let rapier_context = rapier_context.single().expect("No RapierContext found");
    let dt = time.delta_secs();
    let gravity = 32.0;
    let radius = 0.2;

    for (mut transform, mut drop) in drops.iter_mut() {
        resolve_drop_overlap(&voxel_world, &chunk_query, &mut transform, radius);

        drop.velocity.y -= gravity * dt;

        let mut target_pos = transform.translation + drop.velocity * dt;
        if drop.velocity.y <= 0.0 {
            let ray_origin = transform.translation;
            let ray_dir = -Vec3::Y;
            let max_toi = (transform.translation.y - target_pos.y).max(0.0) + radius;

            if let Some((_entity, toi)) = rapier_context.cast_ray(
                ray_origin,
                ray_dir,
                max_toi,
                true,
                QueryFilter::default().exclude_sensors(),
            ) {
                let hit_y = ray_origin.y - toi + radius;
                if hit_y > target_pos.y {
                    target_pos.y = hit_y;
                    drop.velocity.y = 0.0;
                }
            }
        }

        transform.translation = target_pos;
    }
}

pub fn pickup_drops(
    mut commands: Commands,
    mut inventories: Query<(&GlobalTransform, &mut Inventory), With<PickupDrops>>,
    drops: Query<(Entity, &Transform, &DropItem)>,
    sound_assets: Res<SoundAssets>,
    settings: Res<Settings>,
) {
    let pickup_radius = 1.2;

    for (drop_entity, drop_transform, drop) in drops.iter() {
        for (picker_transform, mut inventory) in inventories.iter_mut() {
            let distance = picker_transform
                .translation()
                .distance(drop_transform.translation);
            if distance > pickup_radius {
                continue;
            }

            if add_to_inventory(&mut inventory, drop.item_type) {
                commands.entity(drop_entity).despawn();
                play_sound(
                    &mut commands,
                    sound_assets.pickup_item.clone(),
                    settings.master_volume,
                );
                break;
            }
        }
    }
}

fn add_to_inventory(inventory: &mut Inventory, item_type: ItemType) -> bool {
    let max_stack = 64;

    for slot in &mut inventory.slots {
        if slot.item_type == item_type && slot.count < max_stack {
            slot.count += 1;
            return true;
        }
    }

    for slot in &mut inventory.slots {
        if slot.item_type == ItemType::None {
            slot.item_type = item_type;
            slot.count = 1;
            return true;
        }
    }

    false
}

// update footsteps params
#[derive(SystemParam)]
pub struct FootstepParams<'w, 's> {
    pub time: Res<'w, Time>,
    pub voxel_world: Res<'w, VoxelWorld>,
    pub chunk_query: Query<'w, 's, &'static Chunk>,
    pub player_query: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static mut FootstepTimer,
            &'static Velocity,
            &'static CharacterController,
        ),
        With<Player>,
    >,
    pub sound_assets: Res<'w, SoundAssets>,
    pub settings: Res<'w, Settings>,
    pub commands: Commands<'w, 's>,
    pub settings_menu:
        Query<'w, 's, &'static Visibility, With<crate::player::settings_menu::SettingsMenu>>,
}

pub fn update_footsteps(mut params: FootstepParams) {
    if let Ok(visibility) = params.settings_menu.single()
        && *visibility != Visibility::Hidden
    {
        return;
    }

    let Ok((transform, mut footstep_timer, velocity, controller)) =
        params.player_query.single_mut()
    else {
        return;
    };

    if !controller.is_grounded {
        return;
    }

    footstep_timer.timer.tick(params.time.delta());

    let moving = velocity.linvel.xz().length_squared() > 0.1;
    if !moving {
        return;
    }

    if !footstep_timer.timer.is_finished() {
        return;
    }

    let voxel_pos = VoxelWorld::world_to_voxel_pos(transform.translation);
    let chunk_pos = VoxelWorld::world_to_chunk_pos(transform.translation);
    let local_pos = VoxelWorld::voxel_to_local_pos(voxel_pos);

    let Some(&chunk_entity) = params.voxel_world.chunks.get(&chunk_pos) else {
        return;
    };
    let Ok(chunk) = params.chunk_query.get(chunk_entity) else {
        return;
    };

    let voxel = chunk.get_voxel(local_pos);

    let step_sound = match voxel {
        VoxelType::Grass => Some(params.sound_assets.step_grass.clone()),
        VoxelType::Dirt => Some(params.sound_assets.step_dirt.clone()),
        VoxelType::Stone
        | VoxelType::CoalOre
        | VoxelType::IronOre
        | VoxelType::GoldOre
        | VoxelType::DiamondOre
        | VoxelType::Bedrock => Some(params.sound_assets.step_stone.clone()),
        _ => Some(params.sound_assets.step_stone.clone()),
    };

    if let Some(sound) = step_sound {
        play_sound(
            &mut params.commands,
            sound,
            params.settings.master_volume * params.settings.footstep_volume,
        );
        footstep_timer.timer.reset();
    }
}

fn play_sound(commands: &mut Commands, sound: Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(sound),
        PlaybackSettings::DESPAWN
            .with_spatial(false)
            .with_volume(Volume::Linear(volume)),
        Transform::default(),
        GlobalTransform::default(),
        crate::world::components::InGameEntity,
    ));
}

fn block_break_sound(voxel: VoxelType, sound_assets: &SoundAssets) -> Option<Handle<AudioSource>> {
    match voxel {
        VoxelType::Grass | VoxelType::Dirt | VoxelType::TallGrass => {
            Some(sound_assets.break_grass.clone())
        }
        VoxelType::Stone
        | VoxelType::CoalOre
        | VoxelType::IronOre
        | VoxelType::GoldOre
        | VoxelType::DiamondOre
        | VoxelType::Bedrock => Some(sound_assets.break_stone.clone()),
        VoxelType::Air => None,
    }
}

fn resolve_drop_overlap(
    voxel_world: &VoxelWorld,
    chunk_query: &Query<&Chunk>,
    transform: &mut Transform,
    radius: f32,
) {
    for _ in 0..4 {
        let voxel_pos = VoxelWorld::world_to_voxel_pos(transform.translation);
        let chunk_pos = VoxelWorld::world_to_chunk_pos(transform.translation);
        let local_pos = VoxelWorld::voxel_to_local_pos(voxel_pos);

        let voxel = voxel_world
            .chunks
            .get(&chunk_pos)
            .and_then(|entity| chunk_query.get(*entity).ok())
            .map(|chunk| chunk.get_voxel(local_pos))
            .unwrap_or(VoxelType::Air);

        if voxel == VoxelType::Air {
            break;
        }

        transform.translation.y += radius * 2.0;
    }
}




type PlayerLookCameraQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static CameraController),
    (Without<CharacterController>, With<Camera>),
>;

pub fn player_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cam_query: PlayerLookCameraQuery,
    mut player_query: Query<&mut Transform, (With<CharacterController>, Without<Camera>)>,
    settings_menu: Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
    command_state: Res<crate::player::inventory_ui::CommandState>,
) {
    if command_state.open {
        return;
    }

    if let Ok(visibility) = settings_menu.single()
        && *visibility != Visibility::Hidden
    {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    if let Ok((mut cam_transform, controller)) = cam_query.single_mut()
        && let Ok(mut player_transform) = player_query.single_mut()
    {
        player_transform.rotate_y(-delta.x * controller.sensitivity * 0.01);

        let mut pitch = cam_transform.rotation.to_euler(EulerRot::YXZ).1;
        pitch -= delta.y * controller.sensitivity * 0.01;
        pitch = pitch.clamp(-1.54, 1.54);

        cam_transform.rotation = Quat::from_rotation_x(pitch);
    }
}
