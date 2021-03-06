use crate::messages::Messages;
use crate::equipment::Equipment;
use crate::equipment::Slot;
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
    Fireball,
    Equipment,
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
  pub equipment: Option<Equipment>
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
      always_visible: false,
      equipment: None
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
      always_visible: true,
      equipment: None
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
      always_visible: true,
      equipment: None,
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
      always_visible: true,
      equipment: None,
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
      always_visible: true,
      equipment: None
    }
  }

  pub fn create_sword(x: i32, y: i32) -> Self {
    Object {
      x,
      y,
      char: '/',
      name: "sword".to_string(),
      color:  tcod::colors::SKY,
      blocks: false,
      alive: false,
      fighter: None,
      item: Some(Item::Equipment),
      always_visible: true,
      equipment: Some(Equipment { equipped: false, slot: Slot::RightHand })
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
      always_visible: true,
      equipment: None
    }
  }

  pub fn equip(&mut self,  messages: &mut Messages) {
    if self.item.is_none() {
      messages.add(
        format!("Can't equip {:?} because it's not an Item.", self),
        tcod::colors::RED,
      );
      return;
    };
    if let Some(ref mut equipment) = self.equipment {
      if !equipment.equipped {
        equipment.equipped = true;
        messages.add(
          format!("Equipped by you on {}.", equipment.slot),
          tcod::colors::LIGHT_GREEN,
        );
      }
    } else {
      messages.add(
        format!("Can't equip {:?} because it's not an Equipment.", self),
        tcod::colors::RED,
      );
    }
  }

  pub fn dequip(&mut self, messages: &mut Messages) {
    if self.item.is_none() {
      messages.add(
        format!("Can't dequip {:?} because it's not an Item.", self),
        tcod::colors::RED,
      );
      return;
    };
    if let Some(ref mut equipment) = self.equipment {
      if equipment.equipped {
          equipment.equipped = false;
          messages.add(
            format!("Dequipped by you from {}.", equipment.slot),
            tcod::colors::LIGHT_YELLOW,
          );
      }
    } else {
        messages.add(
          format!("Can't dequip {:?} because it's not an Equipment.", self),
          tcod::colors::RED,
        );
    }
  }

  pub fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    for (inventory_id, item) in inventory.iter().enumerate() {
      if item
        .equipment
        .as_ref()
        .map_or(false, |e| e.equipped && e.slot == slot)
      {
        return Some(inventory_id);
      }
    }
    None
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