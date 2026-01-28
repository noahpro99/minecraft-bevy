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
            .add_systems(Startup, (setup_ui, load_sound_assets))
            .init_resource::<CommandState>()
            .add_systems(
                Update,
                (
                    spawn_player_when_ready,
                    player_look,
                    player_interact,
                    player_inventory_control,
                    update_inventory_ui,
                    update_health_ui,
                    update_hunger_ui,
                    handle_command_input,
                    pickup_drops,
                    toggle_settings_menu,
                    handle_fov_buttons,
                    handle_render_distance_buttons,
                    handle_master_volume_buttons,
                    handle_footstep_volume_buttons,
                    handle_resume_button,
                    update_sprint_fov,
                    update_footsteps,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    player_move,
                    update_drop_items,
                    update_hunger,
                    handle_player_death,
                ),
            );
    }
}
