use crate::math::{Position, ViewSize, Size};

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub origin: Position,
    pub size: Size<f32>,
}

impl Rect {
    pub fn new(width: f32, height: f32, center_to: ViewSize) -> Self {
        let x = (center_to.width - width) / 2.0;
        let y = (center_to.height - height) / 2.0;
        Rect { x, y, width, height, origin: Position::new(x, y), size: (width, height).into() }
    }

    pub fn get_standard_new() -> Self {
        Rect {x: 0.0, y: 0.0, width: 1.0, height: 1.0, origin: Position::zero(), size: (1.0, 1.0).into() }
    }

    pub fn from_origin_n_size(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect { x, y, width, height, origin: Position::new(x, y), size: (width, height).into() }
    }

    pub fn zero() -> Self {
        Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0, origin: Position::zero(), size: (0.0, 0.0).into() }
    }

    pub fn center_x(&self) -> f32 {
        self.width / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.height / 2.0
    }

    // 一个正交投影坐标是否在区域内
    pub fn is_ortho_intersect(&self, ortho_point: Position) -> bool {
        let x_left = -self.center_x();
        let x_right = self.center_x();
        let y_top = self.center_y();
        let y_bottom = -self.center_y();
        if ortho_point.x >= x_left && ortho_point.x <= x_right && ortho_point.y >= y_bottom && ortho_point.y <= y_top {
            true
        } else {
            false
        }
    }
}
