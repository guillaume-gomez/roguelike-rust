use crate::hud::menu;
use serde::{Deserialize, Serialize};
use crate::target_tile;
use crate::Tcod;
use tcod::Console;

use crate::constants::LEVEL_SCREEN_WIDTH;
use crate::constants::LEVEL_UP_FACTOR;
use crate::constants::LEVEL_UP_BASE;
use crate::constants::FIREBALL_RADIUS;
use crate::constants::FIREBALL_DAMAGE;
use crate::constants::CONFUSE_RANGE;
use crate::constants::CONFUSE_NUM_TURNS;
use crate::constants::LIGHTNING_RANGE;
use crate::constants::LIGHTNING_DAMAGE;
use crate::constants::HEAL_AMOUNT;
use crate::constants::MAX_INVENTORY;
use crate::enemy::Enemy;
use crate::fighter::Fighter;
use crate::game::Game;
use crate::object::Object;
use crate::object::Item;
use crate::enemy::Ai;


enum UseResult {
  UsedUp,
  UsedAndKept,
  Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
  object: Object,
  level: i32,
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
        max_hp: 100,
        hp: 100,
        defense: 1,
        xp: 0,
        power: 4
      }),
      item: None,
      always_visible: true,
      equipment: None
    };
    Player {
      object,
      level: 1
    }
  }

  pub fn draw(&self,  con: &mut dyn Console) {
    self.object.draw(con)
  }

  pub fn attack(&mut self, target: &mut Enemy, game: &mut Game) {
    // a simple formula for attack damage
    let damage = self.get_fighter().map_or(0, |f| f.power) - target.get_fighter().map_or(0, |f| f.defense);
    if damage > 0 {
      // make the target take some damage
      game.messages.add(
        format!("{} attacks {} for {} hit points.", self.object.name, target.get_name(), damage),
        tcod::colors::WHITE
      );
      if let Some(xp) = target.take_damage(damage, game) {
        // yield experience to the player
        self.object.fighter.as_mut().unwrap().xp += xp;
      }
    } else {
      game.messages.add(
        format!("{} attacks {} but it has no effect!", self.object.name, target.get_name()),
        tcod::colors::WHITE
      );
    }
  }

  pub fn move_or_attack(&mut self, dx: i32, dy: i32, game: &mut Game, enemies: &mut [Enemy]) {
    let x = self.object.x + dx;
    let y = self.object.y + dy;
    
    let target_id = enemies
      .iter()
      .position(|object| object.get_fighter().is_some() && object.pos() == (x, y));
    // attack if target found, move otherwise
    match target_id {
      Some(target_id) => {
        self.attack(&mut enemies[target_id], game);
      }
      None => {
        //clone => dirty 
        let object_enemies = enemies
          .iter()
          .map(|obj| obj.get_object().clone())
          .collect::<Vec<Object>>();
        
        self.object.move_by(dx, dy, &game, &object_enemies);
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
        game.messages.add("You died!", tcod::colors::RED);
        self.object.die();
        self.object.char = '%';
        self.object.color = tcod::colors::DARK_RED;
      }
    }
  }

  pub fn pick_item_up(&mut self, object_id: usize, game: &mut Game, collectibles: &mut Vec<Object>) {
    if game.inventory.len() >= MAX_INVENTORY {
      game.messages.add(
        format!(
          "Your inventory is full, cannot pick up {}.",
          collectibles[object_id].name
        ),
        tcod::colors::RED,
      );
    } else {
      let item = collectibles.swap_remove(object_id);
      game.messages.add(format!("You picked up a {}!", item.name), tcod::colors::GREEN);
      game.inventory.push(item);
    }
  }

  pub fn use_item(&mut self, game: &mut Game, tcod: &mut Tcod, inventory_id: usize, collectibles: &mut [Object], enemies: &mut [Enemy] ) {
    
    if let Some(item) = game.inventory[inventory_id].item {
      let on_use = match item {
        Item::Heal => Player::cast_heal,
        Item::Lightning => Player::cast_lightning,
        Item::Confuse => Player::cast_confuse,
        Item::Fireball => Player::cast_fireball,
        Item::Equipment => Player::toggle_equipment,
      };
      match on_use(self, game, tcod, inventory_id, collectibles, enemies) {
        UseResult::UsedUp => {
          // destroy after use, unless it was cancelled for some reason
          game.inventory.remove(inventory_id);
        }
        UseResult::Cancelled => {
          game.messages.add("Cancelled", tcod::colors::WHITE);
        }
        UseResult::UsedAndKept => {} // do nothing
      }
    } else {
      game.messages.add(
        format!("The {} cannot be used.", game.inventory[inventory_id].name),
        tcod::colors::WHITE,
      );
    }
  }

  fn cast_heal(&mut self, game: &mut Game, _tcod: &mut Tcod, _inventory_id: usize, _collectibles: &mut [Object], _enemies: &mut [Enemy]) -> UseResult {
    // heal the player
    if let Some(fighter) = self.get_fighter() {
      if fighter.hp == fighter.max_hp {
        game.messages.add("You are already at full health.", tcod::colors::RED);
        return UseResult::Cancelled;
      }
      game.messages
        .add("Your wounds start to feel better!", tcod::colors::LIGHT_VIOLET);
        self.heal(HEAL_AMOUNT);
      return UseResult::UsedUp;
    }
    UseResult::Cancelled
  }

  // find closest enemy (inside a maximum range and damage it)
  fn cast_lightning(&mut self, game: &mut Game, _tcod: &mut Tcod, _inventory_id: usize, _collectibles: &mut [Object], enemies: &mut [Enemy]) -> UseResult {
    let monster_id = self.closest_monster(_tcod, enemies, LIGHTNING_RANGE);
    if let Some(monster_id) = monster_id {
        // zap it!
        game.messages.add(
            format!(
                "A lightning bolt strikes the {} with a loud thunder! \
                 The damage is {} hit points.",
                enemies[monster_id].get_name(), LIGHTNING_DAMAGE
            ),
            tcod::colors::LIGHT_BLUE,
        );
        if let Some(xp) = enemies[monster_id].take_damage(LIGHTNING_DAMAGE, game) {
          self.object.fighter.as_mut().unwrap().xp += xp;
        }
        UseResult::UsedUp
    } else {
        // no enemy found within maximum range
        game.messages
            .add("No enemy is close enough to strike.", tcod::colors::RED);
        UseResult::Cancelled
    }
  }

  fn cast_confuse(&mut self, game: &mut Game, tcod: &mut Tcod, _inventory_id: usize, collectibles: &mut [Object], enemies: &mut[Enemy] ) -> UseResult {
        // ask the player for a target to confuse
    game.messages.add(
        "Left-click an enemy to confuse it, or right-click to cancel.",
        tcod::colors::LIGHT_CYAN,
    );
    let monster_id = self.target_monster(tcod, game, enemies, collectibles, Some(CONFUSE_RANGE as f32));
    if let Some(monster_id) = monster_id {
        let old_ai = enemies[monster_id].get_ai().take().unwrap_or(Ai::Basic);
        // replace the monster's AI with a "confused" one; after
        // some turns it will restore the old AI
        let new_ai = Some(Ai::Confused {
            previous_ai: Box::new(old_ai),
            num_turns: CONFUSE_NUM_TURNS,
        });
        enemies[monster_id].set_ai(new_ai);
        game.messages.add(
            format!(
                "The eyes of {} look vacant, as he starts to stumble around!",
                enemies[monster_id].get_name()
            ),
            tcod::colors::LIGHT_GREEN,
        );
        UseResult::UsedUp
    } else {
        // no enemy fonud within maximum range
        game.messages
            .add("No enemy is close enough to strike.", tcod::colors::RED);
        UseResult::Cancelled
    }
  }

  fn cast_fireball(&mut self, game: &mut Game, tcod: &mut Tcod, _inventory_id: usize, collectibles: &mut [Object], enemies: &mut[Enemy] ) -> UseResult {
    // ask the player for a target tile to throw a fireball at
    game.messages.add(
      "Left-click a target tile for the fireball, or right-click to cancel.",
      tcod::colors::LIGHT_CYAN,
    );
    let (x, y) = match target_tile(tcod, game, self, enemies, collectibles, None) {
      Some(tile_pos) => tile_pos,
      None => return UseResult::Cancelled,
    };
    game.messages.add(
      format!(
        "The fireball explodes, burning everything within {} tiles!",
        FIREBALL_RADIUS
      ),
      tcod::colors::ORANGE,
    );

    let mut xp_to_gain = 0;
    for enemy in enemies {
      if enemy.distance(x, y) <= FIREBALL_RADIUS as f32 && enemy.get_fighter().is_some() {
        game.messages.add(
          format!(
            "The {} gets burned for {} hit points.",
            enemy.get_name(), FIREBALL_DAMAGE
          ),
          tcod::colors::ORANGE,
        );
        if let Some(xp) = enemy.take_damage(FIREBALL_DAMAGE, game) {
          xp_to_gain += xp;
        }
      }
    }
    self.object.fighter.as_mut().unwrap().xp += xp_to_gain;
    UseResult::UsedUp
  }


  fn toggle_equipment(&mut self, game: &mut Game, _tcod: &mut Tcod, inventory_id: usize, _collectibles: &mut [Object], _enemies: &mut[Enemy] ) -> UseResult {
    let equipment = match game.inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };
    if equipment.equipped {
        game.inventory[inventory_id].dequip(&mut game.messages);
    } else {
        game.inventory[inventory_id].equip(&mut game.messages);
    }
    UseResult::UsedAndKept
}

  pub fn closest_monster(&self, tcod: &Tcod, enemies: &[Enemy], max_range: i32) -> Option<usize> {
    let mut closest_enemy = None;
    let mut closest_dist = (max_range + 1) as f32; // start with (slightly more than) maximum range

    for (id, enemy) in enemies.iter().enumerate() {
      if enemy.get_fighter().is_some() && enemy.get_ai().is_some() && tcod.fov.is_in_fov(enemy.get_x(), enemy.get_y())
      {
        // calculate distance between this object and the player
        let dist = self.object.distance_to(enemy.get_object());
        if dist < closest_dist {
           // it's closer, so remember it
          closest_enemy = Some(id);
          closest_dist = dist;
        }
      }
    }
    closest_enemy
  }

  fn target_monster(&self, tcod: &mut Tcod, game: &mut Game, enemies: &[Enemy], collectibles: &[Object], max_range: Option<f32>) -> Option<usize> {
    loop {
      match target_tile(tcod, game, self, enemies, collectibles, max_range) {
        Some((x, y)) => {
          // return the first clicked monster, otherwise continue looping
          for (id, enemy) in enemies.iter().enumerate() {
            if enemy.pos() == (x, y) && enemy.get_fighter().is_some() {
              return Some(id);
            }
          }
        }
        None => return None,
      }
    }
  }

  pub fn drop_item(&self, inventory_id: usize, game: &mut Game, collectibles: &mut Vec<Object>) {
    let mut item = game.inventory.remove(inventory_id);
    item.set_pos(self.get_x(), self.get_y());
    game.messages.add(format!("You dropped a {}.", item.name), tcod::colors::YELLOW);
    collectibles.push(item);
  }
 
  /// heal by the given amount, without going over the maximum
  pub fn heal(&mut self, amount: i32) {
    if let Some(ref mut fighter) = self.object.fighter {
      fighter.hp += amount;
      if fighter.hp > fighter.max_hp {
        fighter.hp = fighter.max_hp;
      }
    }
  }

  pub fn level_up(&mut self, tcod: &mut Tcod, game: &mut Game) {
    let level_up_xp = LEVEL_UP_BASE + self.level * LEVEL_UP_FACTOR;
  // see if the player's experience is enough to level-up
  if self.object.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
      // it is! level up
      self.level += 1;
      game.messages.add(
        format!(
          "Your battle skills grow stronger! You reached level {}!",
          self.level
        ),
        tcod::colors::YELLOW,
      );
      let fighter = self.object.fighter.as_mut().unwrap();
      let mut choice = None;
      while choice.is_none() {
        // keep asking until a choice is made
        choice = menu(
          "Level up! Choose a stat to raise:\n",
          &[
              format!("Constitution (+20 HP, from {})", fighter.max_hp),
              format!("Strength (+1 attack, from {})", fighter.power),
              format!("Agility (+1 defense, from {})", fighter.defense),
          ],
          LEVEL_SCREEN_WIDTH,
          &mut tcod.root,
        );
      }
      fighter.xp -= level_up_xp;
      match choice.unwrap() {
        0 => {
          fighter.max_hp += 20;
          fighter.hp += 20;
        }
        1 => {
          fighter.power += 1;
        }
        2 => {
          fighter.defense += 1;
        }
        _ => unreachable!(),
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

  pub fn get_name(&self) -> String {
    self.object.name.to_string()
  }

  pub fn get_object(&self) -> &Object {
    &self.object
  }

  pub fn distance(&self, x: i32, y: i32) -> f32 {
    self.object.distance(x, y)
  }

  pub fn get_level(&self) -> i32 {
    self.level
  }

  pub fn level_up_xp(&self) -> i32 {
    LEVEL_UP_BASE + self.level * LEVEL_UP_FACTOR
  }
}