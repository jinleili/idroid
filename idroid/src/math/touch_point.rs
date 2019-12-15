#[derive(Copy, Clone, Debug)]
pub struct TouchPoint {
    pub pos: super::Position,
    pub force: f32,
}