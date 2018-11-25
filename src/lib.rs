
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::ops::Deref;
use std::collections::HashMap;

use std::cmp::min;
use std::cmp::max;

pub mod path_storage;
pub mod conv_stroke;
pub use path_storage::*;
pub use conv_stroke::*;

const POLY_SUBPIXEL_SHIFT : i64 = 8;
const POLY_SUBPIXEL_SCALE : i64 = 1<<POLY_SUBPIXEL_SHIFT;
const POLY_SUBPIXEL_MASK  : i64 = POLY_SUBPIXEL_SCALE - 1;



pub trait PixelData<'a> {
    fn pixeldata(&'a self) -> &'a [u8];
}

pub trait Color: std::fmt::Debug {
    fn red(&self) -> f64;
    fn green(&self) -> f64;
    fn blue(&self) -> f64;
    fn alpha(&self) -> f64;
    fn is_transparent(&self) -> bool { self.alpha() == 0.0 }
    fn is_opaque(&self) -> bool { self.alpha() >= 1.0 }
}

#[derive(Debug,Default,Copy,Clone)]
pub struct Rgb8([u8;3]);
impl Deref for Rgb8 {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

fn blend(fg: Rgb8, bg: Rgb8, alpha: f64) -> Rgb8 {
    let v : Vec<_> = fg.iter().zip(bg.iter())
        .map(|(&fg,&bg)| (f64::from(fg), f64::from(bg)) )
        .map(|(fg,bg)| alpha * fg + (1.0 - alpha) * bg)
        .map(|v| v as u8)
        .collect();
    Rgb8::new([v[0],v[1],v[2]])
}


impl Rgb8 {
    pub fn white() -> Self {
        Self::new([255,255,255])
    }
    pub fn black() -> Self {
        Self::new([0,0,0])
    }
    pub fn new(rgb: [u8; 3]) -> Self {
        Rgb8 ( rgb )
    }
    pub fn gray(g: u8) -> Self {
        Self::new([g,g,g])
    }
    pub fn from_wavelength_gamma(w: f64, gamma: f64) -> Self {
        let (r,g,b) =
            if w >= 380.0 && w <= 440.0 {
                (-1.0 * (w-440.0) / (440.0-380.0), 0.0, 1.0)
            } else if w >= 440.0 && w <= 490.0 {
                (0.0, (w-440.0)/(490.0-440.0), 1.0)
            } else if w >= 490.0 && w <= 510.0 {
                (0.0, 1.0, -1.0 * (w-510.0)/(510.0-490.0))
            } else if w >= 510.0 && w <= 580.0 {
                ((w-510.0)/(580.0-510.0), 1.0, 0.0)
            } else if w >= 580.0 && w <= 645.0 {
                (1.0, -1.0*(w-645.0)/(645.0-580.0), 0.0)
            } else if w >= 645.0 && w <= 780.0 {
                (1.0, 0.0, 0.0)
            } else {
                (0.,0.,0.)
            };
        let scale =
            if w > 700.0 {
                0.3 + 0.7 * (780.0-w)/(780.0-700.0)
            } else if w < 420.0 {
                0.3 + 0.7 * (w-380.0)/(420.0-380.0)
            } else {
                1.0
            };
        let r = (r * scale).powf(gamma) * 255.0;
        let g = (g * scale).powf(gamma) * 255.0;
        let b = (b * scale).powf(gamma) * 255.0;
        Self::new ( [r as u8, g as u8, b as u8] )
    }
}

fn color_u8_to_f64(x: u8) -> f64 {
    f64::from(x) / 255.0
}

impl Color for Rgb8 {
    fn   red(&self) -> f64 { color_u8_to_f64(self.0[0]) }
    fn green(&self) -> f64 { color_u8_to_f64(self.0[1]) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.0[2]) }
    fn alpha(&self) -> f64 { 1.0 }
}

#[derive(Debug,Copy,Clone)]
pub struct Gray8(u8);
impl Deref for Gray8 {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}
impl Gray8 {
    pub fn new(g: u8) -> Self {
        Gray8( g )
    }
}

#[derive(Debug,Default,Copy,Clone)]
pub struct Rgba8([u8;4]);

impl Deref for Rgba8 {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}
impl Rgba8 {
    pub fn white() -> Self {
        Self::new([255,255,255,255])
    }
    pub fn black() -> Self {
        Self::new([0,0,0,255])
    }
    pub fn new(rgba: [u8; 4]) -> Self {
        Rgba8 ( rgba )
    }
    pub fn from_wavelength_gamma(w: f64, gamma: f64) -> Self {
        let rgb = &*Rgb8::from_wavelength_gamma(w, gamma);
        Self::new([rgb[0],rgb[1],rgb[2],255])
    }
}

impl Color for Rgba8 {
    fn   red(&self) -> f64 { color_u8_to_f64(self.0[0]) }
    fn green(&self) -> f64 { color_u8_to_f64(self.0[1]) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.0[2]) }
    fn alpha(&self) -> f64 { color_u8_to_f64(self.0[3]) }
}

impl From<Rgba8> for Rgb8 {
    fn from(c: Rgba8) -> Rgb8 {
        let v = c.0;
        Rgb8::new( [v[0],v[1],v[2]] )
    }
}


#[derive(Debug,Default)]
pub struct RenderingBuffer {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
}

impl RenderingBuffer {
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        RenderingBuffer {
            width, height, bpp, data: vec![0u8; width * height * bpp]
        }
    }
    pub fn row_ptr(&mut self, i: usize) -> &mut [u8] {
        let row = i * self.width * self.bpp;
        &mut self.data[row .. ]
    }
    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|v| *v = 255);
    }
    pub fn fill(&mut self) {

    }
}

#[derive(Debug,Default)]
pub struct PixfmtRgb24 {
    pub rbuf: RenderingBuffer,
}

use std::ops::Index;
use std::ops::IndexMut;

impl Index<(usize,usize)> for RenderingBuffer {
    type Output = [u8];
    fn index(&self, index: (usize, usize)) -> &[u8] {
        let i = ((index.1 * self.width) + index.0) * self.bpp;
        &self.data[i..]
    }
}
impl IndexMut<(usize,usize)> for RenderingBuffer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut [u8] {
        let i = ((index.1 * self.width) + index.0) * self.bpp;
        &mut self.data[i..]
    }
}
fn fpart(x: f64) -> f64 {
    x - x.floor()
}
fn rfpart(x: f64) -> f64 {
    1.0 - fpart(x)
}
fn ipart(x: f64) -> f64 {
    x.floor()
}

fn endpoint(x: f64, y: f64, gradient: f64) -> (f64,f64,f64,usize,usize,f64,f64) {
    let xend = x.round();
    let yend = y + gradient * (xend - x);
    let xgap = rfpart(x + 0.5);
    let v1 = xgap * rfpart(yend);
    let v2 = xgap *  fpart(yend);

    (xend,yend,xgap,
     xend as usize,
     ipart(yend) as usize,
     v1, v2)
}

