use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{assets::*, enemy::*, player::*, utils::*};

mod assets;
mod enemy;
mod graphics;
mod player;
mod utils;

pub enum GameState {
    Running,
    Die(f32),
    Escape(f32),
}
impl GameState {
    fn running(&self) -> bool {
        matches!(self, GameState::Running)
    }
}

struct Game<'a> {
    assets: &'a Assets,
    world: World,
    player: Player,
    pixel_camera: Camera2D,
    world_camera_bg: Camera2D,
    world_camera_fg: Camera2D,
    stars: StarsBackground,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    escape_pod_door: Vec2,
    escape_pod: Vec2,
    state: GameState,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        let world = World::default();

        let world_width = ((world.x_max - world.x_min) * 16) as f32 + 16.0 * 16.0;
        let world_height = ((world.y_max - world.y_min) * 16) as f32 + 16.0 * 16.0;

        // render world
        let mut world_camera_bg = create_camera(world_width, world_height);
        world_camera_bg.target = vec2(
            (world.x_min + world.x_max + 16) as f32 / 2.0 * 16.0,
            (world.y_min + world.y_max + 16) as f32 / 2.0 * 16.0,
        );
        set_camera(&world_camera_bg);
        clear_background(BLACK.with_alpha(0.0));

        for chunk in &world.background {
            chunk.draw(assets);
        }
        for chunk in &world.collision {
            chunk.draw(assets);
        }
        for chunk in &world.background_details {
            chunk.draw(assets);
        }
        let mut world_camera_fg = create_camera(world_width, world_height);
        world_camera_fg.target = vec2(
            (world.x_min + world.x_max + 16) as f32 / 2.0 * 16.0,
            (world.y_min + world.y_max + 16) as f32 / 2.0 * 16.0,
        );
        set_camera(&world_camera_fg);
        clear_background(BLACK.with_alpha(0.0));
        for chunk in &world.details {
            chunk.draw(assets);
        }

        let pixel_camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);

        let mut player = Player::new();
        player.pos = world.get_interactable_spawn(16).unwrap();

        Self {
            escape_pod_door: world.get_interactable_spawn(128).unwrap() + vec2(0.0, 8.0),
            escape_pod: world.get_interactable_spawn(129).unwrap(),
            player,
            assets,
            world,
            pixel_camera,
            world_camera_bg,
            world_camera_fg,
            enemies: Vec::with_capacity(10), // todo: adjust capcacity later on?
            stars: StarsBackground::new(),
            projectiles: Vec::with_capacity(10),
            state: GameState::Running,
        }
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_x = mouse_x / scale_factor;
        let mouse_y = mouse_y / scale_factor;

        if self.state.running() {
            self.player.update(
                delta_time,
                &mut self.world,
                &mut self.enemies,
                &mut self.projectiles,
                (mouse_x, mouse_y),
            );
        }
        match &mut self.state {
            GameState::Die(t) => *t += delta_time,
            GameState::Escape(t) => *t += delta_time,
            _ => {}
        }
        self.pixel_camera.target = self.player.camera_pos.floor();
        set_camera(&self.pixel_camera);
        clear_background(BLACK);
        self.stars.draw(delta_time, self.player.camera_pos);

        // draw world texture
        draw_texture_ex(
            &self.world_camera_bg.render_target.as_ref().unwrap().texture,
            (self.world.x_min * 16) as f32,
            (self.world.y_min * 16) as f32,
            WHITE,
            DrawTextureParams::default(),
        );
        let mut can_take_weapon = false;

        for (locker_pos, slot) in self.world.lockers.iter_mut() {
            if (self.player.pos + vec2(-8.0, 8.0)).distance_squared(*locker_pos) < 512.0 {
                draw_texture_ex(
                    &self.assets.locker.get_at_time(1),
                    locker_pos.x,
                    locker_pos.y - 48.0 + 16.0,
                    WHITE,
                    DrawTextureParams::default(),
                );
                if let Some(weapon) = slot {
                    can_take_weapon = true;
                    self.assets.tileset.draw_tile(
                        locker_pos.x + 8.0,
                        locker_pos.y - 8.0,
                        WEAPONS.iter().position(|f| f == weapon).unwrap() as f32,
                        7.0,
                        None,
                    );
                    if is_key_pressed(KeyCode::E) {
                        std::mem::swap(&mut self.player.weapon, slot);
                    }
                }
            } else {
                draw_texture_ex(
                    &self.assets.locker.get_at_time(0),
                    locker_pos.x,
                    locker_pos.y - 48.0 + 16.0,
                    WHITE,
                    DrawTextureParams::default(),
                );
            }
        }

        for ((x, y), entity) in self.world.tile_entities.iter_mut() {
            let pos = vec2(*x as f32, *y as f32) * 16.0;
            (entity.draw)(entity, self.assets, pos);
        }
        if self.state.running() {
            self.player.draw(self.assets, (mouse_x, mouse_y));
        }
        self.enemies.retain_mut(|enemy| {
            enemy.update(
                delta_time,
                &mut self.player,
                &self.world,
                self.assets,
                &mut self.projectiles,
            );
            enemy.draw(self.assets);
            enemy.health > 0.0
        });

        self.projectiles.retain_mut(|projectile| {
            projectile.update(
                self.assets,
                &mut self.enemies,
                &mut self.player,
                &self.world,
                delta_time,
            )
        });
        draw_texture_ex(
            &self.world_camera_fg.render_target.as_ref().unwrap().texture,
            (self.world.x_min * 16) as f32,
            (self.world.y_min * 16) as f32,
            WHITE,
            DrawTextureParams::default(),
        );
        let escaping_animation = if let GameState::Escape(time) = &self.state {
            *time
        } else {
            0.0
        };
        graphics::draw_escape_pod(
            self.assets,
            escaping_animation,
            &mut self.player,
            self.escape_pod,
            self.escape_pod_door,
            delta_time,
        );
        if let GameState::Die(time) = &self.state {
            if graphics::draw_death(self.assets, *time, &self.player, (mouse_x, mouse_y)) {
                *self = Self::new(self.assets)
            }
        }
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &self.pixel_camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor,
                    SCREEN_HEIGHT * scale_factor,
                )),
                ..Default::default()
            },
        );
        let by_escape_pod =
            self.state.running() && self.player.pos.distance_squared(self.escape_pod_door) < 256.0;
        if by_escape_pod && is_key_pressed(KeyCode::E) {
            self.state = GameState::Escape(0.0);
        }
        if self.player.health <= 0.0 && self.state.running() {
            self.state = GameState::Die(0.0);
        }
        if self.state.running() {
            graphics::draw_ui(self.assets, &self.player, can_take_weapon, by_escape_pod);
        }
    }
}

