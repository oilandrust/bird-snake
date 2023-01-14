use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

pub const GRID_TO_WORLD_UNIT: f32 = 25.;
pub const GRID_TO_WORLD_UNIT_INVERSE: f32 = 1. / GRID_TO_WORLD_UNIT;
pub const SNAKE_SIZE: Vec2 = Vec2::splat(GRID_TO_WORLD_UNIT);
pub const GRID_CELL_SIZE: Vec2 = SNAKE_SIZE;
pub const MOVE_START_VELOCITY: f32 = 4.0;
pub const JUMP_START_VELOCITY: f32 = 65.0;
pub const GRAVITY: f32 = 300.0;

pub const UP: IVec2 = IVec2::Y;
pub const DOWN: IVec2 = IVec2::NEG_Y;
pub const RIGHT: IVec2 = IVec2::X;
pub const LEFT: IVec2 = IVec2::NEG_X;

// https://coolors.co/palette/565264-706677-a6808c-ccb7ae-d6cfcb
pub const DARK_COLOR_PALETTE: [Color; 5] = [
    Color::rgb(0.3372549, 0.32156864, 0.39215687),
    Color::rgb(0.4392157, 0.4, 0.46666667),
    Color::rgb(0.6509804, 0.5019608, 0.54901963),
    Color::rgb(0.8, 0.7176471, 0.68235296),
    Color::rgb(0.8392157, 0.8117647, 0.79607844),
];

// https://coolors.co/palette/f94144-f3722c-f8961e-f9844a-f9c74f-90be6d-43aa8b-4d908e-577590-277da1
pub const BRIGHT_COLOR_PALETTE: [Color; 10] = [
    Color::rgb(0.9764706, 0.25490198, 0.26666668),
    Color::rgb(0.9529412, 0.44705883, 0.17254902),
    Color::rgb(0.972549, 0.5882353, 0.11764706),
    Color::rgb(0.9764706, 0.5176471, 0.2901961),
    Color::rgb(0.9764706, 0.78039217, 0.30980393),
    Color::rgb(0.5647059, 0.74509805, 0.42745098),
    Color::rgb(0.2627451, 0.6666667, 0.54509807),
    Color::rgb(0.3019608, 0.5647059, 0.5568628),
    Color::rgb(0.34117648, 0.45882353, 0.5647059),
    Color::rgb(0.15294118, 0.49019608, 0.6313726),
];

pub const WALL_COLOR: Color = DARK_COLOR_PALETTE[0];
pub const SNAKE_COLORS: [Color; 2] = [BRIGHT_COLOR_PALETTE[5], BRIGHT_COLOR_PALETTE[2]];

pub fn to_world(position: IVec2) -> Vec2 {
    (position.as_vec2() + 0.5) * GRID_TO_WORLD_UNIT
}

pub fn to_grid(position: Vec2) -> IVec2 {
    (position * GRID_TO_WORLD_UNIT_INVERSE - 0.5)
        .round()
        .as_ivec2()
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct GameConstants {
    #[inspector(min = 0.0, max = 10.0)]
    pub move_velocity: f32,

    #[inspector(min = 0.0, max = 300.0)]
    pub jump_velocity: f32,

    #[inspector(min = 0.0, max = 900.0)]
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
pub struct GameConstantsPlugin;

impl Plugin for GameConstantsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameConstants>();
        app.insert_resource(GameConstants::default());
    }
}
