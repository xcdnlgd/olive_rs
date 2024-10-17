use core::f32;
use olive_rs::renderer::Renderer;

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

#[cfg(not(feature = "wasm"))]
fn get_timer() -> impl FnMut() -> f32 {
    use std::thread;
    use std::time::{Duration, Instant};
    const TARGET_FPS: f32 = 60.0;
    const TARGET_FRAME_TIME: f32 = 1.0 / TARGET_FPS;
    #[cfg(feature = "sdl")]
    const CAP_FPS: bool = false;
    #[cfg(feature = "term")]
    const CAP_FPS: bool = true;
    let mut last_loop_start_timepoint = Instant::now();
    let mut calculation_start_timepoint = Instant::now();
    #[cfg(feature = "sdl")]
    let mut accumulated_time = 0f32;
    #[cfg(feature = "sdl")]
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

        #[cfg(feature = "sdl")]
        {
            accumulated_time += dt;
            frame_count += 1;
            if accumulated_time > 1f32 {
                println!("FPS: {:.2}", frame_count as f32 / accumulated_time);
                accumulated_time = 0f32;
                frame_count = 0;
            }
        }

        // println!("dt: {}", dt);
        // println!("FPS: {}", 1.0 / dt);
        // println!();
        dt
    }
}

#[cfg(feature = "sdl")]
fn main() {
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::{render::Canvas, video::Window, Sdl};
    fn create_canvase(sdl: &Sdl) -> Canvas<Window> {
        let video_subsystem = sdl.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", WIDTH, HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        window.into_canvas().build().unwrap()
    }
    fn show(buffer: &[u32], canvas: &mut Canvas<Window>) {
        use bytemuck::cast_slice;
        use sdl2::pixels::PixelFormatEnum::ABGR8888;
        let pitch = (WIDTH * 4) as usize;
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_target(ABGR8888, WIDTH, HEIGHT)
            .unwrap();
        texture.update(None, cast_slice(buffer), pitch).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
    sdl2::hint::set("SDL_VIDEODRIVER", "wayland,x11");
    let sdl_context = sdl2::init().unwrap();
    let mut canvas = create_canvase(&sdl_context);

    let mut buffer = [0u32; WIDTH as usize * HEIGHT as usize];
    let mut timer = get_timer();

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

        render(&mut buffer, dt);

        // show on screen
        show(&buffer, &mut canvas);
    }
}

#[cfg(feature = "term")]
fn main() {
    #[macro_export]
    macro_rules! static_assert {
        ($cond:expr $(,)?) => {
            const _: () = assert!($cond,);
        };
    }
    const SCALE_DOWN_FACTOR: u32 = 15;
    static_assert!(HEIGHT % SCALE_DOWN_FACTOR == 0);
    static_assert!(WIDTH % SCALE_DOWN_FACTOR == 0);
    const ROWS: u32 = HEIGHT / SCALE_DOWN_FACTOR;
    const COLS: u32 = WIDTH / SCALE_DOWN_FACTOR;
    fn show(buffer: &[u32]) {
        for r in 0..ROWS {
            let r = r * HEIGHT / ROWS;
            for c in 0..COLS {
                let c = c * WIDTH / COLS;
                print!("{}", color2char(buffer[(r * WIDTH + c) as usize]));
                print!("{}", color2char(buffer[(r * WIDTH + c) as usize]));
            }
            println!();
        }
        print!("\x1b[{ROWS}A");
        print!("\x1b[{COLS}D");
    }
    fn color2char(color: u32) -> char {
        #[allow(non_upper_case_globals)]
        const table: &str = " .:a@#";
        let r = (color & 0x000000ff) as f32;
        let g = ((color & 0x0000ff00) >> 8) as f32;
        let b = ((color & 0x00ff0000) >> (8 * 2)) as f32;
        let brightness = (0.2126 * r + 0.7152 * g + 0.0722 * b).round() as usize;
        let i = brightness * table.len() / 256;
        table.as_bytes()[i] as char
    }
    let mut buffer = [0u32; WIDTH as usize * HEIGHT as usize];
    let mut timer = get_timer();
    loop {
        // handle time, fps cap
        let dt = timer();

        render(&mut buffer, dt);

        // show on screen
        show(&buffer);
    }
}
