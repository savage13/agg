//! Rendering Base

use crate::color::*;
use crate::DrawPixel;
use crate::Color;
use std::cmp::min;
use std::cmp::max;


/// Rendering Base
///
#[derive(Debug)]
pub struct RenderingBase<T> {
    /// Pixel Format
    pub pixf: T,
}

impl<T> RenderingBase<T> where T: DrawPixel {
    /// Create new Rendering Base from Pixel Format
    pub fn new(pixf: T) -> RenderingBase<T> {
        RenderingBase { pixf }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.pixf.as_bytes()
    }
    /// Set Image to a single color
    pub fn clear(&mut self, color: Rgba8) {
        self.pixf.fill(color);
    }
    /// Get Image size
    pub fn limits(&self) -> (i64,i64,i64,i64) {
        let w = self.pixf.width() as i64;
        let h = self.pixf.height() as i64;
        (0, w-1, 0, h-1)
    }
    /// Blend a color along y-row from x1 to x2
    pub fn blend_hline<C: Color>(&mut self, x1: i64, y: i64, x2: i64, c: C, cover: u64) {
        let (xmin,xmax,ymin,ymax) = self.limits();
        let (x1,x2) = if x2 > x1 { (x1,x2) } else { (x2,x1) };
        if y > ymax || y < ymin || x1 > xmax || x2 < xmin {
            return;
        }
        let x1 = max(x1, xmin);
        let x2 = min(x2, xmax);
        self.pixf.blend_hline(x1, y, x2 - x1 + 1, c, cover);
    }
    /// Blend a color from (x,y) with variable covers
    pub fn blend_solid_hspan<C: Color>(&mut self, x: i64, y: i64, len: i64, c: C, covers: &[u64]) {
        //eprintln!("blend_color_hspan, cover_full {:?}", covers);
        //eprintln!("RENBASE BLEND_SOLID_HSPAN x,y {:4} {:4} len {:4}", x, y, len );
        //eprintln!("DRAW: blend_solid_hspan() x,y {} {} len {} RENBASE", x, y, len );
        let (xmin,xmax,ymin,ymax) = self.limits();
        if y > ymax || y < ymin {
            return;
        }
        let (mut x, mut len, mut off) = (x,len, 0);
        if x < xmin {
            len -= xmin - x;
            if len <= 0 {
                return;
            }
            off = off + xmin - x; // Woah!!!!
            x = xmin;
        }
        if x + len > xmax {
            //eprintln!("X+LEN > XMAX");
            len = xmax - x + 1;
            if len <= 0 {
                return;
            }
        }
        let covers_win = &covers[off as usize .. (off+len) as usize];
        //eprintln!("RENBASE BLEND_SOLID_HSPAN x,y {:4} {:4} OFF {:4} LEN {:4} {:4} ", x, y, off, len, covers.len());
        assert!(len as usize <= covers[off as usize ..].len());
        self.pixf.blend_solid_hspan(x, y, len, c, covers_win);
        //eprintln!("RENBASE BLEND SOLID HSPAN DONE");
    }
    /// Blend a color from (x,y) with variable covers
    pub fn blend_solid_vspan<C: Color>(&mut self, x: i64, y: i64, len: i64, c: C, covers: &[u64]) {
        //eprintln!("RENBASE BLEND_SOLID_VSPAN x,y {:4} {:4} len {:4}", x, y, len );
        //eprintln!("DRAW: blend_solid_vspan() x,y {} {} len {} RENBASE", x, y, len );
        let (xmin,xmax,ymin,ymax) = self.limits();
        if x > xmax || x < xmin {
            return;
        }
        let (mut y, mut len, mut off) = (y,len, 0);
        if y < ymin {
            len -= ymin - y;
            if len <= 0 {
                return;
            }
            off = off + ymin - y; // Woah!!!!
            y = ymin;
        }
        if y + len > ymax {
            //eprintln!("X+LEN > XMAX");
            len = ymax - y + 1;
            if len <= 0 {
                return;
            }
        }
        let covers_win = &covers[off as usize .. (off+len) as usize];
        //eprintln!("RENBASE BLEND_SOLID_VSPAN x,y {:4} {:4} OFF {:4} LEN {:4} {:4}", x, y, off, len, covers.len());
        assert!(len as usize <= covers[off as usize ..].len());
        self.pixf.blend_solid_vspan(x, y, len, c, covers_win);
        //eprintln!("RENBASE BLEND SOLID HSPAN DONE");
    }

    pub fn blend_color_vspan<C: Color>(&mut self, x: i64, y: i64, len: i64, colors: &[C], covers: &[u64], cover: u64) {
        let (xmin,xmax,ymin,ymax) = self.limits();
        if x > xmax || x < xmin {
            return;
        }
        let (mut y, mut len, mut off) = (y,len, 0);
        if y < ymin {
            len -= ymin - y;
            if len <= 0 {
                return;
            }
            off = off + ymin - y; // Woah!!!!
            y = ymin;
        }
        if y + len > ymax {
            //eprintln!("X+LEN > XMAX");
            len = ymax - y + 1;
            if len <= 0 {
                return;
            }
        }
        let covers_win = if covers.is_empty() {
            &[]
        } else {
            &covers[off as usize .. (off+len) as usize]
        };
        let colors_win = &colors[off as usize .. (off+len) as usize];
        self.pixf.blend_color_vspan(x, y, len, colors_win, covers_win, cover);
    }
    pub fn blend_color_hspan<C: Color>(&mut self, x: i64, y: i64, len: i64, colors: &[C], covers: &[u64], cover: u64) {
        let (xmin,xmax,ymin,ymax) = self.limits();
        if y > ymax || y < ymin {
            return;
        }
        let (mut x, mut len, mut off) = (x,len, 0);
        if x < xmin {
            len -= xmin - x;
            if len <= 0 {
                return;
            }
            off = off + xmin - x; // Woah!!!!
            x = xmin;
        }
        if x + len > xmax {
            //eprintln!("X+LEN > XMAX");
            len = xmax - x + 1;
            if len <= 0 {
                return;
            }
        }
        let covers_win = if covers.is_empty() {
            &[]
        } else {
            &covers[off as usize .. (off+len) as usize]
        };
        let colors_win = &colors[off as usize .. (off+len) as usize];
        self.pixf.blend_color_hspan(x, y, len, colors_win, covers_win, cover);
    }
}

