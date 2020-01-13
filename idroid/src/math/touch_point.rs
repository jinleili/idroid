#[derive(Copy, Clone, Debug)]
pub struct TouchPoint {
    pub pos: super::Position,
    pub force: f32,
    // 基于压力及笔刷大小计算出来
    pub stamp: f32,
    // 与上一点的距离
    pub distance: f32,
    // 产生点的时间间隔
    // 需要在 native 层计算好，时间是双精度数，传递到 rust 后再计算容易导致数据错误
    pub interval: f32,
    // 每 0.016 秒移动的距离，
    // 速度用于计算笔画粗细及下墨的多少
    pub speed: f32,
    // ty = 1: 表示为结束点
    // 开始点通过是不是第一二个点能识别出来，不需要单独标记
    pub ty: i32,
}

impl TouchPoint {
    pub fn new(pos: crate::math::Position, force: f32, interval: f32) -> Self {
        TouchPoint { pos, force, stamp: 0.0, distance: 0.0, interval, speed: 0.0, ty: -1 }
    }

    // 生成结束点
    pub fn new_end(pos: crate::math::Position) -> Self {
        TouchPoint { pos, force: 0.0, stamp: 0.0, distance: 0.0, interval: 0.0, speed: 0.0, ty: 1 }
    }

    // 通过上一点更新当前点的信息
    pub fn update(&mut self, last: &TouchPoint) {
        // let interval = self.timestamp - last.timestamp;
        let dis = self.pos.distance(&last.pos);
        self.speed = dis / self.interval;
        self.distance = dis;
        // println!("interval: {}, speed: {}", last.interval, self.speed);
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
