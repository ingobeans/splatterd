use std::{borrow::Borrow, collections::HashMap, f32::consts::PI};

use macroquad::prelude::*;

use crate::{
    assets::{Assets, BARRIER, Chunk, World},
    enemy::{ENEMIES, Enemy},
    utils::*,
};

fn tile_to_chunk(pos: (i16, i16)) -> (i16, i16) {
    let cx = ((pos.0 as f32 / 16.0).floor() * 16.0) as i16;
    let cy = ((pos.1 as f32 / 16.0).floor() * 16.0) as i16;
    (cx, cy)
}

fn vec2_to_tile(pos: Vec2) -> (i16, i16) {
    let cx = (pos.x / 16.0).floor() as i16;
    let cy = (pos.y / 16.0).floor() as i16;
    (cx, cy)
}

fn get_connected_spawners(chunks: &[Chunk], start: (i16, i16)) -> Vec<((i16, i16), i16)> {
    fn recurse(
        chunks: &[Chunk],
        start: (i16, i16),
        checked: &mut Vec<(i16, i16)>,
        result: &mut Vec<((i16, i16), i16)>,
    ) {
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for dir in dirs {
            let pos = (start.0 + dir.0, start.1 + dir.1);
            if checked.contains(&pos) {
                continue;
            }

            let tile = get_tile(chunks, pos.0, pos.1) - 1;
            if tile > -1 {
                checked.push(pos);
                recurse(chunks, pos, checked, result);
                if tile != 32 {
                    result.push(((pos.0, pos.1), tile));
                }
            }
        }
    }
    let mut result = Vec::new();
    recurse(chunks, start, &mut Vec::new(), &mut result);
    result
}

#[derive(PartialEq)]
pub struct ProjectileType {
    pub animation_index: usize,
    pub speed: f32,
    pub damage: f32,
}
pub struct Projectile {
    pub ty: &'static ProjectileType,
    pub pos: Vec2,
    pub dir: Vec2,
    pub time: f32,
    pub friendly: bool,
}
impl Projectile {
    pub fn update(
        &mut self,
        assets: &Assets,
        enemies: &mut [Enemy],
        player: &mut Player,
        world: &World,
        splatter: &mut Vec<(Vec2, Vec2, f32)>,
        delta_time: f32,
    ) -> bool {
        self.pos += self.dir * self.ty.speed * delta_time;

        if self.friendly {
            if let Some(enemy) = enemies
                .iter_mut()
                .find(|enemy| enemy.pos.distance_squared(self.pos) < 256.0)
            {
                if enemy.emerging {
                    return false;
                }
                enemy.health -= self.ty.damage;
                splatter.push((enemy.pos, self.dir, rand::gen_range(0, 6) as f32));
                return false;
            }
        } else if player.pos.distance_squared(self.pos) < 256.0 {
            player.health -= self.ty.damage;
            return false;
        }

        let (tx, ty) = vec2_to_tile(self.pos);
        let (cx, cy) = tile_to_chunk((tx, ty));
        if let Some(chunk) = world.collision.iter().find(|f| f.x == cx && f.y == cy)
            && let Some(tile) = chunk.tile_at((tx - cx) as _, (ty - cy) as _).map(|f| f - 1)
            && tile > -1
        {
            return false;
        }
        draw_texture_ex(
            assets.projectiles.animations[self.ty.animation_index]
                .get_at_time((self.time * 1000.0) as u32),
            self.pos.x.floor() - 8.0,
            self.pos.y.floor() - 8.0,
            WHITE,
            DrawTextureParams {
                rotation: self.dir.to_angle(),
                ..Default::default()
            },
        );
        true
    }
}

pub static ENERGY_BALL: ProjectileType = ProjectileType {
    animation_index: 0,
    speed: 160.0,
    damage: 4.0,
};
pub static ENERGY_SHOT: ProjectileType = ProjectileType {
    animation_index: 2,
    speed: 200.0,
    damage: 2.0,
};
pub static ALIEN_BALL: ProjectileType = ProjectileType {
    animation_index: 1,
    speed: 100.0,
    damage: 4.0,
};
#[derive(PartialEq)]
pub struct Weapon {
    pub projectile: &'static ProjectileType,
    pub attack_delay: f32,
    pub multishot: Option<(u8, f32)>,
}
pub static GUN: Weapon = Weapon {
    projectile: &ENERGY_BALL,
    attack_delay: 1.0 / 3.0,
    multishot: None,
};
pub static RIFLE: Weapon = Weapon {
    projectile: &ENERGY_SHOT,
    attack_delay: 1.0 / 7.0,
    multishot: None,
};
pub static SHOTGUN: Weapon = Weapon {
    projectile: &ENERGY_BALL,
    attack_delay: 0.6,
    multishot: Some((3, PI / 5.0)),
};
pub static WEAPONS: &[&Weapon] = &[&GUN, &RIFLE, &SHOTGUN];

