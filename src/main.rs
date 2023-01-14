use automated_test_pluggin::AutomatedTestPluggin;
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use dev_tools_pluggin::DevToolsPlugin;
use game_constants_pluggin::*;
use level_pluggin::{LevelPluggin, StartLevelEventWithIndex};
use movement_pluggin::MovementPluggin;
use snake_pluggin::SnakePluggin;

mod automated_test_pluggin;
mod commands;
mod dev_tools_pluggin;
mod game_constants_pluggin;
mod level_pluggin;
mod level_template;
mod levels;
mod movement_pluggin;
mod snake_pluggin;
mod test_levels;
mod undo;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 0)]
    level: usize,

    #[arg(long, default_value_t = false)]
    test: bool,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();

    app.insert_resource(ClearColor(DARK_COLOR_PALETTE[4]))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Snake".to_string(),
                width: 640.0,
                height: 420.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(TweeningPlugin)
        .add_plugin(GameConstantsPlugin)
        .add_plugin(DevToolsPlugin)
        .add_plugin(SnakePluggin)
        .add_plugin(LevelPluggin)
        .add_plugin(MovementPluggin)
        .add_system(bevy::window::close_on_esc);

    let start_level = args.level;
    let start_game = move |mut event_writer: EventWriter<StartLevelEventWithIndex>| {
        event_writer.send(StartLevelEventWithIndex(start_level));
    };

    if args.test {
        app.add_plugin(AutomatedTestPluggin);
    } else {
        app.add_startup_system(start_game);
    };

    app.run();
}