pub fn prelerp(a: f64, b: f64, t: f64)  {
    let (_a,_b,_t) = (a as f64, b as f64, t as f64);
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64{
    let mut v = (b-a) * t + a;
    eprintln!("BLEND PIX: {} {} {} => {}", a, b, t, v);
    if v < 0.0 {
        v = 0.0;
    }
    if v >= 1.0 {
        v = 1.0;
    }
    v
}


pub fn mult_cover(alpha: f64, cover: f64) -> f64 {
    alpha * cover
}

pub fn blend_pix<C1: Color, C2: Color>(p: &C1, c: &C2, cover: u64) -> Rgba8 {

    assert!(c.alpha() >= 0.0);
    assert!(c.alpha() <= 1.0);

    let alpha = mult_cover(c.alpha(), cover as f64 / 255.0);
    eprintln!("BLEND PIX: ALPHA COVER {} {} => {}", c.alpha(), cover as f64 / 255.0, alpha);
    eprintln!("BLEND PIX: {:?}", p);
    eprintln!("BLEND PIX: {:?}", c);
    assert!(alpha >= 0.0);
    assert!(alpha <= 1.0);
    let red   = lerp(p.red(),   c.red(),   alpha);
    let green = lerp(p.green(), c.green(), alpha);
    let blue  = lerp(p.blue(),  c.blue(),  alpha);
    let alpha = lerp(p.alpha(), c.alpha(), alpha);

    Rgba8([(red   * 255.0) as u8,
           (green * 255.0) as u8,
           (blue  * 255.0) as u8,
           (alpha * 255.0) as u8])
}

impl PixfmtRgb24 {
    pub fn clear(&mut self) {
        self.rbuf.clear();
    }
    pub fn fill(&mut self, c: Rgb8) {
        let w = self.rbuf.width;
        let h = self.rbuf.height;
        for i in 0 .. w {
            for j in 0 .. h {
                self.set((i,j), &c);
            }
        }
    }
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        Self { rbuf: RenderingBuffer::new(width, height, bpp) }
    }
    pub fn from(rbuf: RenderingBuffer) -> Self {
        Self { rbuf }
    }
    pub fn blend_hline<C: Color>(&mut self, x: i64, y: i64, len: i64, c: &C, cover: u64) {
        if c.is_transparent() {
            return;
        }
        let (x,y,len) = (x as usize, y as usize, len as usize);
        let cover_mask = 255;
        if c.is_opaque() && cover == cover_mask {
            for i in 0 .. len {
                self.set((x+i,y), c);
            }
        } else {
            for i in 0 .. len {
                let pix = self.get((x+i, y));
                let pix = blend_pix(&pix, c, cover);
                self.set((x+i,y), &pix);
            }
        }
    }
    pub fn blend_solid_hspan<C: Color>(&mut self, x: i64, y: i64, _len: i64, c: &C, covers: &[u64]) {
        if c.is_transparent() {
            return;
        }
        for (i, &cover) in covers.iter().enumerate() {
            self.blend_hline(x+i as i64,y,1,c,cover);
        }
    }
    pub fn copy_pixel(&mut self, x: usize, y: usize, c: Rgb8) {
        self.set((x,y), &c);
    }
    pub fn copy_hline(&mut self, x: usize, y: usize, n: usize, c: Rgb8) {
        for i in 0 .. n {
            self.set((x+i,y), &c);
        }
    }
    pub fn copy_vline(&mut self, x: usize, y: usize, n: usize, c: Rgb8) {
        for i in 0 .. n {
            self.set((x,y+i), &c);
        }
    }

    pub fn blend_color_hspan(&mut self, x: usize, y: usize, _n: usize,
                             c: &[Rgb8], _cover: usize) {
        for (i,ci) in c.iter().enumerate() {
            self.set((x+i,y), ci);
        }
    }

    pub fn set<C: Color>(&mut self, id: (usize, usize), c: &C) {
        self.rbuf[id][0] = (c.red()   * 255.0) as u8;
        self.rbuf[id][1] = (c.green() * 255.0) as u8;
        self.rbuf[id][2] = (c.blue()  * 255.0) as u8;
    }
    pub fn line_sp_aa(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        let steep = (x2-x1).abs() < (y2-y1).abs();
        let (x1,y1,x2,y2) = if steep   { (y1,x1,y2,x2) } else { (x1,y1,x2,y2) };
        let (x1,y1,x2,y2) = if x2 < x1 { (x2,y2,x1,y1) } else { (x1,y1,x2,y2) };
        let dx = x2-x1;
        let dy = y2-y1;
        let gradient = if dx.abs() <= 1e-6 { 1.0 } else { dy/dx };

        let white = Rgb8::white();
        // Handle First Endpoint
        let (_xend, yend, _xgap, xpx11, ypx11, v1, v2) = endpoint(x1,y1,gradient);
        let v1 = blend(c, white, v1);
        let v2 = blend(c, white, v2);
        if steep {
            self.set((ypx11,  xpx11), &v1);
            self.set((ypx11+1,xpx11), &v2);
        } else {
            self.set((xpx11,  ypx11),  &v1);
            self.set((xpx11,  ypx11+1),&v2);
        }
        let mut intery = yend + gradient;
        // Handle Second Endpoint

        let (_xend, _yend, _xgap, xpx12, ypx12, v1, v2) = endpoint(x2,y2,gradient);
        let v1 = blend(c, white, v1);
        let v2 = blend(c, white, v2);
        if steep {
            self.set((ypx12,  xpx12),   &v1);
            self.set((ypx12+1,xpx12),   &v2);
        } else {
            self.set((xpx12,  ypx12),   &v1);
            self.set((xpx12,  ypx12+1), &v2);
        }
        // In Between Points
        for xp in xpx11 + 1 .. xpx12 {
            let yp = ipart(intery) as usize;
            let (p0,p1) = if steep { ((yp,xp),(yp+1,xp)) } else { ((xp,yp),(xp,yp+1)) };

            let (v1,v2) = ( rfpart(intery), fpart(intery) );
            let v0 = blend(c, self.get(p0), v1);
            let v1 = blend(c, self.get(p1), v2);
            self.set(p0,&v0);
            self.set(p1,&v1);

            intery += gradient;
        }

    }
    pub fn get(&self, id: (usize, usize)) -> Rgb8 {
        let p = &self.rbuf[id];
        Rgb8::new( [p[0], p[1], p[2]] )
    }
    pub fn line_sp(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        println!("({}, {}) - ({}, {})", x1,y1,x2,y2);
        let x1 = (x1 * 256.0).round() as i64 / 256;
        let y1 = (y1 * 256.0).round() as i64 / 256;
        let x2 = (x2 * 256.0).round() as i64 / 256;
        let y2 = (y2 * 256.0).round() as i64 / 256;
        println!("   ({}, {}) - ({}, {})", x1,y1,x2,y2);

        let steep = (x2-x1).abs() < (y2-y1).abs();
        let (x1,y1,x2,y2) = if steep   { (y1,x1,y2,x2) } else { (x1,y1,x2,y2) };
        let (x1,y1,x2,y2) = if x2 < x1 { (x2,y2,x1,y1) } else { (x1,y1,x2,y2) };

        let count = (x2-x1).abs();
        let count = std::cmp::max(count, 1);
        let dy = y2-y1;

        let mut left = dy / count;
        let mut rem  = dy % count;
        let mut xmod = rem;
        let mut y = y1;
        //println!("   count, left, rem, dy: {} {} {} {}", count, left, rem, dy);
        if xmod <= 0 {
            xmod += count;
            rem  += count;
            left -= 1;
        }
        xmod -= count;

        for x in x1..x2 {
            if steep {
                self.set((y as usize, x as usize), &c);
            } else {
                self.set((x as usize, y as usize), &c);
            }
            xmod += rem;
            y += left;
            if xmod > 0 {
                xmod -= count;
                y += 1;
            }
        }
    }
    /// https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm#C.2B.2B
    
    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        let steep = (y2-y1).abs() > (x2-x1).abs();

        let (x1,y1,x2,y2) = if steep { (y1,x1,y2,x2) } else { (x1,y1,x2,y2) };
        let (x1,y1,x2,y2) = if x1>x2 { (x2,y2,x1,y1) } else { (x1,y1,x2,y2) };
        let dx = x2-x1;
        let dy = (y2-y1).abs();
        let mut error = dx / 2.0;

        let pos   = y1<y2;
        let mut y = y1.floor() as usize;
        let x1    = x1.floor() as usize;
        let x2    = x2.floor() as usize;
        for x in x1 .. x2 {
            if steep {
                self.set((y,x), &c);
            } else {
                self.set((x,y), &c);
            }
            error -= dy;
            if error <= 0.0 {
                y = if pos { y+1 } else { y-1 };
                error += dx;
            }
        }
    }
}

