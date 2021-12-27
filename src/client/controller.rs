use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};

use crate::Player;

pub struct ControlHandler {
	event_receiver: Receiver<(f64, WindowEvent)>,
	mouse_x: f64,
	mouse_y: f64,

	mappings: HashMap<KeyMapping, EventKey>,
	events: Vec<Event>,
}

impl ControlHandler {
	pub fn new(event_handler: Receiver<(f64, WindowEvent)>) -> ControlHandler {
		Self {
			event_receiver: event_handler,
			mouse_x: 0.0,
			mouse_y: 0.0,
			mappings: HashMap::new(),
			events: vec![],
		}
	}

	pub fn register(&mut self, action: Event) -> EventKey {
		let pos = self.events.len();
		let key = EventKey { id: pos as u32 };
		self.mappings.insert(action.default_key.clone(), key.clone());
		self.events.push(action);
		key
	}

	pub fn update<R, E>(&mut self, mut resize_event: R, mut scroll_event: E)
		where
			R: FnMut((i32, i32)),
			E: FnMut((f32, f32)),
	{
		for (_, event) in glfw::flush_messages(&self.event_receiver) {
			match event {
				WindowEvent::Key(_, _, action, modifiers) | WindowEvent::MouseButton(_, action, modifiers) => {
					let mapping = match event {
						WindowEvent::MouseButton(button, _, _) => {
							KeyMapping::new_mouse(button, modifiers)
						}
						WindowEvent::Key(key, _, _, _) => KeyMapping::new_board(key, modifiers),
						_ => panic!("what"),
					};
					let function = self.mappings.get(&mapping);
					function.map(|func| {
						let event = &mut self.events[func.id as usize];
						match action {
							Action::Press => match event.event_type {
								EventType::Toggle { ref mut state } => {
									*state = !*state;
								}
								EventType::Request { ref mut requests } => {
									*requests += 1;
								}
								EventType::Persistent { ref mut pressed } => {
									*pressed = true;
								}
							},
							Action::Release => match event.event_type {
								EventType::Persistent { ref mut pressed } => {
									*pressed = false;
								}
								_ => {}
							},
							// what the f*!# does this mean.
							Action::Repeat => {
							}
						}
					});
				}
				WindowEvent::CursorPos(x, y) => {
					self.mouse_x = x;
					self.mouse_y = y;
				}
				WindowEvent::Scroll(x, y) => {
					scroll_event((x as f32, y as f32));
				}
				WindowEvent::Size(width, height) => {
					resize_event((width, height));
				}
				_ => {}
			}
		}
	}

	pub fn acquire(&self, key: &EventKey) -> &EventType {
		&self.events[key.id as usize].event_type
	}

	pub fn finish(&mut self) {
		for x in &mut self.events {
			match x.event_type {
				EventType::Request { ref mut requests } => *requests = 0,
				_ => {}
			}
		}
	}
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct EventKey {
	id: u32,
}

pub struct Event {
	event_type: EventType,
	default_key: KeyMapping,
	language_key: &'static str,
}

impl Event {
	pub fn new(event_type: EventType, default_key: KeyMapping, language_key: &'static str) -> Self {
		Self {
			event_type,
			default_key,
			language_key,
		}
	}
}

pub enum EventType {
	Toggle { state: bool },
	Request { requests: u32 },
	Persistent { pressed: bool },
}

impl EventType {
	pub fn new_toggle(default: bool) -> EventType {
		EventType::Toggle { state: default }
	}

	pub fn new_request() -> EventType {
		EventType::Request { requests: 0 }
	}

	pub fn new_persistent() -> EventType {
		EventType::Persistent { pressed: false }
	}
}

pub struct ControlMetadata {
	mouse_x: i32,
	mouse_y: i32,
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum KeyMapping {
	KeyBoard {
		key: Key,
		modifiers: Modifiers,
	},
	Mouse {
		button: MouseButton,
		modifiers: Modifiers,
	},
	// your mom
	Joystick,
}

impl KeyMapping {
	pub fn new_board(key: Key, modifiers: Modifiers) -> Self {
		Self::KeyBoard { key, modifiers }
	}

	pub fn key(key: Key) -> Self {
		Self::KeyBoard {
			key,
			modifiers: Modifiers::empty(),
		}
	}

	pub fn new_mouse(button: MouseButton, modifiers: Modifiers) -> Self {
		Self::Mouse { button, modifiers }
	}
}

/// All of the game movement is handled here.
pub struct MovementAction {
	up: EventKey,
	down: EventKey,
	left: EventKey,
	right: EventKey,
}

impl MovementAction {
	pub fn new(control_handler: &mut ControlHandler) -> MovementAction {
		MovementAction {
			up: control_handler.register(Event::new(
				EventType::new_persistent(),
				KeyMapping::key(Key::W),
				"ui.up",
			)),
			down: control_handler.register(Event::new(
				EventType::new_persistent(),
				KeyMapping::key(Key::S),
				"ui.down",
			)),
			left: control_handler.register(Event::new(
				EventType::new_persistent(),
				KeyMapping::key(Key::A),
				"ui.left",
			)),
			right: control_handler.register(Event::new(
				EventType::new_persistent(),
				KeyMapping::key(Key::D),
				"ui.right",
			)),
		}
	}

	pub fn event_apply(&self, control_handler: &ControlHandler, player: &mut Player) {
		let mut vel_x = 0f32;
		let mut vel_y = 0f32;

		if let EventType::Persistent { pressed } = control_handler.acquire(&self.up) {
			if *pressed {
				vel_y += 1f32;
			}
		}

		if let EventType::Persistent { pressed } = control_handler.acquire(&self.down) {
			if *pressed {
				vel_y -= 1f32;
			}
		}

		if let EventType::Persistent { pressed } = control_handler.acquire(&self.left) {
			if *pressed {
				vel_x -= 1f32;
			}
		}

		if let EventType::Persistent { pressed } = control_handler.acquire(&self.right) {
			if *pressed {
				vel_x += 1f32;
			}
		}

		player.vel_x = vel_x;
		player.vel_y = vel_y;
	}
}