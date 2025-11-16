use std::{collections::VecDeque, f32::consts::PI, sync::LazyLock};

use crate::{
    assets::{Assets, World},
    player::{ALIEN_BALL, Player, Projectile, ProjectileType, update_physicsbody},
};
use macroquad::prelude::*;

pub struct EnemyType {
    pub health: f32,
    pub states: Vec<EnemyState>,
}
pub enum ProjectileFiring {
    None,
    Forwards(&'static ProjectileType),
    Around(&'static ProjectileType, u8),
}
pub enum StateChangeCondition {
    Never,
    Always,
    HitWall,
    NearPlayer,
    AnimationFinish,
}
pub enum EnemyMovement {
    Chase,
    None,
    Pathfind,
    Straight,
}
pub struct EnemyState {
    pub animation_id: usize,
    pub speed: f32,
    pub movement: EnemyMovement,
    pub projectile_firing: ProjectileFiring,
    pub change_state: StateChangeCondition,
    pub damage_on_exit: Option<f32>,
}

pub static ENEMIES: LazyLock<Vec<EnemyType>> = LazyLock::new(|| {
    let greeno: EnemyType = EnemyType {
        states: vec![
            EnemyState {
                animation_id: 0,
                speed: 25.0,
                movement: EnemyMovement::Chase,
                projectile_firing: ProjectileFiring::None,
                change_state: StateChangeCondition::NearPlayer,
                damage_on_exit: None,
            },
            EnemyState {
                animation_id: 1,
                speed: 0.0,
                movement: EnemyMovement::Chase,
                projectile_firing: ProjectileFiring::None,
                change_state: StateChangeCondition::AnimationFinish,
                damage_on_exit: Some(15.0),
            },
        ],
        health: 20.0,
    };
    let dog: EnemyType = EnemyType {
        states: vec![
            EnemyState {
                animation_id: 2,
                speed: 80.0,
                movement: EnemyMovement::Chase,
                projectile_firing: ProjectileFiring::None,
                change_state: StateChangeCondition::NearPlayer,
                damage_on_exit: None,
            },
            EnemyState {
                animation_id: 3,
                speed: 0.0,
                movement: EnemyMovement::Chase,
                projectile_firing: ProjectileFiring::None,
                change_state: StateChangeCondition::AnimationFinish,
                damage_on_exit: Some(5.0),
            },
        ],
        health: 9.0,
    };
    let shooter: EnemyType = EnemyType {
        states: vec![EnemyState {
            animation_id: 4,
            speed: 0.0,
            movement: EnemyMovement::Chase,
            projectile_firing: ProjectileFiring::Forwards(&ALIEN_BALL),
            change_state: StateChangeCondition::AnimationFinish,
            damage_on_exit: None,
        }],
        health: 9.0,
    };
    let bigo: EnemyType = EnemyType {
        states: vec![
            EnemyState {
                animation_id: 5,
                speed: 0.0,
                movement: EnemyMovement::Chase,
                projectile_firing: ProjectileFiring::None,
                change_state: StateChangeCondition::Always,
                damage_on_exit: None,
            },
            EnemyState {
                animation_id: 5,
                speed: 160.0,
                movement: EnemyMovement::Straight,
                projectile_firing: ProjectileFiring::None,
                change_state: StateChangeCondition::HitWall,
                damage_on_exit: None,
            },
            EnemyState {
                animation_id: 6,
                speed: 0.0,
                movement: EnemyMovement::Chase,
                projectile_firing: ProjectileFiring::Around(&ALIEN_BALL, 10),
                change_state: StateChangeCondition::AnimationFinish,
                damage_on_exit: Some(10.0),
            },
        ],
        health: 80.0,
    };
    vec![greeno, dog, shooter, bigo]
});

pub struct Enemy {
    pub ty: &'static EnemyType,
    pub pos: Vec2,
    pub health: f32,
    pub animation_time: f32,
    pub direction: Vec2,
    pub path: Option<VecDeque<(i16, i16)>>,
    pub time_til_pathfind: f32,
    pub velocity: Vec2,
    pub emerging: bool,
    pub state: usize,
}
impl Enemy {
    pub fn new(ty: &'static EnemyType, pos: Vec2) -> Self {
        Self {
            ty,
            pos,
            health: ty.health,
            animation_time: 0.0,
            direction: vec2(1.0, 0.0),
            path: None,
            time_til_pathfind: 0.0,
            emerging: true,
            velocity: Vec2::ZERO,
            state: 0,
        }
    }
    fn current_state(&self) -> &'static EnemyState {
        &self.ty.states[self.state % self.ty.states.len()]
    }
    pub fn update(
        &mut self,
        delta_time: f32,
        player: &mut Player,
        world: &World,
        assets: &Assets,
        projectiles: &mut Vec<Projectile>,
    ) {
        self.animation_time += delta_time;
        if self.emerging && self.animation_time < HOLE_TIME {
            return;
        } else if self.emerging {
            self.emerging = false;
        }
        let delta = player.pos - self.pos;
        let mut hit_wall = false;
        let mut target = player.pos + 8.0;
        if delta.length() > 0.0 {
            self.time_til_pathfind -= delta_time;

            if matches!(self.current_state().movement, EnemyMovement::Pathfind)
                && (self.path.is_none() || self.time_til_pathfind <= 0.0)
            {
                self.time_til_pathfind = 2.0;
                self.path = world
                    .pathfind(self.pos, player.pos + 8.0)
                    .map(|f| f.0.into());
            }
            if let Some(path) = &mut self.path
                && let Some((x, y)) = path.get(1)
            {
                let next = vec2(*x as f32 * 16.0, *y as f32 * 16.0);
                if next.distance(self.pos) < 4.0 {
                    path.pop_front();
                }
                target = next;
            }
        }
        if matches!(self.current_state().movement, EnemyMovement::Straight) {
            target = self.pos + self.direction;
        }
        let distance = target.distance_squared(self.pos);
        if distance > 0.0 && !matches!(self.current_state().movement, EnemyMovement::None) {
            self.direction = (target - self.pos).normalize();
            self.velocity = (target - self.pos).normalize() * self.current_state().speed;
            let v = self.velocity;
            self.pos = update_physicsbody(self.pos, &mut self.velocity, delta_time, world);
            if self.velocity.length_squared() < v.length_squared() {
                hit_wall = true;
            }
        }

        if match self.current_state().change_state {
            StateChangeCondition::Always => true,
            StateChangeCondition::Never => false,
            StateChangeCondition::AnimationFinish => {
                matches!(
                    self.current_state().change_state,
                    StateChangeCondition::AnimationFinish
                ) && self.animation_time * 1000.0
                    >= assets.enemies.animations[self.current_state().animation_id].total_length
                        as f32
            }
            StateChangeCondition::NearPlayer => player.pos.distance_squared(self.pos) < 144.0,
            StateChangeCondition::HitWall => {
                hit_wall || player.pos.distance_squared(self.pos) < 144.0
            }
        } {
            if let Some(damage) = self.current_state().damage_on_exit
                && player.pos.distance_squared(self.pos) < 144.0
            {
                player.health -= damage;
            }
            match &self.current_state().projectile_firing {
                ProjectileFiring::None => {}
                ProjectileFiring::Forwards(projectile) => {
                    let new = Projectile {
                        ty: projectile,
                        pos: self.pos,
                        dir: self.direction,
                        time: 0.0,
                        friendly: false,
                    };
                    projectiles.push(new);
                }
                ProjectileFiring::Around(projectile, amt) => {
                    let angle = 2.0 * PI / *amt as f32;
                    for i in 0..*amt {
                        let angle = angle * i as f32 + self.direction.to_angle();
                        let new = Projectile {
                            ty: projectile,
                            pos: self.pos,
                            dir: Vec2::from_angle(angle),
                            time: 0.0,
                            friendly: false,
                        };
                        projectiles.push(new);
                    }
                }
            }
            self.state += 1;
            self.animation_time = 0.0;
        }
    }
    pub fn draw(&mut self, assets: &Assets) {
        if self.emerging && self.animation_time < HOLE_TIME {
            let max_hole_diameter = 20.0;
            let diameter = (self.animation_time / HOLE_EMERGE_TIME * max_hole_diameter)
                .min(max_hole_diameter)
                .floor();
            draw_ellipse(
                self.pos.x.floor(),
                self.pos.y.floor() + 8.0,
                diameter,
                diameter / 2.0,
                0.0,
                BLACK,
            );
            if self.animation_time > HOLE_EMERGE_TIME {
                let amt = (self.animation_time - HOLE_EMERGE_TIME) / (HOLE_TIME - HOLE_EMERGE_TIME);
                let amt = (amt - 1.0).powi(5) + 1.0;
                let pos = self.pos.floor() + vec2(0.0, 13.0 - amt * 13.0);
                draw_texture_ex(
                    assets.enemies.animations[self.current_state().animation_id]
                        .get_at_time((self.animation_time * 1000.0) as u32),
                    pos.x.floor() - 16.0,
                    pos.y.floor() - 16.0,
                    WHITE,
                    DrawTextureParams {
                        flip_x: self.direction.x > 0.0,
                        ..Default::default()
                    },
                );
            }
            return;
        }
        draw_texture_ex(
            assets.enemies.animations[self.current_state().animation_id]
                .get_at_time((self.animation_time * 1000.0) as u32),
            self.pos.x.floor() - 16.0,
            self.pos.y.floor() - 16.0,
            WHITE,
            DrawTextureParams {
                flip_x: self.direction.x > 0.0,
                ..Default::default()
            },
        );
        let width = 25.0;
        let height = 4.0;
        let pos = self.pos.floor() - 16.0 + vec2(0.0, -4.0) + (32.0 - width) / 2.0;
        draw_rectangle(pos.x - 1.0, pos.y - 1.0, width + 2.0, height + 2.0, BLACK);
        draw_rectangle(
            pos.x,
            pos.y,
            self.health / self.ty.health * width,
            height,
            HEALTHBAR_COLOR,
        );
    }
}
pub const HEALTHBAR_COLOR: Color = Color::from_hex(0x39741f);
const HOLE_EMERGE_TIME: f32 = 0.7;
const HOLE_TIME: f32 = 1.8;
