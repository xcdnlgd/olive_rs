use core::f32;
use olive_rs::renderer::Renderer;

#[cfg(feature = "term")]
const SCALE_DOWN_FACTOR: u32 = 15;

const FACTOR: u32 = 600;
const WIDTH: u32 = FACTOR;
const HEIGHT: u32 = FACTOR;
const CIRCLE_RADIUS: f32 = 0.005;
const GRID_COUNT: u32 = 10;
const GRID_PADDING: f32 = 0.08;
const GRID_SIZE: f32 = (GRID_COUNT - 1) as f32 * GRID_PADDING;
const Z_START: f32 = 0.5;

#[allow(non_upper_case_globals)]
static mut angle: f32 = 0f32;

pub fn render(buffer: &mut [u32], dt: f32) {
    let mut renderer = Renderer::new(buffer, WIDTH, HEIGHT);

    // logic update
    unsafe {
        angle += 0.25 * dt * f32::consts::PI;

        // The rest of the game loop goes here...
        renderer.fill(0xff202020);
        for iy in 0..GRID_COUNT {
            let y = -GRID_SIZE / 2f32 + (iy as f32 * GRID_PADDING);
            for ix in 0..GRID_COUNT {
                let x = -GRID_SIZE / 2f32 + (ix as f32 * GRID_PADDING);
                // for cz in 0..GRID_COUNT {
                for iz in 0..GRID_COUNT {
                    let z = Z_START + (iz as f32 * GRID_PADDING);

                    let cx = 0f32;
                    let cz = Z_START + GRID_SIZE / 2f32;

                    let dx = x - cx;
                    let dz = z - cz;

                    let mag = (dx * dx + dz * dz).sqrt();
                    let dir = f32::atan2(dz, dx) + angle;

                    let x = dir.cos() * mag + cx;
                    let z = dir.sin() * mag + cz;

                    let x = x / z;
                    let y = y / z;

                    let r = ix * 255 / GRID_COUNT;
                    let g = iy * 255 / GRID_COUNT;
                    let b = iz * 255 / GRID_COUNT;

                    let color: u32 = 0xFF000000 | (r) | (g << (8)) | (b << (2 * 8));

                    renderer.fill_circle(
                        ((x + 1f32) / 2f32 * WIDTH as f32) as i32,
                        ((y + 1f32) / 2f32 * HEIGHT as f32) as i32,
                        (CIRCLE_RADIUS / z * FACTOR as f32) as u32,
                        color,
                    )
                }
            }
        }
    }
}

pub fn init() {}

include!("../common/main.rs");
