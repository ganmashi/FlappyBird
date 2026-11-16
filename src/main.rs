use macroquad::prelude::*;

const BIRD_X: f32 = 130.0;
const BIRD_R: f32 = 15.0;
const GRAVITY: f32 = 1150.0;
const JUMP_V: f32 = -390.0;
const MAX_FALL: f32 = 540.0;
const PIPE_W: f32 = 74.0;
const GAP_H: f32 = 190.0;
const PIPE_SPEED: f32 = 165.0;
const PIPE_INTERVAL: f32 = 1.55;
const GROUND_H: f32 = 90.0;

#[derive(PartialEq, Clone, Copy)]
enum State {
    Ready,
    Playing,
    Over,
}

struct Pipe {
    x: f32,
    top: f32,
    scored: bool,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Rust".to_owned(),
        window_width: 800,
        window_height: 480,
        high_dpi: true,
        ..Default::default()
    }
}

fn rng(seed: &mut u64) -> f32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*seed >> 33) as f32) / 4_294_967_295.0
}

fn rect_hit(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let tw = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (screen_width() - tw) / 2.0, y, size, color);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = State::Ready;
    let mut bird_y = screen_height() * 0.45;
    let mut vy = 0.0f32;
    let mut pipes: Vec<Pipe> = Vec::new();
    let mut spawn_timer = 0.0f32;
    let mut score = 0i32;
    let mut best = 0i32;
    let mut seed = 0x1234_5678_9ABC_DEF0u64;
    let mut scroll = 0.0f32;

    loop {
        let dt = get_frame_time().min(1.0 / 30.0);
        let w = screen_width();
        let h = screen_height();
        let ground_y = h - GROUND_H;

        let tap = is_mouse_button_pressed(MouseButton::Left)
            || is_key_pressed(KeyCode::Space)
            || touches()
                .iter()
                .any(|t| t.phase == macroquad::input::TouchPhase::Started);

        match state {
            State::Ready => {
                bird_y = h * 0.45 + (get_time() as f32 * 3.0).sin() * 10.0;
                if tap {
                    state = State::Playing;
                    vy = JUMP_V;
                }
            }
            State::Playing => {
                if tap {
                    vy = JUMP_V;
                }
                vy = (vy + GRAVITY * dt).min(MAX_FALL);
                bird_y += vy * dt;

                spawn_timer += dt;
                if spawn_timer >= PIPE_INTERVAL {
                    spawn_timer = 0.0;
                    let margin = 60.0;
                    let span = (ground_y - GAP_H - margin * 2.0).max(1.0);
                    let top = margin + rng(&mut seed) * span;
                    pipes.push(Pipe { x: w + 20.0, top, scored: false });
                }

                for p in pipes.iter_mut() {
                    p.x -= PIPE_SPEED * dt;
                    if !p.scored && p.x + PIPE_W < BIRD_X {
                        p.scored = true;
                        score += 1;
                        if score > best {
                            best = score;
                        }
                    }
                }
                pipes.retain(|p| p.x + PIPE_W > -20.0);

                let bx = BIRD_X - BIRD_R;
                let by = bird_y - BIRD_R;
                let bs = BIRD_R * 2.0;
                let mut dead = bird_y + BIRD_R >= ground_y;
                for p in &pipes {
                    if rect_hit(bx, by, bs, bs, p.x, 0.0, PIPE_W, p.top)
                        || rect_hit(
                            bx, by, bs, bs,
                            p.x, p.top + GAP_H, PIPE_W, ground_y - p.top - GAP_H,
                        )
                    {
                        dead = true;
                    }
                }
                if dead {
                    state = State::Over;
                }
            }
            State::Over => {
                if tap {
                    state = State::Playing;
                    bird_y = h * 0.45;
                    vy = JUMP_V;
                    pipes.clear();
                    spawn_timer = 0.0;
                    score = 0;
                }
            }
        }

        let speed = if state == State::Playing { PIPE_SPEED } else { 24.0 };
        scroll = (scroll + speed * dt) % 48.0;

        // background
        clear_background(SKYBLUE);
        draw_circle(w * 0.82, h * 0.18, 34.0, Color::new(1.0, 0.94, 0.62, 1.0));
        draw_circle(w * 0.20, h * 0.24, 28.0, Color::new(1.0, 1.0, 1.0, 0.85));
        draw_circle(w * 0.24, h * 0.23, 20.0, Color::new(1.0, 1.0, 1.0, 0.85));
        draw_circle(w * 0.57, h * 0.13, 22.0, Color::new(1.0, 1.0, 1.0, 0.8));
        draw_circle(w * 0.61, h * 0.12, 16.0, Color::new(1.0, 1.0, 1.0, 0.8));

        // pipes
        let pipe_col = Color::new(0.22, 0.66, 0.28, 1.0);
        let pipe_lip = Color::new(0.16, 0.52, 0.22, 1.0);
        for p in &pipes {
            draw_rectangle(p.x, 0.0, PIPE_W, p.top, pipe_col);
            draw_rectangle(p.x - 4.0, p.top - 22.0, PIPE_W + 8.0, 22.0, pipe_lip);
            let bottom_y = p.top + GAP_H;
            draw_rectangle(p.x, bottom_y, PIPE_W, ground_y - bottom_y, pipe_col);
            draw_rectangle(p.x - 4.0, bottom_y, PIPE_W + 8.0, 22.0, pipe_lip);
        }

        // ground
        draw_rectangle(0.0, ground_y, w, GROUND_H, Color::new(0.85, 0.76, 0.52, 1.0));
        draw_rectangle(0.0, ground_y, w, 8.0, Color::new(0.72, 0.62, 0.38, 1.0));
        let mut x = -scroll;
        while x < w {
            draw_rectangle(x, ground_y + 8.0, 24.0, 6.0, Color::new(0.78, 0.68, 0.44, 1.0));
            x += 48.0;
        }

        // bird
        let bird_col = GOLD;
        draw_circle(BIRD_X, bird_y, BIRD_R, bird_col);
        let wing_off = ((get_time() * 12.0) as f32).sin() * 4.0;
        draw_rectangle(BIRD_X - 10.0, bird_y + 2.0 + wing_off, 14.0, 7.0, Color::new(0.95, 0.78, 0.20, 1.0));
        draw_circle(BIRD_X + 6.0, bird_y - 5.0, 4.2, WHITE);
        draw_circle(BIRD_X + 7.0, bird_y - 5.0, 2.0, BLACK);
        draw_rectangle(BIRD_X + BIRD_R - 2.0, bird_y - 1.0, 10.0, 5.0, Color::new(0.95, 0.42, 0.18, 1.0));

        // hud
        draw_text(&format!("{}", score), w / 2.0 - 10.0, 58.0, 42.0, WHITE);
        match state {
            State::Ready => {
                draw_centered("FLAPPY RUST", h * 0.30, 38.0, WHITE);
                draw_centered("点击屏幕 / 空格键 起飞", h * 0.30 + 44.0, 20.0, WHITE);
            }
            State::Over => {
                draw_rectangle(w / 2.0 - 150.0, h * 0.30, 300.0, 150.0, Color::new(0.0, 0.0, 0.0, 0.45));
                draw_centered("GAME OVER", h * 0.30 + 46.0, 34.0, WHITE);
                draw_centered(&format!("得分 {}   最佳 {}", score, best), h * 0.30 + 84.0, 20.0, WHITE);
                draw_centered("点击屏幕 再来一局", h * 0.30 + 122.0, 18.0, WHITE);
            }
            State::Playing => {}
        }

        next_frame().await;
    }
}
