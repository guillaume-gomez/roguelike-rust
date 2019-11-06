use crate::fighter::Fighter;
use crate::game::Game;
use crate::game::Map;
use tcod::colors::Color;
use tcod::console::Console;
use tcod::console::BackgroundFlag;

#[derive(Clone, Debug, PartialEq)]
enum Ai {
    Basic,
}

#[derive(Debug)]
pub struct Object {
  x: i32,
  y: i32,
  char: char,
  color: Color,
  name: String,
  blocks: bool,
  alive: bool,
  fighter: Option<Fighter>,
  ai: Option<Ai>, 
}

impl Object {
  pub fn new(x: i32, y: i32, char: char, color: Color, name: &str, blocks: bool) -> Self {
    Object { 
      x, 
      y, 
      char,
      color,
      name: name.to_string(),
      blocks,
      alive: false,
      fighter: None,
      ai: None,
    }
  }

  pub fn create_player(x: i32, y: i32) -> Self {
    Object { 
      x, 
      y, 
      char: '%', 
      color: tcod::colors::GREEN, 
      name: "player".to_string(), 
      blocks: true,
      alive: true,
      fighter:  Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
      }),
      ai: None
    }
  }

  fn move_by(&mut self, dx: i32, dy: i32, game: &Game, other_objects: &[Object]) {
    let (x, y) = self.pos();
    if !is_blocked(x + dx, y + dy, &game.map, other_objects) {
      self.set_pos(x + dx, y + dy);
    }
  }

  pub fn move_or_attack(&mut self, dx: i32, dy: i32, game: &Game, other_objects: &[Object]) {
    
    let target_id = other_objects.iter().position(|object| object.pos() == self.pos());

    // attack if target found, move otherwise
    match target_id {
      Some(target_id) => {
        println!(
            "The {} laughs at your puny efforts to attack him!",
            other_objects[target_id].name
        );
      }
      None => {
        self.move_by(dx, dy, &game, other_objects);
      }
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

  pub fn is_blocked(&self) -> bool {
    self.blocks
  }

  pub fn alive(&mut self) {
    self.alive = true;
  }

  pub fn die(&mut self) {
    self.alive = false;
  }

  pub fn is_alive(&self) -> bool {
    self.alive
  }

}

fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
  // first test the map tile
  if map[x as usize][y as usize].is_blocked() {
    return true;
  }
  // now check for any blocking objects
  objects
    .iter()
    .any(|object| object.is_blocked() && object.pos() == (x, y))
}