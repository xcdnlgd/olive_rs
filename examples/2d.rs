use core::f32;

use olive_rs::renderer::Renderer;

static mut ANGLE: f32 = 0f32;
const ROTATION_SPEED: f32 = 0.5;
const BALL_R: u32 = 50;
static mut BALL_X: f32 = (WIDTH / 2) as f32;
static mut BULL_X_SPEED: f32 = 200f32;
static mut BALL_Y: f32 = (HEIGHT / 2) as f32;
static mut BULL_Y_SPEED: f32 = 200f32;

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
        renderer.begin_blending();
        renderer.fill_triangle(x0, y0, x1, y1, x2, y2, 0xaa0000ff);
        renderer.fill_circle_aa(BALL_X as i32, BALL_Y as i32, BALL_R, 0x6900ff00);
        renderer.end_blending();
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
    const SCALE_DOWN_FACTOR: u32 = 20;
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
        let r = color & 0x000000ff;
        let g = (color & 0x0000ff00) >> 8;
        let b = (color & 0x00ff0000) >> (8 * 2);
        let brightness = ((r + r + r + b + g + g + g + g) >> 3) as usize;
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
