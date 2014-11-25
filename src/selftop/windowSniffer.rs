#![feature(globs)]

use x11wrapper::display::{Display};


pub struct WindowSniffer<'a> {
	pub current_window: Window,
	pub display: Display<'a>,
	pub prev_window: Option<Window>
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
    pub mouse_motion: uint,
    pub keys: uint,
}

