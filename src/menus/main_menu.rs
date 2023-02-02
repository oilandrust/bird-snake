use bevy::{app::AppExit, prelude::*};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem},
    state::NextState,
};

use crate::{despawn_with, GameState};

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
fn button_interact_visual_system(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *color = BackgroundColor(Color::rgb(0.75, 0.75, 0.75));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::rgb(0.8, 0.8, 0.8));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::rgb(1.0, 1.0, 1.0));
            }
        }
    }
}

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

fn button_exit_system(mut ev: EventWriter<AppExit>) {
    ev.send(AppExit);
}

fn button_game_system(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Game));
}

fn button_select_level_system(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::SelectLevelMenu));
}

fn setup_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let button_style = Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(8.0)),
        margin: UiRect::all(Val::Px(4.0)),
        flex_grow: 1.0,
        ..Default::default()
    };
    let button_text_style = TextStyle {
        font: assets.load("Sansation-Regular.ttf"),
        font_size: 24.0,
        color: Color::BLACK,
    };

    let menu = commands
        .spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::rgb(0.5, 0.5, 0.5)),
                style: Style {
                    size: Size::new(Val::Auto, Val::Auto),
                    margin: UiRect::all(Val::Auto),
                    align_self: AlignSelf::Center,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            MainMenu,
        ))
        .id();

    let start_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                ..Default::default()
            },
            EnterButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Start", button_text_style.clone()),
                ..Default::default()
            });
        })
        .id();

    let select_level_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                ..Default::default()
            },
            SelectLevelButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Select Level", button_text_style.clone()),
                ..Default::default()
            });
        })
        .id();

    let mut children = vec![start_button, select_level_button];

    #[cfg(not(target_arch = "wasm32"))]
    {
        let exit_button = commands
            .spawn((
                ButtonBundle {
                    style: button_style,
                    ..Default::default()
                },
                ExitButton,
            ))
            .with_children(|btn| {
                btn.spawn(TextBundle {
                    text: Text::from_section("Exit Game", button_text_style.clone()),
                    ..Default::default()
                });
            })
            .id();
        children.push(exit_button);
    }

    commands.entity(menu).push_children(&children);
}
