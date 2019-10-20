use tcod::map::{FovAlgorithm, Map as FovMap};
use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

mod object;
mod tile;
mod game;
mod rect;
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

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

fn handle_keys(tcod: &mut Tcod, game: &Game, object: &mut Object) -> bool {
    // todo: handle keys
    let key = tcod.root.wait_for_keypress(true);
    match key {
        // movement keys
        Key { code: Up, .. } => object.move_by(0, -1, game),
        Key { code: Down, .. } => object.move_by(0, 1, game),
        Key { code: Left, .. } => object.move_by(-1, 0, game),
        Key { code: Right, .. } => object.move_by(1, 0, game),

        _ => {}
    }
    false
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object], fov_recompute: bool) {
  if fov_recompute {
    // recompute FOV if needed (the player moved or something)
    let player = &objects[0];
    tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
  }
  
  // draw all objects in the list
  for object in objects {
    if tcod.fov.is_in_fov(object.get_x(), object.get_y()) {
      object.draw(&mut tcod.con);
    }
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
  let character = Object::new(0, 0, '%', colors::GREEN, "player", true);
  let mut objects = vec![character];
  let game = Game::new(&mut objects);

  while !tcod.root.window_closed() {
    tcod.con.clear();
    let player = &mut objects[0];
    previous_player_position = (player.get_x(), player.get_y());
    let exit = handle_keys(&mut tcod, &game, player);
    if exit {
      break;
    }
    tcod.con.set_default_foreground(colors::WHITE);
    let fov_recompute = previous_player_position != (player.get_x(), player.get_y());
    render_all(&mut tcod, &game, &objects, fov_recompute);
    tcod.root.flush();
    tcod.root.wait_for_keypress(true);
  }
}
