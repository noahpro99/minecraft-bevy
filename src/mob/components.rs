use bevy::prelude::*;

#[derive(Component)]
pub struct Mob {
    pub mob_type: MobType,
    pub max_speed: f32,
    pub wander_timer: f32,
    pub attack_cooldown: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MobType {
    #[default]
    Cow,
    Slime,
}

#[derive(Component)]
pub struct MobState {
    pub state: MobBehavior,
    pub timer: f32,
    pub target_entity: Option<Entity>,
    pub contact_timer: f32, // Time spent close to target
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MobBehavior {
    Idle,
    Wandering,
    Love,
    Parent, // Just gave birth
    Attacking,
}

impl Default for MobState {
    fn default() -> Self {
        Self {
            state: MobBehavior::Idle,
            timer: 0.0,
            target_entity: None,
            contact_timer: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct MobSpawner {
    pub timer: f32,
    pub spawn_interval: f32,
    pub global_cap: usize,
}

impl Default for MobSpawner {
    fn default() -> Self {
        Self {
            timer: 0.0,
            spawn_interval: 1.0,
            global_cap: 100,
        }
    }
}
