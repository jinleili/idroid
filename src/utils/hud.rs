use std::time::{Duration, Instant};

// const fixed_time_stamp: Duration = Duration::new(0, 16666667);

pub struct HUD {
    frame_start: Instant,
}

#[allow(dead_code)]
impl HUD {
    pub fn new() -> Self {
        HUD { frame_start: Instant::now() }
    }

    pub fn start_frame_timer(&mut self) {
        self.frame_start = Instant::now();
    }

    pub fn stop_frame_timer(&mut self) -> Duration {
        Instant::now() - self.frame_start
    }
}
