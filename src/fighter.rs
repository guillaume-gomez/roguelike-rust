use tcod::colors;

use crate::game::Game;
use crate::object::Object;
use crate::death_callback::DeathCallback;



// combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
  pub max_hp: i32,
  pub hp: i32,
  pub defense: i32,
  pub power: i32,
  pub on_death: DeathCallback,
}

impl Fighter {
  pub fn player_death(player: &mut Object, game: &mut Game) {
    // the game ended!
    game.messages.add("You died!", colors::RED);

    // for added effect, transform the player into a corpse!
    player.display_death();
  }

  pub fn monster_death(monster: &mut Object, game: &mut Game) {
    // transform it into a nasty corpse! it doesn't block, can't be
    // attacked and doesn't move
    game.messages.add(format!("{} is dead!", monster.get_name()), colors::ORANGE);
    monster.display_death();
    monster.remove_enemy_interactions();
  }
}