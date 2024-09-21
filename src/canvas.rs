use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[allow(non_camel_case_types)]
pub type u31 = u32;

#[derive(Clone)]
struct RectArea {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}
impl RectArea {
    fn box_clip(&self, limit: &Self) -> Option<Self> {
        let mut x_min = limit.x0;
        let mut x_max = limit.x1;
        if x_max < x_min {
            std::mem::swap(&mut x_min, &mut x_max);
        }
        let mut y_min = limit.y0;
        let mut y_max = limit.y1;
        if y_max < y_min {
            std::mem::swap(&mut y_min, &mut y_max);
        }

        // Cohen–Sutherland
        // 	        left	central right
        // top	    1001	1000    1010
        // central	0001	0000	0010
        // bottom	0101	0100	0110
        const INSIDE: u8 = 0b0000;
        const LEFT: u8 = 0b0001;
        const RIGHT: u8 = 0b0010;
        const BOTTOM: u8 = 0b0100;
        const TOP: u8 = 0b1000;

        let outcode = |x, y| {
            let mut code = INSIDE;
            if x < x_min {
                code |= LEFT;
            } else if x > x_max {
                code |= RIGHT;
            }
            if y < y_min {
                code |= BOTTOM;
            } else if y > y_max {
                code |= TOP;
            }
            code
        };

        let mut line = self.clone();

        let mut outcode_start = outcode(line.x0, line.y0);
        let mut outcode_end = outcode(line.x1, line.y1);
        loop {
            if (outcode_start | outcode_end) == 0 {
                // bitwise OR is 0: both points inside window
                return Some(line);
            } else if (outcode_start & outcode_end) != 0 {
                // bitwise AND is not 0: see the top comment, the line is fully outside the window
                return None;
            }

            // At least one endpoint is outside the clip rectangle; pick it.
            // outcode_center is 0b0000
            let outcode_out = u8::max(outcode_start, outcode_end);

            let (x_s, y_s) = (line.x0, line.y0);
            let (x_e, y_e) = (line.x1, line.y1);
            // Now find the intersection point;
            // use formulas:
            // No need to worry about divide-by-zero because, in each case, the
            // outcode bit being tested guarantees the denominator is non-zero
            let dx = x_e - x_s;
            let dy = y_e - y_s;
            let x;
            let y;
            if (outcode_out & TOP) != 0 {
                // point above the window
                x = x_s + (y_max - y_s) / dy * dx;
                y = y_max;
            } else if (outcode_out & BOTTOM) != 0 {
                // point below the window
                x = x_s + (y_min - y_s) / dy * dx;
                y = y_min;
            } else if (outcode_out & RIGHT) != 0 {
                // point is to the right of the window
                y = y_s + (x_max - x_s) / dx * dy;
                x = x_max;
            } else if (outcode_out & LEFT) != 0 {
                // point is to the left of the window
                y = y_s + (x_min - x_s) / dx * dy;
                x = x_min;
            } else {
                panic!("what!!!?");
            }

            // Now we move outside point to intersection point to clip
            // and get ready for next pass.
            if outcode_start == outcode_out {
                outcode_start = outcode(x, y);
                line.x0 = x;
                line.y0 = y;
            } else {
                outcode_end = outcode(x, y);
                line.x1 = x;
                line.y1 = y;
            }
        }
    }
}

pub struct Canvas {
    pub width: u31,
    pub height: u31,
    buffer: Vec<u32>,
}
impl Canvas {
    pub fn new(width: u31, height: u31) -> Self {
        Self {
            width,
            height,
            buffer: vec![0u32; (width * height) as usize],
        }
    }
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u31;
        let y = y as u31;
        if x < self.width && y < self.height {
            self.buffer[(y * self.width + x) as usize] = color;
        }
    }
    #[inline]
    fn draw_pixel_unchecked(&mut self, x: u31, y: u31, color: u32) {
        self.buffer[(y * self.width + x) as usize] = color;
    }
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let (x0, y0, x1, y1) = if let Some(RectArea { x0, y0, x1, y1 }) = (RectArea {
            x0: x0 as f32,
            y0: y0 as f32,
            x1: x1 as f32,
            y1: y1 as f32,
        })
        .box_clip(&RectArea {
            x0: 0f32,
            y0: 0f32,
            x1: self.width as f32 - 0.1,
            y1: self.height as f32 - 0.1,
        }) {
            (x0 as i32, y0 as i32, x1 as i32, y1 as i32)
        } else {
            return;
        };
        self.draw_pixel_unchecked(x1 as u31, y1 as u31, color);

        // bresenham
        let sx = if x1 < x0 { -1 } else { 1 };
        let sy = if y1 < y0 { -1 } else { 1 };

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();

        let mut x = x0;
        let mut y = y0;
        if dy < dx {
            let mut error = -dx;
            while x != x1 {
                self.draw_pixel_unchecked(x as u31, y as u31, color);
                error += dy + dy;
                if error >= 0 {
                    y += sy;
                    error -= dx + dx;
                }
                x += sx;
            }
        } else {
            let mut error = -dy;
            while y != y1 {
                self.draw_pixel_unchecked(x as u31, y as u31, color);
                error += dx + dx;
                if error >= 0 {
                    x += sx;
                    error -= dy + dy;
                }
                y += sy;
            }
        }
    }
    pub fn fill(&mut self, color: u32) {
        self.buffer.iter_mut().for_each(|pixel| *pixel = color);
    }
    pub fn fill_circle(&mut self, center_x: i32, center_y: i32, r: u31, color: u32) {
        let r = r as i32;

        if center_x + r < 0
            || center_y + r < 0
            || (center_x - r) as u31 > self.width
            || (center_y - r) as u31 > self.height
        {
            return;
        }

        // bresenham
        // Taylor Expansion to get rid of sqrt
        // and other approximation
        let mut x0 = 0;
        let mut y0 = r;
        let mut d = 3 - 2 * r;

        // only formulate the arc above the line y=x in the first quadrant.
        // iterate x
        while y0 >= x0 {
            for i in center_x - x0..=center_x + x0 {
                self.draw_pixel(i, center_y + y0, color);
            }
            for i in center_x - y0..=center_x + y0 {
                self.draw_pixel(i, center_y + x0, color);
            }
            for i in center_x - y0..=center_x + y0 {
                self.draw_pixel(i, center_y - x0, color);
            }
            for i in center_x - x0..=center_x + x0 {
                self.draw_pixel(i, center_y - y0, color);
            }
            if d < 0 {
                d += 4 * x0 + 6;
            } else {
                d += 4 * (x0 - y0) + 10;
                y0 -= 1;
            }
            x0 += 1;
        }
    }
    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: u31, h: u31, color: u32) {
        let xn = (x0 + w as i32).clamp(0, self.width as i32) as u31;
        let yn = (y0 + h as i32).clamp(0, self.height as i32) as u31;
        let x0 = x0.max(0) as u31;
        let y0 = y0.max(0) as u31;
        for y in y0..yn {
            for x in x0..xn {
                self.draw_pixel_unchecked(x as u31, y as u31, color);
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
