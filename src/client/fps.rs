use std::time::Instant;

pub struct FpsCounter {
    old_fps: Instant,
    frames: u128,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        Self { old_fps: Instant::now(), frames: 0 }
    }
    pub fn tick(&mut self) {
        self.frames = self.frames + 1;
        if self.old_fps.elapsed().as_millis() > 1000 {
            println!("FPS: {}", self.frames);
            self.frames = 0;
            self.old_fps = Instant::now();
        }
    }
}