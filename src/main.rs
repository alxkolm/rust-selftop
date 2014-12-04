#![feature(globs)]
extern crate libc;
use libc::*;
use x11::xlib;
use x11::xtst;
use x11wrapper::{Display};
use std::time::Duration;
use std::io::Timer;
use selftop::{MotionSniffer, WindowSniffer, UserEvent};
use std::collections::HashMap;
mod x11;
mod x11wrapper;
mod selftop;

struct XRecordDatum {
    xtype: u8,
    code: u8,
    unknown1: u8,
    unknown2: u8
}

static mut display_control: Display<'static> = Display {display: 0 as *mut xlib::Display};
static mut display_data: x11wrapper::Display<'static> = x11wrapper::Display {display: 0 as *mut xlib::Display};

static mut event_count:u64 = 0;
static mut event_key:u64 = 0;
static mut event_button:u64 = 0;
static mut event_motion:u64 = 0;
static mut prev_time:uint = 0;

static mut motion_sniffer: MotionSniffer = MotionSniffer{
	last_event_time: 0,
	motion_count: 0
};

struct Exchange {
	a: int
}

fn main() {
	// Start X Record event loop
	xRecordBootstrap();
}

fn xRecordBootstrap () {
	unsafe {
		let mut a:  i8 = 0;
		display_control = Display::new();
		display_data = Display::new();

		xlib::XSetErrorHandler(Some(errorCallback));

		xlib::XSynchronize(display_control.display, 1);

		
		// Check presence of Record extension
		let ext_name = "RECORD";
		let arg2:*mut c_int = &mut 1;
		let arg3:*mut c_int = &mut 1;
		let arg4:*mut c_int = &mut 1;
		let has_record = xlib::XQueryExtension(
			display_control.display,
			ext_name.to_c_str().as_ptr() as *const i8,
			arg2,
			arg3,
			arg4);
		let extension = xlib::XInitExtension(
			display_control.display,
			ext_name.to_c_str().as_ptr() as *const i8);
		if extension.is_null() {
			panic!("XInitExtension() failed!");
		}

		// Get version
		let mut versionMajor: c_int = 0;
		let mut versionMinor: c_int = 0;
		xtst::XRecordQueryVersion(
			display_control.display,
			&mut versionMajor,
			&mut versionMinor);
		println!("RECORD extension version {}.{}", versionMajor, versionMinor);

		// Prepare record range
		let mut recordRange: xtst::XRecordRange = *xtst::XRecordAllocRange();
		let mut recordRangePtr: *mut *mut xtst::XRecordRange = std::mem::transmute(&mut &mut recordRange);
		recordRange.device_events.first = xtst::KeyPress; // KeyPress
		recordRange.device_events.last = xtst::MotionNotify; // MotionNotify
		recordRange.delivered_events.first = xtst::EnterNotify;
		recordRange.delivered_events.last = xtst::EnterNotify;
		
		// Create context
		let context = xtst::XRecordCreateContext(
			display_control.display,
			0,
			&mut xtst::XRecordAllClients,
			1,
			recordRangePtr,
			1
		);
		if context == 0 {
			panic!("Fail create Record context\n");
		}

		let mut windowSniffer = WindowSniffer::new();
		// Run
		let res = xtst::XRecordEnableContext(display_data.display, context, Some(recordCallback), std::mem::transmute(&mut windowSniffer));
		if res == 0 {
			panic!("Cound not enable the Record context!\n");
		}
		xtst::XRecordFreeContext(display_data.display, context);

		// without this timer process consume 100% CPU
		// let mut timer = Timer::new().unwrap();
		// let periodic = timer.periodic(Duration::milliseconds(1000));
		// loop {
		// 	periodic.recv();
		// 	println!(
		// 		"Total {}, Key: {}, Button: {}, Motion: {} ({}) ",
		// 		event_count,
		// 		event_key,
		// 		event_button,
		// 		motion_sniffer.motion_count,
		// 		event_motion);
		// 	xtst::XRecordProcessReplies(display_data.display);
		// }
	}
}

