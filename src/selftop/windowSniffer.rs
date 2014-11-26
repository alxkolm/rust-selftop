#![feature(globs)]

use x11wrapper::display::{Display};
use std::collections::HashMap;


pub struct WindowSniffer<'a> {
	pub current_window: Option<Window>,
	pub prev_window: Option<Window>,
	pub windows: HashMap<Window, Counter>
}

impl<'a> WindowSniffer<'a> {
	pub fn new() -> WindowSniffer<'a> {
		WindowSniffer {
			current_window: None,
			prev_window: None,
			windows: HashMap::new()
		}
	}
	pub fn processEvent(&mut self, window: Window, event: UserEvent) {
		if !self.windows.contains_key(&window) {
			let mut c = Counter{mouse_motions: 0, keys: 0, clicks: 0};
			self.windows.insert(window.clone(), c);
		}

		let mut counter = self.windows.get_mut(&window);

		match counter {
			Some(ref mut c) => {
				match event {
					MotionEvent => {
						(*c).mouse_motions += 1;
					},
					KeyEvent(keycode) => {
						(*c).keys += 1;
					},
					ClickEvent(buttoncode) => {
						(*c).clicks += 1;
					}
				}
				
			},
			None => {}
		}
		
	}
}



#[deriving(Hash, Eq, Clone)]
pub struct Window {
    pub wm_name: Option<String>,
    pub class: Option<Vec<String>>,
}

impl PartialEq for Window {
	fn eq(&self, other: &Window) -> bool {
		self.wm_name == other.wm_name
	}
}

pub struct Counter {
    pub mouse_motions: uint,
    pub keys: uint,
    pub clicks: uint,
}

pub enum UserEvent {
	MotionEvent,
	KeyEvent{keycode: u8},
	ClickEvent{buttoncode: u8}
}