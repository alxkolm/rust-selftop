use x11::xlib;
use x11wrapper::window::{Window};

pub struct Display<'a> {
    pub display: *mut xlib::Display,
}

impl<'a> Display<'a> {
	pub fn new() -> Display<'a> {
		Display {display: unsafe {
			let mut a:  i8 = 0;
			let dpy = xlib::XOpenDisplay(&a);
			if dpy.is_null() {
				panic!("XOpenDisplay() failed!");
			}
			dpy
		}}
	}
	pub fn get_input_focus(&self) -> Window{
		let current_window: *mut xlib::Window = &mut 0;
		let revert_to_return: *mut i32 = &mut 0;
		unsafe{xlib::XGetInputFocus(self.display, current_window, revert_to_return)};
		Window {id: unsafe{*current_window as uint}, display: self.display}
	}

	pub fn window(&self, xid: uint) -> Window {
		Window {id: xid, display: self.display}
	}
}