//! Painter module

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    #[inline]
    pub fn from(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }
}

/// Painter interface, to be passed to client code so it can perform painting.
/// *This is not meant to be implemented by client code.*
pub trait Painter {
    fn get_screen_width(&self) -> i32;

    fn get_screen_height(&self) -> i32;

    /// Draw a single pixel.
    /// This is the only abstract method. The others are based on this one.
    fn draw_pixel(&mut self, x: i32, y: i32, color: RGB);

    fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: RGB) {
        if w > 0 && h > 0 {
            let x2 = x + w - 1;
            let y2 = y + h - 1;
            for xx in x..=x2 {
                self.draw_pixel(xx, y, color);
                self.draw_pixel(xx, y2, color);
            }
            for yy in (y + 1)..y2 {
                self.draw_pixel(x, yy, color);
                self.draw_pixel(x2, yy, color);
            }
        }
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: RGB) {
        if w > 0 && h > 0 {
            for yy in y..(y + h) {
                for xx in x..(x + w) {
                    self.draw_pixel(xx, yy, color);
                }
            }
        }
    }

    // (very basic, using floats, can be improved)
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: RGB) {
        if x1 == x2 {
            self.draw_vert_line(x1, y1, y2, color);
            return;
        }

        if y1 == y2 {
            self.draw_horiz_line(x1, x2, y1, color);
            return;
        }

        let dist_x = if x1 < x2 { x2 - x1 } else { x1 - x2 };
        let dist_y = if y1 < y2 { y2 - y1 } else { y1 - y2 };
        let dist_max = if dist_x > dist_y { dist_x } else { dist_y };
        let dx = ((x2 - x1) as f64) / (dist_max as f64);
        let dy = ((y2 - y1) as f64) / (dist_max as f64);
        let mut x = (x1 as f64) + 0.5;
        let mut y = (y1 as f64) + 0.5;
        self.draw_pixel(x1, y1, color);
        for _ in 0..dist_max {
            x += dx;
            y += dy;
            self.draw_pixel(x as i32, y as i32, color);
        }
    }

    fn draw_horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: RGB) {
        if x1 == x2 {
            self.draw_pixel(x1, y, color);
            return;
        }
        let (xmin, xmax) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in xmin..=xmax {
            self.draw_pixel(x, y, color);
        }
    }

    fn draw_vert_line(&mut self, x: i32, y1: i32, y2: i32, color: RGB) {
        if y1 == y2 {
            self.draw_pixel(x, y1, color);
            return;
        }
        let (ymin, ymax) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in ymin..=ymax {
            self.draw_pixel(x, y, color);
        }
    }

    fn draw_circle(&mut self, x: i32, y: i32, r: i32, color: RGB) {
        self.draw_ellipse(x, y, r, r, color);
    }

    fn fill_circle(&mut self, x: i32, y: i32, r: i32, color: RGB) {
        self.fill_ellipse(x, y, r, r, color);
    }

    // (very basic, using floats, can be improved)
    fn draw_ellipse(&mut self, x: i32, y: i32, rx: i32, ry: i32, color: RGB) {
        let r = if rx > ry { rx } else { ry };
        let mut r2 = r * r;
        let mut sub = 1;
        let imax = ((r as f64) / 1.4142135 + 0.5) as i32;

        for qx in 0..=imax {
            let qy = ((r2 as f64).sqrt() + 0.5) as i32;
            r2 -= sub;
            sub += 2;

            let px1 = if rx == r { qx } else { qx * rx / r };
            let px2 = if rx == r { qy } else { qy * rx / r };
            let py1 = if ry == r { qy } else { qy * ry / r };
            let py2 = if ry == r { qx } else { qx * ry / r };

            self.draw_pixel(x + px1, y + py1, color);
            self.draw_pixel(x + px1, y - py1, color);
            self.draw_pixel(x - px1, y + py1, color);
            self.draw_pixel(x - px1, y - py1, color);

            self.draw_pixel(x + px2, y + py2, color);
            self.draw_pixel(x + px2, y - py2, color);
            self.draw_pixel(x - px2, y + py2, color);
            self.draw_pixel(x - px2, y - py2, color);
        }
    }

    // (very basic, using floats, can be improved)
    fn fill_ellipse(&mut self, x: i32, y: i32, rx: i32, ry: i32, color: RGB) {
        let r = if rx > ry { rx } else { ry };
        let mut r2 = r * r;
        let mut sub = 1;
        let imax = ((r as f64) / 1.4142135 + 0.5) as i32;

        for qx in 0..=imax {
            let qy = ((r2 as f64).sqrt() + 0.5) as i32;
            r2 -= sub;
            sub += 2;

            let px1 = if rx == r { qx } else { qx * rx / r };
            let px2 = if rx == r { qy } else { qy * rx / r };
            let py1 = if ry == r { qy } else { qy * ry / r };
            let py2 = if ry == r { qx } else { qx * ry / r };

            self.draw_horiz_line(x - px1, x + px1, y + py1, color);
            self.draw_horiz_line(x - px1, x + px1, y - py1, color);
            self.draw_horiz_line(x - px2, x + px2, y + py2, color);
            self.draw_horiz_line(x - px2, x + px2, y - py2, color);
        }
    }
}
