#[derive(Copy, Clone, Debug)]
pub struct TouchPoint {
    pub pos: super::Position,
    pub force: f32,
    // 基于压力及笔刷大小计算出来
    pub stemp: f32,
    // ty = 1: 表示为结束点
    // 开始点通过是不是第一二个点能识别出来，不需要单独标记
    pub ty: i32,
}

impl TouchPoint {
    pub fn new(pos: crate::math::Position, force: f32) -> Self {
        TouchPoint {
            pos,
            force,
            stemp: 0.0,
            ty: -1,
        }
    }

    // 生成结束点
    pub fn new_end(pos: crate::math::Position) -> Self {
        TouchPoint {
            pos,
            force: 0.0,
            stemp: 0.0,
            ty: 1,
        }
    }

    // 是否为结束点
    pub fn is_the_end(&self) -> bool {
        if self.ty == 1 {
            true
        } else {
            false
        }
    }
}
