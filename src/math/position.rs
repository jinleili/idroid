use super::view_size::ViewSize;
use super::Size;

use nalgebra_glm as glm;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }

    pub fn zero() -> Self {
        Position::new(0.0, 0.0)
    }

    pub fn is_equal_zero(&self) -> bool {
        if self.x == 0.0 && self.y == 0.0 {
            true
        } else {
            false
        }
    }

    // 加减乘除运算
    pub fn add(&self, other: &Position) -> Self {
        Position::new(self.x + other.x, self.y + other.y)
    }

    pub fn minus(&self, other: &Position) -> Self {
        Position::new(self.x - other.x, self.y - other.y)
    }

    pub fn multiply_f(&self, param: f32) -> Self {
        Position::new(self.x * param, self.y * param)
    }

    pub fn divide_f(&self, param: f32) -> Self {
        Position::new(self.x / param, self.y / param)
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Position::new(self.x + dx, self.y + dy)
    }

    // 取整
    pub fn round(&self) -> Self {
        Position::new(self.x.round(), self.y.round())
    }

    // 翻转 y 坐标
    pub fn flip_y(&self, viewport_height: u32) -> Self {
        Position::new(self.x, viewport_height as f32 - self.y)
    }

    // 基于斜率及距离，计算点的坐标
    pub fn new_by_slope_n_dis(&self, slope: f32, distance: f32) -> Self {
        Position::new(
            self.x + distance * slope.cos(),
            self.y + distance * slope.sin(),
        )
    }

    // 求矢量的模
    pub fn vector_mod(&self) -> f32 {
        (self.x.powf(2.0) + self.y.powf(2.0)).sqrt()
    }

    pub fn mod_to(&self, mod_param: f32) -> Self {
        Position::new(self.x % mod_param, self.y % mod_param)
    }

    pub fn distance(&self, other: &Position) -> f32 {
        ((self.x - other.x).powf(2.0) + (self.y - other.y).powf(2.0)).sqrt()
    }

    pub fn length(&self) -> f32 {
        (self.x.powf(2.0) + self.y.powf(2.0)).sqrt()
    }

    pub fn ortho_in(&self, view_size: ViewSize) -> Self {
        // 转换成匹配正交投影的像素坐标
        Position::new(self.x - view_size.center_x(), view_size.center_y() - self.y)
    }

    pub fn slope_with(&self, last: &Position) -> f32 {
        (self.y - last.y) / (self.x - last.x)
    }

    pub fn slope_ridian(&self, last: &Position) -> f32 {
        // atan2 求出的θ取值范围是[-PI, PI]
        let radian = (self.y - last.y).atan2(self.x - last.x);
        radian
    }

    pub fn cross_multiply(&self, other: &Position) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn into_vec2(self) -> glm::TVec2<f32> {
        glm::TVec2::new(self.x, self.y)
    }

    // 将像素坐标转换成NDC空间中的坐标
    // 这个空间可能不是当前可见视口，需要传入实际 reander target 的尺寸
    pub fn get_std_coord(&self, viewport_size: Size<f32>) -> Self {
        let half_w = viewport_size.width / 2.0;
        let half_h = viewport_size.height / 2.0;
        // 像素在NDC空间对应的值
        let x = (self.x - half_w) / half_w;
        let mut y = (self.y - half_h) / half_h;
        // 反转 y 坐标
        y *= -1.0;

        Position::new(x, y)
    }
}

impl From<Position> for [f32; 2] {
    fn from(vs: Position) -> Self {
        [vs.x, vs.y]
    }
}

impl From<[f32; 2]> for Position {
    fn from(vs: [f32; 2]) -> Self {
        Position::new(vs[0], vs[1])
    }
}

impl From<&[f32; 2]> for Position {
    fn from(vs: &[f32; 2]) -> Self {
        Position::new(vs[0], vs[1])
    }
}

impl From<(f32, f32)> for Position {
    fn from(vs: (f32, f32)) -> Self {
        Position::new(vs.0, vs.1)
    }
}

impl From<glm::TVec2<f32>> for Position {
    fn from(vec2: glm::TVec2<f32>) -> Self {
        let vs: [f32; 2] = vec2.into();
        Position::new(vs[0], vs[1])
    }
}