pub struct PixfmtGray8 {
    pub rbuf: RenderingBuffer
}

impl PixfmtGray8 {
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        Self{ rbuf: RenderingBuffer::new(width, height, bpp) }
    }
    pub fn copy_hline(&mut self, x: usize, y: usize, n: usize, c: Gray8) {
        for i in 0 .. n {
            self.rbuf[(x+i,y)][0] = *c;
        }
    }
}

pub struct AlphaMaskAdaptor {
    pub rgb: PixfmtRgb24,
    pub alpha: PixfmtGray8,
}

impl AlphaMaskAdaptor {
    pub fn new(rgb: PixfmtRgb24, alpha: PixfmtGray8) -> Self {
        Self { rgb, alpha }
    }
    /// From https://stackoverflow.com/a/746937 :
    /// out = alpha * new + (1 - alpha) * old
    ///   p[j]  = out
    ///   alpha = alpha
    ///   new   = c
    ///   old   = p[j]
    pub fn blend_color_hspan(&mut self, x: usize, y: usize, n: usize,
                             colors: &[Rgb8], _cover: usize) {
        //for i in 0 .. n {
        assert_eq!(n, colors.len());
        for (i, color) in colors.iter().enumerate() {
            let pix = &mut self.rgb.get((x+i,y));
            let alpha = u64::from(self.alpha.rbuf[(x+i,y)][0]);
            let pix = blend_pix(pix, color, alpha);
            self.rgb.set((x+i,y), &pix);
        }
    }
}
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
        eprintln!("RENBASE BLEND SOLID HSPAN x,y {} {} len {} {}", x, y, len, covers.len() );
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


#[derive(Debug, Default)]
pub struct RenderingScanlineBinSolid {
    pub base: RenderingBase,
    pub color: Rgba8,
}

#[derive(Debug, Default)]
pub struct RenderingScanlineAASolid {
    pub base: RenderingBase,
    pub color: Rgba8,
}

pub fn render_scanline_bin_solid<C: Color>(sl: &ScanlineU8,
                                           ren: &mut RenderingBase,
                                           color: &C) {
    let cover_full = 255;
    for span in &sl.spans {
        eprintln!("RENDER SCANLINE BIN SOLID: Span x,y,len {} {} {} {}",
                  span.x, sl.y, span.len, span.covers.len());
        ren.blend_hline(span.x, sl.y, span.x - 1 + span.len.abs(),
                        color, cover_full);
    }
}

pub fn render_scanline_aa_solid<C: Color>(sl: &ScanlineU8,
                                          ren: &mut RenderingBase,
                                          color: &C) {
    let y = sl.y;
    for span in & sl.spans {
        eprintln!("RENDER SCANLINE AA SOLID: Span x,y,len {} {} {} {}", span.x, y, span.len, span.covers.len());
        let x = span.x;
        if span.len > 0 {
            //eprintln!("RENDER SCANLINE AA SOLID: {} {}", span.len, span.covers.len());
            ren.blend_solid_hspan(x, y, span.len, color, &span.covers);
        } else {
            ren.blend_hline(x, y, x-span.len-1, color, span.covers[0]);
        }
    }
}

pub trait RenderingScanline {
    fn render(&mut self, sl: &ScanlineU8);
    fn prepare(&self) {
    }
}

impl RenderingScanline for RenderingScanlineAASolid {
    fn render(&mut self, sl: &ScanlineU8) {
        render_scanline_aa_solid(sl, &mut self.base, &self.color);
    }
}
impl RenderingScanline for RenderingScanlineBinSolid {
    fn render(&mut self, sl: &ScanlineU8) {
        render_scanline_bin_solid(sl, &mut self.base, &self.color);
    }
}
impl RenderingScanlineBinSolid {
    pub fn with_base(base: RenderingBase) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
    pub fn color(&mut self, color: Rgba8) {
        self.color = color;
    }
}
impl RenderingScanlineAASolid {
    pub fn with_base(base: RenderingBase) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
    pub fn color(&mut self, color: Rgba8) {
        self.color = color;
    }
}
impl<'a> PixelData<'a> for RenderingScanlineAASolid {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.base.pixf.rbuf.data
    }
}
impl<'a> PixelData<'a> for RenderingScanlineBinSolid {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.base.pixf.rbuf.data
    }
}

#[derive(Debug,Default)]
pub struct Rectangle {
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
}
impl Rectangle {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64) -> Self {
        let (x1, x2) = if x1 > x2 { (x2,x1) } else { (x1,x2) };
        let (y1, y2) = if y1 > x2 { (y2,y1) } else { (y1,y2) };
        Self { x1,y1,x2,y2 }
    }
    pub fn clip_flags(&self, x: i64, y: i64) -> u8 {
        clip_flags(x,y, self.x1, self.y1, self.x2, self.y2)
    }
}

