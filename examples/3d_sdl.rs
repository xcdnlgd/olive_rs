use core::f32;

use bytemuck::cast_slice;
use olive_rs::renderer::u31;
use olive_rs::renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{render::Canvas, video::Window, Sdl};

fn create_canvase(sdl: &Sdl) -> Canvas<Window> {
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    window.into_canvas().build().unwrap()
}

fn render(renderer: &Renderer, canvas: &mut Canvas<Window>) {
    use sdl2::pixels::PixelFormatEnum::ABGR8888;
    let buffer = renderer.get_buffer();
    let pitch = (WINDOW_WIDTH * 4) as usize;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(ABGR8888, WINDOW_WIDTH, WINDOW_HEIGHT)
        .unwrap();
    texture.update(None, cast_slice(buffer), pitch).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();
}

fn get_timer() -> impl FnMut() -> f32 {
    use std::thread;
    use std::time::{Duration, Instant};
    const TARGET_FPS: f32 = 60.0;
    const TARGET_FRAME_TIME: f32 = 1.0 / TARGET_FPS;
    const CAP_FPS: bool = false;
    let mut last_loop_start_timepoint = Instant::now();
    let mut calculation_start_timepoint = Instant::now();
    let mut accumulated_time = 0f32;
    let mut frame_count = 0;
    move || {
        let loop_start_timepoint = Instant::now();
        let dt = (loop_start_timepoint - last_loop_start_timepoint).as_secs_f32();
        last_loop_start_timepoint = loop_start_timepoint;
        if CAP_FPS {
            let calculation_time = (Instant::now() - calculation_start_timepoint).as_secs_f32();
            // println!("calculation_time: {}", calculation_time);
            if calculation_time < TARGET_FRAME_TIME {
                let sleep_time = TARGET_FRAME_TIME - calculation_time;
                // println!("sleep_time: {}", sleep_time);
                thread::sleep(Duration::from_micros((sleep_time * 1e6) as u64));
            }
            calculation_start_timepoint = Instant::now();
        }
        accumulated_time += dt;
        frame_count += 1;
        if accumulated_time > 1f32 {
            println!("FPS: {:.2}", frame_count as f32 / accumulated_time);
            accumulated_time = 0f32;
            frame_count = 0;
        }

        // println!("dt: {}", dt);
        // println!("FPS: {}", 1.0 / dt);
        // println!();
        dt
    }
}

const FACTOR: u32 = 512;
const WINDOW_WIDTH: u32 = FACTOR;
const WINDOW_HEIGHT: u32 = FACTOR;
const CIRCLE_RADIUS: f32 = 0.005;
const GRID_COUNT: u31 = 10;
const GRID_PADDING: f32 = 0.08;
const GRID_SIZE: f32 = (GRID_COUNT - 1) as f32 * GRID_PADDING;
const CIRCLE_COLOR: u32 = 0xff0000ff;
const Z_START: f32 = 0.5;

pub fn main() {
    sdl2::hint::set("SDL_VIDEODRIVER", "wayland,x11");
    let sdl_context = sdl2::init().unwrap();
    let mut canvas = create_canvase(&sdl_context);

    let mut buffer = [0u32; WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize];
    let mut renderer = Renderer::new(&mut buffer, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut timer = get_timer();

    let mut angle = 0f32;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        // handle time, fps cap
        let dt = timer();

        // handle event
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // logic update
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
                        ((x + 1f32) / 2f32 * WINDOW_WIDTH as f32) as i32,
                        ((y + 1f32) / 2f32 * WINDOW_HEIGHT as f32) as i32,
                        (CIRCLE_RADIUS / z * FACTOR as f32) as u31,
                        color,
                    )
                }
            }
        }
        // renderer.fill_circle_aa(WINDOW_WIDTH as i32 / 2, WINDOW_HEIGHT as i32 / 2, 100, 0xff0000ff);

        // show on screen
        render(&renderer, &mut canvas);
    }
}
