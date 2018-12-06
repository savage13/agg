//! Renderer

use scan::ScanlineU8;
use base::RenderingBase;
use color::Rgba8;
use POLY_SUBPIXEL_SCALE;
use POLY_SUBPIXEL_SHIFT;

use PixelData;
use VertexSource;
use Render;
use Rasterize;
use Color;
use PixfmtFunc;
use Pixel;

/// Aliased Renderer
#[derive(Debug)]
pub struct RenderingScanlineBinSolid<'a,T> where T: PixfmtFunc + Pixel, T: 'a {
    pub base: &'a mut RenderingBase<T>,
    pub color: Rgba8,
}
/// Anti-Aliased Renderer
#[derive(Debug)]
pub struct RenderingScanlineAASolid<'a,T> where T: PixfmtFunc + Pixel, T: 'a {
    pub base: &'a mut RenderingBase<T>,
    pub color: Rgba8,
}

/// Render a single Scanline (y-row) without Anti-Aliasing (Binary?)
fn render_scanline_bin_solid<T,C: Color>(sl: &ScanlineU8,
                                           ren: &mut RenderingBase<T>,
                                         color: &C)
    where T: PixfmtFunc + Pixel
{
    let cover_full = 255;
    for span in &sl.spans {
        eprintln!("RENDER SCANLINE BIN SOLID: Span x,y,len {} {} {} {}",
                  span.x, sl.y, span.len, span.covers.len());
        ren.blend_hline(span.x, sl.y, span.x - 1 + span.len.abs(),
                        color, cover_full);
    }
}

/// Render a single Scanline (y-row) with Anti Aliasing
fn render_scanline_aa_solid<T,C: Color>(sl: &ScanlineU8,
                                          ren: &mut RenderingBase<T>,
                                        color: &C)
    where T: PixfmtFunc + Pixel 
{
    let y = sl.y;
    for span in & sl.spans {
        eprintln!("RENDER SCANLINE AA SOLID: Span x,y,len {} {} {} {}", span.x, y, span.len, span.covers.len());
        let x = span.x;
        if span.len > 0 {
            // eprintln!("RENDER SCANLINE AA SOLID: {} {}",
            //           span.len, span.covers.len());
            ren.blend_solid_hspan(x, y, span.len, color, &span.covers);
        } else {
            ren.blend_hline(x, y, x-span.len-1, color, span.covers[0]);
        }
    }
}


impl<'a,T> Render for RenderingScanlineAASolid<'a,T> where T: PixfmtFunc + Pixel {
    /// Render a single Scanline Row
    fn render(&mut self, sl: &ScanlineU8) {
        render_scanline_aa_solid(sl, &mut self.base, &self.color);
    }
    /// Set the current color
    fn color<C: Color>(&mut self, color: &C) {
        self.color = Rgba8::new(color.red8(), color.green8(),
                                color.blue8(), color.alpha8());
    }

}
impl<'a,T> Render for RenderingScanlineBinSolid<'a,T> where T: PixfmtFunc + Pixel {
    /// Render a single Scanline Row
    fn render(&mut self, sl: &ScanlineU8) {
        render_scanline_bin_solid(sl, &mut self.base, &self.color);
    }
    /// Set the current Color
    fn color<C: Color>(&mut self, color: &C) {
        self.color = Rgba8::new(color.red8(),color.green8(),
                                color.blue8(), color.alpha8());
    }
}
impl<'a,T> RenderingScanlineBinSolid<'a,T> where T: PixfmtFunc + Pixel {
    /// Create a new Renderer from a Rendering Base
    pub fn with_base(base: &'a mut RenderingBase<T>) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
}
impl<'a,T> RenderingScanlineAASolid<'a,T> where T: PixfmtFunc + Pixel {
    /// Create a new Renderer from a Rendering Base
    pub fn with_base(base: &'a mut RenderingBase<T>) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
}
impl<'a,T> PixelData<'a> for RenderingScanlineAASolid<'a,T> where T: PixfmtFunc + Pixel {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.base.pixf.rbuf().data
    }
}
impl<'a,T> PixelData<'a> for RenderingScanlineBinSolid<'a,T> where T: PixfmtFunc + Pixel  {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.base.pixf.rbuf().data
    }
}