pub struct Player {
    pub weapon: Option<&'static Weapon>,
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub animation_time: f32,
    pub walking: bool,
    pub moving_left: bool,
    pub health: f32,
    pub spawned_spawners: Vec<(i16, i16)>,
    pub attack_counter: f32,
}
impl Player {
    pub fn new() -> Self {
        Self {
            weapon: None,
            pos: Vec2::ZERO,
            camera_pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            animation_time: 0.0,
            walking: false,
            moving_left: false,
            health: 100.0,
            spawned_spawners: Vec::new(),
            attack_counter: 0.0,
        }
    }
    pub fn update(
        &mut self,
        delta_time: f32,
        world: &mut World,
        enemies: &mut Vec<Enemy>,
        projectiles: &mut Vec<Projectile>,
        mouse: (f32, f32),
    ) {
        self.animation_time += delta_time;
        self.walking = false;
        let axis = get_input_axis();
        if axis.length() > 0.0 {
            self.walking = true;
            if axis.x < 0.0 {
                self.moving_left = true;
            } else if axis.x > 0.0 {
                self.moving_left = false;
            }
            self.velocity += axis.normalize() * delta_time * 3600.0;
        }
        self.attack_counter -= delta_time;
        if self.attack_counter <= 0.0
            && let Some(weapon) = self.weapon
            && is_mouse_button_down(MouseButton::Left)
        {
            self.attack_counter = weapon.attack_delay;
            let mut new = Vec::new();
            let multishot = weapon.multishot.unwrap_or((1, 0.0));
            let per_angle = multishot.1 / multishot.0 as f32;
            for i in 0..multishot.0 {
                let angle = (vec2(mouse.0, mouse.1)
                    - vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0))
                .normalize()
                .to_angle()
                    + i as f32 * per_angle
                    - per_angle * multishot.0 as f32 / 2.0;

                new.push(Projectile {
                    ty: weapon.projectile,
                    time: 0.0,
                    pos: self.pos + 8.0,
                    dir: Vec2::from_angle(angle),
                    friendly: true,
                });
            }
            projectiles.append(&mut new);
        }
        let friction = if axis.length() == 0.0 { 20.0 } else { 10.0 } * delta_time;
        self.velocity = self
            .velocity
            .clamp_length_max(2.0 * 70.0)
            .lerp(Vec2::ZERO, friction);
        let new = update_physicsbody(self.pos, &mut self.velocity, delta_time, world);
        self.walking &= self.velocity.length_squared() > 0.1;
        self.pos = new;
        let (tx, ty) = vec2_to_tile(self.pos);
        let (cx, cy) = tile_to_chunk((tx, ty));
        let mut new_spawned = Vec::new();
        let mut tile_entities = HashMap::new();
        let mut new_enemies = Vec::new();
        std::mem::swap(&mut tile_entities, &mut world.tile_entities);
        if let Some(chunk) = world.interactable.iter().find(|f| f.x == cx && f.y == cy)
            && let Some(tile) = chunk.tile_at((tx - cx) as _, (ty - cy) as _).map(|f| f - 1)
            && tile > -1
            && tile == 32
        {
            let tiles = get_connected_spawners(&world.interactable, (tx, ty));
            for ((x, y), tile) in tiles
                .into_iter()
                .filter(|(p, _)| !self.spawned_spawners.contains(p))
            {
                match tile {
                    96..111 => {
                        new_spawned.push((x, y));
                        let enemy = Enemy::new(
                            &ENEMIES[tile as usize - 96],
                            vec2(x as f32 * 16.0, y as f32 * 16.0),
                        );
                        new_enemies.push(enemy);
                    }
                    64 => {
                        if enemies.is_empty() && self.weapon.is_some() {
                            tile_entities.retain(|p, _| p != &(x, y));
                        }
                    }
                    82 | 83 => {
                        tile_entities.insert((x, y), BARRIER.instantiate(tile));
                    }
                    _ => panic!(),
                }
            }
        }
        enemies.append(&mut new_enemies);
        std::mem::swap(&mut tile_entities, &mut world.tile_entities);
        self.spawned_spawners.append(&mut new_spawned);
        self.camera_pos = self.pos
    }
    pub fn draw(&self, assets: &Assets, mouse: (f32, f32)) {
        draw_texture_ex(
            assets.player.animations[if self.walking { 1 } else { 0 }]
                .get_at_time((self.animation_time * 1000.0) as u32),
            self.pos.x.floor(),
            self.pos.y.floor(),
            WHITE,
            DrawTextureParams {
                flip_x: mouse.0 < SCREEN_WIDTH / 2.0,
                ..Default::default()
            },
        );
        if let Some(weapon) = &self.weapon {
            assets.tileset.draw_tile(
                self.pos.x.floor() + 7.0,
                self.pos.y.floor(),
                WEAPONS.iter().position(|f| f == weapon).unwrap() as f32,
                7.0,
                Some(&DrawTextureParams {
                    rotation: (vec2(mouse.0, mouse.1)
                        - vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0))
                    .to_angle(),
                    flip_y: mouse.0 < SCREEN_WIDTH / 2.0,
                    pivot: Some(self.pos.floor() + 8.0),
                    ..Default::default()
                }),
            );
        }
    }
}
fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}
fn get_tile<T: Borrow<Chunk>>(chunks: &[T], x: i16, y: i16) -> i16 {
    let cx = ((x as f32 / 16.0).floor() * 16.0) as i16;
    let cy = ((y as f32 / 16.0).floor() * 16.0) as i16;
    let Some(chunk) = chunks.iter().find(|f| {
        let f: &Chunk = (*f).borrow();
        f.x == cx && f.y == cy
    }) else {
        return 0;
    };
    let chunk = chunk.borrow();
    let local_x = x - chunk.x;
    let local_y = y - chunk.y;
    chunk.tile_at(local_x as _, local_y as _).unwrap_or(0)
}

