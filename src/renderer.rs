#![allow(clippy::too_many_arguments)]

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use phf::phf_map;

const AA_RES: i32 = 2;
const AA_PADDING: f32 = 1f32 / (AA_RES + 1) as f32;

pub struct Renderer<'b> {
    buffer: &'b mut [u32],
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    draw_horizontal_line_unchecked_fn: fn(&mut Self, x0: u32, x1: u32, y: u32, color: u32),
    draw_pixel_unchecked_fn: fn(&mut Self, x: u32, y: u32, color: u32),
    copy_fn: fn(this: &mut Self, source: &Self),
    aa_color_fn: fn(count: u8, color: u32) -> u32,
}
impl<'b> Renderer<'b> {
    pub fn new(buffer: &'b mut [u32], width: u32, height: u32) -> Self {
        assert_eq!((width * height) as usize, buffer.len());
        const BLENDING_ENABLED: bool = false;
        Self {
            buffer,
            width,
            height,
            stride: width,
            draw_horizontal_line_unchecked_fn: Self::m_draw_horizontal_line_unchecked::<
                BLENDING_ENABLED,
            >,
            draw_pixel_unchecked_fn: Self::m_draw_pixel_unchecked::<BLENDING_ENABLED>,
            copy_fn: Self::m_copy::<BLENDING_ENABLED>,
            aa_color_fn: aa_color::<BLENDING_ENABLED>,
        }
    }
    // TODO: when the borrower checker get smarter, receive &'b mut self
    // now use let sub = Renderer::sub_canvas(renderer.get_buffer_mut(), 15, 15, 50, 50, BUFFER_WIDTH, BUFFER_HEIGHT);
    pub fn sub_canvas(
        buffer: &'b mut [u32],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        buffer_width: u32,
        buffer_height: u32,
    ) -> Self {
        assert!(x < buffer_width);
        assert!(x + width <= buffer_width);
        assert!(y < buffer_height);
        assert!(y + height <= buffer_height);
        let stride = buffer_width;
        let start_index = (y * stride + x) as usize;
        const BLENDING_ENABLED: bool = false;
        Self {
            buffer: &mut buffer[start_index..],
            width,
            height,
            stride,
            draw_horizontal_line_unchecked_fn: Self::m_draw_horizontal_line_unchecked::<
                BLENDING_ENABLED,
            >,
            draw_pixel_unchecked_fn: Self::m_draw_pixel_unchecked::<BLENDING_ENABLED>,
            copy_fn: Self::m_copy::<BLENDING_ENABLED>,
            aa_color_fn: aa_color::<BLENDING_ENABLED>,
        }
    }
    pub fn get_buffer(&self) -> &[u32] {
        self.buffer
    }
    pub fn get_buffer_mut(&mut self) -> &mut [u32] {
        self.buffer
    }
    pub fn get_row_unchecked(&self, y: u32) -> &[u32] {
        let start = y * self.stride;
        let end = start + self.width;
        let start = start as usize;
        let end = end as usize;
        &self.buffer[start..end]
    }
    pub fn get_row_mut_unchecked(&mut self, y: u32) -> &mut [u32] {
        let start = y * self.stride;
        let end = start + self.width;
        let start = start as usize;
        let end = end as usize;
        &mut self.buffer[start..end]
    }
    pub fn begin_blending(&mut self) {
        self.draw_horizontal_line_unchecked_fn = Self::m_draw_horizontal_line_unchecked::<true>;
        self.draw_pixel_unchecked_fn = Self::m_draw_pixel_unchecked::<true>;
        self.copy_fn = Self::m_copy::<true>;
        self.aa_color_fn = aa_color::<true>;
    }
    pub fn end_blending(&mut self) {
        self.draw_horizontal_line_unchecked_fn = Self::m_draw_horizontal_line_unchecked::<false>;
        self.draw_pixel_unchecked_fn = Self::m_draw_pixel_unchecked::<false>;
        self.copy_fn = Self::m_copy::<false>;
        self.aa_color_fn = aa_color::<false>;
    }
    #[inline]
    pub fn copy(&mut self, source: &Self) {
        (self.copy_fn)(self, source);
    }
    fn m_copy<const BLENDING_ENABLED: bool>(&mut self, source: &Self) {
        let w = self.width as usize;
        let h = self.height as usize;
        let sw = source.width as usize;
        let sh = source.height as usize;
        for y in 0..h {
            let sy = y * sh / h;
            let start = sy * sw;
            self.get_row_mut_unchecked(y as u32)
                .iter_mut()
                .enumerate()
                .for_each(|(x, pixel)| {
                    let sx = x * sw / w;
                    if BLENDING_ENABLED {
                        blend_color(pixel, source.buffer[start + sx]);
                    } else {
                        *pixel = source.buffer[start + sx];
                    }
                });
        }
    }
    fn draw_horizontal_line(&mut self, mut x0: i32, mut x1: i32, y: i32, color: u32) {
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
        if 0 <= y && (y as u32) < self.height {
            let y = y as u32;
            let x0 = x0.max(0) as u32;
            let xn = (x1 as u32).min(self.width);
            self.draw_horizontal_line_unchecked(x0, xn, y, color);
        }
    }
    #[inline]
    pub fn draw_horizontal_line_unchecked(&mut self, x0: u32, x1: u32, y: u32, color: u32) {
        (self.draw_horizontal_line_unchecked_fn)(self, x0, x1, y, color);
    }
    fn m_draw_horizontal_line_unchecked<const BLENDING_ENABLED: bool>(
        &mut self,
        x0: u32,
        xn: u32,
        y: u32,
        color: u32,
    ) {
        let start_i = (y * self.stride + x0) as usize;
        let end_i = (y * self.stride + xn) as usize;
        self.buffer[start_i..=end_i].iter_mut().for_each(|pixel| {
            if BLENDING_ENABLED {
                blend_color(pixel, color);
            } else {
                *pixel = color;
            }
        });
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
        self.draw_pixel_unchecked(x1 as u32, y1 as u32, color);

        let mut ray = Ray::new(x0, y0, x1, y1);
        while !ray.reached {
            let (x, y) = ray.next_xy();
            self.draw_pixel_unchecked(x as u32, y as u32, color)
        }
    }
    pub fn fill(&mut self, color: u32) {
        self.fill_rect(0, 0, self.width as i32, self.height as i32, color);
    }
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
        let (mut cx0, mut cy0) = ray0.next_xy();
        let (mut cx1, mut cy1) = ray1.next_xy();
        let (mut cx2, mut cy2) = ray2.next_xy();
        if y1 != y0 {
            while row <= ray0.y1 {
                while cy0 != row {
                    (cx0, cy0) = ray0.next_xy();
                }
                while cy2 != row {
                    (cx2, cy2) = ray2.next_xy();
                }
                self.draw_horizontal_line(cx0, cx2, row, color);
                row += 1;
            }
        }
        if y1 != y2 {
            while row <= ray1.y1 {
                while cy1 != row {
                    (cx1, cy1) = ray1.next_xy();
                }
                while cy2 != row {
                    (cx2, cy2) = ray2.next_xy();
                }
                self.draw_horizontal_line(cx1, cx2, row, color);
                row += 1;
            }
        }
    }
    pub fn fill_triangle_aa(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: u32,
    ) {
        let ((x_min, y_min), (x_max, y_max)) = triangle_bunding_box(x0, y0, x1, y1, x2, y2);
        if let Some(((x_min, y_min), (x_max, y_max))) = normalize_rect(
            x_min,
            y_min,
            x_max - x_min + 1,
            y_max - y_min + 1,
            self.width,
            self.height,
        ) {
            let x0 = x0 as f32 + 0.5;
            let y0 = y0 as f32 + 0.5;
            let x1 = x1 as f32 + 0.5;
            let y1 = y1 as f32 + 0.5;
            let x2 = x2 as f32 + 0.5;
            let y2 = y2 as f32 + 0.5;
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    self.draw_pixel_unchecked_aa(x, y, color, |x, y| {
                        xy_in_triangle(x, y, x0, y0, x1, y1, x2, y2)
                    });
                }
            }
        }
    }
    pub fn fill_triangle_mix(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        mut c0: u32,
        mut x1: i32,
        mut y1: i32,
        mut c1: u32,
        mut x2: i32,
        mut y2: i32,
        mut c2: u32,
    ) {
        sort_by_y3(
            &mut x0, &mut y0, &mut c0, &mut x1, &mut y1, &mut c1, &mut x2, &mut y2, &mut c2,
        );
        self.draw_pixel(x2, y2, c2);
        let mut ray0 = Ray::new(x0, y0, x1, y1);
        let mut ray1 = Ray::new(x1, y1, x2, y2);
        let mut ray2 = Ray::new(x0, y0, x2, y2);
        let mut row = y0;
        let (mut cx0, mut cy0) = ray0.next_xy();
        let (mut cx1, mut cy1) = ray1.next_xy();
        let (mut cx2, mut cy2) = ray2.next_xy();
        if y1 != y0 {
            let x0 = x0 as f32;
            let y0 = y0 as f32;
            let x1 = x1 as f32;
            let y1 = y1 as f32;
            let x2 = x2 as f32;
            let y2 = y2 as f32;
            while row <= ray0.y1 {
                while cy0 != row {
                    (cx0, cy0) = ray0.next_xy();
                }
                while cy2 != row {
                    (cx2, cy2) = ray2.next_xy();
                }
                // self.draw_horizontal_line(cx0, cx2, row, color);
                if cx2 < cx0 {
                    std::mem::swap(&mut cx0, &mut cx2);
                }
                for x in cx0..cx2 {
                    let (u, v, w) = barycentric(x as f32, row as f32, x0, y0, x1, y1, x2, y2);
                    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                        let color = mix_color3(c0, c1, c2, u, v, w);
                        self.draw_pixel(x, row, color);
                    }
                }
                row += 1;
            }
        }
        if y1 != y2 {
            let x0 = x0 as f32;
            let y0 = y0 as f32;
            let x1 = x1 as f32;
            let y1 = y1 as f32;
            let x2 = x2 as f32;
            let y2 = y2 as f32;
            while row <= ray1.y1 {
                while cy1 != row {
                    (cx1, cy1) = ray1.next_xy();
                }
                while cy2 != row {
                    (cx2, cy2) = ray2.next_xy();
                }
                // self.draw_horizontal_line(cx1, cx2, row, color);
                if cx2 < cx1 {
                    std::mem::swap(&mut cx1, &mut cx2);
                }
                for x in cx1..=cx2 {
                    let (u, v, w) = barycentric(x as f32, row as f32, x0, y0, x1, y1, x2, y2);
                    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                        let color = mix_color3(c0, c1, c2, u, v, w);
                        self.draw_pixel(x, row, color);
                    }
                }
                row += 1;
            }
        }
    }
    pub fn fill_triangle_mix_aa(
        &mut self,
        x0: i32,
        y0: i32,
        c0: u32,
        x1: i32,
        y1: i32,
        c1: u32,
        x2: i32,
        y2: i32,
        c2: u32,
    ) {
        let ((x_min, y_min), (x_max, y_max)) = triangle_bunding_box(x0, y0, x1, y1, x2, y2);
        if let Some(((x_min, y_min), (x_max, y_max))) = normalize_rect(
            x_min,
            y_min,
            x_max - x_min + 1,
            y_max - y_min + 1,
            self.width,
            self.height,
        ) {
            let x0 = x0 as f32 + 0.5;
            let y0 = y0 as f32 + 0.5;
            let x1 = x1 as f32 + 0.5;
            let y1 = y1 as f32 + 0.5;
            let x2 = x2 as f32 + 0.5;
            let y2 = y2 as f32 + 0.5;
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    self.draw_pixel_unchecked_mix_aa(x, y, |x, y| {
                        let (u, v, w) = barycentric(x, y, x0, y0, x1, y1, x2, y2);
                        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                            let color = mix_color3(c0, c1, c2, u, v, w);
                            Some(color)
                        } else {
                            None
                        }
                    });
                }
            }
        }
    }
    pub fn fill_circle(&mut self, center_x: i32, center_y: i32, r: u32, color: u32) {
        if r == 0 {
            return;
        }
        if r == 1 {
            self.draw_pixel(center_x, center_y, color);
            return;
        }
        let r = r as i32;

        if center_x + r < 0
            || center_y + r < 0
            || center_x - r >= self.width as i32
            || center_y - r >= self.height as i32
        {
            return;
        }

        // bresenham
        // Taylor Expansion to get rid of sqrt
        // and other approximation
        let mut x0 = 0;
        let mut y0 = r;
        let mut last_y = y0;
        let mut d = 3 - 2 * r;

        // horizontal_center_line draw once
        self.draw_horizontal_line(center_x - y0, center_x + y0, center_y, color);
        if d < 0 {
            d += 4 * x0 + 6;
        } else {
            d += 4 * (x0 - y0) + 10;
            y0 -= 1;
        }
        x0 += 1;

        // only formulate the arc above the line y=x in the first quadrant.
        // iterate x
        while y0 >= x0 {
            // avoid draw multiple times
            if y0 != last_y {
                let last_x = x0 - 1;
                self.draw_horizontal_line(
                    center_x - last_x,
                    center_x + last_x,
                    center_y + last_y,
                    color,
                );
                self.draw_horizontal_line(
                    center_x - last_x,
                    center_x + last_x,
                    center_y - last_y,
                    color,
                );
                last_y = y0;
            }
            self.draw_horizontal_line(center_x - y0, center_x + y0, center_y + x0, color);
            self.draw_horizontal_line(center_x - y0, center_x + y0, center_y - x0, color);
            if d < 0 {
                d += 4 * x0 + 6;
            } else {
                d += 4 * (x0 - y0) + 10;
                y0 -= 1;
            }
            x0 += 1;
        }
        // because we are always drawing last_y in the loop, so we miss one y
        let last_x = x0 - 1;
        if last_x == last_y {
            // the missing y have been drawn by last x
            return;
        }
        self.draw_horizontal_line(
            center_x - last_x,
            center_x + last_x,
            center_y + last_y,
            color,
        );
        self.draw_horizontal_line(
            center_x - last_x,
            center_x + last_x,
            center_y - last_y,
            color,
        );
    }
    pub fn fill_circle_aa(&mut self, center_x: i32, center_y: i32, r: u32, color: u32) {
        let r = r as i32;
        if let Some(((x0, y0), (x1, y1))) = normalize_rect(
            center_x - r,
            center_y - r,
            2 * r + 1,
            2 * r + 1,
            self.width,
            self.height,
        ) {
            let r = r as f32;
            let center_x = center_x as f32 + 0.5;
            let center_y = center_y as f32 + 0.5;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    self.draw_pixel_unchecked_aa(x, y, color, |x, y| {
                        let dx = x - center_x;
                        let dy = y - center_y;
                        dx * dx + dy * dy <= r * r
                    });
                }
            }
        }
    }
    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: i32, h: i32, color: u32) {
        if let Some(((x0, y0), (x1, y1))) = normalize_rect(x0, y0, w, h, self.width, self.height) {
            (y0..=y1).for_each(|y| {
                self.draw_horizontal_line_unchecked(x0, x1, y, color);
            });
        }
    }
    pub fn fill_text(&mut self, text: &str, x0: i32, y0: i32, glyph_size: usize, color: u32) {
        let glyph_size = glyph_size as i32;
        for (i, ref ch) in text.chars().enumerate() {
            let x = x0 + (i * (DEFAULT_FONT_WIDTH + DEFAULT_FONT_SPACING)) as i32 * glyph_size;
            let glyph = &DEFAULT_FONT_GLYPHS[ch];
            for (dy, row) in glyph.iter().enumerate() {
                for (dx, &pixel) in row.iter().enumerate() {
                    if pixel == 1 {
                        self.fill_rect(
                            x + dx as i32 * glyph_size,
                            y0 + dy as i32 * glyph_size,
                            glyph_size,
                            glyph_size,
                            color,
                        )
                    }
                }
            }
        }
    }
    pub fn save_to_ppm_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        write!(file, "P6\n{} {} 255\n", self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.buffer[(y * self.width + x) as usize];
                let rgb: [u8; 3] = [
                    ((pixel) & 0xFF) as u8,
                    ((pixel >> 8) & 0xFF) as u8,
                    ((pixel >> (8 * 2)) & 0xFF) as u8,
                ];
                file.write_all(&rgb)?;
            }
        }
        Ok(())
    }
    #[inline]
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        if x < self.width && y < self.height {
            self.draw_pixel_unchecked(x, y, color);
        }
    }
    #[inline]
    fn draw_pixel_unchecked(&mut self, x: u32, y: u32, color: u32) {
        (self.draw_pixel_unchecked_fn)(self, x, y, color);
    }
    fn m_draw_pixel_unchecked<const BLENDING_ENABLED: bool>(&mut self, x: u32, y: u32, color: u32) {
        if BLENDING_ENABLED {
            blend_color(&mut self.buffer[(y * self.stride + x) as usize], color);
        } else {
            self.buffer[(y * self.stride + x) as usize] = color;
        }
    }
    #[inline]
    fn aa_color(&self, count: u8, color: u32) -> u32 {
        (self.aa_color_fn)(count, color)
    }
    fn draw_pixel_unchecked_aa(
        &mut self,
        x: u32,
        y: u32,
        color: u32,
        condition: impl Fn(f32, f32) -> bool,
    ) {
        let mut count_aa = 0;
        for sub_x in 1..=AA_RES {
            for sub_y in 1..=AA_RES {
                let x = x as f32 + sub_x as f32 * AA_PADDING;
                let y = y as f32 + sub_y as f32 * AA_PADDING;
                if condition(x, y) {
                    count_aa += 1;
                }
            }
        }
        if count_aa > 0 {
            let color = self.aa_color(count_aa, color);
            self.m_draw_pixel_unchecked::<true>(x, y, color);
        }
    }
    fn draw_pixel_unchecked_mix_aa(
        &mut self,
        x: u32,
        y: u32,
        color_fn: impl Fn(f32, f32) -> Option<u32>, // Option<color>
    ) {
        let mut count_aa = 0;
        let mut color = None;
        for sub_x in 1..=AA_RES {
            for sub_y in 1..=AA_RES {
                let x = x as f32 + sub_x as f32 * AA_PADDING;
                let y = y as f32 + sub_y as f32 * AA_PADDING;
                if let Some(sub_color) = color_fn(x, y) {
                    count_aa += 1;
                    if color.is_none() {
                        color = Some(sub_color);
                    }
                }
            }
        }
        if let Some(color) = color {
            let color = self.aa_color(count_aa, color);
            self.m_draw_pixel_unchecked::<true>(x, y, color);
        }
    }
}

