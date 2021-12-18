use glfw::Key;

use crate::Player;

pub struct ControlHandler {
    pub up: MovementKey,
    pub down: MovementKey,
    pub left: MovementKey,
    pub right: MovementKey,
}

impl ControlHandler {
    pub fn new() -> ControlHandler {
        Self {
            up: MovementKey {
                direction: Direction::UP,
                key: Key::W,
                state: false
            },
            down: MovementKey {
                direction: Direction::DOWN,
                key: Key::S,
                state: false
            },
            left: MovementKey {
                direction: Direction::LEFT,
                key: Key::A,
                state: false
            },
            right: MovementKey {
                direction: Direction::RIGHT,
                key: Key::D,
                state: false
            }
        }
    }
    pub fn tick(&self, player: &mut Player) {
        if self.up.state {
            player.pos_y = player.pos_y + player.speed;
        }
        if self.down.state {
            player.pos_y = player.pos_y - player.speed;
        }
        if self.left.state {
            player.pos_x = player.pos_x - player.speed;
        }
        if self.right.state {
            player.pos_x = player.pos_x + player.speed;
        }
    }
}

pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct MovementKey {
    direction: Direction,
    key: Key,
    pub state: bool,
}



