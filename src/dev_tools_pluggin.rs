use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use iyes_loopless::prelude::ConditionSet;

use crate::game_constants_pluggin::GameConstants;
use crate::level::level_instance::LevelEntityType;
use crate::level::level_instance::LevelInstance;
use crate::GameState;
use crate::{
    game_constants_pluggin::{to_world, GRID_TO_WORLD_UNIT},
    level::level_template::LevelTemplate,
    snake_pluggin::Snake,
};

pub struct DevToolsPlugin;

#[derive(Default, Resource)]
pub struct DevToolsSettings {
    pub dev_tools_enabled: bool,
    pub inspector_enabled: bool,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DevToolsSettings>()
            // .add_plugin(LogDiagnosticsPlugin::default())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(DebugLinesPlugin::default())
            .add_plugin(EguiPlugin)
            .add_plugin(DefaultInspectorConfigPlugin)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .with_system(toogle_dev_tools_system)
                    .with_system(inspector_ui_system)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .run_if_resource_exists::<LevelInstance>()
                    .with_system(debug_draw_grid_system)
                    .with_system(debug_draw_snake_system)
                    .with_system(debug_draw_level_cells)
                    .into(),
            );
    }
}

fn toogle_dev_tools_system(
    keyboard: Res<Input<KeyCode>>,
    mut dev_tool_settings: ResMut<DevToolsSettings>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        let old_value = dev_tool_settings.dev_tools_enabled;
        dev_tool_settings.dev_tools_enabled = !old_value;
    }

    if keyboard.just_pressed(KeyCode::I) {
        let old_value = dev_tool_settings.inspector_enabled;
        dev_tool_settings.inspector_enabled = !old_value;
    }
}

fn inspector_ui_system(world: &mut World) {
    let dev_tool_settings = world
        .get_resource::<DevToolsSettings>()
        .expect("A dev tools settings resource should be present.");

    if !dev_tool_settings.dev_tools_enabled {
        return;
    }

    if !dev_tool_settings.inspector_enabled {
        return;
    }

    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    egui::Window::new("GameConstants").show(&egui_context, |ui| {
        bevy_inspector_egui::bevy_inspector::ui_for_resource::<GameConstants>(world, ui);
    });

    egui::Window::new("Inspector").show(&egui_context, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector::ui_for_world(world, ui);
        });
    });
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

    for snake in query.iter() {
        for position in snake.parts() {
            let world_grid = to_world(position.0);
            let world_grid = Vec3::new(world_grid.x, world_grid.y, 0.0);

            lines.line_colored(
                world_grid + Vec3::new(5.0, 0.0, 0.0),
                world_grid + Vec3::new(-5.0, 0.0, 0.0),
                0.,
                Color::BLUE,
            );

            lines.line_colored(
                world_grid + Vec3::new(0.0, 5.0, 0.0),
                world_grid + Vec3::new(0.0, -5.0, 0.0),
                0.,
                Color::BLUE,
            );
        }
    }
}

fn debug_draw_level_cells(
    dev_tool_settings: Res<DevToolsSettings>,
    mut lines: ResMut<DebugLines>,
    level: Res<LevelInstance>,
) {
    if !dev_tool_settings.dev_tools_enabled {
        return;
    }

    for (position, value) in level.occupied_cells() {
        let world_grid = to_world(*position);
        let world_grid = Vec3::new(world_grid.x, world_grid.y, 0.0);

        let color = match value {
            LevelEntityType::Food => Color::RED,
            LevelEntityType::Wall => Color::BLACK,
            LevelEntityType::Snake(_) => Color::BLUE,
            LevelEntityType::Spike => Color::DARK_GRAY,
        };

        lines.line_colored(
            world_grid + Vec3::new(5.0, 5.0, 0.0),
            world_grid + Vec3::new(-5.0, -5.0, 0.0),
            0.,
            color,
        );

        lines.line_colored(
            world_grid + Vec3::new(-5.0, 5.0, 0.0),
            world_grid + Vec3::new(5.0, -5.0, 0.0),
            0.,
            color,
        );
    }
}
