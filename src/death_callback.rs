#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
   Player,
   Monster,
}


// deprecrated after splitting Enemy and Player in disctinct Struct
// impl DeathCallback {
//    pub fn callback(self, object: &mut Object, game: &mut Game) {
//      use DeathCallback::*;
//      let callback: fn(&mut Object, &mut Game) = match self {
//          Player => Fighter::player_death,
//          Monster => Fighter::monster_death,
//      };
//      callback(object, game);
//   }
// }
