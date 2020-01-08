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
mod input_output;
mod transition;
mod equipment;

use crate::game::next_level;
use crate::game::initialise_fov;
use crate::game::new_game;
use crate::input_output::load_game;
use crate::input_output::save_game;
use crate::hud::menu;

use crate::hud::inventory_menu;
use crate::hud::render_gui;
use tcod::map::{Map as FovMap};

use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use crate::constants::*;


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



fn play_game(mut tcod: &mut Tcod, mut game: &mut Game, mut player: &mut Player, mut enemies: &mut Vec<Enemy>, mut collectibles: &mut Vec<Object>) {
  // force FOV "recompute" first time through the game loop
  let mut previous_player_position = (-1, -1);

  while !tcod.root.window_closed() {
    match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
      Some((_, Event::Mouse(m))) => tcod.mouse = m,
      Some((_, Event::Key(k))) => tcod.key = k,
      _ => tcod.key = Default::default(),
    }

    tcod.con.clear();
    tcod.panel.clear();
    

    let fov_recompute = previous_player_position != (player.pos());
    render_game(&mut tcod, &game, &player, &enemies, &collectibles, fov_recompute);
    render_gui(&mut tcod, &game, &player, &enemies);
    tcod.root.flush();

    previous_player_position = player.pos();
    let player_action = handle_keys(&mut tcod, &mut game, &mut player, &mut enemies, &mut collectibles);
    if player_action == PlayerAction::Exit {
      save_game(game, player, enemies, collectibles).unwrap();
      break;
    }

    // let monsters take their turn
    if player.is_alive() && player_action != PlayerAction::DidntTakeTurn {
      for id in 0..enemies.len() {
        if enemies[id].get_ai().is_some() {
            let mut enemies_without_enemy = enemies.clone();
            enemies_without_enemy.clone_from_slice(&enemies);
            let mut enemy = enemies_without_enemy.remove(id);
            enemy.ai_take_turn(&tcod, &mut game, &enemies_without_enemy, &mut player);
            
            // update the original other Enemy array
            enemies[id] = enemy;
        }
      }
      player.level_up(tcod, game);
    }
  }
}

fn main_menu(mut tcod: &mut Tcod) {
  let img = tcod::image::Image::from_file("menu_background.png") 
      .ok()
      .expect("Background image not found");

  // show the background image, at twice the regular console resolution
  tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

  tcod.root.set_default_foreground(tcod::colors::LIGHT_YELLOW);
  tcod.root.print_ex(
      SCREEN_WIDTH / 2,
      SCREEN_HEIGHT / 2 - 4,
      BackgroundFlag::None,
      TextAlignment::Center,
      "TOMBS OF THE ANCIENT KINGS",
  );
  tcod.root.print_ex(
      SCREEN_WIDTH / 2,
      SCREEN_HEIGHT - 2,
      BackgroundFlag::None,
      TextAlignment::Center,
      "By Yours Truly",
  );

  while !tcod.root.window_closed() {
    // show the background image, at twice the regular console resolution
    tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

    // show options and wait for the player's choice
    let choices = &["Play a new game", "Continue last game", "Quit"];
    let choice = menu("", choices, 24, &mut tcod.root);

    match choice {
      Some(0) => {
          // new game
          let (mut game, mut player, mut enemies, mut collectibles) = new_game(&mut tcod);
          play_game(&mut tcod, &mut game, &mut player, &mut enemies, &mut collectibles);
      }
      Some(1) => {
        // load game
        match load_game() {
          Ok((mut game, mut player, mut enemies, mut collectibles)) => {
            initialise_fov(tcod, &game.map);
            play_game(&mut tcod, &mut game, &mut player, &mut enemies, &mut collectibles);
          }
          Err(_e) => {
            msgbox("\nNo saved game to load.\n", 24, &mut tcod.root);
            continue
          }
        }
      }
      Some(2) => {
          // quit
          break;
      }
      _ => {}
    }
  }
}




fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}

