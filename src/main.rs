#![feature(globs)]
extern crate libc;
extern crate rustbox;


use libc::*;
use x11::xlib;
use x11::xtst;
use x11wrapper::{Display};
use std::time::Duration;
use std::old_io::Timer;
use std::ffi;
use selftop::{MotionSniffer, WindowSniffer, UserEvent};
use std::collections::HashMap;
use rustbox::{Style,Color};
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
            std::mem::transmute(ext_name),
            arg2,
            arg3,
            arg4);
        let extension = xlib::XInitExtension(
            display_control.display,
            std::mem::transmute(ext_name));
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
        rustbox::init();
        
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
        //  periodic.recv();
        //  println!(
        //      "Total {}, Key: {}, Button: {}, Motion: {} ({}) ",
        //      event_count,
        //      event_key,
        //      event_button,
        //      motion_sniffer.motion_count,
        //      event_motion);
        //  xtst::XRecordProcessReplies(display_data.display);
        // }
    }
}

extern "C" fn recordCallback(pointer:*mut i8, raw_data: *mut xtst::XRecordInterceptData) {
    unsafe {
        let sniffer: &mut WindowSniffer = std::mem::transmute(pointer);
        
        let data = &*raw_data;
        

        if data.category != xtst::XRecordFromServer {
            return;
        }
        event_count += 1;
        let xdatum = &*(data.data as *mut XRecordDatum);

        // Detect wm_name
        
        let window = get_current_window();
        // (*sniffer).processEvent(window);
        // if window.is_none() {
        //  return;
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

        if (data.server_time as uint) - prev_time > 1000 {
            prev_time = data.server_time as uint;
            redrawScreenRustBox(sniffer);

        }
        
        xtst::XRecordFreeData(raw_data);
        
    }
}

extern "C" fn errorCallback(display: *mut xlib::Display, errors: *mut xlib::XErrorEvent) -> i32 {
    return 0;
}

fn redrawScreenRustBox(sniffer: &WindowSniffer) {
    rustbox::clear();
    let mut total = 0;
    let mut current_line = 0;

    let width = rustbox::width();
    let height = rustbox::height();
    // Width of columns
    let pid_width = 5;
    let class_width = 20;
    let keys_width = 5;
    let clicks_width = 5;
    let motions_width = 5;
    let time_width = 9;
    let wmname_width = width - pid_width - class_width - keys_width - clicks_width - motions_width - time_width - 7;

    let mut items = Vec::new();

    for (window, counter) in sniffer.windows.iter() {
        total += counter.timer;

        let pid = match (*window).pid {
            Some(pid) => {
                pid
            },
            None => {0}
        };

        let class = match (*window).class {
            Some(ref class) => {
                (*class)[class.len()-1].clone()
            },
            None => {"".to_string()}
        };
        
        let wmname = match (*window).wm_name {
            Some(ref wm_name) => {
                (*wm_name).clone()
            },
            None => {"".to_string()}
        };

        items.push((pid, class, wmname, counter.keys, counter.clicks,counter.motionSniffer.motion_count, counter.timer));
    }

    // sort by time, desc order
    items.sort_by(|a, b| {let (_,_,_,_,_,_, timeA) = (*a).clone(); let (_,_,_,_,_,_, timeB) = (*b).clone(); timeB.cmp(&timeA)});

    let mut displayTotal = 0;
    for item in items.iter() {
        let (pid, class, wmname, keys, clicks, motions, timer) = (*item).clone();
        let line = format!(
            "{: <7$.7$} {: <8$.8$} {: <9$.9$} {: <10$.10$} {: <11$.11$} {: <12$.12$} {: <13$.13$}",
            pid, class, wmname, keys, clicks, motions, format_time_span(timer),
            pid_width, class_width, wmname_width, keys_width, clicks_width, motions_width, time_width
        );
        rustbox::print(0, current_line, Style::Normal, Color::Default, Color::Default, line);
        displayTotal += timer;
        current_line += 1;
        if current_line + 2 > height {
            break;
        }
    }
    rustbox::print(0, current_line, Style::Normal, Color::Default, Color::Default, format!("Total: {}, shown: {}, not shown: {}", format_time_span(total), format_time_span(displayTotal), format_time_span(total-displayTotal)));

    rustbox::present();
    match rustbox::peek_event(0) {
        rustbox::Event::KeyEvent(_, _, ch) => {
                match std::char::from_u32(ch) {

                    Some('q') => {
                        // temporary hack for quit
                        rustbox::shutdown();
                        panic!("temporary hack for quit");
                    },
                    _ => {}
                }
            },
        _ => {}
    }
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