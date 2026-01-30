use crate::mob::components::{Mob, MobBehavior, MobSpawner, MobState, MobType};
use crate::player::components::{Health, Player};
use crate::player::inventory_ui::KillEvent;
use crate::world::components::{GameTime, InGameEntity};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{Rng, thread_rng};

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

pub fn mob_spawner_system(
    time: Res<Time>,
    mut spawner: ResMut<MobSpawner>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mob_query: Query<Entity, With<Mob>>,
    player_query: Query<&Transform, With<Player>>,
    game_time: Res<GameTime>,
) {
    spawner.timer += time.delta_secs();

    if spawner.timer >= spawner.spawn_interval {
        spawner.timer = 0.0;

        let current_mobs = mob_query.iter().count();
        if current_mobs < spawner.global_cap {
            let player_pos = if let Ok(t) = player_query.single() {
                t.translation
            } else {
                Vec3::ZERO
            };

            let mut rng = thread_rng();
            let angle_adjusted = (game_time.time - 0.25) * std::f32::consts::TAU;
            let is_night = angle_adjusted.sin() <= 0.0;

            let spawn_dist = rng.gen_range(20.0..40.0);
            let spawn_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let spawn_x = player_pos.x + spawn_angle.cos() * spawn_dist;
            let spawn_z = player_pos.z + spawn_angle.sin() * spawn_dist;
            let spawn_y = get_terrain_height(spawn_x, spawn_z);
            let position = Vec3::new(spawn_x, spawn_y, spawn_z);

            if is_night {
                if rng.gen_bool(0.9) {
                    spawn_mob_typed(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        position,
                        MobType::Slime,
                    );
                }
            }
            // Cows no longer spawn via timer system, only on chunk generation.
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
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>,
) {
    let mut rng = thread_rng();
    let (player_entity, player_transform, mut player_health) =
        if let Ok(p) = player_query.single_mut() {
            p
        } else {
            return;
        };

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
                    // Check if player is close for slimes to attack
                    if matches!(mob.mob_type, MobType::Slime)
                        && transform.translation.distance(player_transform.translation) < 10.0
                    {
                        state.state = MobBehavior::Attacking;
                        state.target_entity = Some(player_entity);
                        state.timer = 10.0; // Stay in attack mode for a bit
                    } else {
                        state.state = MobBehavior::Wandering;
                        state.timer = rng.gen_range(2.0..5.0);
                        mob.wander_timer = rng.gen_range(1.0..3.0);
                    }
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

                let dist = transform.translation.distance(player_transform.translation);

                // If target is player and player is far, stop attacking
                if state.target_entity == Some(player_entity) && dist > 15.0 {
                    state.state = MobBehavior::Idle;
                    state.timer = rng.gen_range(1.0..2.0);
                    state.target_entity = None;
                }

                // Deal damage if close enough
                if matches!(mob.mob_type, MobType::Slime)
                    && state.target_entity == Some(player_entity)
                    && dist < 1.2
                    && mob.attack_cooldown <= 0.0
                {
                    player_health.current = player_health.current.saturating_sub(1);
                    mob.attack_cooldown = 1.0; // 1 second cooldown
                    println!("Slime attacked player! Health: {}", player_health.current);
                }

                if state.timer <= 0.0 {
                    state.state = MobBehavior::Idle;
                    state.timer = rng.gen_range(1.0..2.0);
                    state.target_entity = None;
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
        spawn_mob_typed(
            &mut commands,
            &mut meshes,
            &mut materials,
            pos,
            MobType::Cow,
        );
    }
}

pub fn mob_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut mob_query: Query<(Entity, &Mob, &MobState, &Transform, &mut Velocity)>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Mob>)>,
    settings: Res<crate::player::settings_menu::Settings>,
    rapier_context: ReadRapierContext,
) {
    let player_data = player_query.iter().next();
    let player_transform = player_data.map(|(_, t)| t);
    let despawn_dist = (settings.render_distance as f32 + 4.0) * 16.0;
    let despawn_dist_sq = despawn_dist * despawn_dist;

    let rapier_context = rapier_context.single().unwrap();

    let mob_transforms: Vec<(Entity, Vec3)> = mob_query
        .iter()
        .map(|(e, _, _, t, _)| (e, t.translation))
        .collect();

    for (entity, mob, state, transform, mut velocity) in mob_query.iter_mut() {
        // Distance-based despawning
        if let Some(player_t) = player_transform {
            if transform.translation.distance_squared(player_t.translation) > despawn_dist_sq {
                commands.entity(entity).despawn();
                continue;
            }
        }

        let mut move_dir = Vec3::ZERO;
        let mut should_jump = false;

        if (state.state == MobBehavior::Love || state.state == MobBehavior::Attacking)
            && let Some(target_entity) = state.target_entity
        {
            let target_pos = if Some(target_entity) == player_data.map(|(e, _)| e) {
                player_transform.map(|t| t.translation)
            } else {
                // Check if target is another mob
                mob_transforms
                    .iter()
                    .find(|(e, _)| *e == target_entity)
                    .map(|(_, p)| *p)
            };

            if let Some(pos) = target_pos {
                move_dir = (pos - transform.translation).normalize_or_zero();
            }
        } else if mob.wander_timer > 0.0 {
            move_dir = Vec3::new(
                (time.elapsed_secs() * 0.5).sin(),
                0.0,
                (time.elapsed_secs() * 0.3).cos(),
            )
            .normalize();
        }

        if move_dir != Vec3::ZERO {
            velocity.linvel.x = move_dir.x * mob.max_speed;
            velocity.linvel.z = move_dir.z * mob.max_speed;

            // Check if there's an obstacle in front to jump over
            let ray_pos = transform.translation + Vec3::Y * 0.1;
            let ray_dir = move_dir;
            let max_toi = 1.0;

            if let Some((_hit_entity, _toi)) = rapier_context.cast_ray(
                ray_pos,
                ray_dir,
                max_toi,
                true,
                QueryFilter::new().exclude_collider(entity),
            ) {
                // Obstacle detected, check if we're grounded before jumping
                if velocity.linvel.y.abs() < 0.1 {
                    should_jump = true;
                }
            } else {
                // Also check a bit higher up to see if it's a 1-block wall
                let ray_pos_high = transform.translation + Vec3::Y * 0.6;
                if let Some((_hit_entity, _toi)) = rapier_context.cast_ray(
                    ray_pos_high,
                    ray_dir,
                    max_toi,
                    true,
                    QueryFilter::new().exclude_collider(entity),
                ) {
                    if velocity.linvel.y.abs() < 0.1 {
                        should_jump = true;
                    }
                }
            }
        } else {
            velocity.linvel.x *= 0.9;
            velocity.linvel.z *= 0.9;
        }

        if should_jump {
            velocity.linvel.y = 9.6;
        }
    }
}

