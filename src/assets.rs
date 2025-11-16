use std::{collections::HashMap, iter::Map};

use asefile::{self, AsepriteFile};
use image::EncodableLayout;
use macroquad::prelude::*;

use crate::{
    player::{WEAPONS, Weapon},
    utils::*,
};

pub struct Assets {
    pub tileset: Spritesheet,
    pub player: AnimationsGroup,
    pub enemies: AnimationsGroup,
    pub projectiles: AnimationsGroup,
    pub locker: Animation,
    pub tooltip: Texture2D,
    pub escape_pod: AnimationsGroup,
    pub escape_pod_tooltip: Texture2D,
    pub healthbar: Texture2D,
    pub win: Texture2D,
    pub menu: Animation,
    pub die: Animation,
}
impl Default for Assets {
    fn default() -> Self {
        Self {
            tileset: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/tileset.ase"), None),
                16.0,
            ),
            player: AnimationsGroup::from_file(include_bytes!("../assets/player.ase")),
            enemies: AnimationsGroup::from_file(include_bytes!("../assets/enemies.ase")),
            escape_pod: AnimationsGroup::from_file(include_bytes!("../assets/escape_pod.ase")),
            projectiles: AnimationsGroup::from_file(include_bytes!("../assets/projectiles.ase")),
            locker: Animation::from_file(include_bytes!("../assets/locker.ase")),
            tooltip: load_ase_texture(include_bytes!("../assets/tooltip.ase"), None),
            escape_pod_tooltip: load_ase_texture(
                include_bytes!("../assets/escape_pod_tooltip.ase"),
                None,
            ),
            healthbar: load_ase_texture(include_bytes!("../assets/healthbar.ase"), None),
            win: load_ase_texture(include_bytes!("../assets/win.ase"), None),
            menu: Animation::from_file(include_bytes!("../assets/menu.ase")),
            die: Animation::from_file(include_bytes!("../assets/die.ase")),
        }
    }
}

pub struct StarsBackground {
    stars: Vec<(Vec2, f32)>,
}
pub struct AnimationsGroup {
    #[expect(dead_code)]
    pub file: AsepriteFile,
    pub animations: Vec<Animation>,
}
impl AnimationsGroup {
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        let mut tag_frames = Vec::new();
        let mut offset = 0;

