use crate::object::Object;
use crate::fighter::Fighter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
   Player,
   Monster,
}

impl DeathCallback {
   pub fn callback(self, object: &mut Object) {
     use DeathCallback::*;
     let callback: fn(&mut Object) = match self {
         Player => Fighter::player_death,
         Monster => Fighter::monster_death,
     };
     callback(object);
  }
}
