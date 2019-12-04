use std::error::Error;
use crate::game::Game;
use crate::player::Player;
use crate::enemy::Enemy;
use crate::object::Object;
use std::fs::File;
use std::io::{Read, Write};


pub fn save_game(game: &Game, player: &Player, enemies: &[Enemy], collectibles: &[Object]) -> Result<(), Box<dyn Error>> {
  let save_data = serde_json::to_string(&(game, player, enemies, collectibles))?;
  let mut file = File::create("savegame")?;
  file.write_all(save_data.as_bytes())?;
  Ok(())
}

pub fn load_game() -> Result<(Game, Player, Vec<Enemy>, Vec<Object>), Box<dyn Error>> {
  let mut json_save_state = String::new();
  let mut file = File::open("savegame")?;
  file.read_to_string(&mut json_save_state)?;
  let result = serde_json::from_str::<(Game, Player, Vec<Enemy>, Vec<Object>)>(&json_save_state)?;
  Ok(result)
}