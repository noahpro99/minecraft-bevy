use bevy::prelude::*;
use bevy::window::CursorOptions;

#[derive(Resource)]
pub struct Settings {
    pub fov: f32,
    pub render_distance: i32,
    pub master_volume: f32,
    pub footstep_volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fov: 120.0,
            render_distance: 7,
            master_volume: 0.5,
            footstep_volume: 0.3,
        }
    }
}

#[derive(Component)]
pub struct SettingsMenu;

#[derive(Component)]
pub struct FovText;

#[derive(Component)]
pub struct FovDecreaseButton;

#[derive(Component)]
pub struct FovIncreaseButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitToMenuButton;

#[derive(Component)]
pub struct RenderDistanceText;

#[derive(Component)]
pub struct RenderDistanceDecreaseButton;

#[derive(Component)]
pub struct RenderDistanceIncreaseButton;

#[derive(Component)]
pub struct MasterVolumeText;

#[derive(Component)]
pub struct MasterVolumeDecreaseButton;

#[derive(Component)]
pub struct MasterVolumeIncreaseButton;

#[derive(Component)]
pub struct FootstepVolumeText;

#[derive(Component)]
pub struct FootstepVolumeDecreaseButton;

#[derive(Component)]
pub struct FootstepVolumeIncreaseButton;

pub fn toggle_settings_menu(
    key: Res<ButtonInput<KeyCode>>,
    mut settings_menu_query: Query<&mut Visibility, With<SettingsMenu>>,
    mut cameras: Query<&mut Projection, With<Camera3d>>,
    settings: Res<Settings>,
    mut window_query: Query<(Entity, &mut Window, &mut CursorOptions)>,
) {
    if key.just_pressed(KeyCode::Escape) {
        if let Ok(mut visibility) = settings_menu_query.single_mut() {
            if *visibility == Visibility::Hidden {
                *visibility = Visibility::Visible;

                // Release cursor
                if let Ok((_, _, mut cursor)) = window_query.single_mut() {
                    cursor.grab_mode = bevy::window::CursorGrabMode::None;
                    cursor.visible = true;
                }

                // Apply current FOV to camera
                for mut projection in cameras.iter_mut() {
                    if let Projection::Perspective(perspective) = &mut *projection {
                        perspective.fov = settings.fov.to_radians();
                    }
                }
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn handle_resume_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<ResumeButton>),
    >,
    mut settings_menu_query: Query<&mut Visibility, With<SettingsMenu>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Ok(mut visibility) = settings_menu_query.single_mut() {
                    *visibility = Visibility::Hidden;
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }
}

pub fn handle_quit_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<QuitToMenuButton>),
    >,
    mut next_state: ResMut<NextState<crate::main_menu::AppState>>,
    mut window_query: Query<(Entity, &mut Window, &mut CursorOptions)>,
    player_query: Query<
        (&Transform, &crate::player::components::Inventory),
        With<crate::player::components::Player>,
    >,
    mut world_settings: ResMut<crate::main_menu::WorldSettings>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Manually trigger inventory save before state transition to be safe
                if let Ok((transform, inventory)) = player_query.single() {
                    world_settings.player_position = Some(transform.translation);
                    world_settings.inventory = Some(inventory.clone());
                }

                // Ensure cursor is released before switching states
                if let Ok((_, _, mut cursor)) = window_query.single_mut() {
                    cursor.grab_mode = bevy::window::CursorGrabMode::None;
                    cursor.visible = true;
                }
                next_state.set(crate::main_menu::AppState::MainMenu);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.6, 0.3, 0.3));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.4, 0.2, 0.2));
            }
        }
    }
}

pub fn handle_fov_buttons(
    mut cameras: Query<&mut Projection, With<Camera3d>>,
    mut settings: ResMut<Settings>,
    mut fov_text_query: Query<&mut Text, With<FovText>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&FovDecreaseButton>,
            Option<&FovIncreaseButton>,
        ),
        Changed<Interaction>,
    >,
) {
    let mut delta: f32 = 0.0;

    for (interaction, mut color, is_decrease, is_increase) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if is_decrease.is_some() {
                    delta -= 5.0;
                }
                if is_increase.is_some() {
                    delta += 5.0;
                }
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }

    if delta.abs() < f32::EPSILON {
        return;
    }

    settings.fov = (settings.fov + delta).clamp(30.0, 180.0);

    for mut projection in cameras.iter_mut() {
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = settings.fov.to_radians();
        }
    }

    if let Ok(mut text) = fov_text_query.single_mut() {
        text.0 = format!("{:.0}°", settings.fov);
    }
}

pub fn handle_render_distance_buttons(
    mut settings: ResMut<Settings>,
    mut text_query: Query<&mut Text, With<RenderDistanceText>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&RenderDistanceDecreaseButton>,
            Option<&RenderDistanceIncreaseButton>,
        ),
        Changed<Interaction>,
    >,
) {
    let mut delta: i32 = 0;

    for (interaction, mut color, is_decrease, is_increase) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if is_decrease.is_some() {
                    delta -= 1;
                }
                if is_increase.is_some() {
                    delta += 1;
                }
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }

    if delta == 0 {
        return;
    }

    settings.render_distance = (settings.render_distance + delta).clamp(2, 16);

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!("{} chunks", settings.render_distance);
    }
}

pub fn handle_master_volume_buttons(
    mut settings: ResMut<Settings>,
    mut text_query: Query<&mut Text, With<MasterVolumeText>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&MasterVolumeDecreaseButton>,
            Option<&MasterVolumeIncreaseButton>,
        ),
        Changed<Interaction>,
    >,
) {
    let mut delta: f32 = 0.0;

    for (interaction, mut color, is_decrease, is_increase) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if is_decrease.is_some() {
                    delta -= 0.1;
                }
                if is_increase.is_some() {
                    delta += 0.1;
                }
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }

    if delta.abs() < f32::EPSILON {
        return;
    }

    settings.master_volume = (settings.master_volume + delta).clamp(0.0, 1.0);

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!("{:.0}%", settings.master_volume * 100.0);
    }
}

pub fn handle_footstep_volume_buttons(
    mut settings: ResMut<Settings>,
    mut text_query: Query<&mut Text, With<FootstepVolumeText>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&FootstepVolumeDecreaseButton>,
            Option<&FootstepVolumeIncreaseButton>,
        ),
        Changed<Interaction>,
    >,
) {
    let mut delta: f32 = 0.0;

    for (interaction, mut color, is_decrease, is_increase) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if is_decrease.is_some() {
                    delta -= 0.1;
                }
                if is_increase.is_some() {
                    delta += 0.1;
                }
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }

    if delta.abs() < f32::EPSILON {
        return;
    }

    settings.footstep_volume = (settings.footstep_volume + delta).clamp(0.0, 1.0);

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!("{:.0}%", settings.footstep_volume * 100.0);
    }
}
