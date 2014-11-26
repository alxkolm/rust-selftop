#![feature(globs)]

use x11wrapper::display::{Display};
use std::collections::HashMap;
use selftop::motionSniffer::*;



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
		// add window if not exists
		if !self.windows.contains_key(&window) {
			let mut c = Counter{
				motionSniffer: MotionSniffer {
					last_event_time: 0,
					motion_count: 0
				},
				keys: 0,
				clicks: 0
			};
			self.windows.insert(window.clone(), c);
		}

		// get counter
		let mut counter = self.windows.get_mut(&window);

		match counter {
			Some(ref mut c) => {
				match event {
					UserEvent::MotionEvent{time} => {
						(*c).motionSniffer.processEvent(time);
					},
					UserEvent::KeyEvent{keycode, time} => {
						(*c).keys += 1;
					},
					UserEvent::ClickEvent{buttoncode, time} => {
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
    pub keys: uint,
    pub clicks: uint,
    pub motionSniffer: MotionSniffer
}

pub enum UserEvent {
	MotionEvent{time: uint},
	KeyEvent{keycode: u8, time: uint},
	ClickEvent{buttoncode: u8, time: uint}
}