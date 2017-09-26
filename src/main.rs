extern crate rand;
extern crate tcod;

use tcod::console::*;
use tcod::colors;
use tcod::map::{Map as FovMap, FovAlgorithm};

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

// fov
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;  // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true;  // light walls or not
const TORCH_RADIUS: i32 = 10;

const COLOR_DARK_WALL: colors::Color = colors::Color {
    r: 0,
    g: 0,
    b: 200,
};
const COLOR_DARK_GROUND: colors::Color = colors::Color {
    r: 0,
    g: 50,
    b: 0,
};
const COLOR_LIGHT_WALL: colors::Color = colors::Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_LIGHT_GROUND: colors::Color = colors::Color {
    r: 200,
    g: 180,
    b: 50,
};

fn main() {
    let mut root = Root::initializer()
        .font("./fonts/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("i wish i was asleep")
        .init();
    tcod::system::set_fps(LIMIT_FPS);
    let mut con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let (mut tmap, (start_x, start_y)) = make_tilemap();
    let player = Object::new(start_x, start_y, '@', colors::WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '#', colors::YELLOW);
    let mut objects = [player, npc];

    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(x,
                        y,
                        !tmap[y * MAP_WIDTH + x].block_sight,
                        !tmap[y * MAP_WIDTH + x].blocked);
        }
    }
    let mut previous_player_position = (-1, -1);

    while !root.window_closed() {
        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        render_all(&mut root,
                   &mut con,
                   &objects,
                   &mut tmap,
                   &mut fov_map,
                   fov_recompute);
        root.flush();

        // erase all objects at their old locations, before they move
        for object in &objects {
            object.clear(&mut con)
        }

        // handle keys and exit game if needed
        previous_player_position = (objects[0].x, objects[0].y);
        let exit = handle_keys(&mut root, &mut objects[0]);
        if exit {
            break;
        }
    }
}

fn handle_keys(root: &mut Root, player: &mut Object) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let toggle_fullscreen = |root: &mut Root| {
        let fullscreen = root.is_fullscreen();
        root.set_fullscreen(!fullscreen);
    };

    let key = root.wait_for_keypress(true);
    match key {
        Key { code: Enter, alt: true, .. } => {
            // Alt+Enter: toggle fullscreen
            toggle_fullscreen(root);
        }
        Key { code: Escape, .. } => return true,  // exit game

        // movement keys
        Key { printable: 'w', .. } => player.move_by(0, -1),
        Key { printable: 's', .. } => player.move_by(0, 1),
        Key { printable: 'a', .. } => player.move_by(-1, 0),
        Key { printable: 'd', .. } => player.move_by(1, 0),

        _ => {}
    }

    false
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        // returns true if this rectangle intersects with another one
        (self.x1 <= other.x2) && (self.x2 >= other.x1) && (self.y1 <= other.y2) &&
        (self.y2 >= other.y1)
    }

    const ROOM_MAX_SIZE: i32 = 10;
    const ROOM_MIN_SIZE: i32 = 6;
    const MAX_ROOMS: i32 = 30;
}

fn create_room(room: Rect, map: &mut TileMap) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[y * MAP_WIDTH + x] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut TileMap) {
    for x in std::cmp::min(x1, x2)..(std::cmp::max(x1, x2) + 1) {
        map[y * MAP_WIDTH + x] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut TileMap) {
    for y in std::cmp::min(y1, y2)..(std::cmp::max(y1, y2) + 1) {
        map[y * MAP_WIDTH + x] = Tile::empty();
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
        }
    }
}

struct TileMap {
    data: Vec<Tile>,
}

impl TileMap {
    pub fn from_vec(data: Vec<Tile>) -> TileMap {
        TileMap { data: data }
    }
}

use std::ops::{Index, IndexMut};

impl Index<i32> for TileMap {
    type Output = Tile;

    fn index(&self, idx: i32) -> &Tile {
        &self.data[idx as usize]
    }
}

impl IndexMut<i32> for TileMap {
    fn index_mut<'a>(&'a mut self, idx: i32) -> &'a mut Tile {
        &mut self.data[idx as usize]
    }
}

fn make_tilemap() -> (TileMap, (i32, i32)) {
    use rand::Rng;

    // fill map with "blocked" tiles
    let map = vec![Tile::wall(); (MAP_HEIGHT * MAP_WIDTH) as usize];
    let mut tmap = TileMap::from_vec(map);
    let mut rooms = vec![];
    let mut starting_position = (0, 0);

    for _ in 0..Rect::MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(Rect::ROOM_MIN_SIZE, Rect::ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(Rect::ROOM_MIN_SIZE, Rect::ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));
        if failed {
            // this means there are intersections, so this room is invalid (just skip)
            continue;
        }

        // "paint" it to the map's tiles
        create_room(new_room, &mut tmap);

        // center coordinates of the new room, will be useful later
        let (new_x, new_y) = new_room.center();

        if rooms.is_empty() {
            // this is the first room, where the player starts at
            starting_position = (new_x, new_y);
        } else {
            // all rooms after the first:
            // connect it to the previous room with a tunnel

            // center coordinates of the previous room
            let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

            // toss a coin (random bool value -- either true or false)
            if rand::random() {
                // first move horizontally, then vertically
                create_h_tunnel(prev_x, new_x, prev_y, &mut tmap);
                create_v_tunnel(prev_y, new_y, new_x, &mut tmap);
            } else {
                // first move vertically, then horizontally
                create_v_tunnel(prev_y, new_y, prev_x, &mut tmap);
                create_h_tunnel(prev_x, new_x, new_y, &mut tmap);
            }
        }

        // finally, append the new room to the list
        rooms.push(new_room);
    }
    (tmap, starting_position)
}

fn render_all(root: &mut Root,
              con: &mut Offscreen,
              objects: &[Object],
              tmap: &mut TileMap,
              fov_map: &mut FovMap,
              fov_recompute: bool) {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[0];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);

        // go through all tiles, and set their background color
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let visible = fov_map.is_in_fov(x, y);
                let wall = tmap[y * MAP_WIDTH + x].block_sight;
                let color = match (visible, wall) {
                    // outside of field of view:
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    // inside fov:
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };
                let explored = &mut tmap[y * MAP_WIDTH + x].explored;
                if visible {
                    // since it's visible, explore it
                    *explored = true;
                }
                if *explored {
                    // show explored tiles only (any visible tile is explored already)
                    con.set_char_background(x, y, color, BackgroundFlag::Set);
                }
                // con.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    // draw all objects in the list
    for object in objects {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }
    }

    // blit the contents of "con" to the root console
    let source_xy = (0, 0);
    let source_dimensions = (MAP_WIDTH, MAP_HEIGHT);
    let dest_xy = (0, 0);
    let foreground_alpha = 1.0;
    let bg_alpha = 1.0;
    blit(con,
         source_xy,
         source_dimensions,
         root,
         dest_xy,
         foreground_alpha,
         bg_alpha);
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: colors::Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: colors::Color) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        // move by the given amount
        self.x += dx;
        self.y += dy;
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    /// Erase the character that represents this object
    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }
}
