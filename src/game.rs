use args::Args;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin, AudioSource};
use bevy_tweening::TweeningPlugin;
use gameplay::camera_plugin::CameraPlugin;
use gameplay::game_constants_pluggin::*;
use gameplay::level_pluggin::{
    ClearLevelEvent, LevelEntity, LevelPluggin, StartLevelEventWithIndex,
    StartTestLevelEventWithIndex,
};
use gameplay::movement_pluggin::MovementPluggin;
use gameplay::snake_pluggin::SnakePluggin;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};
use menus::main_menu::MainMenuPlugin;
use menus::select_level_menu::{NextLevel, SelectLevelMenuPlugin};
use menus::MenuPlugin;
use tools::dev_tools_pluggin::DevToolsPlugin;

pub mod args;
mod gameplay;
mod level;
mod menus;
mod render_water;
mod tools;

// Don't touch this piece, needed for Web
#[cfg(target_arch = "wasm32")]
mod web_main;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    MainMenu,
    SelectLevelMenu,
    Game,
}

pub struct GamePlugin {
    args: Args,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_exit_system(GameState::Game, despawn_with::<LevelEntity>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .with_system(back_to_menu_on_escape_system)
                    .into(),
            )
            .add_plugin(LevelPluggin)
            .add_plugin(SnakePluggin)
            .add_plugin(MovementPluggin)
            .add_plugin(GameConstantsPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(DevToolsPlugin)
            .add_plugin(TweeningPlugin)
            .insert_resource(self.args.clone())
            .insert_resource(NextLevel(self.args.level.unwrap_or(0)));

        //if let Some(args::Commands::Test { test_case: _ }) = self.args.command {
        //app.add_plugin(AutomatedTestPluggin);
        //}

        app.add_enter_system(GameState::Game, enter_game_system);
    }
}

fn enter_game_system(
    args: Res<Args>,
    next_level: Res<NextLevel>,
    // mut start_test_case_event: EventWriter<StartTestCaseEventWithIndex>,
    mut start_test_level_event: EventWriter<StartTestLevelEventWithIndex>,
    mut start_level_event: EventWriter<StartLevelEventWithIndex>,
) {
    match args.command {
        Some(args::Commands::Test { test_case: _ }) => {
            // let start_test_case = test_case.unwrap_or(0);
            // start_test_case_event.send(StartTestCaseEventWithIndex(start_test_case));
        }
        None => {
            if let Some(test_level) = args.test_level {
                start_test_level_event.send(StartTestLevelEventWithIndex(test_level));
                return;
            }
        }
    };

    start_level_event.send(StartLevelEventWithIndex(next_level.0));
}

fn back_to_menu_on_escape_system(
    mut event_clear_level: EventWriter<ClearLevelEvent>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        event_clear_level.send(ClearLevelEvent);
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn run(app: &mut App, args: &Args) {
    let start_state = if args.command.is_none() && args.level.is_none() && args.test_level.is_none()
    {
        GameState::MainMenu
    } else {
        GameState::Game
    };

    app.insert_resource(ClearColor(DARK_COLOR_PALETTE[4]))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Bird Snake".to_string(),
                width: 1080.0,
                height: 720.0,
                ..default()
            },
            ..default()
        }))
        .add_loopless_state_before_stage(CoreStage::PreUpdate, start_state)
        .add_plugin(MenuPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(SelectLevelMenuPlugin)
        .add_plugin(GamePlugin { args: args.clone() })
        .add_plugin(AudioPlugin)
        .add_startup_system(load_assets)
        .run();
}

#[derive(Resource)]
pub struct Assets {
    pub background_noise: Handle<AudioSource>,
    pub move_effect_1: Handle<AudioSource>,
    pub move_effect_2: Handle<AudioSource>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let background_noise: Handle<AudioSource> = asset_server.load("beach.mp3");

    commands.insert_resource(Assets {
        background_noise: background_noise.clone(),
        move_effect_1: asset_server.load("effects1.mp3"),
        move_effect_2: asset_server.load("effects2.mp3"),
    });

    audio.play(background_noise).looped().with_volume(0.04);
}
