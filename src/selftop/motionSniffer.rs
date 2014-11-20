// mod x11wrapper::window::{Window};
type Time = u64;
pub struct MotionSniffer {
    pub last_event_time: Time,
    // pub last_window: Window,
    pub motion_count: u64
}

impl MotionSniffer {
	pub fn processEvent(&mut self, time: Time){
	
		let delta = time - self.last_event_time;

		if delta > 200 {
			self.motion_count += 1;
		}

		self.last_event_time = time;
	}
}