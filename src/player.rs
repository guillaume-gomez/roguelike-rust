use tcod::Console;
use crate::fighter::Fighter;
use crate::game::Game;
use crate::object::Object;
use crate::death_callback::DeathCallback;

#[derive(Debug, Clone)]
pub struct Player {
  object: Object
}

impl Player {
  pub fn new(x: i32, y: i32) -> Self {
    let object = Object { 
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
    };
    Player {
      object
    }
  }

  pub fn draw(&self,  con: &mut dyn Console) {
    self.object.draw(con)
  }

  pub fn attack(&mut self, target: &mut Object, game: &mut Game) {
    // a simple formula for attack damage
    let damage = self.get_fighter().map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
    if damage > 0 {
      // make the target take some damage
      game.messages.add(
        format!("{} attacks {} for {} hit points.", self.object.name, target.name, damage),
        tcod::colors::WHITE
      );
      target.take_damage(damage, game);
    } else {
      game.messages.add(
        format!("{} attacks {} but it has no effect!", self.object.name, target.name),
        tcod::colors::WHITE
      );
    }
  }

  pub fn move_or_attack(&mut self, dx: i32, dy: i32, game: &mut Game, other_objects: &mut [Object]) {
    let x = self.object.x + dx;
    let y = self.object.y + dy;
    
    let target_id = other_objects
      .iter()
      .position(|object| object.fighter.is_some() && object.pos() == (x, y));
    // attack if target found, move otherwise
    match target_id {
      Some(target_id) => {
        self.attack(&mut other_objects[target_id], game);
      }
      None => {
        self.object.move_by(dx, dy, &game, other_objects);
      }
    }
  }

  pub fn take_damage(&mut self, damage: i32, game: &mut Game) {
    if let Some(fighter) = self.object.fighter.as_mut() {
      if damage > 0 {
          fighter.hp -= damage;
      }
    }
    // check for death, call the death function
    if let Some(fighter) = self.object.fighter {
      if fighter.hp <= 0 {
        self.die();
        fighter.on_death.callback(&mut self.object, game);
      }
    }
  }

  pub fn set_pos(&mut self, x: i32, y: i32) {
    self.object.x = x;
    self.object.y = y;
  }

  pub fn pos(&self) -> (i32, i32) {
    (self.object.x, self.object.y)
  }

   pub fn get_x(&self) -> i32 {
    self.object.x
  }

  pub fn get_y(&self) -> i32{
    self.object.y
  }

  pub fn get_fighter(&self) -> Option<&Fighter> {
    self.object.fighter.as_ref()
  }

  pub fn is_alive(&self) -> bool {
    self.object.alive
  }

  pub fn die(&mut self) {
    self.object.alive = false;
  }

  pub fn get_name(&self) -> String {
    self.object.name.to_string()
  }
}