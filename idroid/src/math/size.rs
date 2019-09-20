#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub x: u32,
    pub y: u32,
}

impl Size {
    pub fn new(x: u32, y: u32) -> Self {
        Size {x, y}
    }

    pub fn count(&self) -> u32 {
        self.x * self.y
    }
}

impl From<[u32; 2]> for Size {
    fn from(vs: [u32; 2]) -> Self {
        Size::new(vs[0], vs[1])
    }
}

impl From<Size> for [u32; 2] {
    fn from(s: Size) -> Self {
        [s.x, s.y]
    }
}

impl From<(u32, u32)> for Size {
    fn from(data: (u32, u32)) -> Self {
        Size::new(data.0, data.1)
    }
}

impl From<Size> for (u32, u32) {
    fn from(s: Size) -> Self {
        (s.x, s.y)
    }
}
