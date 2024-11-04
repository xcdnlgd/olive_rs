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

#[cfg(feature = "term")]
const SCALE_DOWN_FACTOR: u32 = 20;

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

include!("../common/main.rs");
