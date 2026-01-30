mod main_menu;
mod mob;
mod player;
mod world;

use crate::main_menu::MainMenuPlugin;
use crate::player::PlayerPlugin;
use crate::world::WorldPlugin;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PresentMode};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "EXPLR".into(),
                resolution: (1280, 720).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_asset::<AudioSource>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
        .insert_resource(TimestepMode::Interpolated {
            dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 8,
        })
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(MainMenuPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(crate::mob::MobPlugin)
        .add_systems(Startup, configure_rapier)
        .add_systems(Update, grab_cursor)
        .run();
}

fn configure_rapier(
    mut configuration: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    if let Ok(mut configuration) = configuration.single_mut() {
        configuration.gravity = Vec3::new(0.0, -32.0, 0.0);
    }
}

fn grab_cursor(
    mut window_query: Query<(Entity, &mut Window, &mut CursorOptions)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    settings_menu: Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
    app_state: Res<State<crate::main_menu::AppState>>,
) {
    if *app_state.get() != crate::main_menu::AppState::InGame {
        if let Ok((_entity, mut _window, mut cursor)) = window_query.single_mut() {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
        return;
    }

    if let Ok((_entity, mut _window, mut cursor)) = window_query.single_mut() {
        let Ok(menu_visibility) = settings_menu.single() else {
            return;
        };
        let menu_visible = *menu_visibility != Visibility::Hidden;

        if menu_visible {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
            return;
        }

        if mouse_input.just_pressed(MouseButton::Left) {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }

        if key_input.just_pressed(KeyCode::Escape) {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}