/// See https://en.wikipedia.org/wiki/Liang-Barsky_algorithm
/// See https://en.wikipedia.org/wiki/Cyrus-Beck_algorithm

const INSIDE : u8 = 0b0000;
const LEFT   : u8 = 0b0001;
const RIGHT  : u8 = 0b0010;
const BOTTOM : u8 = 0b0100;
const TOP    : u8 = 0b1000;
pub fn clip_flags(x: i64, y: i64, x1: i64, y1: i64, x2: i64, y2: i64) -> u8 {
    let mut code = INSIDE;
    if x < x1 { code |= LEFT; }
    if x > x2 { code |= RIGHT; }
    if y < y1 { code |= BOTTOM; }
    if y > y2 { code |= TOP; }
    code
}

#[derive(Debug,Default)]
pub struct Clip {
    x1: i64,
    y1: i64,
    clip_box: Option<Rectangle>,
    clip_flag: u8,
}

pub fn mul_div(a: i64, b: i64, c: i64) -> i64 {
    let (a,b,c) = (a as f64, b as f64, c as f64);
    (a * b / c).round() as i64
}
impl Clip {
    pub fn new() -> Self {
        Self {x1: 0, y1: 0,
              clip_box: None,
              clip_flag: INSIDE }
    }
    pub fn line_clip_y(&self, ras: &mut RasterizerCell,
                       x1: i64, y1: i64,
                       x2: i64, y2: i64,
                       f1: u8, f2: u8) {
        let b = match self.clip_box {
            None => return,
            Some(ref b) => b,
        };
        let f1 = f1 & (TOP|BOTTOM);
        let f2 = f2 & (TOP|BOTTOM);
        // Fully Visible in y
        if f1 == INSIDE && f2 == INSIDE {
            eprintln!("ras.line_to_d({:.2} , {:.2});//1", x1>>8,y1>>8);
            eprintln!("ras.line_to_d({:.2} , {:.2});//2", x2>>8,y2>>8);
            ras.line(x1,y1,x2,y2);
        } else {
            // Both points above or below clip box
            if f1 == f2 {
                return;
            }
            let (mut tx1, mut ty1, mut tx2, mut ty2) = (x1,y1,x2,y2);
            if f1 == BOTTOM {
                tx1 = x1 + mul_div(b.y1-y1, x2-x1, y2-y1);
                ty1 = b.y1;
            }
            if f1 == TOP {
                tx1 = x1 + mul_div(b.y2-y1, x2-x1, y2-y1);
                ty1 = b.y2;
            }
            if f2 == BOTTOM {
                tx2 = x1 + mul_div(b.y1-y1, x2-x1, y2-y1);
                ty2 = b.y1;
            }
            if f2 == TOP {
                tx2 = x1 + mul_div(b.y2-y1, x2-x1, y2-y1);
                ty2 = b.y2;
            }
            eprintln!("ras.line_to_d({:.2} , {:.2}); //3", tx1>>8,ty1>>8);
            eprintln!("ras.line_to_d({:.2} , {:.2}); //4", tx2>>8,ty2>>8);
            ras.line(tx1,tx2,ty1,ty2);
        }
    }
    pub fn line_to(&mut self, ras: &mut RasterizerCell, x2: i64, y2: i64) {
        //eprintln!("ras.line_to_d({}, {}); // LINE TO: {} {}",
        //          x2 / POLY_SUBPIXEL_SCALE, y2 / POLY_SUBPIXEL_SCALE,
        //          x2, y2);
        if let Some(ref b) = self.clip_box {
            eprintln!("LINE CLIPPING ON");
            let f2 = b.clip_flags(x2,y2);
            // Both points above or below clip box
            let fy1 = (TOP | BOTTOM) & self.clip_flag;
            let fy2 = (TOP | BOTTOM) & f2;
            if fy1 != INSIDE && fy1 == fy2 {
                eprintln!("LINE OUTSIDE CLIP BOX {:?}", b);
                eprintln!("LINE xlim {} {} x1 {} x2 {} f {:04b}", b.x1,b.x2,self.x1,x2, self.clip_flag);
                eprintln!("LINE ylim {} {} y1 {} y2 {} f {:04b}", b.y1,b.y2,self.y1,y2,f2);
                self.x1 = x2;
                self.y1 = y2;
                self.clip_flag = f2;
                return;
            }
            let (x1,y1,f1) = (self.x1, self.y1, self.clip_flag);
            eprintln!("LINE CLIP: L {} R {} T {} B {} -- {} {}", f1 & LEFT, f1 & RIGHT, f1 & TOP, f1 & BOTTOM, x1>>8, y1>>8);
            eprintln!("LINE CLIP: L {} R {} T {} B {} -- {} {}", f2 & LEFT, f2 & RIGHT, f2 & TOP, f2 & BOTTOM, x2>>8, y2>>8);
            match (f1 & (LEFT|RIGHT), f2 & (LEFT|RIGHT)) {
                (INSIDE,INSIDE) => self.line_clip_y(ras, x1,y1,x2,y2,f1,f2),
                (INSIDE,RIGHT) => {
                    let y3 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x2, y3);
                    self.line_clip_y(ras, x1,   y1, b.x2, y3, f1, f3);
                    self.line_clip_y(ras, b.x2, y3, b.x2, y2, f3, f2);
                },
                (RIGHT,INSIDE) => {
                    let y3 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x2, y3);
                    self.line_clip_y(ras, b.x2, y1, b.x2, y3, f1, f3);
                    self.line_clip_y(ras, b.x2, y3,   x2, y2, f3, f2);
                },
                (INSIDE,LEFT) => {
                    let y3 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x1, y3);
                    self.line_clip_y(ras, x1,   y1, b.x1, y3, f1, f3);
                    self.line_clip_y(ras, b.x1, y3, b.x1, y2, f3, f2);
                },
                (RIGHT,LEFT) => {
                    let y3 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let y4 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x2, y3);
                    let f4 = b.clip_flags(b.x1, y4);
                    self.line_clip_y(ras, b.x2, y1, b.x2, y3, f1, f3);
                    self.line_clip_y(ras, b.x2, y3, b.x1, y4, f3, f4);
                    self.line_clip_y(ras, b.x1, y4, b.x1, y2, f4, f2);
                },
                (LEFT,INSIDE) => {
                    let y3 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x1, y3);
                    self.line_clip_y(ras, b.x1, y1, b.x1, y3, f1, f3);
                    self.line_clip_y(ras, b.x1, y3,   x2, y2, f3, f2);
                },
                (LEFT,RIGHT) => {
                    let y3 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let y4 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x1, y3);
                    let f4 = b.clip_flags(b.x2, y4);
                    self.line_clip_y(ras, b.x1, y1, b.x1, y3, f1, f3);
                    self.line_clip_y(ras, b.x1, y3, b.x2, y4, f3, f4);
                    self.line_clip_y(ras, b.x2, y4, b.x2, y2, f4, f2);
                },
                (LEFT,LEFT)   => self.line_clip_y(ras, b.x1,y1,b.x1,y2,f1,f2),
                (RIGHT,RIGHT) => self.line_clip_y(ras, b.x2,y1,b.x2,y2,f1,f2),

                (_,_) => unreachable!("f1,f2 {:?} {:?}", f1,f2),
            }
            self.clip_flag = f2;
        } else {
            ras.line(self.x1, self.y1, x2, y2);
        }
        self.x1 = x2;
        self.y1 = y2;
    }
    pub fn move_to(&mut self, x2: i64, y2: i64) {
        eprintln!("//ras.move_to_d({}, {}); // MOVE TO: {} {}", x2/POLY_SUBPIXEL_SCALE, y2/POLY_SUBPIXEL_SCALE, x2, y2);
        self.x1 = x2;
        self.y1 = y2;
        if let Some(ref b) = self.clip_box {
            self.clip_flag = clip_flags(x2,y2, b.x1,b.y1,b.x2,b.y2);
        }
    }
    pub fn clip_box(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) {
        self.clip_box = Some( Rectangle::new(x1, y1, x2, y2) );
    }
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum FillingRule {
    NonZero,
    EvenOdd,
}
impl Default for FillingRule {
    fn default() -> FillingRule {
        FillingRule::NonZero
    }
}
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum PathStatus {
    Initial,
    Closed,
    MoveTo,
    LineTo
}
impl Default for PathStatus {
    fn default() -> PathStatus {
        PathStatus::Initial
    }
}


