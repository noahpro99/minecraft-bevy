use crate::main_menu::WorldSettings;
use crate::mob::components::{Mob, MobBehavior, MobSpawner, MobState, MobType};
use crate::player::components::{Health, Player};
use crate::world::components::{GameTime, InGameEntity};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};

fn get_terrain_height(x: f32, z: f32) -> f32 {
    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;
    // Simple approximation matching world generation
    let wave = (x * frequency).sin() + (z * frequency).cos();
    let height = (base_height + wave * amplitude * 0.5).round();
    height + 1.0 // spawn on surface
}

pub fn spawn_mob_typed(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    mob_type: MobType,
) {
    let (size, color, health) = match mob_type {
        MobType::Cow => (Vec3::splat(0.6), Color::srgb(0.8, 0.6, 0.4), 10), // brown
        MobType::Slime => (Vec3::splat(0.4), Color::srgb(0.2, 0.8, 0.2), 5), // green
    };

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x * 2.0, size.y * 2.0, size.z * 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })),
        Transform::from_translation(position),
        Mob {
            mob_type,
            max_speed: if matches!(mob_type, MobType::Slime) {
                1.5
            } else {
                2.0
            },
            wander_timer: 0.0,
            attack_cooldown: 0.0,
        },
        Health {
            current: health,
            max: health,
        },
        MobState::default(),
        RigidBody::Dynamic,
        Collider::cuboid(size.x, size.y, size.z),
        Velocity::default(),
        Friction {
            coefficient: 0.7,
            combine_rule: CoefficientCombineRule::Min,
        },
        Restitution {
            coefficient: 0.1,
            combine_rule: CoefficientCombineRule::Min,
        },
        LockedAxes::ROTATION_LOCKED,
        InGameEntity,
    ));
}

pub fn spawn_mob(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let mut rng = thread_rng();
    let mob_type = if rng.gen_bool(0.5) {
        MobType::Cow
    } else {
        MobType::Slime
    };
    spawn_mob_typed(commands, meshes, materials, position, mob_type);
}

pub fn mob_spawner_system(
    time: Res<Time>,
    mut spawner: ResMut<MobSpawner>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mob_query: Query<Entity, With<Mob>>,
    player_query: Query<&Transform, With<Player>>,
    game_time: Res<GameTime>,
    world_settings: Res<WorldSettings>,
) {
    spawner.timer += time.delta_secs();

    if spawner.timer >= spawner.spawn_interval {
        spawner.timer = 0.0;

        let current_mobs = mob_query.iter().count();
        if current_mobs < spawner.max_mobs {
            let player_pos = if let Ok(t) = player_query.single() {
                t.translation
            } else {
                Vec3::ZERO
            };

            let mut rng = thread_rng();
            let angle = (game_time.time - 0.25) * std::f32::consts::TAU;
            let is_night = angle.sin() <= 0.0;

            // Try to spawn a mob at a distance from player
            let spawn_dist = rng.gen_range(20.0..40.0);
            let spawn_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let spawn_x = player_pos.x + spawn_angle.cos() * spawn_dist;
            let spawn_z = player_pos.z + spawn_angle.sin() * spawn_dist;
            let spawn_y = get_terrain_height(spawn_x, spawn_z);
            let position = Vec3::new(spawn_x, spawn_y, spawn_z);

            if is_night && rng.gen_bool(0.9) {
                spawn_mob_typed(&mut commands, &mut meshes, &mut materials, position, MobType::Slime);
            } else {
                let freq = 0.05;
                let seed = world_settings.seed as f32;
                let hotspot = ((spawn_x * freq + seed).sin() + (spawn_z * freq + seed).cos()) > 0.4;

                if hotspot {
                    for _ in 0..rng.gen_range(5..12) {
                        let offset = Vec3::new(rng.gen_range(-5.0..5.0), 0.0, rng.gen_range(-5.0..5.0));
                        let p = position + offset;
                        let py = get_terrain_height(p.x, p.z);
                        spawn_mob_typed(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Vec3::new(p.x, py, p.z),
                            MobType::Cow,
                        );
                    }
                } else if rng.gen_bool(0.4) {
                    spawn_mob_typed(&mut commands, &mut meshes, &mut materials, position, MobType::Cow);
                }
            }
        }
    }
}

