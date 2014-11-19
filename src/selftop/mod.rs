// mod x11wrapper::window::{Window};
type Time = u64;
pub struct MotionSniffer {
    pub last_event_time: Time,
    // pub last_window: Window,
    pub motion_count: u64
}

impl MotionSniffer {
	// pub fn new () -> MotionSniffer<'a>{
	// 	// let time: Time = 0;
	// 	// let mcount: u64 =0;
	// 	MotionSniffer{
	// 		last_event_time: &mut 0,
	// 		motion_count: &mut 0
	// 	}
	// }
	pub fn processEvent(&mut self, time: Time){
	
		let delta = time - self.last_event_time;

		if delta > 200 || self.last_event_time == 0 {
			self.motion_count += 1;
		}
		
		self.last_event_time = time;
	}
}