#[derive(Debug,Copy,Clone,PartialEq, Default)]
pub struct Cell { // cell_aa
    x: i64,
    y: i64,
    cover: i64,
    area: i64,
}

impl Cell {
    pub fn new() -> Self {
        Cell { x: std::i64::MAX,
               y: std::i64::MAX,
               cover: 0,
               area: 0
        }
    }
    pub fn at(x: i64, y: i64) -> Self {
        let mut c = Cell::new();
        c.x = x;
        c.y = y;
        c
    }
    pub fn equal(&self, x: i64, y: i64) -> bool {
        self.x - x == 0 && self.y - y == 0
    }
    pub fn is_empty(&self) -> bool {
        self.cover == 0 && self.area == 0
    }
}

#[derive(Debug,Default)]
pub struct RasterizerCell {
    cells: Vec<Cell>,
    pub min_x: i64,
    pub max_x: i64,
    pub min_y: i64,
    pub max_y: i64,
    sorted_y: Vec<Vec<Cell>>,
}


impl RasterizerCell {
    pub fn new() -> Self {
        Self { cells: vec![],
               min_x: std::i64::MAX,
               min_y: std::i64::MAX,
               max_x: std::i64::MIN,
               max_y: std::i64::MIN,
               sorted_y: vec![],
        }
    }
    pub fn reset(&mut self) {
        self.max_x = std::i64::MIN;
        self.max_y = std::i64::MIN;
        self.min_x = std::i64::MAX;
        self.min_y = std::i64::MAX;
        self.sorted_y.clear(); // Not sure if this should be cleared
        self.cells.clear();    // Not sure if this should be cleared
    }

    pub fn total_cells(&self) -> usize {
        self.cells.len()
    }
    pub fn sort_cells(&mut self) {
        eprintln!("SORT_CELLS MAX_Y: {} N: {}", self.max_y, self.cells.len());
        if ! self.sorted_y.is_empty() || self.max_y < 0 {
            return;
        }
        // Distribute into 
        self.sorted_y = vec![vec![]; (self.max_y+1) as usize];
        for c in self.cells.iter() {
            let y = c.y as usize;
            eprintln!("SORT_CELLS SORTING {:?}", c);
            self.sorted_y[y].push(c.clone());
        }
        // Sort by the x value
        for i in 0 .. self.sorted_y.len() {
            eprintln!("SORT_CELLS: y: {} len: {}", i, self.sorted_y[i].len());
            self.sorted_y[i].sort_by(|a,b| (a.x).cmp(&b.x));
        }
    }
    pub fn scanline_num_cells(&self, y: i64) -> usize {
        self.sorted_y[y as usize].len()
    }
    pub fn scanline_cells(&self, y: i64) -> &[Cell] {
        & self.sorted_y[y as usize]
    }

    //pub fn add_curr_cell(&mut self, new_cell: Cell) {
    //    self.cells.push( new_cell );
    //}
    pub fn curr_cell_is_set(&self, x: i64, y: i64) -> bool {
        match self.cells.last() {
            None      => true,
            Some(cur) => {
                //eprintln!("SET_CURR_CELL: {} {} EQUAL: {} EMPTY: {}", x, y, cur.equal(x,y), !cur.is_empty());
                ! cur.equal(x,y) && ! cur.is_empty()
            }
        }
    }
    pub fn curr_cell_not_equal(&self, x: i64, y: i64) -> bool {
        match self.cells.last() {
            None      => true,
            Some(cur) => {
                //eprintln!("XXXX {:?}", cur);
                ! cur.equal(x,y)

            }
        }
    }

    pub fn pop_last_cell_if_empty(&mut self) {
        let n = self.cells.len();
        if n == 0 {
            return;
        }
        if self.cells[n-1].area == 0 && self.cells[n-1].cover == 0 {
            self.cells.pop();
        } else {
            self.show_last_cell();
        }
    }
    pub fn show_last_cell(&self) {
        if let Some(c) = self.cells.last() {
            eprintln!("ADD_CURR_CELL: {} {} area {} cover {} len {}", c.x,c.y,c.area,c.cover, self.cells.len());
        }
    }
    pub fn set_curr_cell(&mut self, x: i64, y: i64)  {
        //eprintln!("SET_CURR_CELL: {} {}", x,y);
        if self.curr_cell_not_equal(x, y) {
            //eprintln!("ADD_CURR_CELL: {} {} {} ", x,y, self.cells.len()+1);
            self.pop_last_cell_if_empty();
            self.cells.push( Cell::at(x,y) );
        }
    }

