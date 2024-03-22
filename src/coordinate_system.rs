pub struct CoordinateSystem {
    frame_width: f32,
    frame_height: f32,
    scale: f32
}

impl CoordinateSystem {
    // 8k Image resolution: 8262×5803
    pub const REL_X: f32 = 1.0 / 8262.0;
    pub const REL_Y: f32 = 1.0 / 5803.0;

    pub fn new(frame_width: f32, frame_height: f32, scale: f32) -> Self {
        Self {
            frame_width,
            frame_height,
            scale
        }
    }

    pub fn x_dist_pixels(&self, dist: f32) -> f32 {
        self.x_dist_percent(dist * Self::REL_X)
    }

    pub fn y_dist_pixels(&self, dist: f32) -> f32 {
        self.y_dist_percent(dist * Self::REL_Y)
    }

    pub fn x_dist_percent(&self, percent: f32) -> f32 {
        percent * self.frame_width * self.scale
    }

    pub fn y_dist_percent(&self, percent: f32) -> f32 {
        percent * self.frame_height * self.scale
    }
}