use tcod::map::{FovAlgorithm, Map as FovMap};

use tcod::colors::Color;
use tcod::console::*;

mod object;
mod tile;
mod game;
mod rect;
mod fighter;
mod death_callback;
use object::Object;
use game::Game;


// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50 };

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50 };


// size of the map (duplicated from game, TODO create a constant file)
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object, other_objects: &mut[Object]) -> PlayerAction {
  use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let key = tcod.root.wait_for_keypress(true);
    match (key, key.text(), player.is_alive()) {
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
            _,
        ) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }
        (Key { code: Escape, .. }, _, _) => Exit, // exit game

        // movement keys
        (Key { code: Up, .. }, _, true) => {
           player.move_or_attack(0, -1, game, other_objects);
            TookTurn
        }
        (Key { code: Down, .. }, _, true) => {
            player.move_or_attack(0, 1, game, other_objects);
            TookTurn
        }
        (Key { code: Left, .. }, _, true) => {
            player.move_or_attack(-1, 0, game, other_objects);
            TookTurn
        }
        (Key { code: Right, .. }, _, true) => {
            player.move_or_attack(1, 0, game, other_objects);
            TookTurn
        }

        _ => DidntTakeTurn,
    }
}

fn render_all(tcod: &mut Tcod, game: &Game, player: &Object, objects: &[Object], fov_recompute: bool) {
  if fov_recompute {
    // recompute FOV if needed (the player moved or something)
    tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
  }


  //render player
  player.draw(&mut tcod.con);
  // draw all objects in the list
  
  // TODO UNCOMMENT NEXT TIME
  for object in objects {
    //if tcod.fov.is_in_fov(object.get_x(), object.get_y()) {
      object.draw(&mut tcod.con);
    //}
  }
  // go through all tiles, and set their background color
  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      let visible = tcod.fov.is_in_fov(x, y);
      let wall = game.map[x as usize][y as usize].is_block_sight();
      let color = match (visible, wall) {
          // outside of field of view:
          (false, true) => COLOR_DARK_WALL,
          (false, false) => COLOR_DARK_GROUND,
          // inside fov:
          (true, true) => COLOR_LIGHT_WALL,
          (true, false) => COLOR_LIGHT_GROUND,
      };
      tcod.con.set_char_background(x, y, color, BackgroundFlag::Set);
    }
  }
  blit(
    &tcod.con,
    (0, 0),
    (MAP_WIDTH, MAP_HEIGHT),
    &mut tcod.root,
    (0, 0),
    1.0,
    1.0,
  );
}

fn main() {
  let root = Root::initializer()
  .font("arial10x10.png", FontLayout::Tcod)
  .font_type(FontType::Greyscale)
  .size(SCREEN_WIDTH, SCREEN_HEIGHT)
  .title("RogueLike-rust")
  .init();

  let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
  let fov = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
  let mut tcod = Tcod { root, con, fov };
  tcod::system::set_fps(LIMIT_FPS);

  let mut previous_player_position = (-1, -1);
  let mut player = Object::create_player(0, 0);
  let mut other_objects = vec![];
  let game = Game::new(&mut player, &mut other_objects);

  while !tcod.root.window_closed() {
    tcod.con.clear();
    
    let fov_recompute = previous_player_position != (player.pos());
    render_all(&mut tcod, &game, &player, &other_objects, fov_recompute);
    tcod.root.flush();

    previous_player_position = player.pos();
    let player_action = handle_keys(&mut tcod, &game, &mut player, &mut other_objects);
    if player_action == PlayerAction::Exit {
      break;
    }

    // let monsters take their turn
    if player.is_alive() && player_action != PlayerAction::DidntTakeTurn {
      for id in 0..other_objects.len() {
        if other_objects[id].get_ai().is_some() {
            let mut other_objects_without_enemy = other_objects.clone();
            other_objects_without_enemy.clone_from_slice(&other_objects);
            let mut enemy = other_objects_without_enemy.remove(id);
            enemy.ai_take_turn(&tcod, &game, &other_objects_without_enemy, &mut player);
            
            // update the original other object array
            other_objects[id] = enemy;
        }
      }
    }
  }
}
