use bevy::prelude::*;
use std::collections::VecDeque;

use crate::{
    game_constants_pluggin::{to_world, SNAKE_SIZE},
    level::Level,
    level_pluggin::{LevelEntity, StartLevelEvent},
};

#[derive(Component)]
pub struct SnakePart(pub usize);

#[derive(Component)]
pub struct Snake {
    pub parts: VecDeque<(IVec2, IVec2)>,
}

impl Snake {
    pub fn from_parts(parts: Vec<(IVec2, IVec2)>) -> Self {
        Self {
            parts: VecDeque::from(parts),
        }
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn head_position(&self) -> IVec2 {
        self.parts.front().unwrap().0
    }

    pub fn is_standing(&self) -> bool {
        (self.parts.front().unwrap().0.y - self.parts.back().unwrap().0.y)
            == (self.len() - 1) as i32
    }

    pub fn occupies_position(&self, position: IVec2) -> bool {
        self.parts.iter().any(|part| part.0 == position)
    }

    pub fn fall_one_unit(&mut self) {
        for (position, _) in self.parts.iter_mut() {
            *position += IVec2::NEG_Y;
        }
    }
}

pub fn spawn_snake_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEvent>,
    level: Res<Level>,
) {
    if event_start_level.iter().next().is_none() {
        return;
    }

    for (index, part) in level.initial_snake.iter().enumerate() {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,
                    custom_size: Some(SNAKE_SIZE),
                    ..default()
                },
                transform: Transform {
                    translation: to_world(part.0).extend(0.0),
                    ..default()
                },
                ..default()
            })
            .insert(SnakePart(index))
            .insert(LevelEntity);
    }

    commands
        .spawn(Snake::from_parts(level.initial_snake.clone()))
        .insert(LevelEntity);
}
