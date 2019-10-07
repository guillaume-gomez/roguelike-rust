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

  pub fn center(&self) -> (i32, i32) {
    let center_x = (self.x1 + self.x2) / 2;
    let center_y = (self.y1 + self.y2) / 2;
    (center_x, center_y)
  }

  pub fn intersects_with(&self, other: &Rect) -> bool {
    // returns true if this rectangle intersects with another one
    (self.x1 <= other.x2)
    && (self.x2 >= other.x1)
    && (self.y1 <= other.y2)
    && (self.y2 >= other.y1)
  }
}