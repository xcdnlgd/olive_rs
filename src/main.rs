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
    let mut canvas = Canvas::new(800, 600);
    canvas.fill(0xFF_00_FF_00);
    canvas.save_to_ppm_file("a.ppm").unwrap();
}
