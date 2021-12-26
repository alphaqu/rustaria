use crate::misc::util::CallbackResponse;

pub trait Tickable {
    fn tick(&self) -> CallbackResponse;
}

pub struct TickHandler<'a> {
    data: Vec<&'a dyn Tickable>,
}

impl<'a> TickHandler<'a> {
    pub fn new() -> TickHandler<'a> {
        Self {
            data: Vec::new()
        }
    }

    pub fn tick(&mut self) {
        self.data.retain(|tickable| match tickable.tick() {
            CallbackResponse::Continue => true,
            CallbackResponse::Stop => false,
        });
    }

    pub fn add_tickable(&mut self, tickable: &'a dyn Tickable) {
        self.data.push(tickable);
    }
}
