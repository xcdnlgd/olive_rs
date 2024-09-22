use olive_rs::renderer::{u31, Renderer};

const WIDTH: u31 = 800;
const HEIGHT: u31 = 600;
const BUFFER_LEN: usize = (WIDTH * HEIGHT) as usize;

const COLS: u31 = 8;
const ROWS: u31 = 6;

const CELL_WIDTH: u31 = WIDTH / COLS;
const CELL_HEIGHT: u31 = HEIGHT / ROWS;

const BACKGROUND_COLOR: u32 = 0xFF_202020;
const FOREGROUND_COLOR: u32 = 0xff_0000ff;
const RED: u32 = 0xff_0000ff;
const GREEN: u32 = 0xff_00ff00;
const BLUE: u32 = 0xff_ff0000;

fn checker_example() {
    let mut buffer = [0u32; BUFFER_LEN];
    let mut renderer = Renderer::new(&mut buffer, WIDTH, HEIGHT);
    let file = "checher.ppm";
    renderer.fill(BACKGROUND_COLOR);
    for y in 0..ROWS {
        for x in 0..COLS {
            let color = if (x + y) % 2 == 0 {
                FOREGROUND_COLOR
            } else {
                BACKGROUND_COLOR
            };
            renderer.fill_rect(
                (x * CELL_WIDTH) as i32,
                (y * CELL_HEIGHT) as i32,
                CELL_WIDTH as i32,
                CELL_HEIGHT as i32,
                color,
            );
        }
    }
    renderer
        .save_to_ppm_file(file)
        .unwrap_or_else(|e| panic!("cannot open {file}: {}", e));
}

fn rect_example() {
    let file = "rect.ppm";
    let mut buffer = [0u32; BUFFER_LEN];
    let mut renderer = Renderer::new(&mut buffer, WIDTH, HEIGHT);
    renderer.fill(BACKGROUND_COLOR);
    renderer.fill_rect(0, 0, WIDTH as i32 * 3 / 4, HEIGHT as i32 * 3 / 4, RED);
    renderer.fill_rect(WIDTH as i32 - 1, HEIGHT as i32 - 1, -(WIDTH as i32 * 3 / 4), -(HEIGHT as i32 * 3 / 4), GREEN);
    renderer
        .save_to_ppm_file(file)
        .unwrap_or_else(|e| panic!("cannot open {file}: {}", e));
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn circle_example() {
    let mut buffer = [0u32; BUFFER_LEN];
    let mut renderer = Renderer::new(&mut buffer, WIDTH, HEIGHT);
    let file = "circle.ppm";
    renderer.fill(BACKGROUND_COLOR);
    let r = CELL_HEIGHT.min(CELL_WIDTH) / 2;
    for y in 0..ROWS {
        let v = y as f32 / ROWS as f32;
        for x in 0..COLS {
            let u = x as f32 / COLS as f32;
            let t = (u + v) / 2f32;
            let r = r as f32;
            let r = lerp(r / 5f32, r, t);
            renderer.fill_circle(
                (x * CELL_WIDTH + CELL_WIDTH / 2) as i32,
                (y * CELL_HEIGHT + CELL_HEIGHT / 2) as i32,
                r as u31,
                FOREGROUND_COLOR,
            );
        }
    }
    renderer.save_to_ppm_file(file).unwrap();
}

fn line_example() {
    let mut buffer = [0u32; BUFFER_LEN];
    let mut renderer = Renderer::new(&mut buffer, WIDTH, HEIGHT);
    let file = "line.ppm";
    renderer.fill(BACKGROUND_COLOR);
    renderer.draw_line(0, 0, WIDTH as i32, HEIGHT as i32, FOREGROUND_COLOR);
    renderer.draw_line(WIDTH as i32, 0, 0, HEIGHT as i32, FOREGROUND_COLOR);
    renderer.draw_line(
        WIDTH as i32 / 2,
        0,
        WIDTH as i32 / 2,
        HEIGHT as i32,
        FOREGROUND_COLOR,
    );
    renderer.draw_line(
        0,
        HEIGHT as i32 / 2,
        WIDTH as i32,
        HEIGHT as i32 / 2,
        FOREGROUND_COLOR,
    );
    renderer.save_to_ppm_file(file).unwrap();
}

fn triangle_example() {
    let mut buffer = [0u32; BUFFER_LEN];
    let mut renderer = Renderer::new(&mut buffer, WIDTH, HEIGHT);
    let file = "triangle.ppm";
    renderer.fill(BACKGROUND_COLOR);
    renderer.fill_triangle(10, 10, 80, 10, 10, 80, FOREGROUND_COLOR);
    renderer.fill_triangle(80, 10, 10, 80, 80, 80, FOREGROUND_COLOR);
    renderer.fill_triangle(
        WIDTH as i32 / 2,
        HEIGHT as i32 / 8,
        WIDTH as i32 / 8,
        HEIGHT as i32 / 2,
        WIDTH as i32 * 7 / 8,
        HEIGHT as i32 * 7 / 8,
        RED,
    );
    renderer.fill_triangle(
        WIDTH as i32 / 8,
        HEIGHT as i32 * 7 / 8,
        WIDTH as i32 / 2,
        HEIGHT as i32 / 2,
        WIDTH as i32 * 7 / 8,
        HEIGHT as i32 * 7 / 8,
        GREEN,
    );
    renderer.fill_triangle(
        WIDTH as i32 / 2,
        HEIGHT as i32 / 8,
        WIDTH as i32 * 7 / 8,
        HEIGHT as i32 / 8,
        WIDTH as i32 / 2,
        HEIGHT as i32 / 2,
        BLUE,
    );
    renderer.save_to_ppm_file(file).unwrap();
}

fn main() {
    checker_example();
    circle_example();
    line_example();
    triangle_example();
    rect_example();
}
