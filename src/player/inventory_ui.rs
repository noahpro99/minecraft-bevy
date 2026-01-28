use crate::player::components::{Inventory, InventorySlotIcon};
use crate::player::settings_menu::{
    FovDecreaseButton, FovIncreaseButton, FovText, RenderDistanceDecreaseButton,
    RenderDistanceIncreaseButton, RenderDistanceText, ResumeButton, SettingsMenu,
};
use crate::world::components::VoxelType;
use bevy::image::{ImageLoaderSettings, ImageSampler, TRANSPARENT_IMAGE_HANDLE};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryBar;

#[derive(Component)]
pub struct InventorySlotUi(pub usize);

#[derive(Component)]
pub struct InventorySlotText(pub usize);

#[derive(Component)]
pub struct Crosshair;

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
}

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let grass_icon = asset_server.load_with_settings(
        "textures/grass_block_top.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let dirt_icon = asset_server.load_with_settings(
        "textures/dirt.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let stone_icon = asset_server.load_with_settings(
        "textures/stone.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let coal_ore_icon = asset_server.load_with_settings(
        "textures/coal_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let iron_ore_icon = asset_server.load_with_settings(
        "textures/iron_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let gold_ore_icon = asset_server.load_with_settings(
        "textures/gold_ore.png",
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    let diamond_ore_icon = asset_server.load_with_settings(
        "textures/diamond_ore.png",
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
                width: Val::Vw(100.0),
                height: Val::Px(36.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Visibility::Hidden,
            CommandInputRoot,
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
                width: Val::Vw(100.0),
                height: Val::Px(90.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(36.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Visibility::Hidden,
            CommandHistoryRoot,
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
                width: Val::Vw(100.0),
                height: Val::Px(70.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(5.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            InventoryBar,
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
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Centered Menu Container
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(400.0),
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
    mut text_queries: ParamSet<(
        Query<&mut Text, With<CommandInputText>>,
        Query<&mut Text, With<CommandHistoryText>>,
    )>,
    mut visibility_queries: ParamSet<(
        Query<&Visibility, With<crate::player::settings_menu::SettingsMenu>>,
        Query<&mut Visibility, With<CommandInputRoot>>,
        Query<&mut Visibility, With<CommandHistoryRoot>>,
    )>,
    mut char_events: MessageReader<KeyboardInput>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut sun_query: Query<
        (&mut Transform, &mut DirectionalLight),
        With<crate::world::components::SunLight>,
    >,
) {
    if let Ok(visibility) = visibility_queries.p0().single() {
        if *visibility != Visibility::Hidden {
            return;
        }
    }

    let mut ignore_slash = false;
    if !command_state.open && key_input.just_pressed(KeyCode::Slash) {
        command_state.open = true;
        command_state.buffer.clear();
        command_state.buffer.push('/');
        ignore_slash = true;
    }

    if !command_state.open {
        return;
    }

    if key_input.just_pressed(KeyCode::Escape) {
        command_state.open = false;
        command_state.buffer.clear();
    }

    if key_input.just_pressed(KeyCode::Backspace) {
        command_state.buffer.pop();
        if command_state.buffer.is_empty() {
            command_state.buffer.push('/');
        }
    }

    if key_input.just_pressed(KeyCode::Enter) {
        let result = execute_command(&command_state.buffer, &mut sun_query);
        command_state.history.push(result);
        if command_state.history.len() > 6 {
            command_state.history.remove(0);
        }
        command_state.open = false;
        command_state.buffer.clear();
    }

    if key_input.just_pressed(KeyCode::Space) {
        command_state.buffer.push(' ');
    }

    if command_state.open {
        for event in char_events.read() {
            if !event.state.is_pressed() {
                continue;
            }
            if let Key::Character(character) = &event.logical_key {
                if character.is_empty() {
                    continue;
                }
                if ignore_slash && character == "/" {
                    continue;
                }
                command_state.buffer.push_str(character);
            }
        }
    }

    if let Ok(mut visibility) = visibility_queries.p1().single_mut() {
        *visibility = if command_state.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut visibility) = visibility_queries.p2().single_mut() {
        *visibility = if command_state.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut text) = text_queries.p0().single_mut() {
        text.0 = command_state.buffer.clone();
    }
    if let Ok(mut text) = text_queries.p1().single_mut() {
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
            image.image = match slot.voxel_type {
                VoxelType::Grass => icon_assets.grass.clone(),
                VoxelType::Dirt => icon_assets.dirt.clone(),
                VoxelType::Stone => icon_assets.stone.clone(),
                VoxelType::CoalOre => icon_assets.coal_ore.clone(),
                VoxelType::IronOre => icon_assets.iron_ore.clone(),
                VoxelType::GoldOre => icon_assets.gold_ore.clone(),
                VoxelType::DiamondOre => icon_assets.diamond_ore.clone(),
                VoxelType::Air => TRANSPARENT_IMAGE_HANDLE,
            };
        }
    }
}
