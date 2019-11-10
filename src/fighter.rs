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
  pub fn player_death(player: &mut Object) {
    // the game ended!
    println!("You died!");

    // for added effect, transform the player into a corpse!
    player.display_death();
  }

  pub fn monster_death(monster: &mut Object) {
    // transform it into a nasty corpse! it doesn't block, can't be
    // attacked and doesn't move
    println!("{} is dead!", monster.get_name());
    monster.display_death();
    monster.remove_enemy_interactions();
  }
}