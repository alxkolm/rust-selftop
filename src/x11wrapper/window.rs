extern crate libc;
use x11::xlib;
use x11::xlibint;
use std::mem;
use std::ffi;

pub struct Window<'a> {
    pub id: uint, // XID
    pub display: *mut xlib::Display
}

impl<'a> Window<'a> {
	pub fn get_wm_name(&self) -> Option<String> {
		// let mut a:String = String::new();
		// let wmname = unsafe {
			// let mut window_name: *mut i8 = 0 as *mut i8;
			// let res = xlib::XFetchName(self.display, self.id, &mut window_name);
			// if res != 0 {
			// 	let c_wm_name = CString::new(std::mem::transmute(window_name), false);
			// 	// xlib::XFree(&mut window_name);
			// 	Some(String::from_str(c_wm_name.as_str().unwrap()))
			// } else {
			// 	// Try get _NET_WM_NAME
			// 	None
			// }
			let wmname_c = self.get_property("_NET_WM_NAME", "UTF8_STRING");
			match wmname_c {
				Some(bytes) => match String::from_utf8(bytes){
					Ok(value) => Some(value),
					Err(err) => {None}
				},
				None => None
			}
		// };
		// wmname
	}
	pub fn get_class(&self) -> Option<Vec<String>> {
		let property_c = self.get_property("WM_CLASS", "STRING");

		let chunks = match property_c{
			Some(b) => Window::splitByNullByte(b),
			None => None
		};
		let mut strings: Vec<String> = Vec::new();
		match chunks {
			Some(ref items) => {
				for bytes in items.iter() {
					match String::from_utf8((*bytes).clone()) {
						Ok(value) => {
							strings.push(value);
						}
						Err(_) => {}
					}
				}
			},
			None => {}
		}
		match strings.len() {
			0 => None,
			_ => Some(strings)
		}
	}
	pub fn get_pid(&self) -> Option<u32> {
		let property_c = self.get_property("_NET_WM_PID", "CARDINAL");
		match property_c {
			Some(b) => {
				// hack!
				// convert Vec<u8> to u32
				let mut a: [u8; 4] = [0,0,0,0];
				let mut i = 0;
				for j in b.iter() {
					a[i] = *j;
					i += 1;
				}
				let pid:u32 = unsafe {mem::transmute(a)};
				Some(pid)
			},
			_ => None
		}
	}
	pub fn get_property(&self, property_name: &str, property_type: &str) -> Option<Vec<u8>>{
		if self.id == 0 {
			return None;
		}
		unsafe {
			let xa_property_type: xlibint::Atom = xlib::XInternAtom(self.display, ffi::CString::from_slice(property_type.as_bytes()).as_ptr(), 0);
			let xa_property_name: xlibint::Atom = xlib::XInternAtom(self.display, ffi::CString::from_slice(property_name.as_bytes()).as_ptr(), 0);
			let mut actual_type_return  : xlibint::Atom     = 0;
			let mut actual_format_return: libc::c_int       = 0;
			let mut nitems_return       : libc::c_ulong     = 0;
			let mut bytes_after_return  : libc::c_ulong     = 0;
			let mut tmp                 : libc::c_uchar     = 0u8;
			let mut prop_return         : *mut libc::c_uchar = mem::transmute(&mut tmp);
			let res = xlib::XGetWindowProperty(
				self.display,
				mem::transmute(self.id),
				xa_property_name,
				0,
				4096 / 4,
				0,
				xa_property_type,
				&mut actual_type_return,
				&mut actual_format_return,
				&mut nitems_return,
				&mut bytes_after_return,
				&mut prop_return
				);
			if (xa_property_type != actual_type_return) {
				// println!("Invalid type of {} property", property_name);
				return None;
			}
			let tmp_size = ((actual_format_return as uint) / 8) * (nitems_return as uint);
			
			// let data = c_vec::CVec::new(prop_return, tmp_size as uint);
			let data = ffi::c_str_to_bytes(mem::transmute(prop_return));
			let mut copy_data = Vec::with_capacity(tmp_size as uint);
			for b in data.iter() {
				copy_data.push(*b);
			}
			
			xlib::XFree(prop_return as *mut libc::types::common::c95::c_void);
			
			Some(copy_data)
		}
	}

	fn splitByNullByte (buf: Vec<u8>) -> Option<Vec<Vec<u8>>> {
		let mut strings: Vec<Vec<u8>> = Vec::new();
		let mut current_string = Vec::new();
		for b in buf.iter() {
			if (*b != 0) {
				current_string.push(*b);
			} else {
				strings.push(current_string.clone());
				current_string = Vec::new();
			}
		}
		if (current_string.len() > 0) {
			strings.push(current_string.clone());
		}
		
		match strings.len() {
			0 => None,
			_ => Some(strings)
		}
	}
	
	pub fn get_tree (&self) -> Option<WindowTree> {
		unsafe {
			let mut root: xlib::Window = 0;
			let mut parent: xlib::Window = 0;
			let mut children: *mut xlib::Window = mem::transmute(&mut 0u);
			let mut nchildren: u32 = 0;

			let res = xlib::XQueryTree(
				self.display,
				mem::transmute(self.id),
				&mut root,
				&mut parent,
				&mut children,
				&mut nchildren);

			match res {
				0 => None,
				_ => {
					let childs = match nchildren {
						0 => None,
						_ => {
							// let c = std::c_vec::CVec::new(children, nchildren);
							let mut b: Vec<Window> = Vec::new();
							for i in range(0, nchildren as int){
								b.push(Window{
									id: mem::transmute(*children.offset(i)),
									display: self.display
								});
							}
							Some(b)
						}
					};

					Some(WindowTree {
						parent: Window{
							id: parent as uint,
							display: self.display,
						},
						children: childs
					})
				}
			}
		}
	}
}

pub struct WindowTree<'a> {
    pub parent: Window<'a>,
    pub children: Option<Vec<Window<'a>>>,
}