use std::f32;
use std::f32::consts::PI;

use olive_rs::renderer::Renderer;

#[cfg(feature = "term")]
const SCALE_DOWN_FACTOR: u32 = 20;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const RED: u32 = 0xff0000ff;
const GREEN: u32 = 0xff00ff00;
const BLUE: u32 = 0xffff0000;
const WHITE: u32 = 0xffffffff;

struct Vector2 {
    x: f32,
    y: f32,
}
impl Vector2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}
impl Vector3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

fn project_3d_2d(v3: Vector3) -> Vector2 {
    Vector2::new(v3.x / v3.z, v3.y / v3.z)
}

fn project_2d_screen(v2: Vector2) -> Vector2 {
    Vector2::new(
        (v2.x + 1f32) / 2f32 * WIDTH as f32,
        (-v2.y + 1f32) / 2f32 * HEIGHT as f32,
    )
}

static mut GLOBAL_TIME: f32 = 0.0;

pub fn render(buffer: &mut [u32], dt: f32) {
    unsafe {
        GLOBAL_TIME += dt;

        let mut renderer = Renderer::new(buffer, WIDTH, HEIGHT);
        renderer.fill(0xff181818);

        let z = 1.5f32;

        {
            let p0 = project_2d_screen(project_3d_2d(Vector3::new(
                f32::cos(GLOBAL_TIME) * 0.5,
                -0.5,
                z + f32::sin(GLOBAL_TIME) * 0.5,
            )));
            let p1 = project_2d_screen(project_3d_2d(Vector3::new(
                f32::cos(GLOBAL_TIME + PI) * 0.5,
                -0.5,
                z + f32::sin(GLOBAL_TIME + PI) * 0.5,
            )));
            let p2 = project_2d_screen(project_3d_2d(Vector3::new(0.0, 0.5, z)));

            renderer.fill_triangle_mix(
                p0.x as i32,
                p0.y as i32,
                RED,
                p1.x as i32,
                p1.y as i32,
                GREEN,
                p2.x as i32,
                p2.y as i32,
                BLUE,
            );
        }

        {
            let p0 = project_2d_screen(project_3d_2d(Vector3::new(
                f32::cos(GLOBAL_TIME + PI / 2f32) * 0.5,
                -0.5,
                z + f32::sin(GLOBAL_TIME + PI / 2f32) * 0.5,
            )));
            let p1 = project_2d_screen(project_3d_2d(Vector3::new(
                f32::cos(GLOBAL_TIME + PI + PI / 2f32) * 0.5,
                -0.5,
                z + f32::sin(GLOBAL_TIME + PI + PI / 2f32) * 0.5,
            )));
            let p2 = project_2d_screen(project_3d_2d(Vector3::new(0.0, 0.5, z)));

            renderer.fill_triangle_mix(
                p0.x as i32,
                p0.y as i32,
                RED,
                p1.x as i32,
                p1.y as i32,
                GREEN,
                p2.x as i32,
                p2.y as i32,
                BLUE,
            );
        }
    }
}

pub fn init() {}

include!("../common/main.rs");