pub fn update_physicsbody(pos: Vec2, velocity: &mut Vec2, delta_time: f32, world: &World) -> Vec2 {
    let mut new = pos + *velocity * delta_time;

    let tile_x = pos.x / 16.0;
    let tile_y = pos.y / 16.0;

    let tiles_y = [
        (tile_x.trunc(), ceil_g(new.y / 16.0)),
        (ceil_g(tile_x), ceil_g(new.y / 16.0)),
        (tile_x.trunc(), (new.y / 16.0).trunc()),
        (ceil_g(tile_x), (new.y / 16.0).trunc()),
    ];

    let chunks_pos: [(i16, i16); 4] = std::array::from_fn(|f| {
        let cx = ((tiles_y[f].0 / 16.0).floor() * 16.0) as i16;
        let cy = ((tiles_y[f].1 / 16.0).floor() * 16.0) as i16;
        (cx, cy)
    });

    let chunks: Vec<&Chunk> = world
        .collision
        .iter()
        .filter(|f| chunks_pos.contains(&(f.x, f.y)))
        .collect();

    for (tx, ty) in tiles_y {
        let tile = get_tile(&chunks, tx as i16, ty as i16);
        if tile != 0
            || world
                .tile_entities
                .get(&(tx as i16, ty as i16))
                .is_some_and(|f| f.collision && f.enabled)
        {
            let c = if velocity.y < 0.0 {
                tile_y.floor() * 16.0
            } else {
                tile_y.ceil() * 16.0
            };
            new.y = c;
            velocity.y = 0.0;
            break;
        }
    }
    let tiles_x = [
        ((new.x / 16.0).trunc(), ceil_g(new.y / 16.0)),
        (ceil_g(new.x / 16.0), ceil_g(new.y / 16.0)),
        (ceil_g(new.x / 16.0), (new.y / 16.0).trunc()),
        ((new.x / 16.0).trunc(), (new.y / 16.0).trunc()),
    ];

    let chunks_pos: [(i16, i16); 4] = std::array::from_fn(|f| {
        let cx = ((tiles_x[f].0 / 16.0).floor() * 16.0) as i16;
        let cy = ((tiles_x[f].1 / 16.0).floor() * 16.0) as i16;
        (cx, cy)
    });

    let chunks: Vec<&Chunk> = world
        .collision
        .iter()
        .filter(|f| chunks_pos.contains(&(f.x, f.y)))
        .collect();

    for (tx, ty) in tiles_x {
        let tile = get_tile(&chunks, tx as i16, ty as i16);
        if tile != 0
            || world
                .tile_entities
                .get(&(tx as i16, ty as i16))
                .is_some_and(|f| f.collision && f.enabled)
        {
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 16.0
            } else {
                tile_x.ceil() * 16.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        }
    }
    new
}
