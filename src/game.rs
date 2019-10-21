use std::cmp;
use rand::Rng;
use tcod::colors;

use crate::tile::Tile;
use crate::rect::Rect;
use crate::object::Object;


// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const MAX_ROOM_MONSTERS: i32 = 3;

//#[derive(Clone, Copy)]
pub type Map = Vec<Vec<Tile>>;


pub struct Game {
  pub map: Map,
}

impl Game {
  pub fn new(player: &mut Object, other_objects: &mut Vec<Object> ) -> Self {
    Game { map: make_map(player, other_objects) }
  }
}

fn make_map(player: &mut Object, other_objects: &mut Vec<Object>) -> Map {
  let mut rooms = vec![];
  let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

  for _ in 0..MAX_ROOMS {
    // random width and height
    let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
    let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
    // random position without going out of the boundaries of the map
    let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
    let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

    let new_room = Rect::new(x, y, w, h);

    // run through the other rooms and see if they intersect with this one
    let failed = rooms
      .iter()
      .any(|other_room| new_room.intersects_with(other_room));

    if !failed {
      // this means there are no intersections, so this room is valid

      // "paint" it to the map's tiles
      create_room(new_room, &mut map);

      place_objects(new_room, other_objects);

      // center coordinates of the new room, will be useful later
      let (new_x, new_y) = new_room.center();

      if rooms.is_empty() {
        // this is the first room, where the player starts at
        player.set_pos(new_x, new_y);
      } else {
        // center coordinates of the previous room
        let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

        // toss a coin (random bool value -- either true or false)
        if rand::random() {
          // first move horizontally, then vertically
          create_h_tunnel(prev_x, new_x, prev_y, &mut map);
          create_v_tunnel(prev_y, new_y, new_x, &mut map);
        } else {
          // first move vertically, then horizontally
          create_v_tunnel(prev_y, new_y, prev_x, &mut map);
          create_h_tunnel(prev_x, new_x, new_y, &mut map);
        }
      }
    rooms.push(new_room);
    }
  }
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

fn place_objects(room: Rect, objects: &mut Vec<Object>) {
  // choose random number of monsters
  let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

  for _ in 0..num_monsters {
    // choose random spot for this monster
    let x = rand::thread_rng().gen_range(room.x1() + 1, room.x2());
    let y = rand::thread_rng().gen_range(room.y1() + 1, room.y2());

    let mut monster = if rand::random::<f32>() < 0.8 {  // 80% chance of getting an orc
        // create an orc
        Object::new(x, y, 'o', colors::DESATURATED_GREEN, "orc", true)
    } else {
        Object::new(x, y, 'T', colors::DARKER_GREEN, "other monster", true) 
    };
    monster.alive();
    objects.push(monster);
  }
}