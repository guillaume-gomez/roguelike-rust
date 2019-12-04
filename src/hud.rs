use crate::object::Object;
use crate::constants::*;
use crate::game::Game;
use crate::player::Player;
use tcod::console::*;
use tcod::colors::Color;
use crate::Tcod;
use tcod::map::{Map as FovMap};
use tcod::input::Mouse;
use crate::enemy::Enemy;

fn get_names_under_mouse(mouse: Mouse, enemys: &[Enemy], fov_map: &FovMap) -> String {
  let (x, y) = (mouse.cx as i32, mouse.cy as i32);

  let names = enemys
    .iter()
    .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.get_x(), obj.get_y()))
    .map(|obj| obj.get_name())
    .collect::<Vec<_>>();
  names.join(", ")
}



pub fn render_bar(
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

pub fn render_gui(tcod: &mut Tcod, game: &Game, player: &Player, enemys: &[Enemy] ) {
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


pub fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    // how a menu with each item of the inventory as an option
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory.iter().map(|item| item.name.clone()).collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    // if an item was chosen, return it
    if inventory.len() > 0 {
        inventory_index
    } else {
        None
    }
}


pub fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
  assert!(
    options.len() <= MAX_INVENTORY,
    format!("Cannot have a menu with more than {} options.", MAX_INVENTORY)
  );

  
 // calculate total height for the header (after auto-wrap) and one line per option
  let header_height = if header.is_empty() {
      0
  } else {
      root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
  };
  let height = options.len() as i32 + header_height;

  // create an off-screen console that represents the menu's window
  let mut window = Offscreen::new(width, height);

  // print the header, with auto-wrap
  window.set_default_foreground(tcod::colors::WHITE);
  window.print_rect_ex(
      0,
      0,
      width,
      height,
      BackgroundFlag::None,
      TextAlignment::Left,
      header,
  );
  
  // print all the options
  for (index, option_text) in options.iter().enumerate() {
    let menu_letter = (b'a' + index as u8) as char;
    let text = format!("({}) {}", menu_letter, option_text.as_ref());
    window.print_ex(
        0,
        header_height + index as i32,
        BackgroundFlag::None,
        TextAlignment::Left,
        text,
    );
  }

  // blit the contents of "window" to the root console
  let x = SCREEN_WIDTH / 2 - width / 2;
  let y = SCREEN_HEIGHT / 2 - height / 2;
  blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

  // present the root console to the player and wait for a key-press
  root.flush();
  let key = root.wait_for_keypress(true);

  // convert the ASCII code to an index; if it corresponds to an option, return it
  if key.printable.is_alphabetic() {
      let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
      if index < options.len() {
          Some(index)
      } else {
          None
      }
  } else {
      None
  }
}