const DEFAULT_FONT_WIDTH: usize = 5;
const DEFAULT_FONT_HEIGHT: usize = 8;
const DEFAULT_FONT_SPACING: usize = 1;

type Glyph = [[u8; DEFAULT_FONT_WIDTH]; DEFAULT_FONT_HEIGHT];

#[rustfmt::skip]
static DEFAULT_FONT_GLYPHS: phf::Map<char, Glyph> = phf_map! {
    '0' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,1,1],
        [1,0,1,0,1],
        [1,1,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    '1' => [
        [0,0,1,0,0],
        [0,1,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    '2' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [0,0,0,0,1],
        [0,0,1,1,0],
        [0,1,0,0,0],
        [1,0,0,0,1],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    '3' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [0,0,0,0,1],
        [0,0,1,1,0],
        [0,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    '4' => [
        [0,0,0,1,0],
        [0,0,1,1,0],
        [0,1,0,1,0],
        [1,0,0,1,0],
        [1,1,1,1,1],
        [0,0,0,1,0],
        [0,0,0,1,0],
        [0,0,0,0,0],
    ],
    '5' => [
        [1,1,1,1,1],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,1,1,1,0],
        [0,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    '6' => [
        [0,0,1,1,0],
        [0,1,0,0,0],
        [1,0,0,0,0],
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    '7' => [
        [1,1,1,1,1],
        [1,0,0,0,1],
        [0,0,0,0,1],
        [0,0,0,1,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    '8' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    '9' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,1],
        [0,0,0,1,0],
        [0,1,1,0,0],
        [0,0,0,0,0],
    ],
    'a' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,1,0],
        [0,0,0,0,1],
        [0,1,1,1,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,0],
    ],
    'b' => [
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,1,1,0],
        [1,1,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [0,0,0,0,0],
    ],
    'c' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,0],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'd' => [
        [0,0,0,0,1],
        [0,0,0,0,1],
        [0,1,1,0,1],
        [1,0,0,1,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,0],
    ],
    'e' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,1,1,1,1],
        [1,0,0,0,0],
        [0,1,1,1,1],
        [0,0,0,0,0],
    ],
    'f' => [
        [0,0,1,1,1],
        [0,1,0,0,0],
        [1,1,1,1,1],
        [0,1,0,0,0],
        [0,1,0,0,0],
        [0,1,0,0,0],
        [0,1,0,0,0],
        [0,0,0,0,0],
    ],
    'g' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,1,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,1],
        [1,1,1,1,0],
    ],
    'h' => [
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,1,1,0],
        [1,1,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'i' => [
        [0,0,1,0,0],
        [0,0,0,0,0],
        [1,1,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    'j' => [
        [0,0,0,0,1],
        [0,0,0,0,0],
        [0,0,1,1,1],
        [0,0,0,0,1],
        [0,0,0,0,1],
        [0,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
    ],
    'k' => [
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,1],
        [1,0,0,1,0],
        [1,1,1,0,0],
        [1,0,0,1,0],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'l' => [
        [1,1,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,1,1],
        [0,0,0,0,0],
    ],
    'm' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,1,0,1,0],
        [1,0,1,0,1],
        [1,0,1,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'n' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'o' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'p' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,1,1,0],
        [1,1,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
    ],
    'q' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,0,1],
        [1,0,0,1,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,1],
        [0,0,0,0,1],
    ],
    'r' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,1,1,0],
        [1,1,0,0,1],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [0,0,0,0,0],
    ],
    's' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,1,1,1,1],
        [1,0,0,0,0],
        [0,1,1,1,0],
        [0,0,0,0,1],
        [1,1,1,1,0],
        [0,0,0,0,0],
    ],
    't' => [
        [0,0,1,0,0],
        [0,0,1,0,0],
        [1,1,1,1,1],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,1,1],
        [0,0,0,0,0],
    ],
    'u' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,0],
    ],
    'v' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,0,1,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    'w' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,1,0,1],
        [1,0,1,0,1],
        [0,1,1,1,1],
        [0,0,0,0,0],
    ],
    'x' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,0,0,1],
        [0,1,0,1,0],
        [0,0,1,0,0],
        [0,1,0,1,0],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'y' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,1],
        [0,0,0,0,1],
        [1,1,1,1,0],
    ],
    'z' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [1,1,1,1,1],
        [0,0,0,1,0],
        [0,0,1,0,0],
        [0,1,0,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    'A' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'B' => [
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [0,0,0,0,0],
    ],
    'C' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'D' => [
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [0,0,0,0,0],
    ],
    'E' => [
        [1,1,1,1,1],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,1,1,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    'F' => [
        [1,1,1,1,1],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,1,1,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [0,0,0,0,0],
    ],
    'G' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,0],
        [1,0,0,1,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'H' => [
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'I' => [
        [1,1,1,1,1],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    'J' => [
        [0,0,0,0,1],
        [0,0,0,0,1],
        [0,0,0,0,1],
        [0,0,0,0,1],
        [0,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'K' => [
        [1,0,0,0,1],
        [1,0,0,1,0],
        [1,0,1,0,0],
        [1,1,0,0,0],
        [1,0,1,0,0],
        [1,0,0,1,0],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'L' => [
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    'M' => [
        [1,0,0,0,1],
        [1,1,0,1,1],
        [1,0,1,0,1],
        [1,0,1,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'N' => [
        [1,0,0,0,1],
        [1,1,0,0,1],
        [1,0,1,0,1],
        [1,0,0,1,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'O' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'P' => [
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [1,0,0,0,0],
        [0,0,0,0,0],
    ],
    'Q' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,1,0],
        [0,1,1,0,1],
        [0,0,0,0,0],
    ],
    'R' => [
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'S' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,0],
        [0,1,1,1,0],
        [0,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'T' => [
        [1,1,1,1,1],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    'U' => [
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,1,1,0],
        [0,0,0,0,0],
    ],
    'V' => [
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,0,1,0],
        [0,1,0,1,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    'W' => [
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,0,1,0,1],
        [1,0,1,0,1],
        [1,1,0,1,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'X' => [
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,0,1,0],
        [0,0,1,0,0],
        [0,1,0,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,0,0,0,0],
    ],
    'Y' => [
        [1,0,0,0,1],
        [1,0,0,0,1],
        [0,1,0,1,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    'Z' => [
        [1,1,1,1,1],
        [0,0,0,0,1],
        [0,0,0,1,0],
        [0,0,1,0,0],
        [0,1,0,0,0],
        [1,0,0,0,0],
        [1,1,1,1,1],
        [0,0,0,0,0],
    ],
    ' ' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ],
    '.' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    ',' => [
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,1,0,0,0],
    ],
    '\'' => [
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ],
    '\"' => [
        [0,1,0,1,0],
        [0,1,0,1,0],
        [0,1,0,1,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ],
    '!' => [
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
    '?' => [
        [0,1,1,1,0],
        [1,0,0,0,1],
        [0,0,0,0,1],
        [0,0,0,1,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
        [0,0,1,0,0],
        [0,0,0,0,0],
    ],
};

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

fn sort_by_y3(
    x0: &mut i32,
    y0: &mut i32,
    c0: &mut u32,
    x1: &mut i32,
    y1: &mut i32,
    c1: &mut u32,
    x2: &mut i32,
    y2: &mut i32,
    c2: &mut u32,
) {
    if y0 > y1 {
        std::mem::swap(y0, y1);
        std::mem::swap(x0, x1);
        std::mem::swap(c0, c1);
    }
    if y1 > y2 {
        std::mem::swap(y1, y2);
        std::mem::swap(x1, x2);
        std::mem::swap(c1, c2);
    }
    if y0 > y1 {
        std::mem::swap(y0, y1);
        std::mem::swap(x0, x1);
        std::mem::swap(c0, c1);
    }
}

fn blend_color(bottom_color: &mut u32, top_color: u32) {
    let mut bottom_color_components = [0u8; 4];
    for (i, color_component) in bottom_color_components.iter_mut().enumerate() {
        *color_component = (*bottom_color >> (8 * i) & 0xff) as u8;
    }
    let mut top_color_components = [0u8; 4];
    for (i, color_component) in top_color_components.iter_mut().enumerate() {
        *color_component = (top_color >> (8 * i) & 0xff) as u8;
    }
    let aa = top_color_components[3] as u16;
    bottom_color_components
        .iter_mut()
        .zip(top_color_components.iter())
        .take(3)
        .for_each(|(bottom_color_component, &color_component)| {
            *bottom_color_component = (((color_component as u16) * aa
                + (0xff - aa) * (*bottom_color_component as u16))
                / 0xff) as u8;
        });
    *bottom_color &= 0xff000000;
    for (i, bottom_color_component) in bottom_color_components.iter().enumerate().take(3) {
        *bottom_color |= (*bottom_color_component as u32) << (8 * i);
    }
}

#[inline]
fn red_part(color: u32) -> u32 {
    color & 0xff
}
#[inline]
fn green_part(color: u32) -> u32 {
    (color >> 8) & 0xff
}
#[inline]
fn blue_part(color: u32) -> u32 {
    (color >> (8 * 2)) & 0xff
}
#[inline]
fn alpha_part(color: u32) -> u32 {
    (color >> (8 * 3)) & 0xff
}
#[inline]
fn rgba2color(r: u32, g: u32, b: u32, a: u32) -> u32 {
    let mut color = 0u32;
    color |= r & 0x000000ff;
    color |= (g << 8) & 0x0000ff00;
    color |= (b << (8 * 2)) & 0x00ff0000;
    color |= (a << (8 * 3)) & 0xff000000;
    color
}

fn mix_color3(c0: u32, c1: u32, c2: u32, t0: f32, t1: f32, t2: f32) -> u32 {
    let r0 = red_part(c0) as f32;
    let g0 = green_part(c0) as f32;
    let b0 = blue_part(c0) as f32;
    let a0 = alpha_part(c0) as f32;

    let r1 = red_part(c1) as f32;
    let g1 = green_part(c1) as f32;
    let b1 = blue_part(c1) as f32;
    let a1 = alpha_part(c1) as f32;

    let r2 = red_part(c2) as f32;
    let g2 = green_part(c2) as f32;
    let b2 = blue_part(c2) as f32;
    let a2 = alpha_part(c2) as f32;

    let r = (r0 * t0 + r1 * t1 + r2 * t2) as u32;
    let g = (g0 * t0 + g1 * t1 + g2 * t2) as u32;
    let b = (b0 * t0 + b1 * t1 + b2 * t2) as u32;
    let a = (a0 * t0 + a1 * t1 + a2 * t2) as u32;

    rgba2color(r, g, b, a)
}

fn aa_color<const BLENDING_ENABLED: bool>(count: u8, color: u32) -> u32 {
    let old_alpha = if BLENDING_ENABLED {
        color >> (8 * 3) & 0xff
    } else {
        255
    } as f32;
    let t = count as f32 / (AA_RES * AA_RES) as f32;
    let alpha = (t * old_alpha) as u32;
    color & 0x00ffffff | (alpha << (8 * 3))
}

/// ```
/// if let Some((x0, y0), (x1, y1)) = normalize_rect(x, y, w, h, bound_width, bound_height) {
///     for y in y0..=y1 {
///         for x in x0..=x1 {
///             // do things on (x, y)
///         }
///     }
/// }
/// ```
fn normalize_rect(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bound_width: u32,
    bound_height: u32,
) -> Option<((u32, u32), (u32, u32))> {
    if w == 0 || h == 0 {
        return None;
    }
    let x1 = if w > 0 { x + w - 1 } else { x + w + 1 };
    let y1 = if h > 0 { y + h - 1 } else { y + h + 1 };
    let mut x0 = x.clamp(0, bound_width as i32 - 1);
    let mut y0 = y.clamp(0, bound_height as i32 - 1);
    let mut x1 = x1.clamp(0, bound_width as i32 - 1);
    let mut y1 = y1.clamp(0, bound_height as i32 - 1);
    if x1 < x0 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y1 < y0 {
        std::mem::swap(&mut y0, &mut y1);
    }
    Some(((x0 as u32, y0 as u32), (x1 as u32, y1 as u32)))
}

fn triangle_bunding_box(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) -> ((i32, i32), (i32, i32)) {
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }

    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
    }
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }

    ((x0, y0), (x2, y2))
}

#[inline]
fn xy_in_triangle(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
    let (u, v, w) = barycentric(x, y, x0, y0, x1, y1, x2, y2);
    u >= 0.0 && v >= 0.0 && w >= 0.0
}

// return (x, y, z)
#[inline]
fn vector3_a_cross_b(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> (f32, f32, f32) {
    let x = ay * bz - az * by;
    let y = az * bx - ax * bz;
    let z = ax * by - ay * bx;
    (x, y, z)
}

// return (u, v, w)
#[inline]
fn barycentric(
    x: f32,
    y: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> (f32, f32, f32) {
    // Barycentric coordinate system
    // https://github.com/ssloy/tinyrenderer/wiki/Lesson-2:-Triangle-rasterization-and-back-face-culling#:~:text=It%20means%20that%20we%20are%20looking%20for%20a%20vector%20(u%2Cv%2C1)%20that%20is%20orthogonal%20to%20(ABx%2CACx%2CPAx)%20and%20(ABy%2CACy%2CPAy)%20at%20the%20same%20time!
    let (x, y, z) = vector3_a_cross_b(x1 - x0, x2 - x0, x0 - x, y1 - y0, y2 - y0, y0 - y);
    let v = x / z;
    let w = y / z;
    let u = 1.0 - w - v;
    (u, v, w)
}