        for i in 0..ase.num_tags() {
            let tag = ase.get_tag(i).unwrap();
            let (start, end) = (tag.from_frame() as usize, tag.to_frame() as usize);
            let mut total_length = 0;
            let included_frames: Vec<(Texture2D, u32)> = frames
                .extract_if((start - offset)..(end - offset + 1), |_| true)
                .collect();
            for f in included_frames.iter() {
                total_length += f.1;
            }
            offset += end.abs_diff(start) + 1;
            tag_frames.push(Animation {
                frames: included_frames,
                total_length,
            });
        }
        Self {
            file: ase,
            animations: tag_frames,
        }
    }
}
pub struct Animation {
    frames: Vec<(Texture2D, u32)>,
    pub total_length: u32,
}
impl Animation {
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        let mut total_length = 0;
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            total_length += duration;
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        Self {
            frames,
            total_length,
        }
    }
    pub fn get_at_time(&self, mut time: u32) -> &Texture2D {
        time %= self.total_length;
        for (texture, length) in self.frames.iter() {
            if time >= *length {
                time -= length;
            } else {
                return texture;
            }
        }
        panic!()
    }
}
const MAX_STAR_SPEED: f32 = 10.0;
const MIN_STAR_SPEED: f32 = 5.0;
impl StarsBackground {
    pub fn new() -> Self {
        let star_density = 0.005;
        let stars_count = (SCREEN_WIDTH * SCREEN_HEIGHT * star_density) as usize;
        let mut stars: Vec<(Vec2, f32)> = Vec::with_capacity(stars_count);
        for _ in 0..stars_count {
            let pos = Vec2::new(
                rand::gen_range(0, SCREEN_WIDTH as usize) as f32,
                rand::gen_range(0, SCREEN_HEIGHT as usize) as f32,
            );
            stars.push((pos, rand::gen_range(MIN_STAR_SPEED, MAX_STAR_SPEED)));
        }
        Self { stars }
    }
    pub fn draw(&mut self, delta_time: f32, offset: Vec2) {
        self.stars.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (pos, star_speed) in self.stars.iter_mut() {
            pos.y += delta_time * *star_speed;
            if pos.y > SCREEN_HEIGHT {
                *pos = Vec2::new(rand::gen_range(0, SCREEN_WIDTH as usize) as f32, 0.0);
            }
            let value = 255
                - ((MAX_STAR_SPEED - *star_speed) / (MAX_STAR_SPEED - MIN_STAR_SPEED) * 250.0)
                    as u8;
            let color = Color::from_rgba(value, value, value, 255);
            draw_rectangle(
                pos.x.floor() + offset.x.floor() - SCREEN_WIDTH / 2.0,
                pos.y.floor() + offset.y.floor() - SCREEN_HEIGHT / 2.0,
                1.0,
                1.0,
                color,
            );
        }
    }
}
pub enum TileEntityUpdateResult {
    None,
}
type TileEntityDraw = &'static dyn Fn(&mut TileEntity, &Assets, Vec2) -> TileEntityUpdateResult;
#[derive(Clone, Copy)]
pub struct TileEntity {
    pub collision: bool,
    pub enabled: bool,
    pub draw: TileEntityDraw,
    pub tile_index: i16,
}
pub const BARRIER: TileEntity = TileEntity {
    collision: true,
    enabled: true,
    tile_index: 0,
    draw: &|this, assets, pos| {
        assets.tileset.draw_tile(
            pos.x,
            pos.y,
            (this.tile_index % 16) as f32,
            (this.tile_index / 16) as f32,
            None,
        );
        TileEntityUpdateResult::None
    },
};
impl TileEntity {
    pub fn instantiate(&self, tile_index: i16) -> Self {
        let mut new = *self;
        new.tile_index = tile_index;
        new
    }
}

pub struct World {
    pub collision: Vec<Chunk>,
    pub details: Vec<Chunk>,
    pub background: Vec<Chunk>,
    pub background_details: Vec<Chunk>,
    pub interactable: Vec<Chunk>,