    pub fn render_hline(&mut self, ey: i64, x1: i64, y1: i64, x2: i64, y2: i64) {
        let ex1 = x1 >> POLY_SUBPIXEL_SHIFT;
        let ex2 = x2 >> POLY_SUBPIXEL_SHIFT;
        let fx1 = x1  & POLY_SUBPIXEL_MASK;
        let fx2 = x2  & POLY_SUBPIXEL_MASK;

        // Horizontal Line
        if y1 == y2 {
            self.set_curr_cell(ex2, ey);
            return;
        }

        // Single Cell
        if ex1 == ex2 {
            //eprintln!("RENDER_HLINE LEN: {}", self.cells.len());
            let m_curr_cell = self.cells.last_mut().unwrap();
            m_curr_cell.cover += y2-y1;
            m_curr_cell.area  += (fx1 + fx2) * (y2-y1);
            eprintln!("INCR0 cover {} area {} dcover {} darea {} x,y {} {}",
                      m_curr_cell.cover,
                      m_curr_cell.area,
                      y2-y1,
                      (fx1 + fx2) * (y2-y1), m_curr_cell.x, m_curr_cell.y);
            return;
        }
        //eprintln!("RENDER_HLINE ADJCENT CELLS SAME LINE {} {}", x1,x2);
        // Adjacent Cells on Same Line
        let (mut p, first, incr, dx) = if x2-x1 < 0 {
            (fx1 * (y2-y1), 0,-1, x1-x2)
        } else {
            ((POLY_SUBPIXEL_SCALE - fx1) * (y2-y1), POLY_SUBPIXEL_SCALE, 1, x2-x1)
        };
        let mut delta = p / dx;
        let mut xmod =  p % dx;

        if xmod < 0 {
            delta -= 1;
            xmod += dx;
        }
        {
            let m_curr_cell = self.cells.last_mut().unwrap();
            m_curr_cell.cover += delta;
            m_curr_cell.area  += (fx1 + first) * delta;
            eprintln!("INCR1 cover {} area {} dcover {} darea {} x,y {} {} ",
                      m_curr_cell.cover,
                      m_curr_cell.area,
                      delta,
                      (fx1 + first) * delta, m_curr_cell.x, m_curr_cell.y);

        }
        let mut ex1 = ex1 + incr;
        self.set_curr_cell(ex1, ey);
        let mut y1 = y1 + delta;

        if ex1 != ex2 {
            p = POLY_SUBPIXEL_SCALE * (y2 - y1 + delta);
            let mut lift = p / dx;
            let mut rem = p % dx;
            if rem < 0 {
                lift -= 1;
                rem += dx;
            }
            xmod -= dx;

            while ex1 != ex2 {
                delta = lift;
                xmod += rem;
                if xmod >= 0 {
                    xmod -= dx;
                    delta += 1;
                }
                {
                    let m_curr_cell = self.cells.last_mut().unwrap();
                    m_curr_cell.cover += delta;
                    m_curr_cell.area  += POLY_SUBPIXEL_SCALE * delta;
                    eprintln!("INCR2 cover {} area {} dcover {} darea {} x,y {} {}",
                              m_curr_cell.cover,
                              m_curr_cell.area,
                              delta,
                              POLY_SUBPIXEL_SCALE * delta, m_curr_cell.x, m_curr_cell.y);
                }
                y1 += delta;
                ex1 += incr;
                self.set_curr_cell(ex1, ey);
            }
        }
        delta = y2-y1;
        {
            let m_curr_cell = self.cells.last_mut().unwrap();
            m_curr_cell.cover += delta;
            m_curr_cell.area  += (fx2 + POLY_SUBPIXEL_SCALE - first) * delta;
            eprintln!("INCR3 cover {} area {} dcover {} darea {} x,y {} {}",
                      m_curr_cell.cover,
                      m_curr_cell.area,
                      delta,
                      (fx2 + POLY_SUBPIXEL_SCALE - first) * delta, m_curr_cell.x, m_curr_cell.y);
        }
    }

    pub fn line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) {
        eprintln!("LINE: {} {} -> {} {}", x1,y1, x2,y2);
        let dx_limit = 16384 << POLY_SUBPIXEL_SHIFT;
        let dx = x2 - x1;
        // Split long lines in half
        if dx >= dx_limit || dx <= -dx_limit {
            let cx = (x1 + x2) / 2;
            let cy = (y1 + y2) / 2;
            self.line(x1, y1, cx, cy);
            self.line(cx, cy, x2, y2);
        }
        let dy = y2-y1;
        let ex1 = x1 >> POLY_SUBPIXEL_SHIFT;
        let ex2 = x2 >> POLY_SUBPIXEL_SHIFT;
        let ey1 = y1 >> POLY_SUBPIXEL_SHIFT;
        let ey2 = y2 >> POLY_SUBPIXEL_SHIFT;
        let fy1 = y1 &  POLY_SUBPIXEL_MASK;
        let fy2 = y2 &  POLY_SUBPIXEL_MASK;

        self.min_x = min(ex2, min(ex1, self.min_x));
        self.min_y = min(ey2, min(ey1, self.min_y));
        self.max_x = max(ex2, max(ex1, self.max_x));
        self.max_y = max(ey2, max(ey1, self.max_y));

        self.set_curr_cell(ex1, ey1);
        //eprintln!("EY1, EY2: {} {}", ey1, ey2);
        // Horizontal Line
        if ey1 == ey2 {
            //eprintln!("LINE EY1 = EY2");
            self.render_hline(ey1, x1, fy1, x2, fy2);
            let n = self.cells.len();
            if self.cells[n-1].area == 0 && self.cells[n-1].cover == 0 {
                self.cells.pop();
            }
            return;
        }

        if dx == 0 {
            eprintln!("LINE DX = 0");
            let ex = x1 >> POLY_SUBPIXEL_SHIFT;
            let two_fx = (x1 - (ex << POLY_SUBPIXEL_SHIFT)) << 1;

            let (first, incr) = if dy < 0 {
                (0, -1)
            } else {
                (POLY_SUBPIXEL_SCALE, 1)
            };
            //let x_from = x1;
            let delta = first - fy1;
            {
                let m_curr_cell = self.cells.last_mut().unwrap();
                m_curr_cell.cover += delta;
                m_curr_cell.area  += two_fx * delta;
            }

            let mut ey1 = ey1 + incr;
            self.set_curr_cell(ex, ey1);
            let delta = first + first - POLY_SUBPIXEL_SCALE;
            let area = two_fx * delta;
            while ey1 != ey2 {
                {
                    let m_curr_cell = self.cells.last_mut().unwrap();
                    m_curr_cell.cover = delta;
                    m_curr_cell.area = area;
                }
                ey1 += incr;
                self.set_curr_cell(ex, ey1);
            }
            let delta = fy2 - POLY_SUBPIXEL_SCALE + first;
            {
                let m_curr_cell = self.cells.last_mut().unwrap();
                m_curr_cell.cover += delta;
                m_curr_cell.area += two_fx * delta;
            }
            return;
        }
        eprintln!("LINE RENDER MULTPLE LINES {} {}", dx, dy);
        // Render Multiple Lines
        let (p,first,incr, dy) = if dy < 0 {
            (fy1 * dx, 0, -1, -dy)
        } else {
            ((POLY_SUBPIXEL_SCALE - fy1) * dx, POLY_SUBPIXEL_SCALE, 1, dy)
        };
        let mut delta = p / dy;
        let mut xmod  = p % dy;
        if xmod < 0 {
            delta -= 1;
            xmod += dy;
        }
        let mut x_from = x1 + delta;
        self.render_hline(ey1, x1, fy1, x_from, first);
        let mut ey1 = ey1 + incr;
        self.set_curr_cell(x_from >> POLY_SUBPIXEL_SHIFT, ey1);
        if ey1 != ey2 {
            let p = POLY_SUBPIXEL_SCALE * dx;
            let mut lift = p / dy;
            let mut rem  = p % dy;
            if rem < 0 {
                lift -= 1;
                rem += dy;
            }
            xmod -= dy;
            while ey1 != ey2 {
                delta = lift;
                xmod += rem;
                if xmod >= 0 {
                    xmod -= dy;
                    delta += 1;
                }
                let x_to = x_from + delta;
                self.render_hline(ey1, x_from, POLY_SUBPIXEL_SCALE - first, x_to, first);
                x_from = x_to;
                ey1 += incr;
                self.set_curr_cell(x_from >> POLY_SUBPIXEL_SHIFT, ey1);
            }
        }
        self.render_hline(ey1, x_from, POLY_SUBPIXEL_SCALE - first, x2, fy2);
        self.pop_last_cell_if_empty();
    }
}

