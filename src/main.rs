use olive_rs::canvas::Canvas;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const COLS: usize = 8;
const ROWS: usize = 6;

const CELL_WIDTH: usize = WIDTH / COLS;
const CELL_HEIGHT: usize = HEIGHT / ROWS;

const BACKGROUND_COLOR: u32 = 0xFF_202020;
const FOREGROUND_COLOR: u32 = 0xff_0000ff;

fn checker_example() {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    let file = "checher.ppm";
    canvas.fill(BACKGROUND_COLOR);
    for y in 0..ROWS {
        for x in 0..COLS {
            let color = if (x + y) % 2 == 0 {
                FOREGROUND_COLOR
            } else {
                BACKGROUND_COLOR
            };
            canvas.fill_rect(
                (x * CELL_WIDTH) as i32,
                (y * CELL_HEIGHT) as i32,
                CELL_WIDTH,
                CELL_HEIGHT,
                color,
            );
        }
    }
    canvas
        .save_to_ppm_file(file)
        .unwrap_or_else(|e| panic!("cannot open {file}: {}", e));
}

fn lerp(a: f32, b: f32, t: f32) -> f32{
    a + (b - a) * t
}

fn circle_example() {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    let file = "circle.ppm";
    canvas.fill(BACKGROUND_COLOR);
    let r = CELL_HEIGHT.min(CELL_WIDTH) / 2;
    for y in 0..ROWS {
        let v = y as f32/ ROWS as f32;
        for x in 0..COLS {
            let u = x as f32/ COLS as f32;
            let t = (u + v) / 2f32;
            let r = r as f32;
            let r = lerp(r / 5f32, r, t);
            canvas.fill_circle(
                (x * CELL_WIDTH + CELL_WIDTH / 2) as i32,
                (y * CELL_HEIGHT + CELL_HEIGHT / 2) as i32,
                r as usize,
                FOREGROUND_COLOR,
            );
        }
    }
    canvas.save_to_ppm_file(file).unwrap();
}

fn main() {
    checker_example();
    circle_example();
}
