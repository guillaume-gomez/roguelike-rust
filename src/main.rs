use tcod::map::{Map as FovMap};
use tcod::colors::Color;
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
use enemy::Enemy;
use player::Player;
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

fn get_names_under_mouse(mouse: Mouse, enemys: &[Enemy], fov_map: &FovMap) -> String {
  let (x, y) = (mouse.cx as i32, mouse.cy as i32);

  let names = enemys
    .iter()
    .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.get_x(), obj.get_y()))
    .map(|obj| obj.get_name())
    .collect::<Vec<_>>();
  names.join(", ")
}

fn handle_keys(tcod: &mut Tcod, game: &mut Game, player: &mut Player, enemys: &mut[Enemy]) -> PlayerAction {
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

    _ => DidntTakeTurn,
  }
}

fn render_game(tcod: &mut Tcod, game: &Game, player: &Player, enemys: &[Enemy], fov_recompute: bool) {
  // prepare to render the GUI panel
  if fov_recompute {
    // recompute FOV if needed (the player moved or something)
    tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
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

fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color,
) {
  // render a bar (HP, experience, etc). First calculate the width of the bar
  let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

  // render the background first
  panel.set_default_background(back_color);
  panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

  // now render the bar on top
  panel.set_default_background(bar_color);
  if bar_width > 0 {
      panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
  }
  // finally, some centered text with the values
  panel.set_default_foreground(tcod::colors::WHITE);
  panel.print_ex(
      x + total_width / 2,
      y,
      BackgroundFlag::None,
      TextAlignment::Center,
      &format!("{}: {}/{}", name, value, maximum),
  );
}

fn render_gui(tcod: &mut Tcod, game: &Game, player: &Player, enemys: &[Enemy] ) {
  tcod.panel.set_default_background(tcod::colors::BLACK);
  tcod.panel.clear();

  // show the player's stats
  let hp = player.get_fighter().map_or(0, |f| f.hp);
  let max_hp = player.get_fighter().map_or(0, |f| f.max_hp);
  render_bar(
    &mut tcod.panel,
    1,
    1,
    BAR_WIDTH,
    "HP",
    hp,
    max_hp,
    tcod::colors::LIGHT_RED,
    tcod::colors::DARKER_RED,
  );

  render_messages(tcod, game);
  render_raycast(tcod, enemys);

  // blit the contents of `panel` to the root console
  blit(
    &tcod.panel,
    (0, 0),
    (SCREEN_WIDTH, PANEL_HEIGHT),
    &mut tcod.root,
    (0, PANEL_Y),
    1.0,
    1.0,
  );
}

fn render_messages(tcod: &mut Tcod, game: &Game) {
  // print the game messages, one line at a time
  let mut y = MSG_HEIGHT as i32;
  for &(ref msg, color) in game.messages.iter().rev() {
    let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    y -= msg_height;
    if y < 0 {
        break;
    }
    tcod.panel.set_default_foreground(color);
    tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
  }
}

fn render_raycast(tcod: &mut Tcod, enemys: &[Enemy]) {
  tcod.panel.set_default_foreground(tcod::colors::LIGHT_GREY);
  tcod.panel.print_ex(
    1,
    0,
    BackgroundFlag::None,
    TextAlignment::Left,
    get_names_under_mouse(tcod.mouse, enemys, &tcod.fov),
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
  let mut game = Game::new(&mut player, &mut enemys);

  // a warm welcoming message!
  game.messages.add(
    "Welcome stranger! Prepare to perish in the Tombs of the Ancient Kings.",
    tcod::colors::RED,
  );

  while !tcod.root.window_closed() {
    match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
      Some((_, Event::Mouse(m))) => tcod.mouse = m,
      Some((_, Event::Key(k))) => tcod.key = k,
      _ => tcod.key = Default::default(),
    }

    tcod.con.clear();
    tcod.panel.clear();
    

    let fov_recompute = previous_player_position != (player.pos());
    render_game(&mut tcod, &game, &player, &enemys, fov_recompute);
    render_gui(&mut tcod, &game, &player, &enemys);
    tcod.root.flush();

    previous_player_position = player.pos();
    let player_action = handle_keys(&mut tcod, &mut game, &mut player, &mut enemys);
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
