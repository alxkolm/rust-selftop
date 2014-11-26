// mod x11wrapper::window::{Window};

pub struct MotionSniffer {
    pub last_event_time: uint,
    pub motion_count: uint
}

impl MotionSniffer {
	pub fn processEvent(&mut self, time: uint){
	
		let delta = time - self.last_event_time;

		if delta > 200 {
			self.motion_count += 1;
		}

		self.last_event_time = time;
	}
}