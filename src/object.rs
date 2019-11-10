use crate::Tcod;
use crate::fighter::Fighter;
use crate::death_callback::DeathCallback;
use crate::game::Game;
use crate::game::Map;
use tcod::colors::Color;
use tcod::console::Console;
use tcod::console::BackgroundFlag;

#[derive(Clone, Debug, PartialEq)]
pub enum Ai {
    Basic,
}

#[derive(Debug, Clone)]
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
        on_death: DeathCallback::Player
      }),
      ai: None
    }
  }

  pub fn create_orc(x: i32, y: i32) -> Self {
    Object { 
      x, 
      y, 
      char: 'o', 
      color: tcod::colors::DESATURATED_GREEN, 
      name: "orc".to_string(), 
      blocks: true,
      alive: true,
      fighter:  Some(Fighter {
        max_hp: 10,
        hp: 10,
        defense: 0,
        power: 3,
        on_death: DeathCallback::Monster,
      }),
      ai: Some(Ai::Basic)
    }
  }

  pub fn create_troll(x: i32, y: i32) -> Self {
    Object { 
      x, 
      y, 
      char: 'T', 
      color: tcod::colors::DARKER_GREEN, 
      name: "troll".to_string(), 
      blocks: true,
      alive: true,
      fighter:  Some(Fighter {
        max_hp: 16,
        hp: 16,
        defense: 1,
        power: 4,
        on_death: DeathCallback::Monster,
      }),
      ai: Some(Ai::Basic)
    }
  }

  fn move_by(&mut self, dx: i32, dy: i32, game: &Game, other_objects: &[Object]) {
    let (x, y) = self.pos();
    if !is_blocked(x + dx, y + dy, &game.map, other_objects) {
      self.set_pos(x + dx, y + dy);
    }
  }

  pub fn take_damage(&mut self, damage: i32) {
    if let Some(fighter) = self.fighter.as_mut() {
      if damage > 0 {
          fighter.hp -= damage;
      }
    }
    // check for death, call the death function
    if let Some(fighter) = self.fighter {
      if fighter.hp <= 0 {
        self.die();
        fighter.on_death.callback(self);
      }
    }
  }

  pub fn attack(&mut self, target: &mut Object) {
    // a simple formula for attack damage
    let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
    if damage > 0 {
      // make the target take some damage
      println!(
        "{} attacks {} for {} hit points.",
        self.name, target.name, damage
      );
      target.take_damage(damage);
    } else {
      println!(
        "{} attacks {} but it has no effect!",
        self.name, target.name
      );
    }
  }

  pub fn move_or_attack(&mut self, dx: i32, dy: i32, game: &Game, other_objects: &mut [Object]) {
    let x = self.x + dx;
    let y = self.y + dy;
    
    let target_id = other_objects
      .iter()
      .position(|object| object.fighter.is_some() && object.pos() == (x, y));
    // attack if target found, move otherwise
    match target_id {
      Some(target_id) => {
        self.attack(&mut other_objects[target_id]);
      }
      None => {
        self.move_by(dx, dy, &game, other_objects);
      }
    }
  }

  pub fn move_towards(&mut self, target_x: i32, target_y: i32, game: &Game, other_objects: &[Object]) {
    // vector from this object to the target, and distance
    let (x, y) = self.pos();
    let dx = target_x - x;
    let dy = target_y - y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    self.move_by(dx, dy, &game, other_objects);
  }

  /// return the distance to another object
  pub fn distance_to(&self, other: &Object) -> f32 {
    let dx = other.x - self.x;
    let dy = other.y - self.y;
    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
  }

  pub fn ai_take_turn(&mut self, tcod: &Tcod, game: &Game, other_objects: &[Object], player: &mut Object) {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = self.pos();
    if tcod.fov.is_in_fov(monster_x, monster_y) {
        if self.distance_to(&player) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = player.pos();
            self.move_towards(player_x, player_y, &game, other_objects);
        } else if player.fighter.map_or(false, |f| f.hp > 0) {
            self.attack(player);
            
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

  pub fn get_ai(&self) -> Option<&Ai> {
    self.ai.as_ref()
  }

  pub fn get_name(&self) -> String {
    self.name.to_string()
  }

  pub fn display_death(&mut self) {
    self.char = '%';
    self.color = tcod::colors::DARK_RED;
  }

  pub fn remove_enemy_interactions(&mut self) {
    self.blocks = false;
    self.fighter = None;
    self.ai = None;
    self.name = format!("remains of {}", self.name);
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