pub fn target_tile(tcod: &mut Tcod, game: &mut Game, player: &Player, enemies: &[Enemy], collectibles: &[Object], max_range: Option<f32>) -> Option<(i32, i32)> {
    use tcod::input::KeyCode::Escape;
    loop {
        // render the screen. this erases the inventory and shows the names of
        // objects under the mouse.
        tcod.root.flush();
        let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
        match event {
            Some(Event::Mouse(m)) => tcod.mouse = m,
            Some(Event::Key(k)) => tcod.key = k,
            None => tcod.key = Default::default(),
        }
        render_game(tcod, &game, &player, &enemies, &collectibles, false);

        let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

        // accept the target if the player clicked in FOV, and in case a range
        // is specified, if it's in that range
        let in_fov = (x < MAP_WIDTH) && (y < MAP_HEIGHT) && tcod.fov.is_in_fov(x, y);
        let in_range = max_range.map_or(true, |range| player.distance(x, y) <= range);
        if tcod.mouse.lbutton_pressed && in_fov && in_range {
            return Some((x, y));
        }

        if tcod.mouse.rbutton_pressed || tcod.key.code == Escape {
            return None; // cancel if the player right-clicked or pressed Escape
        }
    }
}

fn handle_keys(tcod: &mut Tcod, game: &mut Game, player: &mut Player, enemies: &mut Vec<Enemy>, collectibles: &mut Vec<Object>) -> PlayerAction {
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
      player.move_or_attack(0, -1, game, enemies);
      TookTurn
    }
    (Key { code: Down, .. }, _, true) => {
      player.move_or_attack(0, 1, game, enemies);
      TookTurn
    }
    (Key { code: Left, .. }, _, true) => {
      player.move_or_attack(-1, 0, game, enemies);
      TookTurn
    }
    (Key { code: Right, .. }, _, true) => {
      player.move_or_attack(1, 0, game, enemies);
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
    (Key { code: Text, .. }, "i", true) => {
      // show the inventory: if an item is selected, use it
      let inventory_index = inventory_menu(
        &game.inventory,
        "Press the key next to an item to use it, or any other to cancel.\n",
        &mut tcod.root,
      );
      if let Some(inventory_index) = inventory_index {
        player.use_item(game, tcod, inventory_index, collectibles, enemies);
      }
      DidntTakeTurn
    }
    (Key { code: Text, .. }, "d", true) => {
      // show the inventory; if an item is selected, drop it
      let inventory_index = inventory_menu(
        &game.inventory,
        "Press the key next to an item to drop it, or any other to cancel.\n'",
        &mut tcod.root,
      );
      if let Some(inventory_index) = inventory_index {
        player.drop_item(inventory_index, game, collectibles);
      }
      DidntTakeTurn
    }(Key { code: Text, .. }, "<", true) => {
      // go down stairs, if the player is on them
      let player_on_stairs = collectibles
        .iter()
        .any(|object| object.pos() == player.pos() && object.get_name() == "stairs");
      if player_on_stairs {
          next_level(tcod, game, player, enemies, collectibles);
      }
      DidntTakeTurn
    }
    (Key { code: Text, .. }, "c", true) => {
      // show character information
      let level = player.get_level();
      let level_up_xp = player.level_up_xp();
      if let Some(fighter) = player.get_fighter() {
        let msg = format!(
          "Character information
          Level: {}
          Experience: {}
          Experience to level up: {}

          Maximum HP: {}
          Attack: {}
          Defense: {}",
          level, fighter.xp, level_up_xp, fighter.max_hp, fighter.power, fighter.defense
        );
        msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
      }

      DidntTakeTurn
  }
    _ => DidntTakeTurn,
  }
}

fn render_game(tcod: &mut Tcod, game: &Game, player: &Player, enemies: &[Enemy], collectibles: &[Object], fov_recompute: bool) {
  // prepare to render the GUI panel
  if fov_recompute {
    // recompute FOV if needed (the player moved or something)
    tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
  }

  // draw all collectibles
  for collectible in collectibles {
    let (x, y) = collectible.pos();
    if tcod.fov.is_in_fov(x, y) || collectible.always_visible() {
      collectible.draw(&mut tcod.con);
    }
  }

  // draw all enemies in the list
  for enemy in enemies {
    if tcod.fov.is_in_fov(enemy.get_x(), enemy.get_y()) {
      enemy.draw(&mut tcod.con);
    }
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
  tcod::system::set_fps(LIMIT_FPS);

  let root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("RogueLike-rust")
    .init();

  let mut tcod = Tcod {
    root,
    con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
    panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
    fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    key: Default::default(),
    mouse: Default::default(),
  };

  main_menu(&mut tcod);
}
