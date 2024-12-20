use core::f32;

use olive_rs::renderer::Renderer;

#[cfg(feature = "term")]
const SCALE_DOWN_FACTOR: u32 = 20;

static mut ANGLE: f32 = 0f32;
const ROTATION_SPEED: f32 = 0.5;
const BALL_R: u32 = 50;
static mut BALL_X: f32 = (WIDTH / 2) as f32;
static mut BULL_X_SPEED: f32 = 200f32;
static mut BALL_Y: f32 = (HEIGHT / 2) as f32;
static mut BULL_Y_SPEED: f32 = 200f32;

const WHITE: u32 = 0xff_ffffff;
const RED: u32 = 0xff_0000ff;
const GREEN: u32 = 0xff_00ff00;
const BLUE: u32 = 0xff_ff0000;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const BACKGROUND_COLOR: u32 = 0xFF_202020;

fn rotate_point(x: &mut i32, y: &mut i32, angle: f32) {
    let dx = *x - WIDTH as i32 / 2;
    let dy = *y - HEIGHT as i32 / 2;
    let mag = ((dx * dx + dy * dy) as f32).sqrt();
    let dir = f32::atan2(dy as f32, dx as f32) + angle;
    *x = (dir.cos() * mag) as i32 + WIDTH as i32 / 2;
    *y = (dir.sin() * mag) as i32 + HEIGHT as i32 / 2;
}

pub fn render(buffer: &mut [u32], dt: f32) {
    unsafe {
        ANGLE += 2f32 * f32::consts::PI * dt * ROTATION_SPEED;
        if BALL_X + BALL_R as f32 >= WIDTH as f32 || BALL_X - BALL_R as f32 <= 0f32 {
            BULL_X_SPEED *= -1f32;
            BALL_X += dt * BULL_X_SPEED;
        }
        BALL_X += dt * BULL_X_SPEED;
        if BALL_Y + BALL_R as f32 >= HEIGHT as f32 || BALL_Y - BALL_R as f32 <= 0f32 {
            BULL_Y_SPEED *= -1f32;
            BALL_Y += dt * BULL_Y_SPEED;
        }
        BALL_Y += dt * BULL_Y_SPEED;
        let mut renderer = Renderer::new(buffer, WIDTH, HEIGHT);
        renderer.fill(BACKGROUND_COLOR);
        let mut x0 = WIDTH as i32 / 2;
        let mut y0 = HEIGHT as i32 / 8;
        let mut x1 = WIDTH as i32 / 8;
        let mut y1 = HEIGHT as i32 / 2;
        let mut x2 = WIDTH as i32 * 7 / 8;
        let mut y2 = HEIGHT as i32 * 7 / 8;
        rotate_point(&mut x0, &mut y0, ANGLE);
        rotate_point(&mut x1, &mut y1, ANGLE);
        rotate_point(&mut x2, &mut y2, ANGLE);

        renderer.fill_triangle_mix(x0, y0, RED, x1, y1, GREEN, x2, y2, BLUE);
        renderer.fill_text("R", x0, y0, 4, WHITE);
        renderer.fill_text("G", x1, y1, 4, WHITE);
        renderer.fill_text("B", x2, y2, 4, WHITE);

        renderer.begin_blending();
        renderer.fill_circle_aa(BALL_X as i32, BALL_Y as i32, BALL_R, 0x6900ff00);
        renderer.end_blending();
    }
}

pub fn init() {}

include!("../common/main.rs");
