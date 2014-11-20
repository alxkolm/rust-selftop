#![feature(globs)]

use x11wrapper::display::{Display};


pub struct WindowSniffer<'a> {
	pub current_window: Window,
	pub display: Display<'a>,
}

pub struct Window {
    pub wm_name: String,
    pub class: String,
}

impl Window {
	pub fn get_current_window(){

	}
}