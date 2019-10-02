use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn handle_keys(tcod: &mut Tcod, object: &mut Object) -> bool {
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

struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

fn main() {
    let root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("Rust/libtcod tutorial")
    .init();

    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(LIMIT_FPS);

    let mut character = Object::new(30, 40, '%', colors::GREEN);

    while !tcod.root.window_closed() {
        tcod.con.clear();
        let exit = handle_keys(&mut tcod, &mut character);
        if exit {
            break;
        }
        tcod.con.set_default_foreground(colors::WHITE);
        character.draw(&mut tcod.con);
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
