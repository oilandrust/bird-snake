use crate::menus::button_interact_visual_system;
use bevy::{app::AppExit, prelude::*};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem},
    state::NextState,
};

use crate::{despawn_with, GameState};

use super::MenuStyles;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::MainMenu, setup_camera)
            .add_enter_system(GameState::MainMenu, setup_menu)
            .add_exit_system(GameState::MainMenu, despawn_with::<MainMenu>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(bevy::window::close_on_esc)
                    .with_system(button_interact_visual_system)
                    .with_system(button_exit_system.run_if(on_button_interact_system::<ExitButton>))
                    .with_system(
                        button_game_system.run_if(on_button_interact_system::<EnterButton>),
                    )
                    .with_system(
                        button_select_level_system
                            .run_if(on_button_interact_system::<SelectLevelButton>),
                    )
                    .into(),
            );
    }
}

#[derive(Component)]
struct MenuCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MenuCamera, MainMenu));
}

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct EnterButton;

#[derive(Component)]
struct SelectLevelButton;

#[allow(clippy::type_complexity)]
fn on_button_interact_system<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            return true;
        }
    }

    false
}

fn button_exit_system(mut event: EventWriter<AppExit>) {
    event.send(AppExit);
}

fn button_game_system(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Game));
}

fn button_select_level_system(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::SelectLevelMenu));
}

fn setup_menu(mut commands: Commands, menu_styles: Res<MenuStyles>) {
    let menu = commands
        .spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                style: menu_styles.layout_node_style.clone(),
                ..Default::default()
            },
            MainMenu,
        ))
        .id();

    let title = commands
        .spawn((
            TextBundle {
                text: Text::from_section("BirdSnake", menu_styles.title_style.clone()),
                style: menu_styles.button_style.clone(),
                ..Default::default()
            },
            MainMenu,
        ))
        .id();

    let start_button = commands
        .spawn((
            ButtonBundle {
                style: menu_styles.button_style.clone(),
                background_color: BackgroundColor(Color::NONE),
                ..Default::default()
            },
            EnterButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Start", menu_styles.button_text_style.clone()),
                ..Default::default()
            });
        })
        .id();

    let select_level_button = commands
        .spawn((
            ButtonBundle {
                style: menu_styles.button_style.clone(),
                background_color: BackgroundColor(Color::NONE),
                ..Default::default()
            },
            SelectLevelButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Select Level", menu_styles.button_text_style.clone()),
                ..Default::default()
            });
        })
        .id();

    let mut children = vec![title, start_button, select_level_button];

    #[cfg(not(target_arch = "wasm32"))]
    {
        let exit_button = commands
            .spawn((
                ButtonBundle {
                    style: menu_styles.button_style.clone(),
                    background_color: BackgroundColor(Color::NONE),
                    ..Default::default()
                },
                ExitButton,
            ))
            .with_children(|btn| {
                btn.spawn(TextBundle {
                    text: Text::from_section("Exit Game", menu_styles.button_text_style.clone()),
                    ..Default::default()
                });
            })
            .id();
        children.push(exit_button);
    }

    commands.entity(menu).push_children(&children);
}
