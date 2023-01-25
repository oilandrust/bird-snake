use bevy::{app::AppExit, prelude::*};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem},
    state::NextState,
};

use crate::despawn_with;

use super::GameState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Menu, setup_camera)
            .add_enter_system(GameState::Menu, setup_menu)
            .add_exit_system(GameState::Menu, despawn_with::<MainMenu>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Menu)
                    .with_system(bevy::window::close_on_esc)
                    .with_system(button_interact_visual_system)
                    .with_system(button_exit_system.run_if(on_button_interact_system::<ExitButton>))
                    .with_system(
                        button_game_system.run_if(on_button_interact_system::<EnterButton>),
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

fn setup_menu(mut commands: Commands, ass: Res<AssetServer>) {
    let butt_style = Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(8.0)),
        margin: UiRect::all(Val::Px(4.0)),
        flex_grow: 1.0,
        ..Default::default()
    };
    let butt_textstyle = TextStyle {
        font: ass.load("Sansation-Regular.ttf"),
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

    let butt_enter = commands
        .spawn((
            ButtonBundle {
                style: butt_style.clone(),
                ..Default::default()
            },
            EnterButton,
        ))
        .with_children(|btn| {
            btn.spawn(TextBundle {
                text: Text::from_section("Enter Game", butt_textstyle.clone()),
                ..Default::default()
            });
        })
        .id();

    let butt_exit = commands
        .spawn((
            ButtonBundle {
                style: butt_style,
                ..Default::default()
            },
            ExitButton,
        ))
        .with_children(|btn| {
            btn.spawn(TextBundle {
                text: Text::from_section("Exit Game", butt_textstyle.clone()),
                ..Default::default()
            });
        })
        .id();

    commands
        .entity(menu)
        .push_children(&[butt_enter, butt_exit]);
}
