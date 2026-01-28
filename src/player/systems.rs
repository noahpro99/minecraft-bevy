use crate::player::components::{
    CameraController, CharacterController, Inventory, PickupDrops, Player,
};
use crate::world::components::{Chunk, DropItem, NeedsMeshUpdate, VoxelType, CHUNK_SIZE};
use crate::world::resources::VoxelWorld;
use crate::world::systems::{BlockAssets, InitialChunkMeshing};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            CharacterController::default(),
            Inventory::default(),
            PickupDrops,
            Transform::from_xyz(0.0, spawn_height(), 0.0),
            GlobalTransform::default(),
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::cuboid(0.3, 0.9, 0.3),
            Friction::coefficient(0.0),
            LockedAxes::ROTATION_LOCKED,
            Ccd::enabled(),
            TransformInterpolation::default(),
            Velocity::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
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
            ));
        });
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
    mut commands: Commands,
    meshing: Res<InitialChunkMeshing>,
    players: Query<Entity, With<Player>>,
    voxel_world: Res<VoxelWorld>,
    chunk_colliders: Query<(), With<Collider>>,
) {
    if meshing.0 {
        return;
    }
    if !players.is_empty() {
        return;
    }

    let spawn_pos = Vec3::new(0.0, spawn_height(), 0.0);
    let spawn_chunk_pos = VoxelWorld::world_to_chunk_pos(spawn_pos);
    let has_chunk = voxel_world
        .chunks
        .get(&spawn_chunk_pos)
        .map(|entity| chunk_colliders.contains(*entity))
        .unwrap_or(false);
    if !has_chunk {
        return;
    }

    spawn_player(commands);
}

