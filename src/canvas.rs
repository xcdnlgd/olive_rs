use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

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
    pub fn fill_circle(&mut self, center_x: i32, center_y: i32, r: usize, color: u32) {
        // FIXME: usize as i32 may overflow
        let r = r as i32;
        let x0 = (center_x - r).max(0);
        let y0 = (center_y - r).max(0);
        let xn = (center_x + r).min(self.width as i32);
        let yn = (center_y + r).min(self.width as i32);
        for y in y0..yn {
            let dy = center_y - y;
            for x in x0..xn {
                let dx = center_x - x;
                if dx * dx + dy * dy <= r * r {
                    self.buffer[(y * self.width as i32 + x) as usize] = color;
                }
            }
        }
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