struct GameManager<'a> {
    assets: &'a Assets,
    stars: StarsBackground,
    pixel_camera: Camera2D,
    game: Option<Game<'a>>,
    transition_time: f32,
}
impl<'a> GameManager<'a> {
    fn new(assets: &'a Assets) -> Self {
        let mut pixel_camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
        pixel_camera.target = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
        Self {
            assets,
            pixel_camera,
            stars: StarsBackground::new(),
            game: None,
            transition_time: 0.0,
        }
    }
    fn update(&mut self) {
        let transition_length = 1.0;
        let delta_time = get_frame_time();
        let (actual_screen_width, actual_screen_height) = screen_size();
        match &mut self.game {
            Some(game) if self.transition_time > transition_length / 2.0 => {
                game.update();
                if self.transition_time < transition_length {
                    self.transition_time += delta_time;
                    let amt = (self.transition_time - transition_length / 2.0)
                        / (transition_length / 2.0);
                    let amt = 1.0 - (2.0_f32.powf(amt.powi(2)) - 1.0);
                    draw_rectangle(
                        0.0,
                        0.0,
                        actual_screen_width,
                        actual_screen_height,
                        BLACK.with_alpha(amt),
                    );
                }
            }
            _ => {
                if self.transition_time > 0.0 {
                    self.transition_time += delta_time;
                }
                let scale_factor =
                    (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
                let (mouse_x, mouse_y) = mouse_position();
                let mouse_x = mouse_x / scale_factor;
                let mouse_y = mouse_y / scale_factor;
                set_camera(&self.pixel_camera);
                clear_background(BLACK);
                self.stars.draw(delta_time, self.pixel_camera.target);

                let offset = vec2(20.0, 20.0);
                let texture_offset = vec2(6.0, 43.0);
                let button_size = vec2(136.0, 23.0);

                let hovered_play_button = (offset.x + texture_offset.x
                    ..offset.x + texture_offset.x + button_size.x)
                    .contains(&mouse_x)
                    && (offset.y + texture_offset.y..offset.y + texture_offset.y + button_size.y)
                        .contains(&mouse_y);
                draw_texture(
                    &self
                        .assets
                        .menu
                        .get_at_time(if hovered_play_button { 1 } else { 0 }),
                    offset.x,
                    offset.y,
                    WHITE,
                );

                set_default_camera();
                clear_background(BLACK);
                draw_texture_ex(
                    &self.pixel_camera.render_target.as_ref().unwrap().texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            SCREEN_WIDTH * scale_factor,
                            SCREEN_HEIGHT * scale_factor,
                        )),
                        ..Default::default()
                    },
                );
                if self.transition_time > 0.0 {
                    let amt = self.transition_time / (transition_length / 2.0);
                    let amt = 2.0_f32.powf(amt.powi(2)) - 1.0;
                    draw_rectangle(
                        0.0,
                        0.0,
                        actual_screen_width,
                        actual_screen_height,
                        BLACK.with_alpha(amt),
                    );
                }
                if hovered_play_button && is_mouse_button_pressed(MouseButton::Left) {
                    self.game = Some(Game::new(self.assets));
                    self.transition_time += delta_time;
                }
            }
        }
    }
}

#[macroquad::main("space splatter")]
async fn main() {
    let assets = Assets::default();
    let mut game_manager = GameManager::new(&assets);
    loop {
        game_manager.update();
        next_frame().await
    }
}
