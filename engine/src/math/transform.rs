pub use glam::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
    
    pub fn contains_point(&self, point: Vec2) -> bool {
        let half_width = self.scale.x * 0.5;
        let half_height = self.scale.y * 0.5;
        
        let min_x = self.position.x - half_width;
        let max_x = self.position.x + half_width;
        let min_y = self.position.y - half_height;
        let max_y = self.position.y + half_height;
        
        point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}