use bevy::prelude::*;

use bevy_egui::{egui, EguiContext};

use crate::dev_tools::DevToolsSettings;

pub const GRID_TO_WORLD_UNIT: f32 = 25.;
pub const SNAKE_SIZE: Vec2 = Vec2::splat(GRID_TO_WORLD_UNIT);
pub const GRID_CELL_SIZE: Vec2 = SNAKE_SIZE;
pub const MOVE_START_VELOCITY: f32 = 4.0;
pub const JUMP_START_VELOCITY: f32 = 65.0;
pub const GRAVITY: f32 = 300.0;

pub fn to_world(position: IVec2) -> Vec2 {
    (position.as_vec2() + 0.5) * GRID_TO_WORLD_UNIT
}

#[derive(Resource)]
pub struct GameConstants {
    pub move_velocity: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
}

impl Default for GameConstants {
    fn default() -> Self {
        Self {
            move_velocity: MOVE_START_VELOCITY,
            jump_velocity: JUMP_START_VELOCITY,
            gravity: GRAVITY,
        }
    }
}

fn dev_ui_system(
    dev_tool_settings: Res<DevToolsSettings>,
    mut egui_context: ResMut<EguiContext>,
    mut game_constants: ResMut<GameConstants>,
) {
    if !dev_tool_settings.dev_tools_enabled {
        return;
    }

    egui::Window::new("Game Constants").show(egui_context.ctx_mut(), |ui| {
        ui.add(
            egui::Slider::new(&mut game_constants.move_velocity, 0.0..=10.0).text("Move Velocity"),
        );
        ui.add(
            egui::Slider::new(&mut game_constants.jump_velocity, 0.0..=300.0).text("Jump Velocity"),
        );
        ui.add(egui::Slider::new(&mut game_constants.gravity, 0.0..=900.0).text("Gravity"));
    });
}

pub struct GameConstantsPlugin;

impl Plugin for GameConstantsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConstants>()
            .add_system(dev_ui_system);
    }
}
