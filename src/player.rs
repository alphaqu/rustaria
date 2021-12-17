const GRAVITY: f32 = 0.2;

// player position
pub struct PlayerPos {
    x: f32,
    y: f32,
}

pub struct Player {
    pub pos: PlayerPos,
    pub speed: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub controller: Controller,
}

impl Player {
    pub fn tick(&mut self) {
        self.velocity_x = self.controller.get_x_vel(self.speed / 60f32);
        self.velocity_y = self.controller.get_y_vel(self.speed / 60f32);

        self.pos.x = self.pos.x + self.velocity_x;
        self.pos.y = self.pos.y + self.velocity_y;
    }
}