/* pub trait Scale<T> {
    fn upscale(v: f64)   -> T;
    fn downscale(v: i64) -> T;
}*/

/// Render rasterized data to an image using a single color, Binary
pub fn render_scanlines_bin_solid<RAS,C,T>(_ras: &mut RAS,
                                         _sl: &mut ScanlineU8,
                                         _ren: &mut RenderingBase<T>,
                                         _color: &C) 
    where RAS: Rasterize,
          C: Color,
          T: PixfmtFunc + Pixel
{
    unimplemented!();
}

/// Render rasterized data to an image using a single color, Anti-aliased
pub fn render_scanlines_aa_solid<RAS,C,T>(ras: &mut RAS,
                                        sl: &mut ScanlineU8,
                                        ren: &mut RenderingBase<T>,
                                        color: &C) 
    where RAS: Rasterize,
          C: Color,
          T: PixfmtFunc + Pixel
{
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(sl) {
            render_scanline_aa_solid(sl, ren, color);
        }
    }
}

/// Render rasterized data to an image using the current color
pub fn render_scanlines<REN, RAS>(ras: &mut RAS,
                                  sl: &mut ScanlineU8,
                                  ren: &mut REN)
    where REN: Render,
          RAS: Rasterize
{
    //eprintln!("RENDER SCANLINES");
    if ras.rewind_scanlines() {
        //eprintln!("RENDER RESET");
        sl.reset( ras.min_x(), ras.max_x() );
        //eprintln!("RENDER SCANLINES PREPARE");
        ren.prepare();
        //eprintln!("RENDER SCANLINES SWEEP");
        while ras.sweep_scanline(sl) {
            //eprintln!("----------------------------------------------");
            //eprintln!("RENDER SCANLINES RENDER: {:?}", sl);
            ren.render(&sl);
        }
    }
}

/// Render paths after rasterizing to an image using a set of colors
pub fn render_all_paths<REN,RAS,VS,C>(ras: &mut RAS,
                                      sl: &mut ScanlineU8,
                                      ren: &mut REN,
                                      paths: &[VS],
                                      colors: &[C])
    where C: Color,
          REN: Render,
          RAS: Rasterize,
          VS: VertexSource
{
    debug_assert!(paths.len() == colors.len());
    for (path, color) in paths.iter().zip(colors.iter()) {
        ras.reset();
        ras.add_path(path);
        ren.color(color);
        render_scanlines(ras, sl, ren);
    }

}
pub struct RendererPrimatives<'a,T> where T: PixfmtFunc + Pixel, T: 'a {
    pub base: &'a mut RenderingBase<T>,
    pub fill_color: Rgba8,
    pub line_color: Rgba8,
    pub x: i64,
    pub y: i64,
}

impl<'a,T> RendererPrimatives<'a,T> where T: PixfmtFunc + Pixel {
    pub fn with_base(base: &'a mut RenderingBase<T>) -> Self {
        let fill_color = Rgba8::new(0,0,0,255);
        let line_color = Rgba8::new(0,0,0,255);
        Self { base, fill_color, line_color, x: 0, y: 0 }
    }
    pub fn line_color<C: Color>(&mut self, line_color: &C) {
        self.line_color = line_color.into();
    }
    pub fn fill_color<C: Color>(&mut self, fill_color: &C) {
        self.fill_color = fill_color.into();
    }
    pub fn coord(&self, c: f64) -> i64 {
        (c * POLY_SUBPIXEL_SCALE as f64).round() as i64
    }
    pub fn move_to(&mut self, x: i64, y: i64) {
        self.x = x;
        self.y = y;
        eprintln!("DDA MOVE: {} {}", x>>8, y>>8);
    }
    pub fn line_to(&mut self, x: i64, y: i64) {
        eprintln!("DDA LINE: {} {}", x>>8, y>>8);
        let (x0,y0) = (self.x, self.y);
        self.line(x0, y0, x, y);
        self.x = x;
        self.y = y;
    }
    fn line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) {
        //let cover_shift = POLY_SUBPIXEL_SCALE;
        //let cover_size = 1 << cover_shift;
        //let cover_mask = cover_size - 1;
        //let cover_full = cover_mask;
        let color = self.line_color;
        let mut li = BresehamInterpolator::new(x1,y1,x2,y2);
        if li.len == 0 {
            return;
        }
        if li.ver {
            for _ in 0 .. li.len {
                eprintln!("DDA PIX VER {} {}", li.x2, li.y1);
                self.base.pixf.set((li.x2 as usize, li.y1 as usize), &color);
                li.vstep();
            }
        } else {
            for _ in 0 .. li.len {
                eprintln!("DDA PIX HOR {} {} {} {}", li.x1, li.y2, li.func.y, li.func.y >>8);
                self.base.pixf.set((li.x1 as usize, li.y2 as usize), &color);
                li.hstep();
            }
        }
    }
}

