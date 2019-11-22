extern crate rand;
use crate::game::Game;
use crate::player::Player;
use crate::fighter::Fighter;
use crate::object::Object;

use crate::Tcod;
use tcod::colors::Color;
use tcod::Console;

use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub enum Ai {
  Basic,
  Confused {
    previous_ai: Box<Ai>,
    num_turns: i32,
  },
}

#[derive(Debug, Clone)]
pub struct Enemy {
  object: Object,
  ai: Option<Ai>,
}


impl Enemy {
  pub fn new(x: i32, y: i32, char: char, color: Color, name: &str, max_hp: i32, hp: i32, defense: i32, power: i32) -> Self {
    let object = Object { 
      x, 
      y,
      char,
      color,
      name: name.to_string(), 
      blocks: true,
      alive: true,
      fighter:  Some(Fighter {
        max_hp,
        hp,
        defense,
        power
      }),
      item: None,
    };
    Enemy {
      object,
      ai: Some(Ai::Basic)
    }
  }


  pub fn create_orc(x: i32, y: i32) -> Self {
    Enemy::new(
      x,
      y,
      'o',
      tcod::colors::DESATURATED_GREEN,
      &"orc".to_string(),
      10,
      10,
      0,
      3
    )
  }

  pub fn create_troll(x: i32, y: i32) -> Self {
    Enemy::new(
      x,
      y,
      'T',
      tcod::colors::DARKER_GREEN,
      &"troll".to_string(),
      16,
      16,
      1,
      4
    )
  }

  pub fn attack(&mut self, player: &mut Player, game: &mut Game) {
    // a simple formula for attack damage
    let damage = self.get_fighter().map_or(0, |f| f.power) - player.get_fighter().map_or(0, |f| f.defense);
    if damage > 0 {
      // make the target take some damage
      game.messages.add(
        format!("{} attacks {} for {} hit points.", self.object.name, player.get_name(), damage),
        tcod::colors::WHITE
      );
      player.take_damage(damage, game);
    } else {
      game.messages.add(
        format!("{} attacks {} but it has no effect!", self.object.name, player.get_name()),
        tcod::colors::WHITE
      );
    }
  }

   pub fn move_towards(&mut self, target_x: i32, target_y: i32, game: &Game, other_enemies: &[Enemy]) {
    // vector from this object to the target, and distance
    let (x, y) = self.object.pos();
    let dx = target_x - x;
    let dy = target_y - y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    
  //clone => dirty 
  let object_enemies = other_enemies
    .iter()
    .map(|obj| obj.get_object().clone())
    .collect::<Vec<Object>>();

    self.object.move_by(dx, dy, &game, &object_enemies);
  }

  pub fn ai_basic(&mut self, tcod: &Tcod, game: &mut Game, other_enemies: &[Enemy], player: &mut Player) -> Ai {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = self.object.pos();
    if tcod.fov.is_in_fov(monster_x, monster_y) {
      if self.object.distance_to(&player.get_object()) >= 2.0 {
        // move towards player if far away
        let (player_x, player_y) = player.pos();
        self.move_towards(player_x, player_y, &game, other_enemies);
      } else if player.get_fighter().map_or(false, |f| f.hp > 0) {
        self.attack(player, game);
      }
    }
    Ai::Basic
  }

  pub fn ai_take_turn(&mut self, tcod: &Tcod, game: &mut Game, other_enemies: &[Enemy], player: &mut Player) {
    use Ai::*;
    if let Some(ai) = self.ai.take() {
      let new_ai = match ai {
        Basic => self.ai_basic(tcod, game, other_enemies, player),
        Confused {
            previous_ai,
            num_turns,
        } => self.ai_confused(tcod, game, other_enemies, previous_ai, num_turns),
      };
      self.ai = Some(new_ai);
    }
  }

  fn ai_confused(&mut self, _tcod: &Tcod, game: &mut Game, enemies: &[Enemy], previous_ai: Box<Ai>, num_turns: i32) -> Ai {
      let object_enemies = enemies
        .iter()
        .map(|obj| obj.get_object().clone())
        .collect::<Vec<Object>>();
      if num_turns >= 0 {
          // still confused ...
          // move in a random direction, and decrease the number of turns confused
          self.object.move_by(rand::thread_rng().gen_range(-1, 2), rand::thread_rng().gen_range(-1, 2), &game, &object_enemies);
          Ai::Confused {
              previous_ai: previous_ai,
              num_turns: num_turns - 1,
          }
      } else {
          // restore the previous AI (this one will be deleted)
          game.messages.add(
              format!("The {} is no longer confused!", self.get_name()),
              tcod::colors::RED,
          );
          *previous_ai
      }
  }

  pub fn draw(&self,  con: &mut dyn Console) {
    self.object.draw(con)
  }

  // for player and enemy only
  pub fn take_damage(&mut self, damage: i32, game: &mut Game) {
    if let Some(fighter) = self.object.fighter.as_mut() {
      if damage > 0 {
          fighter.hp -= damage;
      }
    }
    // check for death, call the death function
    if let Some(fighter) = self.object.fighter {
      if fighter.hp <= 0 {
        self.object.die();
        self.object.char = '%';
        self.object.color = tcod::colors::DARK_RED;
        self.object.blocks = false;
        self.object.fighter = None;
        self.ai = None;
        
        game.messages.add(format!("{} is dead!", self.object.get_name()), tcod::colors::ORANGE);
        self.object.name = format!("remains of {}", self.object.get_name());
      }
    }
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

  pub fn get_name(&self) -> String {
    self.object.name.to_string()
  }

  pub fn get_fighter(&self) -> Option<&Fighter> {
    self.object.fighter.as_ref()
  }

  pub fn get_object(&self) -> &Object {
    &self.object
  }

  pub fn get_ai(&self) -> Option<&Ai> {
    self.ai.as_ref()
  }


}