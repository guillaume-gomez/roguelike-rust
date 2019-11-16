use crate::hud::render_gui;
use tcod::map::{Map as FovMap};

use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use crate::constants::*;

mod constants;
mod game;
mod rect;
mod tile;
mod object;
mod player;
mod enemy;
mod fighter;
mod messages;
mod hud;
use enemy::Enemy;
use player::Player;
use object::Object;
use game::Game;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
  TookTurn,
  DidntTakeTurn,
  Exit,
}

pub struct Tcod {
  root: Root,
  con: Offscreen,
  panel: Offscreen,
  fov: FovMap,
  key: Key,
  mouse: Mouse,
}


fn handle_keys(tcod: &mut Tcod, game: &mut Game, player: &mut Player, enemys: &mut[Enemy], collectibles: &mut Vec<Object>) -> PlayerAction {
  use tcod::input::KeyCode::*;
  use PlayerAction::*;

  match (tcod.key, tcod.key.text(), player.is_alive()) {
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
      player.move_or_attack(0, -1, game, enemys);
      TookTurn
    }
    (Key { code: Down, .. }, _, true) => {
      player.move_or_attack(0, 1, game, enemys);
      TookTurn
    }
    (Key { code: Left, .. }, _, true) => {
      player.move_or_attack(-1, 0, game, enemys);
      TookTurn
    }
    (Key { code: Right, .. }, _, true) => {
      player.move_or_attack(1, 0, game, enemys);
      TookTurn
    }
    (Key { code: Text, .. }, "g", true) => {
      // pick up an item
      let item_id = collectibles
          .iter()
          .position(|object| object.pos() == player.pos() && object.item.is_some());
      if let Some(item_id) = item_id {
          player.pick_item_up(item_id, game, collectibles);
      }
      DidntTakeTurn
    }

    _ => DidntTakeTurn,
  }
}

fn render_game(tcod: &mut Tcod, game: &Game, player: &Player, enemys: &[Enemy], collectibles: &[Object], fov_recompute: bool) {
  // prepare to render the GUI panel
  if fov_recompute {
    // recompute FOV if needed (the player moved or something)
    tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
  }

  // draw all collectibles
  for collectible in collectibles {
    //if tcod.fov.is_in_fov(collectible.get_x(), collectible.get_y()) {
      collectible.draw(&mut tcod.con);
    //}
  }

  // draw all Enemys in the list
  // TODO UNCOMMENT FOV NEXT TIME
  for enemy in enemys {
    //if tcod.fov.is_in_fov(Enemy.get_x(), Enemy.get_y()) {
      enemy.draw(&mut tcod.con);
    //}
  }
  //render player
  player.draw(&mut tcod.con);
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

  let mut tcod = Tcod {
    root,
    fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
    panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
    key: Default::default(),
    mouse: Default::default(),

  };
  tcod::system::set_fps(LIMIT_FPS);

  let mut previous_player_position = (-1, -1);
  let mut player = Player::new(0, 0);
  let mut enemys = vec![];
  let mut collectibles = vec![];
  let mut game = Game::new(&mut player, &mut enemys, &mut collectibles);

  // a warm welcoming message!
  game.messages.add(
    "Welcome stranger! Prepare to perish in the Tombs of the Ancient Kings.",
    tcod::colors::RED,
  );

  println!("{:?}", collectibles.len() );

  while !tcod.root.window_closed() {
    match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
      Some((_, Event::Mouse(m))) => tcod.mouse = m,
      Some((_, Event::Key(k))) => tcod.key = k,
      _ => tcod.key = Default::default(),
    }

    tcod.con.clear();
    tcod.panel.clear();
    

    let fov_recompute = previous_player_position != (player.pos());
    render_game(&mut tcod, &game, &player, &enemys, &collectibles, fov_recompute);
    render_gui(&mut tcod, &game, &player, &enemys);
    tcod.root.flush();

    previous_player_position = player.pos();
    let player_action = handle_keys(&mut tcod, &mut game, &mut player, &mut enemys, &mut collectibles);
    if player_action == PlayerAction::Exit {
      break;
    }

    // let monsters take their turn
    if player.is_alive() && player_action != PlayerAction::DidntTakeTurn {
      for id in 0..enemys.len() {
        if enemys[id].get_ai().is_some() {
            let mut enemys_without_enemy = enemys.clone();
            enemys_without_enemy.clone_from_slice(&enemys);
            let mut enemy = enemys_without_enemy.remove(id);
            enemy.ai_take_turn(&tcod, &mut game, &enemys_without_enemy, &mut player);
            
            // update the original other Enemy array
            enemys[id] = enemy;
        }
      }
    }
  }
}
