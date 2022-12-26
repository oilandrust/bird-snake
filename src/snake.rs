use bevy::prelude::*;
use std::collections::VecDeque;

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
