pub struct Drag {
    start_x: f32,
    start_y: f32,
    cur_x: f32,
    cur_y: f32,
    passed_threshold: bool,
}

impl Drag {
    const THRESHOLD: f32 = 7.0;

    pub fn new(start_x: f32, start_y: f32) -> Self {
        Self {
            start_x,
            start_y,
            cur_x: start_x,
            cur_y: start_y,
            passed_threshold: false,
        }
    }

    pub fn passed_threshold(&self) -> bool {
        self.passed_threshold
    }

    // TODO change to translation?
    pub fn get_map_offset_delta(&mut self, mouse_x: f32, mouse_y: f32) -> (f32, f32) {
        let delta_x = mouse_x - self.cur_x;
        let delta_y = mouse_y - self.cur_y;

        // println!("mouse {}, {}  cur {}, {}  delta {}, {}", mouse_x, mouse_y, self.cur_x, self.cur_y, delta_x, delta_y);

        if !self.passed_threshold {
            if ((self.start_x - mouse_x).powi(2) + (self.start_y - mouse_y).powi(2)).sqrt() > Self::THRESHOLD {
                self.passed_threshold = true;
            }
        }

        if self.passed_threshold {
            self.cur_x = mouse_x;
            self.cur_y = mouse_y;
            (delta_x, delta_y)
        } else {
            (0.0, 0.0)
        }
    }
}
