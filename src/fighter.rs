use serde::{Deserialize, Serialize};
// combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
  pub max_hp: i32,
  pub hp: i32,
  pub defense: i32,
  pub power: i32,
  pub xp: i32,
}

impl Fighter {

}