extern "C" fn recordCallback(pointer:*mut i8, raw_data: *mut xtst::XRecordInterceptData) {
	unsafe {
		let sniffer: &mut WindowSniffer = std::mem::transmute(pointer);
		
		let data = &*raw_data;
		prev_time = data.server_time as uint;

		if data.category != xtst::XRecordFromServer {
			return;
		}
		event_count += 1;
		let xdatum = &*(data.data as *mut XRecordDatum);

		// Detect wm_name
		
		let window = get_current_window();
		// (*sniffer).processEvent(window);
		// if window.is_none() {
		// 	return;
		// }
		 
		// Count events
		let mut event = match xdatum.xtype {
			xtst::KeyPress     => Some(UserEvent::KeyEvent{time: data.server_time as uint, keycode: 1}),
			xtst::ButtonPress  => Some(UserEvent::ClickEvent{time: data.server_time as uint, buttoncode: 1}),
			xtst::MotionNotify => Some(UserEvent::MotionEvent{time: data.server_time as uint}),
			xtst::EnterNotify  => Some(UserEvent::EnterEvent{time: data.server_time as uint}),
			_                  => None
		};

		match event {
			Some(e) => {
				(*sniffer).processEvent(window, e);
			},
			_ => {}
		}

		redrawScreen(sniffer);
		xtst::XRecordFreeData(raw_data);
	}
}

extern "C" fn errorCallback(display: *mut xlib::Display, errors: *mut xlib::XErrorEvent) -> i32 {
	return 0;
}

fn redrawScreen(sniffer: &WindowSniffer) {
	let mut out = std::io::stdout();
	// Clear screen
	out.write(b"\x1B[2J\x1B[H\x1B[?25l");
	let mut total = 0;
	for (window, counter) in sniffer.windows.iter() {
		// println!("{}", (*window).wm_name);
		// match (*window).wm_name {
		// 	Some(ref wmname) => {
		// 		out.write((*wmname).as_bytes());
		// 	},
		// 	None => {}
		// };
		match (*window).pid {
			Some(ref pid) => {
				out.write((*pid).to_string().as_bytes());
			},
			None => {}
		};

		out.write(b"\t");
		out.write(counter.keys.to_string().as_bytes());
		out.write(b"\t");
		out.write(counter.clicks.to_string().as_bytes());
		out.write(b"\t");
		out.write(counter.motionSniffer.motion_count.to_string().as_bytes());
		out.write(b"\t");
		out.write(format_time_span(counter.timer).as_bytes());
		out.write(b"\n");

		total += counter.timer;
	}
	out.write(b"Total\t");
	out.write(format_time_span(total).as_bytes());
}

fn get_current_window() -> selftop::Window {
	let mut current_window = unsafe {display_control.get_input_focus()};
	let mut parent_window: Option<x11wrapper::window::Window> = None;
	let mut wm_name_str: Option<String> = None;
	
	let mut i = 0u;
	while i < 10 {
		if current_window.id == 0  || current_window.id == 1 {
			break;
		}
		
		wm_name_str = current_window.get_wm_name();
		if wm_name_str.is_none() || wm_name_str.clone().unwrap() == "FocusProxy".to_string() {
			// If not found or wmname is "FocusProxy" dig up to tree
			let tree = current_window.get_tree();
			parent_window = match tree {
				Some(x11wrapper::window::WindowTree{parent: parent, children: _}) => {
					Some(parent)
				},
				_ => None
			}
		} else {
			// Found window with adequate WM_NAME.
			// Exit from while loop.
			break;
		}
					
		current_window = match parent_window {
			Some(win) => win,
			_ => current_window
		};
		
		i += 1;
	}

	selftop::Window {
		wm_name: current_window.get_wm_name(),
		class: current_window.get_class(),
		pid: current_window.get_pid(),
	}
}

fn format_time_span(timeMs: uint) -> String
{
	let secs = timeMs / 1000;
	
	let hours = secs/ (60*60);
	let mins = (secs - (hours*60*60))/60;
	let seconds = secs - (hours*60*60) - (mins*60);
	format!("{}h{}m{}s", hours, mins, seconds)
}