pub fn handle_kill_events(
    mut commands: Commands,
    mut kill_events: MessageReader<KillEvent>,
    mob_query: Query<(Entity, &Mob, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.iter().next();

    for event in kill_events.read() {
        let selector = &event.selector;

        let target_type = if selector.contains("type=cow") {
            Some(MobType::Cow)
        } else if selector.contains("type=slime") {
            Some(MobType::Slime)
        } else {
            None
        };

        let limit = if let Some(pos) = selector.find("limit=") {
            let rest = &selector[pos + 6..];
            let end = rest.find(|c: char| !c.is_numeric()).unwrap_or(rest.len());
            rest[..end].parse::<usize>().ok()
        } else {
            None
        };

        let distance = if let Some(pos) = selector.find("distance=") {
            let rest = &selector[pos + 9..];
            let end = rest
                .find(|c: char| !c.is_numeric() && c != '.')
                .unwrap_or(rest.len());
            rest[..end].parse::<f32>().ok()
        } else {
            None
        };

        let mut targets: Vec<(Entity, f32)> = mob_query
            .iter()
            .filter(|(_, mob, transform)| {
                let type_match = target_type.is_none() || mob.mob_type == target_type.unwrap();
                let dist_match = if let (Some(max_d), Some(pt)) = (distance, player_transform) {
                    transform.translation.distance(pt.translation) <= max_d
                } else {
                    true
                };
                type_match && dist_match
            })
            .map(|(e, _, t)| {
                let d = player_transform
                    .map(|pt| t.translation.distance_squared(pt.translation))
                    .unwrap_or(0.0);
                (e, d)
            })
            .collect();

        // Sort by distance if limit is set
        if limit.is_some() {
            targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        }

        let to_kill = if let Some(l) = limit {
            targets
                .into_iter()
                .take(l)
                .map(|(e, _)| e)
                .collect::<Vec<_>>()
        } else {
            targets.into_iter().map(|(e, _)| e).collect::<Vec<_>>()
        };

        let count = to_kill.len();
        for entity in to_kill {
            commands.entity(entity).despawn();
        }

        if count > 0 {
            println!("[System] Killed {} entities matching {}", count, selector);
        } else {
            println!("[System] No entities found matching {}", selector);
        }
    }
}
