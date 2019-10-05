use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

mod items;
// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

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

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn handle_keys(tcod: &mut Tcod, object: &mut items::object::Object) -> bool {
    // todo: handle keys
    let key = tcod.root.wait_for_keypress(true);
    match key {
        // movement keys
        Key { code: Up, .. } => object.move_by(0, -1),
        Key { code: Down, .. } => object.move_by(0, 1),
        Key { code: Left, .. } => object.move_by(-1, 0),
        Key { code: Right, .. } => object.move_by(1, 0),

        _ => {}
    }
    false
}

fn main() {
    let root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("RogueLike-rust")
    .init();

    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(LIMIT_FPS);

    let character = items::object::Object::new(30, 40, '%', colors::GREEN);
    let npc = items::object::Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::YELLOW);
    let mut objects = [character, npc];

    while !tcod.root.window_closed() {
        tcod.con.clear();
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player);
        if exit {
            break;
        }
        tcod.con.set_default_foreground(colors::WHITE);
        for object in &objects {
            object.draw(&mut tcod.con);
        }
        blit(
            &tcod.con,
            (0, 0),
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0,
            1.0,
        );
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
    }
}
