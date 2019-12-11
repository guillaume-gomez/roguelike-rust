use crate::Tcod;
use serde::{Deserialize, Serialize};
use crate::enemy::Enemy;
use crate::object::is_blocked;
use crate::object::Object;
use crate::object::Item;
use crate::player::Player;
use std::cmp;
use rand::Rng;
use crate::constants::*;
use crate::messages::Messages;
use crate::tile::Tile;
use crate::rect::Rect;
use tcod::console::*;
use rand::distributions::{IndependentSample, Weighted, WeightedChoice};

//#[derive(Clone, Copy)]
pub type Map = Vec<Vec<Tile>>;


#[derive(Serialize, Deserialize)]
pub struct Game {
  pub map: Map,
  pub messages: Messages,
  pub inventory: Vec<Object>,
  pub dungeon_level: u32,
}

impl Game {
  pub fn new(player: &mut Player, enemies: &mut Vec<Enemy>, collectibles: &mut Vec<Object> ) -> Self {
    Game { 
      map: make_map(player, enemies, collectibles),
      messages: Messages::new(),
      inventory: vec![],
      dungeon_level: 1,
    }
  }
}


pub fn initialise_fov(tcod: &mut Tcod, map: &Map) {
  // create the FOV map, according to the generated map
  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      tcod.fov.set(
        x,
        y,
        !map[x as usize][y as usize].is_block_sight(),
        !map[x as usize][y as usize].is_blocked(),
      );
    }
  }
  // unexplored areas start black (which is the default background color)
  tcod.con.clear();
}

pub fn new_game(tcod: &mut Tcod) -> (Game, Player, Vec<Enemy>, Vec<Object>) {
  // create object representing the player
  let mut player = Player::new(0, 0);
  let mut enemies = vec![];
  let mut collectibles = vec![];
  let mut game = Game::new(&mut player, &mut enemies, &mut collectibles);

  // a warm welcoming message!
  game.messages.add(
    "Welcome stranger! Prepare to perish in the Tombs of the Ancient Kings.",
    tcod::colors::RED,
  );
  initialise_fov(tcod, &game.map);
  (game, player, enemies, collectibles)
}


pub fn next_level(tcod: &mut Tcod, game: &mut Game, player: &mut Player, enemies: &mut Vec<Enemy>, collectibles: &mut Vec<Object> ) {
    game.messages.add(
        "You take a moment to rest, and recover your strength.",
        tcod::colors::VIOLET,
    );
    let heal_hp = player.get_fighter().map_or(0, |f| f.max_hp / 2);
    player.heal(heal_hp);

    game.messages.add(
        "After a rare moment of peace, you descend deeper into \
         the heart of the dungeon...",
        tcod::colors::RED,
    );
    game.dungeon_level += 1;
    game.map = make_map(player, enemies, collectibles);
    initialise_fov(tcod, &game.map);
}

fn make_map(player: &mut Player, enemies: &mut Vec<Enemy>, collectibles: &mut Vec<Object>) -> Map {
  collectibles.clear();
  enemies.clear();
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

      place_objects(new_room, enemies, collectibles, &mut map);

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
  // create stairs at the center of the last room
  let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
  let stairs = Object::create_stair(last_room_x, last_room_y);
  collectibles.push(stairs);

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

fn place_objects(room: Rect, enemies: &mut Vec<Enemy>, collectibles: &mut Vec<Object>, map: &Map) {
  // choose random number of monsters
  let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);
  
  //clone => dirty 
  let object_enemies = enemies
    .iter()
    .map(|obj| obj.get_object().clone())
    .collect::<Vec<Object>>();

  let concatenated_objects = [&object_enemies[..], &collectibles[..]].concat();

  for _ in 0..num_monsters {
    // choose random spot for this monster
    let x = rand::thread_rng().gen_range(room.x1() + 1, room.x2());
    let y = rand::thread_rng().gen_range(room.y1() + 1, room.y2());

    // monster random table
    let monster_chances = &mut [
        Weighted {
            weight: 80,
            item: "orc",
        },
        Weighted {
            weight: 20,
            item: "troll",
        },
    ];
    let monster_choice = WeightedChoice::new(monster_chances);

    if !is_blocked(x, y, map, &concatenated_objects) {
      let monster = match monster_choice.ind_sample(&mut rand::thread_rng()) {
          "orc" => {
              // create an orc
              Enemy::create_orc(x, y)
          }
          "troll" => {
              // create a troll
              Enemy::create_troll(x, y)
          }
          _ => unreachable!(),
      };
      enemies.push(monster);
    }
  }

  // choose random number of items
  let num_items = rand::thread_rng().gen_range(0, MAX_ROOM_ITEMS + 1);

  // item random table
  let item_chances = &mut [
      Weighted {
          weight: 70,
          item: Item::Heal,
      },
      Weighted {
          weight: 10,
          item: Item::Lightning,
      },
      Weighted {
          weight: 10,
          item: Item::Fireball,
      },
      Weighted {
          weight:10,
          item: Item::Confuse,
      },
  ];
  let item_choice = WeightedChoice::new(item_chances);

  for _ in 0..num_items {
    // choose random spot for this item
    let x = rand::thread_rng().gen_range(room.x1() + 1, room.x2());
    let y = rand::thread_rng().gen_range(room.y1() + 1, room.y2());

    // only place it if the tile is not blocked
    if !is_blocked(x, y, map, &concatenated_objects) {
      let item = match item_choice.ind_sample(&mut rand::thread_rng()) {
        Item::Heal => { Object::create_potion(x, y) }
        Item::Lightning => { Object::create_lighting_bolt(x, y) }
        Item::Fireball => { Object::create_fireball(x, y) }
        Item::Confuse => { Object::create_confuse_potion(x, y) }
      };
      collectibles.push(item);
    }
  }
}