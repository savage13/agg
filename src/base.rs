
use pixfmt::*;
use color::*;
use PixelData;

use std::cmp::min;
use std::cmp::max;

#[derive(Debug,Default)]
pub struct RenderingBase {
    pub pixf: PixfmtRgb24,
}


impl RenderingBase {
    pub fn with_rgb24(pixf: PixfmtRgb24) -> RenderingBase {
        RenderingBase { pixf }
    }
    pub fn clear(&mut self, color: Rgba8) {
        self.pixf.fill(color.into());
    }
    pub fn limits(&self) -> (i64,i64,i64,i64) {
        let w = self.pixf.rbuf.width as i64;
        let h = self.pixf.rbuf.height as i64;
        (0, w-1, 0, h-1)
    }
    pub fn blend_hline<C: Color>(&mut self, x1: i64, y: i64, x2: i64, c: &C, cover: u64) {
        let (xmin,xmax,ymin,ymax) = self.limits();
        let (x1,x2) = if x2 > x1 { (x1,x2) } else { (x2,x1) };
        if y > ymax || y < ymin || x1 > xmax || x2 < xmin {
            return;
        }
        let x1 = max(x1, xmin);
        let x2 = min(x2, xmax);
        self.pixf.blend_hline(x1, y, x2 - x1 + 1, c, cover);
    }
    pub fn blend_solid_hspan<C: Color>(&mut self, x: i64, y: i64, len: i64, c: &C, covers: &[u64]) {
        eprintln!("BLEND_SOLID_HSPAN x,y {} {} len {} RENBASE", x, y, len );
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
            eprintln!("X+LEN > XMAX");
            len = xmax - x + 1;
            if len <= 0 {
                return;
            }
        }
        eprintln!("RENBASE BLEND SOLID HSPAN x,y {} {} OFF {} LEN {} {}", x, y, off, len, covers.len() );
        //assert_eq!(len as usize, covers[off as usize ..].len());
        self.pixf.blend_solid_hspan(x, y, len, c, &covers[off as usize ..]);
        eprintln!("RENBASE BLEND SOLID HSPAN DONE");
    }
}

impl<'a> PixelData<'a> for RenderingBase {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.pixf.rbuf.data
    }
}
