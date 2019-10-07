use crate::tile::Tile;

// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

//#[derive(Clone, Copy)]
pub type Map = Vec<Vec<Tile>>;

pub struct Game {
  pub map: Map,
}

impl Game {
  pub fn new() -> Self {
    Game { map: make_map() }
  }
}

fn make_map() -> Map {
  // fill map with "unblocked" tiles
  let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
  // place two pillars to test the map
  map[30][22] = Tile::wall();
  map[50][22] = Tile::wall();
  
  map
}