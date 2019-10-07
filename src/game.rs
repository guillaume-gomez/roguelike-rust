use std::cmp;
use crate::tile::Tile;
use crate::rect::Rect;

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
  // fill map with "blocked" tiles
  let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

  // create two rooms
  let room1 = Rect::new(20, 15, 10, 15);
  let room2 = Rect::new(50, 15, 10, 15);
  create_room(room1, &mut map);
  create_room(room2, &mut map);
  create_h_tunnel(25, 55, 23, &mut map);
  map
}

fn create_room(room: Rect, map: &mut Map) {
  // go through the tiles in the rectangle and make them passable
  for x in (room.x1() + 1)..room.x2() {
    for y in (room.y1() + 1)..room.y2() {
      map[x as usize][y as usize] = Tile::empty();
    }
  }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}