pub fn player_look(
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut cam_query: Query<
        (&mut Transform, &CameraController),
        (Without<CharacterController>, With<Camera>),
    >,
    mut player_query: Query<&mut Transform, (With<CharacterController>, Without<Camera>)>,
    settings_menu: Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
    command_state: Res<crate::player::inventory_ui::CommandState>,
) {
    if command_state.open {
        return;
    }

    if let Ok(visibility) = settings_menu.single() {
        if *visibility != Visibility::Hidden {
            return;
        }
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    if let Ok((mut cam_transform, controller)) = cam_query.single_mut() {
        if let Ok(mut player_transform) = player_query.single_mut() {
            player_transform.rotate_y(-delta.x * controller.sensitivity * 0.01);

            let mut pitch = cam_transform.rotation.to_euler(EulerRot::YXZ).1;
            pitch -= delta.y * controller.sensitivity * 0.01;
            pitch = pitch.clamp(-1.54, 1.54);

            cam_transform.rotation = Quat::from_rotation_x(pitch);
        }
    }
}

pub fn player_move(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    mut query: Query<(Entity, &Transform, &mut CharacterController, &mut Velocity)>,
    rapier_context: ReadRapierContext,
    settings_menu: Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
    command_state: Res<crate::player::inventory_ui::CommandState>,
) {
    if command_state.open {
        return;
    }

    if let Ok(visibility) = settings_menu.single() {
        if *visibility != Visibility::Hidden {
            return;
        }
    }

    if let Ok((entity, transform, mut controller, mut velocity)) = query.single_mut() {
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

        // Ground check using Rapier raycast
        let ray_pos = transform.translation;
        let ray_dir = -Vec3::Y;
        let max_toi = 1.05;

        controller.is_grounded = false;
        if let Some((_entity, toi)) = rapier_context.cast_ray(
            ray_pos,
            ray_dir,
            max_toi,
            true,
            QueryFilter::default().exclude_rigid_body(entity),
        ) {
            if toi < max_toi {
                controller.is_grounded = true;
            }
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

pub fn player_interact(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    rapier_context: ReadRapierContext,
    mut chunk_query: Query<&mut Chunk>,
    voxel_world: Res<VoxelWorld>,
    block_assets: Res<BlockAssets>,
    mut inventory_query: Query<&mut Inventory, With<Player>>,
    player_query: Query<(Entity, &GlobalTransform), With<Player>>,
    settings_menu: Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
) {
    if let Ok(visibility) = settings_menu.single() {
        if *visibility != Visibility::Hidden {
            return;
        }
    }

    let right_click = mouse_input.just_pressed(MouseButton::Right);
    let left_click = mouse_input.just_pressed(MouseButton::Left);

    if !right_click && !left_click {
        return;
    }

    let rapier_context = rapier_context.single().expect("No RapierContext found");

    if let Ok((camera_global_transform, _camera)) = camera_query.single() {
        let ray_origin: Vec3 = camera_global_transform.translation();
        let ray_direction: Dir3 = camera_global_transform.forward();

        let filter = if let Ok((player_entity, _)) = player_query.single() {
            QueryFilter::default()
                .exclude_rigid_body(player_entity)
                .exclude_sensors()
        } else {
            QueryFilter::default().exclude_sensors()
        };

        // Offset ray origin slightly forward to avoid self-collision if filter fails
        let ray_origin = ray_origin + *ray_direction * 0.1;

        if let Some((_entity, toi)) =
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
            let world_voxel_pos = if left_click {
                let p: Vec3 = hit_point + *ray_direction * 0.05;
                p.floor().as_ivec3()
            } else {
                let p: Vec3 = hit_point - *ray_direction * 0.05;
                p.floor().as_ivec3()
            };

            let chunk_pos = VoxelWorld::world_to_chunk_pos(world_voxel_pos.as_vec3());
            let local_voxel_pos = VoxelWorld::voxel_to_local_pos(world_voxel_pos);

            // Important: World generation currently only spawns chunks at Y=0
            // We should use the calculated chunk_pos directly.
            if let Some(&chunk_entity) = voxel_world.chunks.get(&chunk_pos) {
                if let Ok(mut chunk) = chunk_query.get_mut(chunk_entity) {
                    if left_click {
                        let voxel = chunk.get_voxel(local_voxel_pos);
                        if voxel != VoxelType::Air {
                            chunk.set_voxel(local_voxel_pos, VoxelType::Air);
                            commands.entity(chunk_entity).insert(NeedsMeshUpdate);
                            mark_neighbor_chunks(
                                &mut commands,
                                &voxel_world,
                                chunk_pos,
                                local_voxel_pos,
                            );
                            spawn_drop_item(&mut commands, &block_assets, world_voxel_pos, voxel);
                        }
                    } else if right_click {
                        if let Ok(mut inventory) = inventory_query.single_mut() {
                            let selected_slot = inventory.selected_slot;
                            let selected_item = inventory.slots[selected_slot].voxel_type;
                            if selected_item != VoxelType::Air
                                && chunk.get_voxel(local_voxel_pos) == VoxelType::Air
                            {
                                // Prevent placing block inside player
                                if let Ok((_player_entity, player_global_transform)) =
                                    player_query.single()
                                {
                                    let voxel_center = world_voxel_pos.as_vec3() + Vec3::splat(0.5);
                                    let player_translation: Vec3 =
                                        player_global_transform.translation();

                                    // Check distance between voxel center and player center
                                    // Player is roughly 1.8 units high, 0.8 units wide (capsule)
                                    let player_center = player_translation;
                                    let dist = player_center.distance(voxel_center);

                                    if dist < 0.8 {
                                        return;
                                    }
                                }

                                chunk.set_voxel(local_voxel_pos, selected_item);
                                commands.entity(chunk_entity).insert(NeedsMeshUpdate);
                                mark_neighbor_chunks(
                                    &mut commands,
                                    &voxel_world,
                                    chunk_pos,
                                    local_voxel_pos,
                                );

                                let slot = &mut inventory.slots[selected_slot];
                                if slot.count > 0 {
                                    slot.count -= 1;
                                    if slot.count == 0 {
                                        slot.voxel_type = VoxelType::Air;
                                    }
                                }
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
    voxel_type: VoxelType,
) {
    let material = match voxel_type {
        VoxelType::Grass => block_assets.grass_material.clone(),
        VoxelType::Dirt => block_assets.dirt_material.clone(),
        VoxelType::Stone => block_assets.stone_material.clone(),
        VoxelType::Glowstone => block_assets.glowstone_material.clone(),
        VoxelType::Air => return,
    };

    let translation = voxel_pos.as_vec3() + Vec3::splat(0.5);
    commands.spawn((
        DropItem {
            voxel_type,
            velocity: Vec3::ZERO,
        },
        Mesh3d(block_assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(translation).with_scale(Vec3::splat(0.4)),
        GlobalTransform::default(),
        Visibility::Visible,
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

            if add_to_inventory(&mut inventory, drop.voxel_type) {
                commands.entity(drop_entity).despawn();
                break;
            }
        }
    }
}

fn add_to_inventory(inventory: &mut Inventory, voxel_type: VoxelType) -> bool {
    let max_stack = 64;

    for slot in &mut inventory.slots {
        if slot.voxel_type == voxel_type && slot.count < max_stack {
            slot.count += 1;
            return true;
        }
    }

    for slot in &mut inventory.slots {
        if slot.voxel_type == VoxelType::Air {
            slot.voxel_type = voxel_type;
            slot.count = 1;
            return true;
        }
    }

    false
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
