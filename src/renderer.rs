use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[allow(non_camel_case_types)]
pub type u31 = u32;

pub struct Renderer<'a> {
    buffer: &'a mut [u32],
    pub width: u31,
    pub height: u31,
}
impl<'a> Renderer<'a> {
    pub fn new(buffer: &'a mut [u32], width: u31, height: u31) -> Self {
        assert_eq!((width * height) as usize, buffer.len());
        Self {
            buffer,
            width,
            height,
        }
    }
    pub fn draw_horizontal_line(&mut self, mut x0: i32, mut x1: i32, y: i32, color: u32) {
        if x1 < x0 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if x1 < 0 {
            return;
        }
        if x0 >= self.width as i32 {
            return;
        }
        x1 += 1;
        if 0 <= y && (y as u31) < self.height {
            let y = y as u31;
            let x0 = x0.max(0) as u31;
            let xn = (x1 as u31).min(self.width);
            let start_i = (y * self.width + x0) as usize;
            let end_i = (y * self.width + xn) as usize;
            self.buffer[start_i..end_i]
                .iter_mut()
                .for_each(|pixel| *pixel = color);
        }
    }
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let (x0, y0, x1, y1) = if let Some(Line2D { x0, y0, x1, y1 }) = (Line2D {
            x0: x0 as f32,
            y0: y0 as f32,
            x1: x1 as f32,
            y1: y1 as f32,
        })
        .box_clip(
            0f32,
            0f32,
            self.width as f32 - 0.1,
            self.height as f32 - 0.1,
        ) {
            (x0 as i32, y0 as i32, x1 as i32, y1 as i32)
        } else {
            return;
        };
        self.draw_pixel_unchecked(x1 as u31, y1 as u31, color);

        let mut ray = Ray::new(x0, y0, x1, y1);
        while !ray.reached {
            let (x, y) = ray.next_xy();
            self.draw_pixel_unchecked(x as u31, y as u31, color)
        }
    }
    pub fn fill(&mut self, color: u32) {
        self.buffer.iter_mut().for_each(|pixel| *pixel = color);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn fill_triangle(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        mut x1: i32,
        mut y1: i32,
        mut x2: i32,
        mut y2: i32,
        color: u32,
    ) {
        sort_by_y(&mut x0, &mut y0, &mut x1, &mut y1, &mut x2, &mut y2);
        self.draw_pixel(x2, y2, color);
        let mut ray0 = Ray::new(x0, y0, x1, y1);
        let mut ray1 = Ray::new(x1, y1, x2, y2);
        let mut ray2 = Ray::new(x0, y0, x2, y2);
        let mut row = y0;
        let (mut x0, mut y0) = ray0.next_xy();
        let (mut x1, mut y1) = ray1.next_xy();
        let (mut x2, mut y2) = ray2.next_xy();
        while row <= ray0.y1 {
            while y0 != row {
                (x0, y0) = ray0.next_xy();
            }
            while y2 != row {
                (x2, y2) = ray2.next_xy();
            }
            self.draw_horizontal_line(x0, x2, row, color);
            row += 1;
        }
        row -= 1;
        while row <= ray1.y1 {
            while y1 != row {
                (x1, y1) = ray1.next_xy();
            }
            while y2 != row {
                (x2, y2) = ray2.next_xy();
            }
            self.draw_horizontal_line(x1, x2, row, color);
            row += 1;
        }
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
    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: i32, h: i32, color: u32) {
        if w == 0 || h == 0 {
            return;
        }
        let x1 = if w > 0 { x0 + w - 1} else { x0 + w + 1 };
        let y1 = if h > 0 { y0 + h - 1} else { y0 + h + 1 };
        let mut x0 = x0.clamp(0, self.width as i32 - 1) as u31;
        let mut y0 = y0.clamp(0, self.height as i32 - 1) as u31;
        let mut x1 = x1.clamp(0, self.width as i32 - 1) as u31;
        let mut y1 = y1.clamp(0, self.height as i32 - 1) as u31;
        if x1 < x0 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if y1 < y0 {
            std::mem::swap(&mut y0, &mut y1);
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
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
}

#[derive(Clone)]
struct Line2D {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}
impl Line2D {
    fn box_clip(&self, x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Option<Self> {
        if x_max < x_min {
            return None;
        }
        if y_max < y_min {
            return None;
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

pub struct Ray {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub reached: bool,
    sx: i32,
    sy: i32,
    dx: i32,
    dy: i32,
    error: i32,
    x: i32,
    y: i32,
    next: fn(&mut Self) -> (i32, i32),
}
impl Ray {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        // bresenham
        let sx = if x1 < x0 { -1 } else { 1 };
        let sy = if y1 < y0 { -1 } else { 1 };

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();

        let x = x0;
        let y = y0;

        let error;
        let next: fn(&mut Self) -> (i32, i32);
        let reached;
        if dy < dx {
            reached = x == x1;
            error = -dx;
            next = Self::iter_x;
        } else {
            reached = y == y1;
            error = -dy;
            next = Self::iter_y;
        }

        Self {
            x0,
            y0,
            x1,
            y1,
            reached,
            sx,
            sy,
            dx,
            dy,
            error,
            x,
            y,
            next,
        }
    }
    pub fn next_xy(&mut self) -> (i32, i32) {
        (self.next)(self)
    }
    fn iter_x(&mut self) -> (i32, i32) {
        let result = (self.x, self.y);
        self.error += self.dy + self.dy;
        if self.error >= 0 {
            self.y += self.sy;
            self.error -= self.dx + self.dx;
        }
        self.x += self.sx;
        if self.x == self.x1 {
            self.reached = true;
        }
        result
    }
    fn iter_y(&mut self) -> (i32, i32) {
        let result = (self.x, self.y);
        self.error += self.dx + self.dx;
        if self.error >= 0 {
            self.x += self.sx;
            self.error -= self.dy + self.dy;
        }
        self.y += self.sy;
        if self.y == self.y1 {
            self.reached = true;
        }
        result
    }
}

fn sort_by_y(x0: &mut i32, y0: &mut i32, x1: &mut i32, y1: &mut i32, x2: &mut i32, y2: &mut i32) {
    if y0 > y1 {
        std::mem::swap(y0, y1);
        std::mem::swap(x0, x1);
    }
    if y1 > y2 {
        std::mem::swap(y1, y2);
        std::mem::swap(x1, x2);
    }
    if y0 > y1 {
        std::mem::swap(y0, y1);
        std::mem::swap(x0, x1);
    }
}
