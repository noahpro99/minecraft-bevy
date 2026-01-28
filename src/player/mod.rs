use bevy::prelude::*;

pub mod components;
pub mod inventory_ui;
pub mod resources;
pub mod settings_menu;
pub mod systems;

use inventory_ui::*;
use resources::*;
use settings_menu::*;
use systems::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>()
            .init_resource::<SoundAssets>()
            .add_systems(
                OnEnter(crate::main_menu::AppState::InGame),
                (setup_ui, load_sound_assets),
            )
            .init_resource::<CommandState>()
            .add_systems(
                Update,
                spawn_player_when_ready.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                player_look.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                player_interact.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                player_inventory_control.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                update_inventory_ui.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                update_health_ui.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                update_hunger_ui.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                (handle_command_input,).run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                pickup_drops.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                toggle_settings_menu.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_fov_buttons.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_render_distance_buttons.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_master_volume_buttons.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_footstep_volume_buttons.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_resume_button.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                handle_quit_button.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                update_sprint_fov.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                Update,
                update_footsteps.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                player_move.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                update_drop_items.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                update_hunger.run_if(in_state(crate::main_menu::AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                handle_player_death.run_if(in_state(crate::main_menu::AppState::InGame)),
            );
    }
}
