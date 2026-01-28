use crate::mob::components::{Mob, MobBehavior, MobSpawner, MobState, MobType};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};

fn get_terrain_height(x: f32, _z: f32) -> f32 {
    let base_height = 14.0;
    let amplitude = 8.0;
    let frequency = 0.04;
    let wave = (x * frequency).sin()
        + (x * frequency).cos()
        + ((x * frequency * 0.5).sin() * 0.5)
        + ((x * frequency * 0.5).cos() * 0.5);
    let height = (base_height + wave * amplitude * 0.5).round();
    height + 1.0 // spawn on surface
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

    let (size, color) = match mob_type {
        MobType::Cow => (Vec3::splat(0.6), Color::srgb(0.8, 0.6, 0.4)), // brown
        MobType::Slime => (Vec3::splat(0.4), Color::srgb(0.2, 0.8, 0.2)), // green
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
            health: if matches!(mob_type, MobType::Cow) {
                10
            } else {
                5
            },
            max_speed: 2.0,
            wander_timer: 0.0,
            attack_cooldown: 0.0,
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
    ));
}

pub fn mob_spawner_system(
    time: Res<Time>,
    mut spawner: ResMut<MobSpawner>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mob_query: Query<Entity, With<Mob>>,
) {
    spawner.timer += time.delta_secs();

    if spawner.timer >= spawner.spawn_interval {
        spawner.timer = 0.0;

        let current_mobs = mob_query.iter().count();
        if current_mobs < spawner.max_mobs {
            let mut rng = thread_rng();
            let x = rng.gen_range(-20.0..20.0);
            let z = rng.gen_range(-20.0..20.0);
            let y = get_terrain_height(x, z);
            let position = Vec3::new(x, y, z);

            spawn_mob(&mut commands, &mut meshes, &mut materials, position);
        }
    }
}

pub fn mob_behavior_system(
    time: Res<Time>,
    mut mob_query: Query<(&mut Mob, &mut MobState, &Transform)>,
) {
    let mut rng = thread_rng();

    for (mut mob, mut state, _transform) in mob_query.iter_mut() {
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
        }
    }
}

pub fn mob_movement_system(time: Res<Time>, mut mob_query: Query<(&Mob, &mut Velocity)>) {
    for (mob, mut velocity) in mob_query.iter_mut() {
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