pub fn mob_behavior_system(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mob_query: Query<(
        Entity,
        &mut Mob,
        &mut MobState,
        &Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let mut rng = thread_rng();

    let love_mobs: Vec<(Entity, Vec3)> = mob_query
        .iter()
        .filter(|(_, mob, state, _, _)| {
            matches!(mob.mob_type, MobType::Cow) && state.state == MobBehavior::Love
        })
        .map(|(e, _, _, t, _)| (e, t.translation))
        .collect();

    let mut to_spawn = Vec::new();

    for (entity, mut mob, mut state, transform, material_handle) in mob_query.iter_mut() {
        state.timer -= time.delta_secs();

        match state.state {
            MobBehavior::Idle => {
                if state.timer <= 0.0 {
                    state.state = MobBehavior::Wandering;
                    state.timer = rng.gen_range(2.0..5.0);
                    mob.wander_timer = rng.gen_range(1.0..3.0);
                }
            }
            MobBehavior::Wandering => {
                mob.wander_timer -= time.delta_secs();

                if mob.wander_timer <= 0.0 {
                    state.state = MobBehavior::Idle;
                    state.timer = rng.gen_range(1.0..3.0);
                }
            }
            MobBehavior::Attacking => {
                mob.attack_cooldown -= time.delta_secs();

                if mob.attack_cooldown <= 0.0 {
                    state.state = MobBehavior::Idle;
                    state.timer = rng.gen_range(1.0..2.0);
                }
            }
            MobBehavior::Love => {
                if state.timer <= 0.0 {
                    state.state = MobBehavior::Idle;
                    state.timer = rng.gen_range(1.0..3.0);
                    state.target_entity = None;
                    state.contact_timer = 0.0;
                    if let Some(material) = materials.get_mut(material_handle) {
                        material.base_color = Color::srgb(0.8, 0.6, 0.4); // Reset to brown
                    }
                    continue;
                }

                // Look for another cow in love
                if state.target_entity.is_none() {
                    let my_pos = transform.translation;
                    let closest = love_mobs.iter().filter(|(e, _)| *e != entity).min_by(
                        |(_, p1), (_, p2)| {
                            my_pos
                                .distance_squared(*p1)
                                .partial_cmp(&my_pos.distance_squared(*p2))
                                .unwrap()
                        },
                    );

                    if let Some((target_e, _)) = closest {
                        state.target_entity = Some(*target_e);
                    }
                }

                // Check for proximity over time
                if let Some(target_e) = state.target_entity {
                    if let Some((_, target_pos)) = love_mobs.iter().find(|(e, _)| *e == target_e) {
                        if transform.translation.distance(*target_pos) < 2.0 {
                            state.contact_timer += time.delta_secs();

                            if state.contact_timer >= 1.5 {
                                // Birth!
                                state.state = MobBehavior::Parent;
                                state.timer = 10.0; // Break from love
                                state.target_entity = None;
                                state.contact_timer = 0.0;
                                if let Some(material) = materials.get_mut(material_handle) {
                                    material.base_color = Color::srgb(0.8, 0.6, 0.4);
                                }

                                // Only one of the pair should spawn the baby
                                if entity < target_e {
                                    to_spawn.push(*target_pos);
                                }
                            }
                        } else {
                            state.contact_timer =
                                (state.contact_timer - time.delta_secs() * 0.5).max(0.0);
                        }
                    } else {
                        state.target_entity = None; // Target lost (no longer in love?)
                        state.contact_timer = 0.0;
                    }
                }
            }
            MobBehavior::Parent => {
                if state.timer <= 0.0 {
                    state.state = MobBehavior::Idle;
                    state.timer = rng.gen_range(1.0..3.0);
                }
            }
        }
    }

    for pos in to_spawn {
        spawn_mob_typed(&mut commands, &mut meshes, &mut materials, pos, MobType::Cow);
    }
}

pub fn mob_movement_system(
    time: Res<Time>,
    mut mob_query: Query<(&Mob, &MobState, &Transform, &mut Velocity)>,
    target_query: Query<&Transform>,
) {
    for (mob, state, transform, mut velocity) in mob_query.iter_mut() {
        if state.state == MobBehavior::Love
            && let Some(target_e) = state.target_entity
            && let Ok(target_transform) = target_query.get(target_e)
        {
            let direction =
                (target_transform.translation - transform.translation).normalize_or_zero();
            velocity.linvel = direction * mob.max_speed;
            continue;
        }

        // Simple wandering movement using physics
        if mob.wander_timer > 0.0 {
            let direction = Vec3::new(
                (time.elapsed_secs() * 0.5).sin(),
                0.0,
                (time.elapsed_secs() * 0.3).cos(),
            )
            .normalize();

            velocity.linvel = direction * mob.max_speed;
        } else {
            velocity.linvel.x *= 0.9; // friction
            velocity.linvel.z *= 0.9;
        }
    }
}
