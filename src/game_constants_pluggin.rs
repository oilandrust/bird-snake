use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

pub const GRID_TO_WORLD_UNIT: f32 = 25.;
pub const GRID_TO_WORLD_UNIT_INVERSE: f32 = 1. / GRID_TO_WORLD_UNIT;
pub const SNAKE_SIZE: Vec2 = Vec2::splat(GRID_TO_WORLD_UNIT);
pub const GRID_CELL_SIZE: Vec2 = SNAKE_SIZE;
pub const MOVE_START_VELOCITY: f32 = 4.0;
pub const JUMP_START_VELOCITY: f32 = 65.0;
pub const GRAVITY: f32 = 300.0;

// https://coolors.co/palette/001219-005f73-0a9396-94d2bd-e9d8a6-ee9b00-ca6702-bb3e03-ae2012-9b2226
pub const COLOR_PALETTE: [Color; 10] = [
    Color::rgb(0., 0.07058824, 0.09803922),
    Color::rgb(0., 0.37254903, 0.4509804),
    Color::rgb(0.039215688, 0.5764706, 0.5882353),
    Color::rgb(0.5803922, 0.8235294, 0.7411765),
    Color::rgb(0.9137255, 0.84705883, 0.6509804),
    Color::rgb(0.93333334, 0.60784316, 0.),
    Color::rgb(0.7921569, 0.40392157, 0.007843138),
    Color::rgb(0.73333335, 0.24313726, 0.011764706),
    Color::rgb(0.68235296, 0.1254902, 0.07058824),
    Color::rgb(0.60784316, 0.13333334, 0.14901961),
];

pub const WALL_COLOR: Color = COLOR_PALETTE[0];
pub const SNAKE_COLORS: [Color; 2] = [COLOR_PALETTE[2], COLOR_PALETTE[6]];

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
