use crate::game::Game;
use tcod::colors::Color;
use tcod::console::Console;
use tcod::console::BackgroundFlag;


#[derive(Debug)]
pub struct Object {
  x: i32,
  y: i32,
  char: char,
  color: Color,
}

impl Object {
  pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
    Object { x, y, char, color }
  }

  pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
    if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].is_blocked() {
      self.x += dx;
      self.y += dy;
    }
  }

  pub fn draw(&self, con: &mut dyn Console) {
    con.set_default_foreground(self.color);
    con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }

  pub fn set_pos(&mut self, x: i32, y: i32) {
    self.x = x;
    self.y = y;
  }

  pub fn get_x(&self) -> i32 {
    self.x
  }

  pub fn get_y(&self) -> i32{
    self.y
  }

  pub fn set_x(&mut self, x: i32) {
    self.x = x;
  }

  pub fn set_y(&mut self, y: i32) {
    self.y = y;
  }

  pub fn pos(&self) -> (i32, i32) {
    (self.x, self.y)
  }

}