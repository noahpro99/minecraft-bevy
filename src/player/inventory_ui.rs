use crate::player::components::{Health, Hunger, Inventory, InventorySlotIcon};
use crate::player::settings_menu::{
    FootstepVolumeDecreaseButton, FootstepVolumeIncreaseButton, FootstepVolumeText,
    FovDecreaseButton, FovIncreaseButton, FovText, MasterVolumeDecreaseButton,
    MasterVolumeIncreaseButton, MasterVolumeText, QuitToMenuButton, RenderDistanceDecreaseButton,
    RenderDistanceIncreaseButton, RenderDistanceText, ResumeButton, SettingsMenu,
};
use crate::world::components::{ItemType, SunLight};
use bevy::image::{ImageLoaderSettings, ImageSampler, TRANSPARENT_IMAGE_HANDLE};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

#[derive(Component)]
#[allow(dead_code)]
pub struct InventoryBar;

#[derive(Component)]
pub struct InventorySlotUi(pub usize);

#[derive(Component)]
pub struct InventorySlotText(pub usize);

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct HungerText;

#[derive(Resource, Default)]
pub struct CommandState {
    pub open: bool,
    pub buffer: String,
    pub history: Vec<String>,
}

#[derive(Component)]
pub struct CommandInputRoot;

#[derive(Component)]
pub struct CommandInputText;

#[derive(Component)]
pub struct CommandHistoryRoot;

#[derive(Component)]
pub struct CommandHistoryText;

#[derive(Resource)]
pub struct InventoryIconAssets {
    pub grass: Handle<Image>,
    pub dirt: Handle<Image>,
    pub stone: Handle<Image>,
    pub coal_ore: Handle<Image>,
    pub iron_ore: Handle<Image>,
    pub gold_ore: Handle<Image>,
    pub diamond_ore: Handle<Image>,
    pub wheat: Handle<Image>,
}

#[derive(Component)]
pub struct HotbarRoot;

