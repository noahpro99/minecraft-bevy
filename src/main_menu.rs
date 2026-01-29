use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::CursorOptions;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct WorldSettings {
    pub name: String,
    pub seed: u64,
    pub player_position: Option<Vec3>,
    pub inventory: Option<crate::player::components::Inventory>,
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct WorldNameInput;

#[derive(Component)]
pub struct WorldSeedInput;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<WorldSettings>()
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (handle_buttons,).run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(Update, (handle_input,).run_if(in_state(AppState::MainMenu)))
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
            .add_systems(
                OnExit(AppState::InGame),
                (save_inventory_on_exit, cleanup_in_game_entities),
            );
    }
}

fn save_inventory_on_exit(
    player_query: Query<
        (&Transform, &crate::player::components::Inventory),
        With<crate::player::components::Player>,
    >,
    mut world_settings: ResMut<WorldSettings>,
) {
    if let Ok((transform, inventory)) = player_query.single() {
        world_settings.player_position = Some(transform.translation);
        world_settings.inventory = Some(inventory.clone());
        save_world_settings(&world_settings);
    }
}

fn cleanup_in_game_entities(
    mut commands: Commands,
    query: Query<Entity, With<crate::world::components::InGameEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
enum MenuButton {
    Create,
    Load(String),
    Delete(String),
}

fn setup_main_menu(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut window_query: Query<(Entity, &mut Window, &mut CursorOptions)>,
) {
    if let Ok((_, _, mut cursor)) = window_query.single_mut() {
        cursor.grab_mode = bevy::window::CursorGrabMode::None;
        cursor.visible = true;
    }

    commands.spawn((Camera2d, MainMenuRoot));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("EXPLR"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // --- CREATE WORLD SECTION ---
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Create New World"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.8, 0.2)),
                    ));

                    // Name Input
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Name: "),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(35.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    BorderColor::all(Color::WHITE),
                                    WorldNameInput,
                                    FocusedInput, // Default focus
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("New World"),
                                        TextFont {
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // Seed Input
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Seed: "),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(35.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                                    WorldSeedInput,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(""),
                                        TextFont {
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // Create Button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(45.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                            MenuButton::Create,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("START ADVENTURE"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            // --- LOAD WORLD SECTION ---
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Load Existing World"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    if let Ok(worlds) = get_worlds() {
                        for world in worlds {
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(10.0),
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(250.0),
                                                height: Val::Px(40.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            MenuButton::Load(world.clone()),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new(world.clone()),
                                                TextFont {
                                                    font_size: 18.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(40.0),
                                                height: Val::Px(40.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                                            MenuButton::Delete(world),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("X"),
                                                TextFont {
                                                    font_size: 18.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
                        }
                    }
                });
        });
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

type InteractionQuery<'a, 'b> = Query<
    'a,
    'b,
    (&'static Interaction, &'static MenuButton),
    (Changed<Interaction>, With<Button>),
>;

fn handle_buttons(
    mut next_state: ResMut<NextState<AppState>>,
    mut world_settings: ResMut<WorldSettings>,
    interaction_query: InteractionQuery,
    input_query: Query<&Children, With<WorldNameInput>>,
    seed_query: Query<&Children, With<WorldSeedInput>>,
    text_query: Query<&Text>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Create => {
                    let mut name = String::new();
                    let mut seed = rand::random::<u64>();

                    if let Ok(children) = input_query.single() {
                        if let Ok(text) = text_query.get(children[0]) {
                            name = text.0.trim().to_string();
                        }
                    }

                    if let Ok(children) = seed_query.single() {
                        let seed_str = if let Ok(text) = text_query.get(children[0]) {
                            text.0.trim()
                        } else {
                            ""
                        };

                        if !seed_str.is_empty() {
                            if let Ok(s) = seed_str.parse::<u64>() {
                                seed = s;
                            } else {
                                use std::collections::hash_map::DefaultHasher;
                                use std::hash::{Hash, Hasher};
                                let mut hasher = DefaultHasher::new();
                                seed_str.hash(&mut hasher);
                                seed = hasher.finish();
                            }
                        }
                    }

                    let mut name = name.trim().to_string();
                    if name.is_empty() {
                        name = "New World".to_string();
                    }

                    let worlds = get_worlds().unwrap_or_default();
                    if worlds.contains(&name) {
                        let mut count = 1;
                        let base_name = name.clone();
                        while worlds.contains(&format!("{} ({})", base_name, count)) {
                            count += 1;
                        }
                        name = format!("{} ({})", base_name, count);
                    }

                    world_settings.name = name;
                    world_settings.seed = seed;
                    world_settings.inventory = None;
                    save_world_settings(&world_settings);
                    next_state.set(AppState::InGame);
                }
                MenuButton::Load(name) => {
                    if let Ok(settings) = load_world_settings(name) {
                        *world_settings = settings;
                        next_state.set(AppState::InGame);
                    }
                }
                MenuButton::Delete(name) => {
                    let mut path = get_worlds_dir();
                    path.push(name);
                    if path.exists() {
                        fs::remove_dir_all(path).ok();
                        next_state.set(AppState::MainMenu); // Refresh
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct FocusedInput;

type InputInteractionQuery<'a, 'b> =
    Query<'a, 'b, (Entity, &'static Interaction), (With<Node>, Changed<Interaction>)>;

fn handle_input(
    mut commands: Commands,
    mut char_events: MessageReader<KeyboardInput>,
    input_query: Query<(Entity, &Children), With<WorldNameInput>>,
    seed_query: Query<(Entity, &Children), With<WorldSeedInput>>,
    focused_query: Query<Entity, With<FocusedInput>>,
    mut text_query: Query<&mut Text>,
    _mouse_input: Res<ButtonInput<MouseButton>>,
    interaction_query: InputInteractionQuery,
) {
    use bevy::input::keyboard::Key;

    // Handle focus on click
    for (entity, interaction) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for focused_entity in focused_query.iter() {
                commands.entity(focused_entity).remove::<FocusedInput>();
            }
            if let Ok((input_entity, _)) = input_query.get(entity) {
                commands.entity(input_entity).insert(FocusedInput);
            } else if let Ok((seed_entity, _)) = seed_query.get(entity) {
                commands.entity(seed_entity).insert(FocusedInput);
            }
        }
    }

    if let Some((_focused_entity, children)) = focused_query.single().ok().and_then(|e| {
        input_query
            .get(e)
            .ok()
            .map(|(_, c)| (e, c))
            .or_else(|| seed_query.get(e).ok().map(|(_, c)| (e, c)))
    }) && let Ok(mut text) = text_query.get_mut(children[0])
    {
        for event in char_events.read() {
            if !event.state.is_pressed() {
                continue;
            }
            if let Key::Character(c) = &event.logical_key {
                text.0.push_str(c.as_str());
            } else if event.key_code == KeyCode::Backspace {
                text.0.pop();
            }
        }
    }
}

fn get_worlds_dir() -> PathBuf {
    let mut path = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".explr");
    path.push("worlds");
    fs::create_dir_all(&path).ok();
    path
}

fn get_worlds() -> Result<Vec<String>, std::io::Error> {
    let dir = get_worlds_dir();
    let mut worlds = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if let Some(name) = entry
            .path()
            .is_dir()
            .then(|| entry.file_name())
            .and_then(|n| n.to_str().map(|s| s.to_string()))
        {
            worlds.push(name);
        }
    }
    Ok(worlds)
}

fn save_world_settings(settings: &WorldSettings) {
    let mut path = get_worlds_dir();
    path.push(&settings.name);
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    if let Ok(json) = serde_json::to_string(settings) {
        fs::write(path, json).ok();
    }
}

fn load_world_settings(name: &str) -> Result<WorldSettings, String> {
    let mut path = get_worlds_dir();
    path.push(name);
    path.push("settings.json");
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}
