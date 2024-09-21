use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    buffer: Vec<u32>,
}
impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0u32; width * height],
        }
    }
    pub fn fill(&mut self, color: u32) {
        self.buffer.iter_mut().for_each(|pixel| *pixel = color);
    }
    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: usize, h: usize, color: u32) {
        let xn = if x0 < 0 {
            w.saturating_sub((-x0) as usize)
        } else {
            w.saturating_add(x0 as usize)
        }
        .min(self.width);
        let yn = if x0 < 0 {
            h.saturating_sub((-y0) as usize)
        } else {
            h.saturating_add(y0 as usize)
        }
        .min(self.height);
        let x0 = x0.max(0) as usize;
        let y0 = y0.max(0) as usize;
        for y in y0..yn {
            for x in x0..xn {
                self.buffer[y * self.width + x] = color;
            }
        }
    }
    pub fn save_to_ppm_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        write!(file, "P6\n{} {} 255\n", self.width, self.height)?;
        for &pixel in self.buffer.iter() {
            let rgb: [u8; 3] = [
                ((pixel) & 0xFF) as u8,
                ((pixel >> 8) & 0xFF) as u8,
                ((pixel >> (8 * 2)) & 0xFF) as u8,
            ];
            file.write_all(&rgb)?;
        }
        Ok(())
    }
}

fn main() {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    let file = "a.ppm";
    let w = 50 * 4;
    let h = 30 * 4;
    canvas.fill(0xFF_202020);
    canvas.fill_rect(
        (WIDTH / 2 - w / 2) as i32,
        (HEIGHT / 2 - h / 2) as i32,
        w,
        h,
        0xFF_0000FF,
    );
    canvas
        .save_to_ppm_file(file)
        .unwrap_or_else(|e| panic!("cannot open {file}: {}", e));
}
