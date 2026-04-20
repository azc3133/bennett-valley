pub const TILE_SIZE: f32 = 32.0;

/// Camera pixel position. `(x, y)` is the world pixel coordinate at the
/// top-left corner of the screen.
///
/// Tiles are drawn at:   `(col * TILE_SIZE - cam.x,  row * TILE_SIZE - cam.y)`
/// Player is drawn at:   `(screen_w / 2.0,  screen_h / 2.0)`  — always fixed.
///
/// Only `pos` interpolates. `target` jumps instantly when the player moves.
#[derive(Debug, Clone, Default)]
pub struct Camera {
    pub pos_x: f32,
    pub pos_y: f32,
    target_x: f32,
    target_y: f32,
    initialized: bool,
}

impl Camera {
    /// Compute the pixel origin that centres tile `(col, row)` on screen.
    fn tile_center_origin(col: usize, row: usize, sw: f32, sh: f32) -> (f32, f32) {
        (
            col as f32 * TILE_SIZE - sw / 2.0 + TILE_SIZE / 2.0,
            row as f32 * TILE_SIZE - sh / 2.0 + TILE_SIZE / 2.0,
        )
    }

    /// Call this whenever the player's logical tile changes.
    /// On the very first call (before the first render) the camera also snaps
    /// so there is no initial slide.
    pub fn set_target(&mut self, col: usize, row: usize, sw: f32, sh: f32) {
        let (tx, ty) = Self::tile_center_origin(col, row, sw, sh);
        self.target_x = tx;
        self.target_y = ty;
        if !self.initialized {
            self.pos_x = tx;
            self.pos_y = ty;
            self.initialized = true;
        }
    }

    /// Smoothly ease camera toward target using exponential interpolation.
    /// Feels smooth at all distances — fast when far, gentle when close.
    pub fn update(&mut self, dt: f32) {
        let smoothing = 5.0; // higher = faster catch-up
        let t = 1.0 - (-smoothing * dt).exp(); // exponential ease
        self.pos_x += (self.target_x - self.pos_x) * t;
        self.pos_y += (self.target_y - self.pos_y) * t;
        // Snap when very close to avoid sub-pixel jitter
        if (self.target_x - self.pos_x).abs() < 0.5 { self.pos_x = self.target_x; }
        if (self.target_y - self.pos_y).abs() < 0.5 { self.pos_y = self.target_y; }
    }

    /// World tile → screen pixel (top-left of tile).
    pub fn world_to_screen(&self, col: usize, row: usize) -> (f32, f32) {
        (
            col as f32 * TILE_SIZE - self.pos_x,
            row as f32 * TILE_SIZE - self.pos_y,
        )
    }

    /// Set target with a horizontal pixel offset (for split-screen centering).
    pub fn set_target_split(&mut self, col: usize, row: usize, sw: f32, sh: f32, off_x: f32) {
        let (tx, ty) = Self::tile_center_origin(col, row, sw, sh);
        self.target_x = tx + off_x;
        self.target_y = ty;
        if !self.initialized {
            self.pos_x = self.target_x;
            self.pos_y = self.target_y;
            self.initialized = true;
        }
    }

    /// Camera offset in world pixels (for converting world pixel coords to screen).
    pub fn offset_x(&self) -> f32 { self.pos_x }
    pub fn offset_y(&self) -> f32 { self.pos_y }
}
