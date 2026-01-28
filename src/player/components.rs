use crate::world::components::ItemType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct Hunger {
    pub current: i32,
    pub max: i32,
    pub timer: f32,
    pub damage_timer: f32,
}

#[derive(Component)]
pub struct FootstepTimer {
    pub timer: Timer,
}

impl Default for FootstepTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct PickupDrops;

#[derive(Component)]
pub struct CameraController {
    pub sensitivity: f32,
}

#[derive(Component)]
pub struct CharacterController {
    pub speed: f32,
    pub jump_force: f32,
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub was_grounded: bool,
    pub fall_start_y: f32,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            speed: 4.317,
            jump_force: 9.6,
            velocity: Vec3::ZERO,
            is_grounded: false,
            was_grounded: false,
            fall_start_y: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item_type: ItemType,
    pub count: u32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub slots: [InventorySlot; 10],
    pub selected_slot: usize,
}

#[derive(Component)]
pub struct InventorySlotIcon(pub usize);

impl Default for Inventory {
    fn default() -> Self {
        let slots = [InventorySlot {
            item_type: ItemType::None,
            count: 0,
        }; 10];
        Self {
            slots,
            selected_slot: 0,
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 20,
            max: 20,
        }
    }
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            current: 20,
            max: 20,
            timer: 0.0,
            damage_timer: 0.0,
        }
    }
}
