pub struct Transition {
  pub level: u32,
  pub value: u32,
}

pub fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
  table
    .iter()
    .rev()
    .find(|transition| level >= transition.level)
    .map_or(0, |transition| transition.value)
}