#[derive(Component)]
#[allow(dead_code)]
pub struct InventoryRoot;

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let grass_icon = asset_server.load_with_settings(
        "textures/block/grass_block_top.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let dirt_icon = asset_server.load_with_settings(
        "textures/block/dirt.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let stone_icon = asset_server.load_with_settings(
        "textures/block/stone.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let coal_ore_icon = asset_server.load_with_settings(
        "textures/block/coal_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let iron_ore_icon = asset_server.load_with_settings(
        "textures/block/iron_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let gold_ore_icon = asset_server.load_with_settings(
        "textures/block/gold_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let diamond_ore_icon = asset_server.load_with_settings(
        "textures/block/diamond_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let wheat_icon = asset_server.load_with_settings(
        "textures/item/wheat.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    commands.insert_resource(InventoryIconAssets {
        grass: grass_icon,
        dirt: dirt_icon,
        stone: stone_icon,
        coal_ore: coal_ore_icon,
        iron_ore: iron_ore_icon,
        gold_ore: gold_ore_icon,
        diamond_ore: diamond_ore_icon,
        wheat: wheat_icon,
    });

    // Crosshair
    commands
        .spawn((
            Node {
                width: Val::Px(22.0),
                height: Val::Px(22.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-11.0),
                    top: Val::Px(-11.0),
                    ..default()
                },
                ..default()
            },
            Crosshair,
            crate::world::components::InGameEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(22.0),
                    height: Val::Px(2.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
            parent.spawn((
                Node {
                    width: Val::Px(2.0),
                    height: Val::Px(22.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
        });

    commands
        .spawn((
            Node {
                width: Val::Px(220.0),
                height: Val::Px(30.0),
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(12.0),
                ..default()
            },
            crate::world::components::InGameEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Health: 20/20"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
                HealthText,
            ));
        });

    commands
        .spawn((
            Node {
                width: Val::Px(220.0),
                height: Val::Px(26.0),
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(40.0),
                ..default()
            },
            crate::world::components::InGameEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Hunger: 20/20"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.8, 0.4)),
                HungerText,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(100.0),
                left: Val::Px(20.0),
                width: Val::Px(400.0),
                height: Val::Px(30.0),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderColor::all(Color::WHITE),
            CommandInputRoot,
            crate::world::components::InGameEntity,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                CommandInputText,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(140.0),
                left: Val::Px(20.0),
                width: Val::Px(400.0),
                flex_direction: FlexDirection::ColumnReverse,
                row_gap: Val::Px(2.0),
                ..default()
            },
            CommandHistoryRoot,
            crate::world::components::InGameEntity,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
                CommandHistoryText,
            ));
        });

    // Inventory Bar Container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            HotbarRoot,
            crate::world::components::InGameEntity,
        ))
        .with_children(|parent| {
            for i in 0..10 {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            margin: UiRect::all(Val::Px(4.5)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                        InventorySlotUi(i),
                        BorderColor::all(Color::WHITE),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                            ImageNode::new(TRANSPARENT_IMAGE_HANDLE),
                            InventorySlotIcon(i),
                        ));
                        parent.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 15.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            InventorySlotText(i),
                        ));
                    });
            }
        });

    // Settings Menu
    commands
        .spawn((
            Node {
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            SettingsMenu,
            crate::world::components::InGameEntity,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Centered Menu Container
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Settings Menu"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // FOV Section
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Field of View"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            parent.spawn((
                                Text::new("120°"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                FovText,
                            ));

                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            FovDecreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("-"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            FovIncreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("+"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });

                            parent.spawn((
                                Text::new("Render Distance"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            parent.spawn((
                                Text::new("7 chunks"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                RenderDistanceText,
                            ));

                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            RenderDistanceDecreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("-"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            RenderDistanceIncreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("+"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
                        });

                    // Master Volume Section
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Master Volume"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            parent.spawn((
                                Text::new("50%"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                MasterVolumeText,
                            ));

                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            MasterVolumeDecreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("-"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            MasterVolumeIncreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("+"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
                        });

                    // Footstep Volume Section
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Footstep Volume"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            parent.spawn((
                                Text::new("30%"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                FootstepVolumeText,
                            ));

                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            FootstepVolumeDecreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("-"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            BorderColor::all(Color::WHITE),
                                            FootstepVolumeIncreaseButton,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("+"),
                                                TextFont {
                                                    font_size: 22.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            BorderColor::all(Color::WHITE),
                            ResumeButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("RESUME"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
                            BorderColor::all(Color::WHITE),
                            QuitToMenuButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("QUIT TO MENU"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    parent.spawn((
                        Text::new("Press ESC to Close"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

pub fn handle_command_input(
    mut command_state: ResMut<CommandState>,
    mut text_params: ParamSet<(
        Query<&mut Text, With<CommandInputText>>,
        Query<&mut Text, With<CommandHistoryText>>,
    )>,
    mut visibility_params: ParamSet<(
        Query<&Visibility, With<SettingsMenu>>,
        Query<&mut Visibility, With<CommandInputRoot>>,
        Query<&mut Visibility, With<CommandHistoryRoot>>,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut char_events: MessageReader<KeyboardInput>,
    _mouse: Res<ButtonInput<KeyCode>>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), With<SunLight>>,
) {
    if let Ok(visibility) = visibility_params.p0().single()
        && *visibility != Visibility::Hidden
    {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Slash) && !command_state.open {
        command_state.open = true;
        command_state.buffer.clear();
        if let Ok(mut visibility) = visibility_params.p1().single_mut() {
            *visibility = Visibility::Visible;
        }
        if let Ok(mut visibility) = visibility_params.p2().single_mut() {
            *visibility = Visibility::Visible;
        }
    }

    if !command_state.open {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        let command = command_state.buffer.clone();
        command_state.history.push(command.clone());
        command_state.open = false;
        if let Ok(mut visibility) = visibility_params.p1().single_mut() {
            *visibility = Visibility::Hidden;
        }
        let response = execute_command(&command, &mut sun_query);
        command_state.history.push(format!("[System] {}", response));
    }

    for event in char_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        if let bevy::input::keyboard::Key::Character(c) = &event.logical_key {
            command_state.buffer.push_str(c.as_str());
        } else if event.key_code == KeyCode::Backspace {
            command_state.buffer.pop();
        } else if event.key_code == KeyCode::Escape {
            command_state.open = false;
            if let Ok(mut visibility) = visibility_params.p1().single_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }

    if let Ok(mut text) = text_params.p0().single_mut() {
        text.0 = format!("> {}", command_state.buffer);
    }
    if let Ok(mut text) = text_params.p1().single_mut() {
        text.0 = command_state.history.join("\n");
    }
}

fn execute_command(
    buffer: &str,
    sun_query: &mut Query<
        (&mut Transform, &mut DirectionalLight),
        With<crate::world::components::SunLight>,
    >,
) -> String {
    let command = buffer.trim().to_lowercase();
    if command == "/time set day" {
        if let Ok((mut transform, mut light)) = sun_query.single_mut() {
            *transform = Transform::from_xyz(80.0, 120.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y);
            light.illuminance = 30000.0;
        }
        return "Set time: day".to_string();
    } else if command == "/time set night" {
        if let Ok((mut transform, mut light)) = sun_query.single_mut() {
            *transform = Transform::from_xyz(-30.0, -10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
            light.illuminance = 2000.0;
        }
        return "Set time: night".to_string();
    }
    format!("Unknown command: {}", buffer.trim())
}

pub fn update_inventory_ui(
    inventory_query: Query<&Inventory>,
    icon_assets: Res<InventoryIconAssets>,
    mut slot_query: Query<(&InventorySlotUi, &mut BackgroundColor, &mut BorderColor)>,
    mut text_query: Query<(&InventorySlotText, &mut Text)>,
    mut icon_query: Query<(&InventorySlotIcon, &mut ImageNode)>,
) {
    if let Ok(inventory) = inventory_query.single() {
        for (slot_ui, mut background, mut border) in slot_query.iter_mut() {
            if inventory.selected_slot == slot_ui.0 {
                *border = BorderColor::all(Color::srgb(1.0, 1.0, 0.0)); // Yellow
                background.0 = Color::srgba(0.5, 0.5, 0.5, 1.0);
            } else {
                *border = BorderColor::all(Color::WHITE);
                background.0 = Color::srgba(0.3, 0.3, 0.3, 1.0);
            }
        }

        for (slot_text, mut text) in text_query.iter_mut() {
            let slot = &inventory.slots[slot_text.0];
            if slot.count > 0 {
                text.0 = slot.count.to_string();
            } else {
                text.0 = "".to_string();
            }
        }

        for (slot_icon, mut image) in icon_query.iter_mut() {
            let slot = &inventory.slots[slot_icon.0];
            image.image = match slot.item_type {
                ItemType::GrassBlock => icon_assets.grass.clone(),
                ItemType::Dirt => icon_assets.dirt.clone(),
                ItemType::Stone => icon_assets.stone.clone(),
                ItemType::CoalOre => icon_assets.coal_ore.clone(),
                ItemType::IronOre => icon_assets.iron_ore.clone(),
                ItemType::GoldOre => icon_assets.gold_ore.clone(),
                ItemType::DiamondOre => icon_assets.diamond_ore.clone(),
                ItemType::Wheat => icon_assets.wheat.clone(),
                ItemType::None => TRANSPARENT_IMAGE_HANDLE,
            };
        }
    }
}

pub fn update_health_ui(
    health_query: Query<&Health>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    let Ok(health) = health_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    text.0 = format!("Health: {}/{}", health.current, health.max);
}

pub fn update_hunger_ui(
    hunger_query: Query<&Hunger>,
    mut text_query: Query<&mut Text, With<HungerText>>,
) {
    let Ok(hunger) = hunger_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    text.0 = format!("Hunger: {}/{}", hunger.current, hunger.max);
}