    pub lockers: Vec<(Vec2, Option<&'static Weapon>)>,
    pub tile_entities: HashMap<(i16, i16), TileEntity>,

    pub x_min: i16,
    pub x_max: i16,
    pub y_min: i16,
    pub y_max: i16,
}

fn get_tile(chunks: &[&Chunk], x: i16, y: i16) -> i16 {
    let cx = ((x as f32 / 16.0).floor() * 16.0) as i16;
    let cy = ((y as f32 / 16.0).floor() * 16.0) as i16;
    let Some(chunk) = chunks.iter().find(|f| f.x == cx && f.y == cy) else {
        return 0;
    };
    let local_x = x - chunk.x;
    let local_y = y - chunk.y;
    chunk.tile_at(local_x as _, local_y as _).unwrap_or(0)
}
type SuccessorIterator = Map<std::vec::IntoIter<(i16, i16)>, fn((i16, i16)) -> ((i16, i16), i16)>;

fn generate_successors(pos: (i16, i16), chunks: &[&Chunk]) -> SuccessorIterator {
    let mut candidates = vec![(pos.0 + 1, pos.1), (pos.0, pos.1 + 1)];
    if pos.0 > 0 {
        candidates.push((pos.0 - 1, pos.1));
    }
    if pos.1 > 0 {
        candidates.push((pos.0, pos.1 - 1));
    }
    candidates.retain(|(cx, cy)| get_tile(chunks, *cx, *cy) == 0);
    fn map_function(p: (i16, i16)) -> ((i16, i16), i16) {
        (p, 1)
    }
    let mapped: SuccessorIterator = candidates.into_iter().map(map_function);
    mapped
}
#[expect(dead_code)]
impl World {
    pub fn pathfind(&self, from: Vec2, to: Vec2) -> Option<(Vec<(i16, i16)>, i16)> {
        let to = to / 16.0;
        let from = from / 16.0;
        let tiles: [Vec2; 4] = [
            from - vec2(1.0, 0.0),
            from - vec2(0.0, 1.0),
            from + vec2(1.0, 0.0),
            from + vec2(0.0, 1.0),
        ];
        let chunks_pos: [(i16, i16); 4] = std::array::from_fn(|f| {
            let cx = ((tiles[f].x / 16.0).floor() * 16.0) as i16;
            let cy = ((tiles[f].y / 16.0).floor() * 16.0) as i16;
            (cx, cy)
        });
        let chunks: Vec<&Chunk> = self
            .collision
            .iter()
            .filter(|f| chunks_pos.contains(&(f.x, f.y)))
            .collect();
        let to = (to.x as i16, to.y as i16);
        pathfinding::prelude::astar(
            &(from.x as i16, from.y as i16),
            |p| generate_successors(*p, &chunks),
            |&(x, y)| (to.0.abs_diff(x) as i16 + to.1.abs_diff(y) as i16) / 3,
            |&p| p == to,
        )
    }
    pub fn get_interactable_spawn(&self, tile_index: i16) -> Option<Vec2> {
        for chunk in self.interactable.iter() {
            for (i, tile) in chunk.tiles.iter().enumerate() {
                if *tile == tile_index + 1 {
                    return Some(Vec2::new(
                        (i as i16 % 16 + chunk.x) as f32 * 16.0,
                        (i as i16 / 16 + chunk.y) as f32 * 16.0,
                    ));
                }
            }
        }
        None
    }
    pub fn set_collision_tile(&mut self, x: i16, y: i16, tile: i16) {
        let cx = ((x as f32 / 16.0).floor() * 16.0) as i16;
        let cy = ((y as f32 / 16.0).floor() * 16.0) as i16;

        let chunk = self
            .collision
            .iter_mut()
            .find(|f| f.x == cx && f.y == cy)
            .unwrap();
        chunk.tiles[(x - chunk.x + (y - chunk.y) * 16) as usize] = tile;
    }
}
impl Default for World {
    fn default() -> Self {
        let xml = include_str!("../assets/station.tmx");
        let collision = get_layer(xml, "Collision");
        let detail = get_layer(xml, "Detail");
        let interactable = get_layer(xml, "Interactable");
        let background = get_layer(xml, "Background");
        let background_details = get_layer(xml, "BackgroundDetails");
        let mut world = World {
            collision: get_all_chunks(collision),
            details: get_all_chunks(detail),
            background: get_all_chunks(background),
            interactable: get_all_chunks(interactable),
            background_details: get_all_chunks(background_details),
            lockers: Vec::new(),
            tile_entities: HashMap::new(),
            x_min: 999,
            y_min: 999,
            y_max: -999,
            x_max: -999,
        };

        // define x y min and max
        for layer in [
            &world.collision,
            &world.details,
            &world.background,
            &world.interactable,
        ] {
            for chunk in layer {
                if chunk.x < world.x_min {
                    world.x_min = chunk.x;
                }
                if chunk.y < world.y_min {
                    world.y_min = chunk.y;
                }
                if chunk.x > world.x_max {
                    world.x_max = chunk.x;
                }
                if chunk.y > world.y_max {
                    world.y_max = chunk.y;
                }
            }
        }

        let tile_entities = get_all_chunks(get_layer(xml, "TileEntities"));
        for chunk in &world.interactable {
            for (index, tile) in chunk.tiles.iter().enumerate() {
                let tile = tile - 1;
                if (112..=127).contains(&tile) {
                    let x = (index % 16) as i16 + chunk.x;
                    let y = (index / 16) as i16 + chunk.y;
                    world.lockers.push((
                        vec2(x as f32 * 16.0, y as f32 * 16.0),
                        Some(WEAPONS[tile as usize - 112]),
                    ));
                }
            }
        }
        for chunk in &tile_entities {
            for (index, tile) in chunk.tiles.iter().enumerate() {
                let tile = tile - 1;
                if tile <= -1 {
                    continue;
                }
                let x = (index % 16) as i16 + chunk.x;
                let y = (index / 16) as i16 + chunk.y;
                if (81..=83).contains(&tile) {
                    world
                        .tile_entities
                        .insert((x, y), BARRIER.instantiate(tile));
                }
            }
        }

        world
    }
}
pub struct Chunk {
    pub x: i16,
    pub y: i16,
    pub tiles: Vec<i16>,
}
impl Chunk {
    pub fn tile_at(&self, x: usize, y: usize) -> Option<i16> {
        if x > 16 {
            return None;
        }
        self.tiles.get(x + y * 16).cloned()
    }
    pub fn draw(&self, assets: &Assets) {
        for (index, tile) in self.tiles.iter().enumerate() {
            if *tile == 0 {
                continue;
            }
            let tile = *tile - 1;
            let x = index % 16;
            let y = index / 16;
            assets.tileset.draw_tile(
                (self.x * 16) as f32 + (x * 16) as f32,
                (self.y * 16) as f32 + (y * 16) as f32,
                (tile % 16) as f32,
                (tile / 16) as f32,
                None,
            );
        }
    }
}
fn get_all_chunks(xml: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut xml = xml.to_string();
    while let Some((current, remains)) = xml.split_once("</chunk>") {
        let new = parse_chunk(current);
        chunks.push(new);
        xml = remains.to_string();
    }

    chunks
}
fn parse_chunk(xml: &str) -> Chunk {
    let (tag, data) = xml
        .split_once("<chunk ")
        .unwrap()
        .1
        .split_once(">")
        .unwrap();

    let x = tag
        .split_once("x=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();
    let y = tag
        .split_once("y=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();

    let mut split = data.split(',');

    let mut chunk = vec![0; 16 * 16];
    for item in &mut chunk {
        let a = split.next().unwrap().trim();
        *item = a.parse().unwrap()
    }
    Chunk { x, y, tiles: chunk }
}

fn get_layer<'a>(xml: &'a str, layer: &str) -> &'a str {
    let split = format!(" name=\"{layer}");
    xml.split_once(&split)
        .unwrap()
        .1
        .split_once(">")
        .unwrap()
        .1
        .split_once("</layer>")
        .unwrap()
        .0
}

fn load_ase_texture(bytes: &[u8], layer: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(0).image()
    } else {
        img.frame(0).image()
    };
    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}
pub struct Spritesheet {
    pub texture: Texture2D,
    pub sprite_size: f32,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: f32) -> Self {
        Self {
            texture,
            sprite_size,
        }
    }
    /// Same as `draw_tile`, except centered
    pub fn draw_sprite(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        self.draw_tile(
            screen_x - self.sprite_size / 2.0,
            screen_y - self.sprite_size / 2.0,
            tile_x,
            tile_y,
            params,
        );
    }
    /// Draws a single tile from the spritesheet
    pub fn draw_tile(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        let mut p = params.cloned().unwrap_or(DrawTextureParams::default());
        p.dest_size = p
            .dest_size
            .or(Some(Vec2::new(self.sprite_size, self.sprite_size)));
        p.source = p.source.or(Some(Rect {
            x: tile_x * self.sprite_size,
            y: tile_y * self.sprite_size,
            w: self.sprite_size,
            h: self.sprite_size,
        }));
        draw_texture_ex(&self.texture, screen_x, screen_y, WHITE, p);
    }
}