/*
pub trait Scale<T> {
    fn upscale(v: f64)   -> T;
    fn downscale(v: i64) -> T;

}
*/

pub struct RasConvInt {
}
impl RasConvInt {
    pub fn upscale(v: f64) -> i64 {
        (v * POLY_SUBPIXEL_SCALE as f64).round() as i64
    }
    pub fn downscale(v: i64) -> i64 {
        v
    }
}

pub trait RasterizerScanline {
    fn rewind_scanlines(&mut self) -> bool;
    fn sweep_scanline(&mut self, sl: &mut ScanlineU8) -> bool;
    fn max_x(&self) -> i64;
    fn min_x(&self) -> i64;
}

#[derive(Debug, Default)]
pub struct RasterizerScanlineAA {
    pub clipper: Clip,
    pub outline: RasterizerCell,
    pub status: PathStatus,
    pub x0: i64,
    pub y0: i64,
    scan_y: i64,
    filling_rule: FillingRule,
    gamma: Vec<u64>,
}


impl RasterizerScanline for RasterizerScanlineAA {
    fn rewind_scanlines(&mut self) -> bool {

        self.close_polygon();
        self.outline.sort_cells();
        if self.outline.total_cells() == 0 {
            false
        } else {
            self.scan_y = self.outline.min_y;
            true
        }
    }
    fn sweep_scanline(&mut self, sl: &mut ScanlineU8) -> bool {
        loop {
            eprintln!("SWEEP SCANLINES: Y: {}", self.scan_y);
            if self.scan_y > self.outline.max_y {
                return false;
            }
            sl.reset_spans();
            let mut num_cells = self.outline.scanline_num_cells( self.scan_y );
            let cells = self.outline.scanline_cells( self.scan_y );

            let mut cover = 0;

            let mut iter = cells.iter();
            //eprintln!("SWEEP SCANLINES: ADDING ITER: {:?} N {}", iter, num_cells);

            if let Some(mut cur_cell) = iter.next() {
                while num_cells > 0 {
                    //eprintln!("SWEEP SCANLINES: ITER: {:?} N {}", iter, num_cells);
                    //let cur_cell = iter.next().unwrap();
                    //num_cells -= 1;

                    let mut x = cur_cell.x;
                    let mut area = cur_cell.area;

                    cover  += cur_cell.cover;
                    eprintln!("SWEEP SCANLINES: {:?} outside cover {} area {}", cur_cell, cover, area);
                    num_cells -= 1;
                    //eprintln!("SWEEP SCANLINES: N(A): {}", num_cells); 
                    //accumulate all cells with the same X
                    while num_cells > 0 {
                        cur_cell = iter.next().unwrap();
                        eprintln!("SWEEP SCANLINES: {:?} inside cover {} area {}", cur_cell, cover, area);
                        if cur_cell.x != x {
                            break;
                        }
                        area += cur_cell.area;
                        cover += cur_cell.cover;
                        num_cells -= 1;
                        //eprintln!("SWEEP SCANLINES: N(B): {}", num_cells); 
                    }
                    eprintln!("SWEEP SCANLINES: {:?} DONE cover {} area {}", cur_cell, cover, area);
                    //eprintln!("SWEEP SCANLINES: ADDING CHECK AREA: {} NUM_CELLS {} x,y {} {}", area, num_cells, x, self.scan_y);
                    if area != 0 {
                        eprintln!("SWEEP SCANLINES: ADDING CELL: x {} y {} area {} cover {}", x,self.scan_y, area, cover);
                        let alpha = self.calculate_alpha((cover << (POLY_SUBPIXEL_SHIFT + 1)) - area);
                        if alpha > 0 {
                            sl.add_cell(x, alpha);
                        }
                        x += 1;
                    }
                    if num_cells > 0 && cur_cell.x > x {
                        let alpha = self.calculate_alpha(cover << (POLY_SUBPIXEL_SHIFT + 1));
                        eprintln!("SWEEP SCANLINES: ADDING SPAN: {} -> {} Y: {} {} {}", x, cur_cell.x, self.scan_y, area, cover);
                        if alpha > 0 {
                            sl.add_span(x, cur_cell.x - x, alpha);
                        }
                    }
                }
            }
            if sl.num_spans() != 0 {
                break;
            }
            self.scan_y += 1;
            eprintln!("SWEEP SCANLINES:  ---------------------");
        }
        sl.finalize(self.scan_y);
        self.scan_y += 1;
        true
    }
    fn min_x(&self) -> i64 {
        self.outline.min_x
    }
    fn max_x(&self) -> i64 {
        self.outline.max_x
    }
}

