use bevy::prelude::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use crate::{
    game_constants_pluggin::{to_world, GameConstants, GRID_TO_WORLD_UNIT},
    level_template::LevelTemplate,
    snake::Snake,
};

pub struct DevToolsPlugin;

#[derive(Default, Resource)]
pub struct DevToolsSettings {
    pub dev_tools_enabled: bool,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DevToolsSettings>()
            // .add_plugin(LogDiagnosticsPlugin::default())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(DebugLinesPlugin::default())
            .add_system(toogle_dev_tools_system)
            .add_system_to_stage(CoreStage::Last, debug_draw_grid_system)
            .add_system_to_stage(CoreStage::Last, debug_draw_snake_system);
    }
}

fn toogle_dev_tools_system(
    keyboard: Res<Input<KeyCode>>,
    mut dev_tool_settings: ResMut<DevToolsSettings>,
    mut inspector_windows: ResMut<InspectorWindows>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        let old_value = dev_tool_settings.dev_tools_enabled;
        dev_tool_settings.dev_tools_enabled = !old_value;
    }

    inspector_windows.window_data_mut::<GameConstants>().visible =
        dev_tool_settings.dev_tools_enabled;
}

fn debug_draw_grid_system(
    dev_tool_settings: Res<DevToolsSettings>,
    level: Res<LevelTemplate>,
    mut lines: ResMut<DebugLines>,
) {
    if !dev_tool_settings.dev_tools_enabled {
        return;
    }

    for j in 0..=level.grid.height() {
        let y = j as f32 * GRID_TO_WORLD_UNIT;
        let start = Vec3::new(0., y, 0.);
        let end = Vec3::new(level.grid.width() as f32 * GRID_TO_WORLD_UNIT, y, 0.);
        lines.line_colored(
            start,
            end,
            0.,
            if j == 0 { Color::RED } else { Color::BLACK },
        );
    }

    for i in 0..=level.grid.width() {
        let x = i as f32 * GRID_TO_WORLD_UNIT;
        let start = Vec3::new(x, 0., 0.);
        let end = Vec3::new(x, level.grid.height() as f32 * GRID_TO_WORLD_UNIT, 0.);
        lines.line_colored(
            start,
            end,
            0.,
            if i == 0 { Color::RED } else { Color::BLACK },
        );
    }
}

fn debug_draw_snake_system(
    dev_tool_settings: Res<DevToolsSettings>,
    mut lines: ResMut<DebugLines>,
    query: Query<&Snake>,
) {
    if !dev_tool_settings.dev_tools_enabled {
        return;
    }

    let Ok(snake) = query.get_single() else {
        return;
    };

    for position in &snake.parts {
        let world_grid = to_world(position.0);
        let world_grid = Vec3::new(world_grid.x, world_grid.y, 0.0);

        lines.line_colored(
            world_grid + Vec3::new(5.0, 5.0, 0.0),
            world_grid + Vec3::new(-5.0, -5.0, 0.0),
            0.,
            Color::BLUE,
        );

        lines.line_colored(
            world_grid + Vec3::new(-5.0, 5.0, 0.0),
            world_grid + Vec3::new(5.0, -5.0, 0.0),
            0.,
            Color::BLUE,
        );
    }
}
