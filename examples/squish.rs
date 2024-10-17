use core::f32;

use image::{ImageBuffer, Rgba};
use olive_rs::renderer::Renderer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const IMAGE_DATA: &[u8] = include_bytes!("../assets/tsodinPog.png");
static mut IMAGE: Vec<u32> = Vec::new();
static mut IMAGE_WIDTH: u32 = 0;
static mut IMAGE_HEIGHT: u32 = 0;

const BACKGROUND_COLOR: u32 = 0xFF_202020;

static mut ANGLE: f32 = 0f32;

pub fn render(buffer: &mut [u32], dt: f32) {
    let d = unsafe {
        ANGLE += dt * 2f32 * f32::consts::PI * 3f32;
        f32::sin(ANGLE)
    };

    let mut renderer = Renderer::new(buffer, WIDTH, HEIGHT);
    let image = unsafe {
        let a: &mut [u32] = IMAGE.as_mut();
        Renderer::new(a, IMAGE_WIDTH, IMAGE_HEIGHT)
    };
    renderer.fill(BACKGROUND_COLOR);
    let width = (400 - (d * 80f32) as i32) as u32;
    let height = (400 + (d * 80f32) as i32) as u32;
    let x = WIDTH / 2 - width / 2;
    let y = HEIGHT - height;
    let mut sub_canvas = Renderer::sub_canvas(
        renderer.get_buffer_mut(),
        x,
        y,
        width,
        height,
        WIDTH,
        HEIGHT,
    );
    sub_canvas.copy(&image);
}

pub fn init() {
    let img = image::load_from_memory_with_format(IMAGE_DATA, image::ImageFormat::Png).unwrap();
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = img.to_rgba8();

    let width = img.width();
    let height = img.height();

    let rgba32: Vec<u32> = img
        .pixels()
        .map(|pixel| {
            let [r, g, b, a] = pixel.0;
            (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32)
        })
        .collect();
    unsafe {
        IMAGE = rgba32;
        IMAGE_WIDTH = width;
        IMAGE_HEIGHT = height;
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
    init();
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
    init();
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