struct BresehamInterpolator {
    /// First point, x position
    x1: i64,
    /// First point, y position
    y1: i64,
    /// Second point, x position
    x2: i64,
    /// Second point, y position
    y2: i64,
    /// Line is primarilly vertical
    ver: bool,
    len: i64,
    inc: i64,
    func: LineInterpolator,
}

impl BresehamInterpolator {
    fn new(x1_hr: i64, y1_hr: i64, x2_hr: i64, y2_hr: i64) -> Self {
        let x1 = x1_hr >> POLY_SUBPIXEL_SHIFT;
        let x2 = x2_hr >> POLY_SUBPIXEL_SHIFT;
        let y1 = y1_hr >> POLY_SUBPIXEL_SHIFT;
        let y2 = y2_hr >> POLY_SUBPIXEL_SHIFT;
        let dy = (y2 - y1).abs();
        let dx = (x2 - x1).abs();
        let ver = dy > dx;
        let len = if ver { dy } else { dx };
        let inc = if ver {
            if y2 > y1 { 1 } else { -1 }
        } else {
            if x2 > x1 { 1 } else { -1 }
        };
        let (z1,z2) = if ver { (x1_hr,x2_hr) } else { (y1_hr,y2_hr) };
        let func = LineInterpolator::new(z1,z2,len);
        eprintln!("DDA: {} {} {} {} LINE", x1_hr, y1_hr, x2_hr, y2_hr);
        let y2 = func.y >> POLY_SUBPIXEL_SHIFT;
        let x2 = func.y >> POLY_SUBPIXEL_SHIFT;
        Self { x1, y1, x2, y2, ver, len, inc, func }
    }
    fn vstep(&mut self) {
        eprintln!("DDA VSTEP {} ({}) {}", self.y1, self.inc, self.func.y);
        self.func.inc();
        self.y1 += self.inc as i64;
        self.x2 = self.func.y >> POLY_SUBPIXEL_SHIFT;
        eprintln!("DDA VSTEP {} ({}) {}<==", self.y1, self.inc, self.func.y);
    }
    fn hstep(&mut self) {
        eprintln!("DDA HSTEP {} ({}) {}", self.x1, self.inc, self.func.y);
        self.func.inc();
        self.x1 += self.inc as i64;
        self.y2 = self.func.y >> POLY_SUBPIXEL_SHIFT;
        eprintln!("DDA HSTEP {} ({}) {}<==", self.x1, self.inc, self.func.y);
    }
}

pub struct LineInterpolator {
    count: i64,
    left: i64,
    rem: i64,
    xmod: i64,
    y: i64
}

impl LineInterpolator {
    pub fn new(y1: i64, y2: i64, count: i64) -> Self {
        let cnt = std::cmp::max(1,count);
        let mut left = (y2 - y1) / cnt;
        let mut rem  = (y2 - y1) % cnt;
        let mut xmod = rem;
        let y = y1;
        eprintln!("DDA: {} {} {} {} {} :: {} {}", y, left, rem, xmod, cnt, y1, y2);
        if xmod <= 0 {
            xmod += count;
            rem  += count;
            left -= 1;
        }
        xmod -= count;
        eprintln!("DDA: {} {} {} {} {} :: {} {}", y, left, rem, xmod, cnt, y1, y2);
        Self { y, left, rem, xmod, count: cnt }
    }
    pub fn inc(&mut self) {
        self.xmod += self.rem;
        self.y += self.left;
        if self.xmod > 0 {
            self.xmod -= self.count;
            self.y += 1;
        }
    }
}
