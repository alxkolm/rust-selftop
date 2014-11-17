// mod x11wrapper::window::{Window};
type Time = u64;
pub struct MotionSniffer {
    pub last_event_time: Time,
    // pub last_window: Window,
    pub motion_count: u64
}

impl MotionSniffer {
	fn new () -> MotionSniffer{
		MotionSniffer{
			last_event_time: 0,
			motion_count: 0
		}
	}
	fn processEvent(&self, time: Time){
		let delta = time - self.last_event_time;
		self.last_event_time = time;

		if delta > 100 {
			self.motion_count += 1;
		}
	}
}