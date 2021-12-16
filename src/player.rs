const GRAVITY: f32 = 0.2;


pub struct PlayerPos {
    pub x: f32,
    pub y: f32,
}

pub struct Controller {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
}

impl Controller {
    pub fn get_y_vel(&self, speed: f32) -> f32 {
        let mut vel = 0f32;
        if self.w {
            vel = vel + speed;
        }
        if self.s {
            vel = vel - speed;
        }
        vel
    }

    pub fn get_x_vel(&self, speed: f32) -> f32 {
        let mut vel = 0f32;
        if self.a {
            vel = vel - speed;
        }
        if self.d {
            vel = vel + speed;
        }
        vel
    }
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