impl RasterizerScanlineAA {
    pub fn new() -> Self {
        return Self::new_with_gamma(|v| v as u64)
    }
    pub fn new_with_gamma<F>(gfunc: F) -> Self
        where F: Fn(f64) -> u64
    {
        let gamma : Vec<_> = (0..256).map(|i| gfunc(i as f64)).collect();
        Self { clipper: Clip::new(), status: PathStatus::Initial,
               outline: RasterizerCell::new(),
               x0: 0, y0: 0, scan_y: 0,
               filling_rule: FillingRule::NonZero,
               gamma: gamma,
        }
    }
    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.clipper.clip_box(RasConvInt::upscale(x1),
                              RasConvInt::upscale(y1),
                              RasConvInt::upscale(x2),
                              RasConvInt::upscale(y2));
    }
    pub fn move_to_d(&mut self, x: f64, y: f64) {
        self.x0 = RasConvInt::upscale( x );
        self.y0 = RasConvInt::upscale( y );
        self.clipper.move_to(self.x0,self.y0);
        self.status = PathStatus::MoveTo;
    }
    pub fn line_to_d(&mut self, x: f64, y: f64) {
        let x = RasConvInt::upscale( x );
        let y = RasConvInt::upscale( y );
        self.clipper.line_to(&mut self.outline, x,y);
        self.status = PathStatus::LineTo;
    }
    pub fn close_polygon(&mut self) {
        eprintln!("CLOSE POLYGON?");
        if self.status == PathStatus::LineTo {
            eprintln!("CLOSE POLYGON: CLOSED {} {}",self.x0>>8, self.y0>>8);
            self.clipper.line_to(&mut self.outline, self.x0, self.y0);
            self.status = PathStatus::Closed;
        }
    }
    pub fn reset(&mut self) {
        self.outline.reset();
        self.status = PathStatus::Initial;
    }
    pub fn add_path<VS: VertexSource>(&mut self, path: &mut VS) {
        path.rewind();
        if ! self.outline.sorted_y.is_empty() {
            self.reset();
        }
        for seg in path.convert() {
            eprintln!("ADD_PATH: {} {} {:?} ", seg.x, seg.y, seg.cmd);
            if seg.cmd == PathCommand::LineTo {
                self.line_to_d(seg.x, seg.y);
            } else if seg.cmd == PathCommand::MoveTo {
                self.move_to_d(seg.x, seg.y);
            } else if seg.cmd == PathCommand::Close {
                self.close_polygon();
            }
            eprintln!("ADD_PATH: {} {} {:?} DONE", seg.x, seg.y, seg.cmd);
        }
    }
    pub fn calculate_alpha(&self, area: i64) -> u64 {
        let aa_shift  = 8;
        let aa_scale  = 1 << aa_shift;
        let aa_scale2 = aa_scale * 2;
        let aa_mask   = aa_scale  - 1;
        let aa_mask2  = aa_scale2 - 1;

        let mut cover = area >> (POLY_SUBPIXEL_SHIFT*2 + 1 - aa_shift);
        cover = cover.abs();
        if self.filling_rule == FillingRule::EvenOdd {
            cover *= aa_mask2;
            if cover > aa_scale {
                cover = aa_scale2 - cover;
            }
        }
        cover = max(0, min(cover, aa_mask));
        self.gamma[cover as usize]
    }


}

#[derive(Debug,Default)]
pub struct Span {
    x: i64,
    len: i64,
    covers: Vec<u64>,
}
#[derive(Debug,Default)]
pub struct ScanlineU8 {
    last_x: i64,
    min_x: i64,
    spans: Vec<Span>,
    covers: HashMap<i64, u64>,
    y: i64,
}

const LAST_X: i64 = 0x7FFF_FFF0;
impl ScanlineU8 {
    pub fn new() -> Self {
        Self { last_x: LAST_X, min_x: 0, y: 0,
               spans: vec![], covers: HashMap::new() }
    }
    pub fn reset_spans(&mut self) {
        self.last_x = LAST_X;
        self.spans.clear();
        self.covers.clear();
    }
    pub fn reset(&mut self, min_x: i64, _max_x: i64) {
        self.last_x = LAST_X;
        self.min_x = min_x;
        self.spans = vec![];
        self.covers = HashMap::new()
    }
    pub fn finalize(&mut self, y: i64) {
        self.y = y;
    }
    pub fn num_spans(&self) -> usize {
        self.spans.len()
    }
    pub fn add_span(&mut self, x: i64, len: i64, cover: u64) {
        let x = x - self.min_x;
        self.covers.insert( x, cover );
        if x == self.last_x + 1 {
            let cur = self.spans.last_mut().unwrap();
            eprintln!("ADD_SPAN: Increasing length of span: {} {} x: {} {}", cur.len, cur.covers.len(), x+self.min_x, len);
            cur.len += len;
            cur.covers.extend(vec![cover; len as usize]);
            eprintln!("ADD_SPAN: Increasing length of span: {} {} x: {}", cur.len, cur.covers.len(), x+self.min_x);
        } else {
            eprintln!("ADD_SPAN: Adding span of length: {} at {}", len, x+self.min_x);
            let span = Span { x: x + self.min_x, len,
                              covers: vec![cover; len as usize] };
            self.spans.push(span);
        }
        self.last_x = x + len - 1;
    }
    pub fn add_cell(&mut self, x: i64, cover: u64) {
        let x = x - self.min_x;
        self.covers.insert( x, cover );
        if x == self.last_x + 1 {
            let cur = self.spans.last_mut().unwrap();
            cur.len += 1;
            cur.covers.push(cover);
        } else {
            //let cover = self.covers.get(&x).unwrap().clone();
            let span = Span { x: x + self.min_x, len: 1,
                              covers: vec![cover] };
            self.spans.push(span);
        }
        self.last_x = x;
    }
}

pub fn render_scanlines_aa_solid<RAS,C>(ras: &mut RAS,
                                            sl: &mut ScanlineU8,
                                            ren: &mut RenderingBase,
                                            color: C) 
    where RAS: RasterizerScanline,
          C: Color
{
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(sl) {
            render_scanline_aa_solid(sl, ren, &color);
        }
    }
}

pub fn render_scanlines<REN, RAS>(ras: &mut RAS,
                                  sl: &mut ScanlineU8,
                                  ren: &mut REN)
    where REN: RenderingScanline,
          RAS: RasterizerScanline
{
    eprintln!("RENDER SCANLINES");
    if ras.rewind_scanlines() {
        eprintln!("RENDER RESET");
        sl.reset( ras.min_x(), ras.max_x() );
        eprintln!("RENDER SCANLINES PREPARE");
        ren.prepare();
        eprintln!("RENDER SCANLINES SWEEP");
        while ras.sweep_scanline(sl) {
            eprintln!("----------------------------------------------");
            eprintln!("RENDER SCANLINES RENDER: {:?}", sl);
            ren.render(&sl);
        }
    }
}

pub fn compare_ppm<P: AsRef<Path>>(f1: P, f2: P) {
    let d1 = fs::read(f1).expect("Unable to read file");
    let d2 = fs::read(f2).expect("Unable to read file");
    for (v1,v2) in d1.iter().zip(d2.iter()) {
        assert_eq!(v1,v2);
    }
}

pub fn write_ppm<P: AsRef<Path>>(buf: &[u8], width: usize, height: usize, filename: P) -> Result<(),std::io::Error> {
    let mut fd = File::create(filename)?;
    write!(fd, "P6 {} {} 255 ", width, height);
    fd.write_all(buf)?;
    Ok(())
}

