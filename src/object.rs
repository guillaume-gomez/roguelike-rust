use crate::fighter::Fighter;
use crate::game::Game;
use crate::game::Map;
use tcod::colors::Color;
use tcod::console::Console;
use tcod::console::BackgroundFlag;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Heal,
    Lightning,
    Confuse,
    Fireball
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
  pub x: i32,
  pub y: i32,
  pub char: char,
  pub color: Color,
  pub name: String,
  pub blocks: bool,
  pub alive: bool,
  pub fighter: Option<Fighter>,
  pub item: Option<Item>,
  pub always_visible: bool,
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
      item: None,
      always_visible: false
    }
  }

  pub fn create_potion(x: i32, y: i32) -> Self {
    Object {
      x,
      y,
      char: '!',
      name: "healing potion".to_string(),
      color: tcod::colors::VIOLET,
      blocks: false,
      alive: false,
      fighter: None,
      item: Some(Item::Heal),
      always_visible: false
    }
  }

  pub fn create_lighting_bolt(x: i32, y: i32) -> Self {
    Object {
      x,
      y,
      char: '#',
      name: "scroll of lightning bolt".to_string(),
      color: tcod::colors::LIGHT_YELLOW,
      blocks: false,
      alive: false,
      fighter: None,
      item: Some(Item::Lightning),
      always_visible: false
    }
  }

  pub fn create_confuse_potion(x: i32, y: i32) -> Self {
    Object {
      x,
      y,
      char: '#',
      name: "scroll of confusion".to_string(),
      color: tcod::colors::LIGHT_YELLOW,
      blocks: false,
      alive: false,
      fighter: None,
      item: Some(Item::Confuse),
      always_visible: false
    }
  }

  pub fn create_fireball(x: i32, y: i32) -> Self {
    Object {
      x,
      y,
      char: '#',
      name: "scroll of fireball".to_string(),
      color:  tcod::colors::LIGHT_YELLOW,
      blocks: false,
      alive: false,
      fighter: None,
      item: Some(Item::Fireball),
      always_visible: false
    }
  }

  pub fn create_stair(x: i32, y: i32) -> Self {
    Object {
      x,
      y,
      char: '<',
      name: "stairs".to_string(),
      color:  tcod::colors::WHITE,
      blocks: false,
      alive: false,
      fighter: None,
      item: None,
      always_visible: true
    }
  }

  pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game, other_objects: &[Object]) {
    let (x, y) = self.pos();
    if !is_blocked(x + dx, y + dy, &game.map, other_objects) {
      self.set_pos(x + dx, y + dy);
    }
  }

  pub fn draw(&self, con: &mut dyn Console) {
    con.set_default_foreground(self.color);
    con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }

  /// return the distance to another object
  pub fn distance_to(&self, object: &Object) -> f32 {
    let dx = object.x - self.x;
    let dy = object.y - self.y;
    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
  }

  pub fn distance(&self, x: i32, y: i32) -> f32 {
    (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
  }

  pub fn set_pos(&mut self, x: i32, y: i32) {
    self.x = x;
    self.y = y;
  }

  pub fn pos(&self) -> (i32, i32) {
    (self.x, self.y)
  }

  pub fn is_blocked(&self) -> bool {
    self.blocks
  }


  pub fn die(&mut self) {
    self.alive = false;
  }

  pub fn get_name(&self) -> String {
    self.name.to_string()
  }

  pub fn always_visible(&self) -> bool {
    self.always_visible
  }
}

pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
  // first test the map tile
  if map[x as usize][y as usize].is_blocked() {
    return true;
  }
  // now check for any blocking objects
  objects
    .iter()
    .any(|object| object.is_blocked() && object.pos() == (x, y))
}