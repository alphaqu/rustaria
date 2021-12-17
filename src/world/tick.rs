pub enum TickResult {
    Continue,
    Stop,
}

pub trait Tickable {
    fn tick(&self) -> TickResult;
}

pub struct TickHandler<'a> {
    data: Vec<&'a dyn Tickable>,
}

impl<'a> TickHandler<'a> {
    pub fn tick(&mut self) {
        self.data.retain(|tickable| match tickable.tick() {
            TickResult::Continue => true,
            TickResult::Stop => false,
        });
    }

    pub fn add_tickable(&mut self, tickable: &'a dyn Tickable) {
        self.data.push(tickable);
    }
}
