use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

mod object;
use object::Object;
mod tile;
mod game;
use game::Game;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

// size of the map (duplicated from game, TODO create a constant file)
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

struct Tcod {
    root: Root,
    con: Offscreen,
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

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    // draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }
    // go through all tiles, and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].is_block_sight();
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
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
    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(LIMIT_FPS);

    let game = Game::new();
    let character = Object::new(30, 40, '%', colors::GREEN);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::YELLOW);
    let mut objects = [character, npc];

    while !tcod.root.window_closed() {
        tcod.con.clear();
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
        tcod.con.set_default_foreground(colors::WHITE);
        render_all(&mut tcod, &game, &objects);
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
    }
}
