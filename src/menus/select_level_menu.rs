use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};

use crate::{despawn_with, level::levels::LEVELS, GameState};

use super::{button_interact_visual_system, MenuStyles};

pub struct SelectLevelMenuPlugin;

impl Plugin for SelectLevelMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::SelectLevelMenu, setup_camera)
            .add_enter_system(GameState::SelectLevelMenu, setup_menu)
            .add_exit_system(GameState::SelectLevelMenu, despawn_with::<SelectLevelMenu>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::SelectLevelMenu)
                    .with_system(back_on_escape)
                    .with_system(button_interact_visual_system)
                    .with_system(on_back_button_interact_system)
                    .with_system(on_level_button_interact_system)
                    .into(),
            );
    }
}

#[derive(Component)]
struct MenuCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MenuCamera, SelectLevelMenu));
}

#[derive(Component)]
struct SelectLevelMenu;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct LevelButton(usize);

#[derive(Resource)]
pub struct NextLevel(pub usize);

#[allow(clippy::type_complexity)]
fn on_back_button_interact_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<BackButton>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            commands.insert_resource(NextState(GameState::MainMenu));
        }
    }
}

pub fn back_on_escape(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

#[allow(clippy::type_complexity)]
fn on_level_button_interact_system(
    mut commands: Commands,
    query: Query<(&Interaction, &LevelButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, level_button) in query.iter() {
        if *interaction == Interaction::Clicked {
            commands.insert_resource(NextState(GameState::Game));
            commands.insert_resource(NextLevel(level_button.0))
        }
    }
}

fn setup_menu(mut commands: Commands, menu_styles: Res<MenuStyles>) {
    let button_style = Style {
        padding: UiRect::all(Val::Px(2.0)),
        margin: UiRect::all(Val::Px(2.0)),
        ..menu_styles.button_style
    };

    let menu = commands
        .spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                style: menu_styles.layout_node_style.clone(),
                ..Default::default()
            },
            SelectLevelMenu,
        ))
        .id();

    let mut buttons: Vec<Entity> = Vec::with_capacity(LEVELS.len() + 1);

    for i in 0..LEVELS.len() {
        buttons.push(
            commands
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: BackgroundColor(Color::NONE),
                        ..Default::default()
                    },
                    LevelButton(i),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("Level {}", i),
                            menu_styles.button_text_style.clone(),
                        ),
                        ..Default::default()
                    });
                })
                .id(),
        );
    }

    buttons.push(
        commands
            .spawn((
                ButtonBundle {
                    style: button_style,
                    background_color: BackgroundColor(Color::NONE),
                    ..Default::default()
                },
                BackButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "Back to Main Menu",
                        menu_styles.button_text_style.clone(),
                    ),
                    ..Default::default()
                });
            })
            .id(),
    );

    commands.entity(menu).push_children(&buttons);
}
