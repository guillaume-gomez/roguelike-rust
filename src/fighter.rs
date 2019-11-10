use crate::object::Object;


// combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
  pub max_hp: i32,
  pub hp: i32,
  pub defense: i32,
  pub power: i32,
  //pub on_death: DeathCallback,
}


// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum DeathCallback {
//   Player,
//   Monster,
// }

// impl DeathCallback {
//   fn callback(self, object: &mut Object) {
//     use DeathCallback::*;
//     let callback: fn(&mut Object) = match self {
//         Player => player_death,
//         Monster => monster_death,
//     };
//     callback(object);
//   }
// }
