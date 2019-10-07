#[derive(Clone, Copy, Debug)]
pub struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn x1(&self) -> i32 { self.x1 }
    pub fn x2(&self) -> i32 { self.x2 }
    pub fn y1(&self) -> i32 { self.y1 }
    pub fn y2(&self) -> i32 { self.y2 }
}