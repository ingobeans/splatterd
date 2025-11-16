use crate::assets::Assets;
use crate::player::Player;
use crate::utils::*;
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;

pub const PLAYER_HEALTH_COLOR: Color = Color::from_hex(0x87d1ef);

pub fn draw_death(assets: &Assets, time: f32, player: &Player, mouse: (f32, f32)) -> bool {
    let player_collapse = assets.player.animations[2].total_length as f32 / 1000.0;
    let fade_out_time = 1.5;
    let die_screen_time = 0.5;

    let p = player.camera_pos.floor() - vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);

    draw_texture(
        assets.player.animations[2]
            .get_at_time((time.min(player_collapse - 0.001) * 1000.0) as u32),
        player.pos.x.floor(),
        player.pos.y.floor(),
        WHITE,
    );

    if time < player_collapse {
        false
    } else if time < player_collapse + fade_out_time {
        let amt = (time - player_collapse) / fade_out_time;
        let amt = 2.0_f32.powf(amt.powi(2)) - 1.0;
        draw_rectangle(p.x, p.y, SCREEN_WIDTH, SCREEN_HEIGHT / 2.0 * amt, BLACK);
        draw_rectangle(
            p.x,
            p.y + SCREEN_HEIGHT - SCREEN_HEIGHT / 2.0 * amt,
            SCREEN_WIDTH,
            SCREEN_HEIGHT / 2.0 * amt,
            BLACK,
        );
        false
    } else {
        draw_rectangle(p.x, p.y, SCREEN_WIDTH, SCREEN_HEIGHT, BLACK);
        let amt = (time - player_collapse - fade_out_time) / die_screen_time;
        let texture = assets.die.get_at_time(0);
        let size = vec2(135.0, 23.0);
        let draw_pos = vec2(
            (SCREEN_WIDTH - texture.width()) / 2.0,
            (SCREEN_HEIGHT - texture.height()) / 2.0,
        );
        let texture_offset = vec2(30.0, 67.0);
        let hovered = (draw_pos.x + texture_offset.x..draw_pos.x + texture_offset.x + size.x)
            .contains(&mouse.0)
            && (draw_pos.y + texture_offset.y..draw_pos.y + texture_offset.y + size.y)
                .contains(&mouse.1);
        let draw_pos = vec2(
            p.x + (SCREEN_WIDTH - texture.width()) / 2.0,
            p.y + (SCREEN_HEIGHT - texture.height()) / 2.0,
        );
        let texture = assets.die.get_at_time(if hovered { 1 } else { 0 });
        draw_texture(&texture, draw_pos.x, draw_pos.y, WHITE.with_alpha(amt));
        hovered && is_mouse_button_pressed(MouseButton::Left)
    }
}
pub fn draw_escape_pod(
    assets: &Assets,
    time: f32,
    player: &mut Player,
    escape_pod: Vec2,
    escape_pod_door: Vec2,
    delta_time: f32,
) {
    let walk_time = 1.5;
    let fly_off_time = 2.0;
    let fade_out_time = 1.5;
    let win_screen_time = 0.5;

    if time == 0.0 {
        draw_texture_ex(
            &assets.escape_pod.animations[0].get_at_time(0),
            escape_pod.x,
            escape_pod.y,
            WHITE,
            DrawTextureParams::default(),
        );
        return;
    }

    let target = escape_pod_door + vec2(16.0, -8.0);
    let mut pos = target;
    let p = pos - vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
    if time < walk_time {
        pos = player.pos.lerp(target, (time - delta_time) / walk_time);
        draw_texture(
            assets.player.animations[1].get_at_time((time * 1000.0) as u32),
            pos.x.floor(),
            pos.y.floor(),
            WHITE,
        );
        pos = player.pos.lerp(target, time / walk_time);

        draw_texture_ex(
            &assets.escape_pod.animations[0].get_at_time(0),
            escape_pod.x,
            escape_pod.y,
            WHITE,
            DrawTextureParams::default(),
        );
    } else if time < walk_time + fly_off_time {
        let amt = (time - walk_time) / fly_off_time;
        let amt = 2.0_f32.powf(amt.powi(3)) - 1.0;
        let pod_pos = escape_pod.lerp(escape_pod + vec2(0.0, 1.0 * SCREEN_HEIGHT), amt);
        draw_texture_ex(
            &assets.escape_pod.animations[1].get_at_time((time * 1000.0) as u32),
            pod_pos.x,
            pod_pos.y,
            WHITE,
            DrawTextureParams::default(),
        );
    } else if time < walk_time + fly_off_time + fade_out_time {
        let amt = (time - walk_time - fly_off_time) / fade_out_time;
        let amt = 2.0_f32.powf(amt.powi(2)) - 1.0;
        draw_rectangle(p.x, p.y, SCREEN_WIDTH, SCREEN_HEIGHT / 2.0 * amt, BLACK);
        draw_rectangle(
            p.x,
            p.y + SCREEN_HEIGHT - SCREEN_HEIGHT / 2.0 * amt,
            SCREEN_WIDTH,
            SCREEN_HEIGHT / 2.0 * amt,
            BLACK,
        );
    } else {
        draw_rectangle(p.x, p.y, SCREEN_WIDTH, SCREEN_HEIGHT, BLACK);
        let amt = (time - walk_time - fly_off_time - fade_out_time) / win_screen_time;
        draw_texture(
            &assets.win,
            p.x + (SCREEN_WIDTH - assets.win.width()) / 2.0,
            p.y + (SCREEN_HEIGHT - assets.win.height()) / 2.0,
            WHITE.with_alpha(amt),
        );
    }
    player.camera_pos = pos.floor();
}

pub fn draw_ui(
    assets: &Assets,
    player: &Player,
    show_item_tooltip: bool,
    show_escape_tooltip: bool,
) {
    let (actual_screen_width, actual_screen_height) = screen_size();
    let scale_factor = (actual_screen_width / SCREEN_WIDTH)
        .min(actual_screen_height / SCREEN_HEIGHT)
        .floor()
        .max(1.0);

    let x = 10.0 * scale_factor;
    let y = 10.0 * scale_factor;
    draw_rectangle(
        x + 8.0 * scale_factor,
        y + 2.0 * scale_factor,
        170.0 * scale_factor,
        20.0 * scale_factor,
        BLACK,
    );
    draw_rectangle(
        x + 8.0 * scale_factor,
        y + 2.0 * scale_factor,
        170.0 * scale_factor * player.health / 100.0,
        20.0 * scale_factor,
        PLAYER_HEALTH_COLOR,
    );
    draw_texture_ex(
        &assets.healthbar,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                assets.tooltip.width() * scale_factor,
                assets.tooltip.height() * scale_factor,
            )),
            ..Default::default()
        },
    );

    let tooltip = if show_item_tooltip {
        Some(&assets.tooltip)
    } else if show_escape_tooltip {
        Some(&assets.escape_pod_tooltip)
    } else {
        None
    };
    if let Some(tooltip) = tooltip {
        let x = (actual_screen_width - tooltip.width() * scale_factor) / 2.0;
        let y = actual_screen_height - tooltip.height() * scale_factor - 4.0 * scale_factor;
        draw_texture_ex(
            &tooltip,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    tooltip.width() * scale_factor,
                    tooltip.height() * scale_factor,
                )),
                ..Default::default()
            },
        );
    }
}
