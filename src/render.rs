//! Renderer

use crate::scan::ScanlineU8;
use crate::base::RenderingBase;
use crate::color::Rgba8;
use crate::POLY_SUBPIXEL_SCALE;
use crate::POLY_SUBPIXEL_MASK;
use crate::POLY_SUBPIXEL_SHIFT;
use crate::POLY_MR_SUBPIXEL_SHIFT;
use crate::clip::Rectangle;
use crate::line_interp::LineParameters;
use crate::raster::len_i64_xy;
use crate::clip::{INSIDE, TOP,BOTTOM,LEFT,RIGHT};
use crate::line_interp::DistanceInterpolator00;
use crate::line_interp::DistanceInterpolator0;
use crate::RenderOutline;
use crate::MAX_HALF_WIDTH;
use crate::line_interp::line_mr;
use crate::pixfmt::Pixfmt;
use crate::raster::RasterizerScanline;

use crate::Source;
use crate::VertexSource;
use crate::Render;
use crate::Color;
use crate::DrawOutline;
use crate::Pixel;
use crate::SetColor;
use crate::AccurateJoins;
use crate::Lines;

use crate::outline::Subpixel;

const LINE_MAX_LENGTH : i64 = 1 << (POLY_SUBPIXEL_SHIFT + 10);

/// Aliased Renderer
#[derive(Debug)]
pub struct RenderingScanlineBinSolid<'a,T> where T: 'a {
    pub base: &'a mut RenderingBase<T>,
    pub color: Rgba8,
}
/// Anti-Aliased Renderer
#[derive(Debug)]
pub struct RenderingScanlineAASolid<'a,T> where T: 'a {
    pub base: &'a mut RenderingBase<T>,
    pub color: Rgba8,
}

/// Render a single Scanline (y-row) without Anti-Aliasing (Binary?)
fn render_scanline_bin_solid<T,C: Color>(sl: &ScanlineU8,
                                         ren: &mut RenderingBase<T>,
                                         color: C)
    where T: Pixel
{
    let cover_full = 255;
    for span in &sl.spans {
        //eprintln!("RENDER SCANLINE BIN SOLID: Span x,y,len {} {} {} {}",
        //          span.x, sl.y, span.len, span.covers.len());
        ren.blend_hline(span.x, sl.y, span.x - 1 + span.len.abs(),
                        color, cover_full);
    }
}

/// Render a single Scanline (y-row) with Anti Aliasing
fn render_scanline_aa_solid<T,C: Color>(sl: &ScanlineU8,
                                        ren: &mut RenderingBase<T>,
                                        color: C)
    where T: Pixel
{
    let y = sl.y;
    for span in & sl.spans {
        let x = span.x;
        if span.len > 0 {
            ren.blend_solid_hspan(x, y, span.len, color, &span.covers);
        } else {
            ren.blend_hline(x, y, x-span.len-1, color, span.covers[0]);
        }
    }
}
#[derive(Debug)]
pub struct RenderData {
    sl: ScanlineU8
}
impl RenderData {
    pub fn new() -> Self {
        Self { sl: ScanlineU8::new() }
    }
}

impl<T> Render for RenderingScanlineAASolid<'_,T> where T: Pixel {
    /// Render a single Scanline Row
    fn render(&mut self, data: &RenderData) {
        render_scanline_aa_solid(&data.sl, &mut self.base, self.color);
    }
    /// Set the current color
    fn color<C: Color>(&mut self, color: &C) {
        self.color = Rgba8::new(color.red8(), color.green8(),
                                color.blue8(), color.alpha8());
    }

}
impl<T> Render for RenderingScanlineBinSolid<'_,T> where T: Pixel {
    /// Render a single Scanline Row
    fn render(&mut self, data: &RenderData) {
        render_scanline_bin_solid(&data.sl, &mut self.base, self.color);
    }
    /// Set the current Color
    fn color<C: Color>(&mut self, color: &C) {
        self.color = Rgba8::new(color.red8(),color.green8(),
                                color.blue8(), color.alpha8());
    }
}
impl<'a,T> RenderingScanlineBinSolid<'a,T> where T: Pixel {
    /// Create a new Renderer from a Rendering Base
    pub fn with_base(base: &'a mut RenderingBase<T>) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.base.as_bytes()
    }

}
impl<'a,T> RenderingScanlineAASolid<'a,T> where T: Pixel {
    /// Create a new Renderer from a Rendering Base
    pub fn with_base(base: &'a mut RenderingBase<T>) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.base.as_bytes()
    }
}

/* pub trait Scale<T> {
    fn upscale(v: f64)   -> T;
    fn downscale(v: i64) -> T;
}*/

/// Render rasterized data to an image using a single color, Binary
pub fn render_scanlines_bin_solid<C,T>(ras: &mut RasterizerScanline,
                                       ren: &mut RenderingBase<T>,
                                       color: C)
    where C: Color,
          T: Pixel
{
    let mut sl = ScanlineU8::new();
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(&mut sl) {
            render_scanline_bin_solid(&sl, ren, color);
        }
    }
}

/// Render rasterized data to an image using a single color, Anti-aliased
pub fn render_scanlines_aa_solid<C,T>(ras: &mut RasterizerScanline,
                                      ren: &mut RenderingBase<T>,
                                      color: C)
    where C: Color,
          T: Pixel
{
    let mut sl = ScanlineU8::new();
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(&mut sl) {
            render_scanline_aa_solid(&sl, ren, color);
        }
    }
}

/// Render rasterized data to an image using the current color
pub fn render_scanlines<REN>(ras: &mut RasterizerScanline,
                             ren: &mut REN)
    where REN: Render
{
    let mut data = RenderData::new();
    if ras.rewind_scanlines() {
        data.sl.reset( ras.min_x(), ras.max_x() );
        ren.prepare();
        while ras.sweep_scanline(&mut data.sl) {
            ren.render(&data);
        }
    }
}

/// Render paths after rasterizing to an image using a set of colors
pub fn render_all_paths<REN,VS,C>(ras: &mut RasterizerScanline,
                                  ren: &mut REN,
                                  paths: &[VS],
                                  colors: &[C])
    where C: Color,
          REN: Render,
          VS: VertexSource
{
    debug_assert!(paths.len() == colors.len());
    for (path, color) in paths.iter().zip(colors.iter()) {
        ras.reset();
        ras.add_path(path);
        ren.color(color);
        render_scanlines(ras, ren);
    }

}
#[derive(Debug)]
pub struct RendererPrimatives<'a,T> where T: 'a {
    base: &'a mut RenderingBase<T>,
    fill_color: Rgba8,
    line_color: Rgba8,
    x: Subpixel,
    y: Subpixel,
}

impl<'a,T> RendererPrimatives<'a,T> where T: Pixel {
    pub fn with_base(base: &'a mut RenderingBase<T>) -> Self {
        let fill_color = Rgba8::new(0,0,0,255);
        let line_color = Rgba8::new(0,0,0,255);
        Self { base, fill_color, line_color,
               x: Subpixel::from(0),
               y: Subpixel::from(0)
        }
    }
    pub fn line_color<C: Color>(&mut self, line_color: C) {
        self.line_color = Rgba8::from_trait(line_color);
    }
    pub fn fill_color<C: Color>(&mut self, fill_color: C) {
        self.fill_color = Rgba8::from_trait(fill_color);
    }
    pub(crate) fn coord(&self, c: f64) -> Subpixel {
        Subpixel::from( (c * POLY_SUBPIXEL_SCALE as f64).round() as i64 )
    }
    pub(crate) fn move_to(&mut self, x: Subpixel, y: Subpixel) {
        self.x = x;
        self.y = y;
        //eprintln!("DDA MOVE: {} {}", x>>8, y>>8);
    }
    pub(crate) fn line_to(&mut self, x: Subpixel, y: Subpixel) {
        //eprintln!("DDA LINE: {} {}", x>>8, y>>8);
        let (x0,y0) = (self.x, self.y);
        self.line(x0, y0, x, y);
        self.x = x;
        self.y = y;
    }
    fn line(&mut self, x1: Subpixel, y1: Subpixel, x2: Subpixel, y2: Subpixel) {
        //let cover_shift = POLY_SUBPIXEL_SCALE;
        //let cover_size = 1 << cover_shift;
        //let cover_mask = cover_size - 1;
        //let cover_full = cover_mask;
        let mask = T::cover_mask();
        let color = self.line_color;
        let mut li = BresehamInterpolator::new(x1,y1,x2,y2);
        if li.len == 0 {
            return;
        }
        if li.ver {
            for _ in 0 .. li.len {
                //self.base.pixf.set((li.x2 as usize, li.y1 as usize), color);
                self.base.blend_hline(li.x2, li.y1, li.x2, color, mask);
                li.vstep();
            }
        } else {
            for _ in 0 .. li.len {
                //eprintln!("DDA PIX HOR {} {} {} {}", li.x1, li.y2, li.func.y, li.func.y >>8);
                //self.base.pixf.set((li.x1 as usize, li.y2 as usize), color);
                self.base.blend_hline(li.x1, li.y2, li.x1, color, mask);
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
    fn new(x1_hr: Subpixel, y1_hr: Subpixel, x2_hr: Subpixel, y2_hr: Subpixel) -> Self {
        let x1 = i64::from(x1_hr);
        let x2 = i64::from(x2_hr);
        let y1 = i64::from(y1_hr);
        let y2 = i64::from(y2_hr);
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
        // XXX  - value() should not be used
        let func = LineInterpolator::new(z1.value(), z2.value(), len);
        //eprintln!("DDA: {} {} {} {} LINE", x1_hr, y1_hr, x2_hr, y2_hr);
        let y2 = func.y >> POLY_SUBPIXEL_SHIFT;
        let x2 = func.y >> POLY_SUBPIXEL_SHIFT;
        Self { x1, y1, x2, y2, ver, len, inc, func }
    }
    fn vstep(&mut self) {
        //eprintln!("DDA VSTEP {} ({}) {}", self.y1, self.inc, self.func.y);
        self.func.inc();
        self.y1 += self.inc as i64;
        self.x2 = self.func.y >> POLY_SUBPIXEL_SHIFT;
        //eprintln!("DDA VSTEP {} ({}) {}<==", self.y1, self.inc, self.func.y);
    }
    fn hstep(&mut self) {
        //eprintln!("DDA HSTEP {} ({}) {}", self.x1, self.inc, self.func.y);
        self.func.inc();
        self.x1 += self.inc as i64;
        self.y2 = self.func.y >> POLY_SUBPIXEL_SHIFT;
        //eprintln!("DDA HSTEP {} ({}) {}<==", self.x1, self.inc, self.func.y);
    }
}

/// Line Interpolator using a Digital differential analyzer (DDA)

/// See [https://en.wikipedia.org/wiki/Digital_differential_analyzer_(graphics_algorithm)]()
#[derive(Debug)]
pub(crate) struct LineInterpolator {
    count: i64,
    left: i64,
    rem: i64,
    xmod: i64,
    pub y: i64,
}

impl LineInterpolator {
    // Values should be in Subpixel coordinates
    pub fn new(y1: i64, y2: i64, count: i64) -> Self { 
        let cnt = std::cmp::max(1,count);
        let mut left = (y2 - y1) / cnt;
        let mut rem  = (y2 - y1) % cnt;
        let mut xmod = rem;
        let y = y1;
        //eprintln!("DRAW: DDA: {} {} {} {} {} :: {} {} ", y, left, rem, xmod, cnt, y1, y2);
        if xmod <= 0 {
            xmod += count;
            rem  += count;
            left -= 1;
        }
        xmod -= count;

        Self { y, left, rem, xmod, count: cnt }
    }
    pub fn adjust_forward(&mut self) {
        self.xmod -= self.count;
    }
    // pub fn adjust_backward(&mut self) {
    //     self.xmod += self.count;
    // }
    pub fn new_foward_adjusted(y1: i64, y2: i64, count: i64) -> Self {
        Self::new(y1, y2, count)
    }
    pub fn new_back_adjusted_2(y: i64, count: i64) -> Self {
        //eprintln!("DRAW: LI::ba2() y,count {} {}", y, count);
        let cnt = std::cmp::max(1,count);
        let mut left = y / cnt;
        let mut rem = y % cnt;
        let mut xmod = rem;
        let m_y = 0;

        if xmod <= 0 {
            xmod += count;
            rem  += count;
            left -= 1;
        }

        Self { y: m_y, left, rem, xmod, count: cnt }
    }
    // pub fn new_back_adjusted_1(y1: i64, y2: i64, count: i64) -> Self {

    //     let mut back = Self::new(y1, y2, count);
    //     back.count += count;
    //     back
    // }
    pub fn inc(&mut self) {
        //eprintln!("DRAW: LI++ y,mod,rem,lft,cnt {} {} {} {} {}", self.y, self.xmod, self.rem, self.left, self.count);
        self.xmod += self.rem;
        self.y += self.left;
        if self.xmod > 0 {
            self.xmod -= self.count;
            self.y += 1;
        }
    }
    pub fn dec(&mut self) {
        //eprintln!("DRAW: LI--");
        if self.xmod <= self.rem {
            self.xmod += self.count;
            self.y -= 1;
        }
        self.xmod -= self.rem;
        self.y -= self.left;
    }
}

#[derive(Debug,Default)]
pub struct LineProfileAA {
    pub min_width: f64,
    pub smoother_width: f64,
    pub subpixel_width: i64,
    pub gamma: Vec<u8>,
    pub profile: Vec<u8>,
}

impl LineProfileAA {
    pub fn new() -> Self {
        let gamma : Vec<_> = (0..POLY_SUBPIXEL_SCALE).map(|x| x as u8).collect();
        Self { min_width: 1.0, smoother_width: 1.0, subpixel_width: 0,
               profile: vec![], gamma }
    }
    pub fn width(&mut self, w: f64) {
        let mut w = w;
        if w < 0.0 {
            w = 0.0;
        }
        if w < self.smoother_width {
            w += w;
        } else {
            w += self.smoother_width;
        }
        w *= 0.5;
        w -= self.smoother_width;
        let mut s = self.smoother_width;
        if w < 0.0 {
            s += w;
            w = 0.0;
        }
        self.set(w, s);
    }
    pub fn profile(&mut self, w: f64) {
        let subpixel_shift = POLY_SUBPIXEL_SHIFT;
        let subpixel_scale = 1 << subpixel_shift;
        self.subpixel_width = (w * subpixel_scale as f64).round() as i64;
        let size = (self.subpixel_width + subpixel_scale * 6) as usize;
        if size > self.profile.capacity() {
            self.profile.resize(size, 0);
        }
    }
    pub fn set(&mut self, center_width: f64, smoother_width: f64) {
        let subpixel_shift = POLY_SUBPIXEL_SHIFT;
        let subpixel_scale = 1 << subpixel_shift;
        let aa_shift = POLY_SUBPIXEL_SHIFT;
        let aa_scale = 1 << aa_shift;
        let aa_mask = aa_scale - 1;

        let mut base_val = 1.0;
        let mut center_width = center_width;
        let mut smoother_width = smoother_width;
        if center_width == 0.0 {
            center_width = 1.0 / subpixel_scale as f64;
        }
        if smoother_width == 0.0 {
            smoother_width = 1.0 / subpixel_scale as f64;
        }

        let width = center_width + smoother_width;
        if width < self.min_width {
            let k = width / self.min_width;
            base_val *= k;
            center_width /= k;
            smoother_width /= k;
        }

        self.profile(center_width + smoother_width);

        let subpixel_center_width : usize   = (center_width * subpixel_scale as f64) as usize;
        let subpixel_smoother_width : usize = (smoother_width * subpixel_scale as f64) as usize;
        let n_smoother = self.profile.len() -
            subpixel_smoother_width -
            subpixel_center_width -
            subpixel_scale*2;

        //let ch = &mut self.profile;

        let ch_center   = subpixel_scale*2;
        let ch_smoother = ch_center + subpixel_center_width;

        let val = self.gamma[(base_val * f64::from(aa_mask)) as usize];
        //eprintln!("-- PROFILE {:4} {:4}", (base_val * aa_mask as f64) as usize, val);
        //ch = ch_center;
        // Fill center portion (on one side)
        for i in 0 .. subpixel_center_width {
            self.profile[ch_center + i] = val;
        }

        for i  in 0 .. subpixel_smoother_width {
            let k = ((base_val - base_val * (i as f64 / subpixel_smoother_width as f64)) * f64::from(aa_mask)) as usize;
            //eprintln!("-- PROFILE {:4}", self.gamma[k]);
            self.profile[ch_smoother + i] = self.gamma[k];
        }

        // Remainder is gamma[0]
        let val = self.gamma[0];
        for i in 0 .. n_smoother  {
            self.profile[ch_smoother + subpixel_smoother_width + i] = val;
        }
        // Copy to other side
        for i in 0 .. subpixel_scale*2 {
            self.profile[ch_center - 1 - i] = self.profile[ch_center + i]
        }
        // for i in 0 .. self.profile.len() {
        //     if i > 0 && i % 10 == 0 {
        //         eprintln!("");
        //     }
        //     eprint!("{:3} ", self.profile[i]);
        // }
        // eprintln!("");
    }
}

#[derive(Debug)]
pub struct RendererOutlineAA<'a,T>  {
    ren: &'a mut RenderingBase<T>,
    color: Rgba8,
    clip_box: Option<Rectangle<i64>>,
    pub profile: LineProfileAA,
}

impl<'a,T> DrawOutline for RendererOutlineAA<'a,T> where T: Pixel {}

impl<'a,T> RendererOutlineAA<'a,T> where T: Pixel {
    pub fn with_base(ren: &'a mut RenderingBase<T>) -> Self {
        let profile = LineProfileAA::new();
        Self { ren, color: Rgba8::black(), clip_box: None, profile }
    }
    fn subpixel_width(&self) -> i64 {
        self.profile.subpixel_width
    }

    fn line0_no_clip(&mut self, lp: &LineParameters) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            self.line0_no_clip(&lp1);
            self.line0_no_clip(&lp2);
            return;
        }
        //eprintln!("DRAW: AA0::new line0_no_clip\n");
        let mut li = lp.interp0(self.subpixel_width());
        //eprintln!("DRAW: line0_no_clip count {} vertical {}", li.count(), li.vertical());
        if li.count() > 0 {
            if li.vertical() {
                while li.step_ver(self) {
                    //eprintln!("DRAW: AA0::new interp vertical\n");
                }
            } else {
                while li.step_hor(self) {
                    //eprintln!("DRAW: AA0::new interp horizontal\n");
                }
            }
        }
    }
    fn line1_no_clip(&mut self, lp: &LineParameters, sx: i64, sy: i64) {
        //eprintln!("DRAW: line1_no_clip() {} {} {} {}", lp.x1, lp.y1, lp.x2, lp.y2);
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            self.line1_no_clip(&lp1, (lp.x1 + sx)>>1, (lp.y1+sy)>>1);
            self.line1_no_clip(&lp2, lp.x2 + (lp.y1 + lp1.y1), lp1.y2 - (lp1.x2-lp1.x1));
            return;
        }
        //eprintln!("DRAW: AA1::new line1_no_clip\n");
        let (sx, sy) = lp.fix_degenerate_bisectrix_start(sx, sy);
        let mut li = lp.interp1(sx, sy, self.subpixel_width());
        //eprintln!("DRAW: line1_no_clip count {} vertical {}", li.count(), li.vertical());
        if li.vertical() {
            while li.step_ver(self) {
                //eprintln!("DRAW: AA1::new interp vertical\n");
            }
        } else {
            while li.step_hor(self) {
                //eprintln!("DRAW: AA1::new interp horizontal\n");
            }
        }
    }
    fn line2_no_clip(&mut self, lp: &LineParameters, ex: i64, ey: i64) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1,lp2) = lp.divide();
            self.line2_no_clip(&lp1, lp1.x2 + (lp1.y2 - lp1.y1), lp1.y2 - (lp1.x2 - lp1.x1));
            self.line2_no_clip(&lp2, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }
        //eprintln!("DRAW: AA2::new line2_no_clip\n");
        let (ex, ey) = lp.fix_degenerate_bisectrix_end(ex, ey);
        let mut li = lp.interp2(ex, ey, self.subpixel_width());
        if li.vertical() {
            while li.step_ver(self) {
                //eprintln!("DRAW: AA2::new interp vertical\n");
            }
        } else {
            while li.step_hor(self) {
                //eprintln!("DRAW: AA2::new interp horizontal\n");
            }
        }
    }
    fn line3_no_clip(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            let mx = lp1.x2 + (lp1.y2 - lp1.y1);
            let my = lp1.y2 - (lp1.x2 - lp1.x1);
            self.line3_no_clip(&lp1, (lp.x1 + sx) >> 1, (lp.y1 + sy) >> 1, mx, my);
            self.line3_no_clip(&lp2, mx, my, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }
        //eprintln!("DRAW: AA3::new line3_no_clip\n");
        let (sx, sy) = lp.fix_degenerate_bisectrix_start(sx, sy);
        let (ex, ey) = lp.fix_degenerate_bisectrix_end(ex, ey);
        let mut li = lp.interp3(sx, sy, ex, ey, self.subpixel_width());
        if li.vertical() {
            while li.step_ver(self) {
                //eprintln!("DRAW: AA3::new interp vertical\n");
            }
        } else {
            while li.step_hor(self) {
                //eprintln!("DRAW: AA3::new interp horizontal\n");
            }
        }
    }

    fn semidot_hline<F>(&mut self, cmp: F, xc1: i64, yc1: i64, xc2: i64, yc2: i64, x1: i64, y1: i64, x2: i64)
    where F: Fn(i64) -> bool
    {
        let mut x1 = x1;
        let mut covers = [0u64; MAX_HALF_WIDTH * 2 + 4];
        let p0 = 0;
        let mut p1 = 0;
        let mut x = x1 << POLY_SUBPIXEL_SHIFT;
        let mut y = y1 << POLY_SUBPIXEL_SHIFT;
        let w = self.subpixel_width();

        let mut di = DistanceInterpolator0::new(xc1, yc1, xc2, yc2, x, y);
        x += POLY_SUBPIXEL_SCALE/2;
        y += POLY_SUBPIXEL_SCALE/2;

        let x0 = x1;
        let mut dx = x - xc1;
        let dy = y - yc1;
        loop {
            let d = ((dx*dx + dy*dy) as f64).sqrt() as i64;
            covers[p1] = 0;
            if cmp(di.dist) && d <= w {
                covers[p1] = self.cover(d);
            }
            p1 += 1;
            dx += POLY_SUBPIXEL_SCALE;
            di.inc_x();
            x1 += 1;
            if x1 > x2 {
                break;
            }
        }
        self.ren.blend_solid_hspan(x0, y1,
                                   (p1 - p0) as i64,
                                   self.color,
                                   &covers);
    }

    fn pie_hline(&mut self, xc: i64, yc: i64, xp1: i64, yp1: i64, xp2: i64, yp2: i64, xh1: i64, yh1: i64, xh2: i64) {
        if let Some(clip_box) = self.clip_box {
            if clip_box.clip_flags(xc, yc) != 0 {
                return;
            }
        }
        let mut xh1 = xh1;
        let mut covers = [0u64; MAX_HALF_WIDTH * 2 + 4];

        let p0 = 0;
        let mut p1 = 0;
        let mut x = xh1 << POLY_SUBPIXEL_SHIFT;
        let mut y = yh1 << POLY_SUBPIXEL_SHIFT;
        let w = self.subpixel_width();

        let mut di = DistanceInterpolator00::new(xc, yc, xp1, yp1, xp2, yp2,
                                                 x, y);
        x += POLY_SUBPIXEL_SCALE/2;
        y += POLY_SUBPIXEL_SCALE/2;

        let xh0 = xh1;
        let mut dx = x - xc;
        let dy = y - yc;
        loop {
            let d = ((dx*dx + dy*dy) as f64).sqrt() as i64;
            //eprintln!("PIE HLINE: {:6},{:6}({:4} {:4}) => {:4}", xc,yc, dx, dy, d);
            covers[p1] = 0;
            if di.dist1 <= 0 && di.dist2 > 0 && d <= w {
                covers[p1] = self.cover(d);
            }
            p1 += 1;
            dx += POLY_SUBPIXEL_SCALE;
            di.inc_x();
            xh1 += 1;
            if xh1 > xh2 {
                break;
            }
        }
        self.ren.blend_solid_hspan(xh0, yh1,
                                   (p1 - p0) as i64,
                                   self.color,
                                   &covers);

    }

}

impl<T> RenderOutline for RendererOutlineAA<'_, T> where T: Pixel {
    fn cover(&self, d: i64) -> u64 {
        let subpixel_shift = POLY_SUBPIXEL_SHIFT;
        let subpixel_scale = 1 << subpixel_shift;
        let index = d + i64::from(subpixel_scale) * 2;
        assert!(index >= 0);
        println!("index {} profile {}", index, self.profile.profile.len());
        //eprintln!("COVER: {}", self.profile.profile[index as usize] as u64);

        u64::from( self.profile.profile[index as usize] )
    }
    fn blend_solid_hspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]) {
        //eprintln!("DRAW: RO::blend_solid_hspan() x,y {} {} len {} covers.len {}", x, y, len, covers.len() );
        self.ren.blend_solid_hspan(x, y, len,  self.color, covers);
    }
    fn blend_solid_vspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]) {
        //eprintln!("DRAW: RO::blend_solid_vspan() x,y {} {} len {} covers.len {}", x, y, len, covers.len() );
        self.ren.blend_solid_vspan(x, y, len,  self.color, covers);
    }
}

impl<T> Lines for RendererOutlineAA<'_, T> where T: Pixel {
    fn line3(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64) {
        //eprintln!("DRAW: line3() {:?}", lp);
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1, lp.y1, lp.x2, lp.y2, clip_box);
            if (flags & 4) == 0 {
                let (mut sx, mut sy, mut ex, mut ey) = (sx,sy,ex,ey);
                if flags != 0{
                    let lp2 = LineParameters::new(x1,y1,x2,y2,
                                                  len_i64_xy(x1, y1, x2, y2));
                    if flags & 1 != 0{
                        sx = x1 + (y2 - y1);
                        sy = y1 - (x2 - x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy - lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    if flags & 2 != 0{
                        ex = x2 + (y2 - y1);
                        ey = y2 - (x2 - x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line3_no_clip(&lp2, sx, sy, ex, ey);
                } else {
                    self.line3_no_clip(&lp, sx, sy, ex, ey);
                }
            }
        } else {
            self.line3_no_clip(&lp, sx, sy, ex, ey);
        }
    }
    fn semidot<F>(&mut self, cmp: F, xc1: i64, yc1: i64, xc2: i64, yc2: i64)
    where F: Fn(i64) -> bool
    {
        if let Some(clip_box) = self.clip_box {
            if clip_box.clip_flags(xc1, yc1) != 0 {
                return;
            }
        }

        let mut r = (self.subpixel_width() + POLY_SUBPIXEL_MASK) >> POLY_SUBPIXEL_SHIFT;
        if r < 1 {
            r = 1;
        }
        let mut ei = EllipseInterpolator::new(r, r);
        let mut dx = 0;
        let mut dy = -r;
        let mut dy0 = dy;
        let mut dx0 = dx;
        let x = xc1 >> POLY_SUBPIXEL_SHIFT;
        let y = yc1 >> POLY_SUBPIXEL_SHIFT;

        loop {
            dx += ei.dx;
            dy += ei.dy;

            if dy != dy0 {
                self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x-dx0, y+dy0, x+dx0);
                self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x-dx0, y-dy0, x+dx0);
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
        self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x-dx0, y+dy0, x+dx0);
    }

    fn pie(&mut self, xc: i64, yc: i64, x1: i64, y1: i64, x2: i64, y2: i64) {
        let mut r = (self.subpixel_width() + POLY_SUBPIXEL_MASK) >> POLY_SUBPIXEL_SHIFT;
        if r < 1 {
            r = 1;
        }
        let mut ei = EllipseInterpolator::new(r, r);
        let mut dx = 0;
        let mut dy = -r;
        let mut dy0 = dy;
        let mut dx0 = dx;
        let x = xc >> POLY_SUBPIXEL_SHIFT;
        let y = yc >> POLY_SUBPIXEL_SHIFT;

        loop {
            dx += ei.dx;
            dy += ei.dy;

            if dy != dy0 {
                self.pie_hline(xc, yc, x1, y1, x2, y2, x-dx0, y+dy0, x+dx0);
                self.pie_hline(xc, yc, x1, y1, x2, y2, x-dx0, y-dy0, x+dx0);
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
        self.pie_hline(xc, yc, x1, y1, x2, y2, x-dx0, y+dy0, x+dx0);

    }
    fn line0(&mut self, lp: &LineParameters) {
        //eprintln!("DRAW: line0() {:?}", lp);
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1,lp.y1,lp.x2,lp.y2,clip_box);
            if flags & 4 == 0 { // Visible
                if flags != 0 { // Not Clipped
                    let lp2 = LineParameters::new(x1, y1, x2, y2,
                                                  len_i64_xy(x1,y1,x2,y2));
                    self.line0_no_clip(&lp2);
                } else {
                    self.line0_no_clip(&lp)
                }
            }
        } else {
            self.line0_no_clip(&lp);
        }
    }
    fn line1(&mut self, lp: &LineParameters, sx: i64, sy: i64) {
        //eprintln!("DRAW: line1() {} {} {} {}", lp.x1, lp.y1, lp.x2, lp.y2);
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1,lp.y1,lp.x2,lp.y2, clip_box);
            if flags & 4 == 0 {
                if flags != 0{
                    let (mut sx, mut sy) = (sx,sy);
                    let lp2 = LineParameters::new(x1,y1,x2,y2, len_i64_xy(x1,y1,x2,y2));
                    if flags & 1 == 0 {
                        sx = x1 + (y2-y1);
                        sy = y1 - (x2-x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy-lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    self.line1_no_clip(&lp2, sx, sy);
                } else {
                    self.line1_no_clip(&lp, sx, sy);
                }
            }
        } else {
            self.line1_no_clip(&lp, sx, sy);
        }
    }
    fn line2(&mut self, lp: &LineParameters, ex: i64, ey: i64) {
        //eprintln!("DRAW: line2() {:?}", lp);
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1,lp.y1,lp.x2,lp.y2, clip_box);
            if flags & 4 == 0 {
                if flags != 0 {
                    let (mut ex,mut ey) = (ex,ey);
                    let lp2 = LineParameters::new(x1,y1,x2,y2, len_i64_xy(x1,y1,x2,y2));
                    if flags & 2 != 0{
                        ex = x2 + (y2-y1);
                        ey = y2 + (x2-x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line2_no_clip(&lp2, ex, ey);
                } else {
                    self.line2_no_clip(&lp, ex, ey);
                }
            }
        } else {
            self.line2_no_clip(&lp, ex, ey);
        }
    }

}

impl<T> SetColor for RendererOutlineAA<'_, T> where T: Pixel {
    fn color<C: Color>(&mut self, color: C) {
        self.color = Rgba8::from_trait(color);
    }
}
impl<T> AccurateJoins for RendererOutlineAA<'_, T> where T: Pixel {
    fn accurate_join_only(&self) -> bool{
        false
    }
}

fn clip_line_segment(x1: i64, y1: i64, x2: i64, y2: i64, clip_box: Rectangle<i64>) -> (i64, i64, i64, i64, u8) {
    let f1 = clip_box.clip_flags(x1,y1);
    let f2 = clip_box.clip_flags(x2,y2);
    let mut ret = 0;
    if f1 == INSIDE && f2 == INSIDE {
        return (x1,y1,x2,y2,0);
    }
    let x_side = LEFT | RIGHT;
    let y_side = TOP  | BOTTOM;
    if f1 & x_side != 0 && f1 & x_side == f2 & x_side {
        return (x1,y1,x2,y2,4); // Outside
    }
    if f1 & y_side != 0 && f1 & y_side == f2 & y_side {
        return (x1,y1,x2,y2,4); // Outside
    }
    let (mut x1, mut y1) = (x1,y1);
    let (mut x2, mut y2) = (x2,y2);
    if f1 != 0 {
        if let Some((x,y)) = clip_move_point(x1, y1, x2, y2, clip_box, x1, y1, f1) {
            x1 = x;
            y1 = y;
        } else {
            return (x1,y1,x2,y2,4);
        }
        if x1 == x2 && y1 == y2 {
            return (x1,y1,x2,y2,4);
        }
        ret |= 1;
    }
    if f2 != 0 {
        if let Some((x,y)) = clip_move_point(x1, y1, x2, y2, clip_box, x2, y2, f2) {
            x2 = x;
            y2 = y;
        } else {
            return (x1,y1,x2,y2,4);
        }
        if x1 == x2 && y1 == y2 {
            return (x1,y1,x2,y2,4);
        }
        ret |= 2;
    }
    (x1,y1,x2,y2,ret)
}

fn clip_move_point(x1: i64, y1: i64, x2: i64, y2: i64, clip_box: Rectangle<i64>, x: i64, y: i64, flags: u8) -> Option<(i64,i64)>{
    let (mut x, mut y) = (x,y);
    if flags & (LEFT | RIGHT) != 0 {
        if x1 == x2 {
            return None;
        } else {
            let x = if flags & LEFT != 0 { clip_box.x1 } else { clip_box.x2 };
            y = ((x - x1) as f64  * (y2-y1) as f64 / (x2-x1) as f64 + y1 as f64) as i64;
        }
    }
    let flags = clip_box.clip_flags(x,y);
    if flags & (TOP | BOTTOM) != 0 {
        if y1 == y2 {
            return None;
        } else {
            let y = if flags & BOTTOM != 0 { clip_box.y1 } else { clip_box.y2 };
            x = ((y - y1) as f64 * (x2-x1) as f64 / (y2-y1) as f64 + x1 as f64) as i64;
        }
    }
    Some((x,y))
}

#[derive(Debug)]
struct EllipseInterpolator {
    rx2: i64,
    ry2: i64,
    two_rx2: i64,
    two_ry2: i64,
    dx: i64,
    dy: i64,
    inc_x: i64,
    inc_y: i64,
    cur_f: i64,
}

impl EllipseInterpolator {
    pub fn new(rx: i64, ry: i64) -> Self {
        let rx2 = rx * rx;
        let ry2 = ry * ry;
        let two_rx2 = rx2 * 2;
        let two_ry2 = ry2 * 2;
        let dx = 0;
        let dy = 0;
        let inc_x = 0;
        let inc_y = -ry * two_rx2;
        let cur_f = 0;

        Self { rx2, ry2, two_rx2, two_ry2, dx, dy, inc_x, inc_y, cur_f }
    }
    pub fn inc(&mut self) {
        let mut mx = self.cur_f + self.inc_x + self.ry2;
        let fx = mx;
        if mx < 0 {
            mx = -mx;
        }

        let mut my = self.cur_f + self.inc_y + self.rx2;
        let fy = my;
        if my < 0  {
            my = -my;
        }

        let mut mxy = self.cur_f + self.inc_x + self.ry2 + self.inc_y + self.rx2;
        let fxy = mxy;
        if mxy < 0 {
            mxy = -mxy;
        }

        let mut min_m = mx;

        let flag = if min_m > my {
            min_m = my;
            false
        } else {
            true
        };

        self.dx = 0;
        self.dy = 0;
        if min_m > mxy {
            self.inc_x += self.two_ry2;
            self.inc_y += self.two_rx2;
            self.cur_f = fxy;
            self.dx = 1;
            self.dy = 1;
            return;
        }

        if flag {
            self.inc_x += self.two_ry2;
            self.cur_f = fx;
            self.dx = 1;
            return;
        }

        self.inc_y += self.two_rx2;
        self.cur_f = fy;
        self.dy = 1;

    }
}


#[derive(Debug)]
pub struct RendererOutlineImg<'a,T> {
    ren: &'a mut RenderingBase<T>,
    pattern: LineImagePatternPow2,
    start: i64,
    scale_x: f64,
    clip_box: Option<Rectangle<i64>>,
}
impl<T> AccurateJoins for RendererOutlineImg<'_, T>  {
    fn accurate_join_only(&self) -> bool{
        true
    }
}

impl<'a,T> DrawOutline for RendererOutlineImg<'a, T> where T: Pixel {}

impl<'a,T> RendererOutlineImg<'a,T> where T: Pixel {
    pub fn with_base_and_pattern(ren: &'a mut RenderingBase<T>, pattern: LineImagePatternPow2) -> Self {
        Self { ren, pattern, start: 0, scale_x: 1.0, clip_box: None  }
    }
    pub fn scale_x(&mut self, scale_x: f64) {
        self.scale_x = scale_x;
    }
    pub fn start_x(&mut self, s: f64) {
        self.start = (s * POLY_SUBPIXEL_SCALE as f64).round() as i64;
    }
    fn subpixel_width(&self) -> i64 {
        self.pattern.line_width()
    }
    fn pattern_width(&self) -> i64 {
        self.pattern.pattern_width()
    }
    // fn width(&self) -> f64 {
    //     self.subpixel_width() as f64 / POLY_SUBPIXEL_SCALE as f64
    // }
    fn pixel(&mut self, x: i64, y: i64) -> Rgba8 {
        //eprintln!("PIXEL {} {}", x, y);
        self.pattern.pixel(x, y)
    }
    fn blend_color_hspan(&mut self, x: i64, y: i64, len: i64, colors: &[Rgba8]) {
        //eprintln!("LENGTH COLORS {}", colors.len());
        //assert_eq!(len as usize, colors.len());
        //for (i,color) in colors.iter().enumerate() {
        self.ren.blend_color_hspan(x, y, len, colors, &[], 255);
        //}
    }
    fn blend_color_vspan(&mut self, x: i64, y: i64, len: i64, colors: &[Rgba8]) {
        //eprintln!("LENGTH COLORS {}", colors.len());
        assert_eq!(len as usize, colors.len());
        self.ren.blend_color_vspan(x, y, len, colors, &[], 255);
        //for (i,color) in colors.iter().enumerate() {
        //    self.ren.blend_solid_hspan(x, y+i as i64, 1, color, &[255]);
        //}
    }
    fn line3_no_clip(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            let mx = lp1.x2 + (lp1.y2 - lp1.y1);
            let my = lp1.y2 - (lp1.x2 - lp1.x1);
            self.line3_no_clip(&lp1, (lp.x1 + sx) >> 1, (lp.y1 + sy) >> 1, mx, my);
            self.line3_no_clip(&lp2, mx, my, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }
        //eprintln!("LINE3: {} {} {} {}", sx, sy, ex, ey);
        let (sx, sy) = lp.fix_degenerate_bisectrix_start(sx, sy);
        let (ex, ey) = lp.fix_degenerate_bisectrix_end(ex, ey);
        //eprintln!("LINE3: {} {} {} {}", sx, sy, ex, ey);
        let mut li = lp.interp_image(sx, sy, ex, ey,
                                 self.subpixel_width(),
                                 self.start,
                                 self.pattern_width(),
                                 self.scale_x);
        if li.vertical() {
            while li.step_ver(self) {}
        } else {
            while li.step_hor(self) {}
        }
        self.start += (lp.len as f64/ self.scale_x).round() as i64;
    }
}
impl<T> SetColor for RendererOutlineImg<'_, T> where T: Pixel {
    fn color<C: Color>(&mut self, _color: C) {
        unimplemented!("no color for outline img");
    }
}
impl<T> Lines for RendererOutlineImg<'_, T> where T: Pixel {
    fn line0(&mut self, _lp: &LineParameters) {
    }
    fn line1(&mut self, _lp: &LineParameters, _sx: i64, _sy: i64) {
    }
    fn line2(&mut self, _lp: &LineParameters, _ex: i64, _ey: i64) {
    }
    fn line3(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64) {
        if let Some(clip_box) = self.clip_box {
            let x1 = lp.x1;
            let y1 = lp.y1;
            let x2 = lp.x2;
            let y2 = lp.y2;
            let (x1,y1,x2,y2,flags) = clip_line_segment(x1, y1, x2, y2, clip_box);
            let start = self.start;
            let (mut sx, mut sy, mut ex, mut ey) = (sx,sy,ex,ey);
            if (flags & 4) == 0 {
                if flags != 0 {
                    let lp2 = LineParameters::new(x1, y1, x2, y2,
                                                  len_i64_xy(x1, y1, x2, y2));
                    if flags & 1 != 0 {
                        self.start += (len_i64_xy(lp.x1, lp.y1, x1, y1) as f64 / self.scale_x as f64).round() as i64;
                        sx = x1 + (y2 - y1);
                        sy = y1 - (x2 - x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy - lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    if flags & 2 != 0{
                        ex = x2 + (y2 - y1);
                        ey = y2 - (x2 - x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line3_no_clip(&lp2, sx, sy, ex, ey);
                } else {
                    self.line3_no_clip(lp, sx, sy, ex, ey);
                }
            }
            self.start = start + (lp.len as f64 / self.scale_x as f64).round() as i64;
        } else {
            //eprintln!("LINE3: {} {} {} {}", sx, sy, ex, ey);
            self.line3_no_clip(lp, sx, sy, ex, ey);
        }
    }
    fn semidot<F>(&mut self, _cmp: F, _xc1: i64, _yc1: i64, _xc2: i64, _yc2: i64) where F: Fn(i64) -> bool {
    }
    fn pie(&mut self, _xc: i64, _y: i64, _x1: i64, _y1: i64, _x2: i64, _y2: i64) {
    }
}

#[derive(Debug)]
pub struct LineImagePattern {
    pix: Pixfmt<Rgba8>,
    filter: PatternFilterBilinear,
    dilation: u64,
    dilation_hr: i64,
    //data: Vec<u8>,
    width: u64,
    height: u64,
    width_hr: i64,
    half_height_hr: i64,
    offset_y_hr: i64,
}

impl LineImagePattern {
    pub fn new(filter: PatternFilterBilinear) -> Self {
        let dilation = filter.dilation() + 1;
        let dilation_hr = (dilation as i64) << POLY_SUBPIXEL_SHIFT;
        Self { filter, dilation, dilation_hr,
               width: 0, height: 0, width_hr: 0,
               half_height_hr: 0, offset_y_hr: 0,
               pix: Pixfmt::new(1,1)
        }
    }
    pub fn create<T>(&mut self, src: &T) where T: Source + Pixel {
        self.height = src.height() as u64;
        self.width  = src.width() as u64;
        self.width_hr = src.width() as i64 * POLY_SUBPIXEL_SCALE;
        self.half_height_hr = src.height() as i64 * POLY_SUBPIXEL_SCALE/2;
        self.offset_y_hr = self.dilation_hr + self.half_height_hr - POLY_SUBPIXEL_SCALE/2;
        self.half_height_hr += POLY_SUBPIXEL_SCALE/2;

        self.pix = Pixfmt::<Rgba8>::new((self.width  + self.dilation * 2) as usize,
                                        (self.height + self.dilation * 2) as usize);
        // Resize and attach input data, hmmmm
        /*
        self.data.resize((self.width + self.dilation * 2) *
                         (self.height + self.dilation * 2));

        self.buf.attach(&self.data[0], self.width  + self.dilation * 2,
                        self.height + self.dilation * 2,
                        self.width  + self.dilation * 2);
         */
        //eprintln!("src {} {} {}", src.rbuf.width, src.rbuf.height, src.rbuf.data.len());
        //eprintln!("dst {} {} {}", self.pix.rbuf.width, self.pix.rbuf.height, self.pix.rbuf.data.len());
        for y in 0 .. self.height as usize {
            //d1 = self.buf.row_ptr(y + self.dilation) + self.dilation;
            let x1 = self.dilation as usize;
            let y1 = y + self.dilation as usize;
            for x in 0 .. self.width as usize {
                /*eprintln!("copy {} {} ({},{}) => {} {} ({},{})",
                          x,y,
                          src.rbuf.width, src.rbuf.height,
                          x1+x,y1,
                          self.pix.rbuf.width, self.pix.rbuf.height);
                 */
                self.pix.set((x1+x,y1), src.get((x,y)));
                //*d1++ = src.pixel(x, y);
            }
        }
        //const color_type* s1;
        //const color_type* s2;
        let none = Rgba8::new(0,0,0,0);
        let dill = self.dilation as usize;
        for y in 0 .. dill {
            //s1 = self.buf.row_ptr(self.height + self.dilation - 1) + self.dilation;
            //s2 = self.buf.row_ptr(self.dilation) + self.dilation;
            //let d1 = self.buf.row_ptr(self.dilation + self.height + y) + self.dilation;
            //let d2 = self.buf.row_ptr(self.dilation - y - 1) + self.dilation;
            let (x1,y1) = (dill, dill + y + self.height as usize);
            let (x2,y2) = (dill, dill - y - 1);
            for x in 0 .. self.width as usize{
                //*d1++ = color_type(*s1++, 0);
                //*d2++ = color_type(*s2++, 0);
                //*d1++ = color_type::no_color();
                //*d2++ = color_type::no_color();
                self.pix.set((x1+x,y1), none);
                self.pix.set((x2+x,y2), none);
            }
        }
        let h = self.height + self.dilation * 2;
        for y in  0 .. h as usize {
            let sx1 = self.dilation as usize;
            let sx2 = (self.dilation + self.width) as usize;
            let dx1 = sx2;
            let dx2 = sx1;
            //s1 = self.buf.row_ptr(y) + self.dilation;
            //s2 = self.buf.row_ptr(y) + self.dilation + self.width;
            //d1 = self.buf.row_ptr(y) + self.dilation + self.width;
            //d2 = self.buf.row_ptr(y) + self.dilation;

            for x in 0 .. self.dilation as usize {
                //*d1++ = *s1++;
                //*--d2 = *--s2;
                self.pix.set((dx1 + x,y), self.pix.get((sx1 + x,y)));
                self.pix.set((dx2 - x - 1,y), self.pix.get((sx2 - x - 1,y)));
            }
        }
    }
    pub fn pattern_width(&self) -> i64 {
        self.width_hr
    }
    pub fn line_width(&self) -> i64 {
        self.half_height_hr
    }
    pub fn width(&self) -> u64 {
        self.height
    }
}

#[derive(Debug)]
pub struct LineImagePatternPow2 {
    base: LineImagePattern,
    mask: u64
}

impl LineImagePatternPow2 {
    pub fn new(filter: PatternFilterBilinear) -> Self {
        let base = LineImagePattern::new( filter );
        Self { base, mask: POLY_SUBPIXEL_MASK as u64}
    }
    pub fn create<T>(&mut self, src: &T) where T: Source + Pixel {
        self.base.create(src);
        self.mask = 1;
        while self.mask < self.base.width {
            self.mask <<= 1;
            self.mask |= 1;
        }
        self.mask <<= POLY_SUBPIXEL_SHIFT - 1;
        self.mask |=  POLY_SUBPIXEL_MASK as u64 ;
        self.base.width_hr = (self.mask + 1) as i64;
    }
    pub fn pattern_width(&self) -> i64 {
        self.base.width_hr
    }
    pub fn line_width(&self) -> i64 {
        self.base.half_height_hr
    }
    pub fn width(&self) -> u64 {
        self.base.height
    }
    pub fn pixel(&self, x: i64, y: i64) -> Rgba8 {
        //eprintln!("PIXEL {} {}", x, y);
        self.base.filter.pixel_high_res(&self.base.pix,
                                        (x & self.mask as i64) + self.base.dilation_hr,
                                        y + self.base.offset_y_hr)
    }

}

#[derive(Debug,Default)]
pub struct PatternFilterBilinear();


impl PatternFilterBilinear {
    pub fn new() -> Self {
        Self{ }
    }
    pub fn dilation(&self) -> u64 {
        1
    }
    pub fn pixel_low_res(&self, pix: &Pixfmt<Rgba8>, x: i64, y: i64) -> Rgba8
    {
        pix.get((x as usize, y as usize))
    }
    pub fn pixel_high_res(&self, pix: &Pixfmt<Rgba8>, x: i64, y: i64) -> Rgba8 {

        //eprintln!("PIXEL HIGH RES: {:6} {:6}", x, y);
        let (mut red, mut green, mut blue, mut alpha) = (0i64, 0i64, 0i64, 0i64);

        let x_lr = (x as usize) >> POLY_SUBPIXEL_SHIFT;
        let y_lr = (y as usize) >> POLY_SUBPIXEL_SHIFT;

        let x = x & POLY_SUBPIXEL_MASK;
        let y = y & POLY_SUBPIXEL_MASK;

        let ptr = pix.get((x_lr,y_lr));

        let weight = (POLY_SUBPIXEL_SCALE - x) * (POLY_SUBPIXEL_SCALE - y);
        red   += weight * i64::from(ptr.r);
        green += weight * i64::from(ptr.g);
        blue  += weight * i64::from(ptr.b);
        alpha += weight * i64::from(ptr.a);
        //eprintln!("PIXEL HIGH RES: {:7} {:7} {:7} {:7} w {:7} p {:7} {:7} {:7} {:7} xy {:4} {:4}", r,g,b,a, weight, ptr.r, ptr.g, ptr.b, ptr.a, x_lr,y_lr);
        let ptr = pix.get((x_lr + 1,y_lr));
        let weight = x * (POLY_SUBPIXEL_SCALE - y);
        red   += weight * i64::from(ptr.r);
        green += weight * i64::from(ptr.g);
        blue  += weight * i64::from(ptr.b);
        alpha += weight * i64::from(ptr.a);
        //eprintln!("PIXEL HIGH RES: {:7} {:7} {:7} {:7} w {:7} p {:7} {:7} {:7} {:7} xy {:4} {:4}", r,g,b,a,weight, ptr.r, ptr.g, ptr.b, ptr.a,x_lr+1,y_lr);
        let ptr = pix.get((x_lr,y_lr+1));
        let weight = (POLY_SUBPIXEL_SCALE - x) * y;
        red   += weight * i64::from(ptr.r);
        green += weight * i64::from(ptr.g);
        blue  += weight * i64::from(ptr.b);
        alpha += weight * i64::from(ptr.a);
        //eprintln!("PIXEL HIGH RES: {:7} {:7} {:7} {:7} w {:7} p {:7} {:7} {:7} {:7} xy {:4} {:4}", r,g,b,a, weight, ptr.r, ptr.g, ptr.b, ptr.a, x_lr, y_lr+1);
        let ptr = pix.get((x_lr+1,y_lr+1));
        let weight = x * y;
        red   += weight * i64::from(ptr.r);
        green += weight * i64::from(ptr.g);
        blue  += weight * i64::from(ptr.b);
        alpha += weight * i64::from(ptr.a);
        //eprintln!("PIXEL HIGH RES: {:7} {:7} {:7} {:7} w {:7} p {:7} {:7} {:7} {:7} xy {:4} {:4}", r,g,b,a, weight, ptr.r, ptr.g, ptr.b, ptr.a, x_lr+1,y_lr+1);
        let red   = (red   >> (POLY_SUBPIXEL_SHIFT * 2)) as u8;
        let green = (green >> (POLY_SUBPIXEL_SHIFT * 2)) as u8;
        let blue  = (blue  >> (POLY_SUBPIXEL_SHIFT * 2)) as u8;
        let alpha = (alpha >> (POLY_SUBPIXEL_SHIFT * 2)) as u8;
        //eprintln!("PIXEL HIGH RES: {:7} {:7} {:7} {:7}", r,g,b,a);
        Rgba8::new(red,green,blue,alpha)
    }
}
#[derive(Debug)]
pub struct LineInterpolatorImage {
    lp: LineParameters,
    li: LineInterpolator,
    di: DistanceInterpolator4,
    //pub plen: i64,
    x: i64,
    y: i64,
    old_x: i64,
    old_y: i64,
    count: i64,
    width: i64,
    max_extent: i64,
    start: i64,
    step: i64,
    //pub dist_pos: [i64; MAX_HALF_WIDTH + 1],
    dist_pos: Vec<i64>,
    //pub colors: [Rgba8; MAX_HALF_WIDTH * 2 + 4],
    colors: Vec<Rgba8>,
}

impl LineInterpolatorImage {
    pub fn new(lp: LineParameters,
               sx: i64, sy: i64, ex: i64, ey: i64,
               subpixel_width: i64,
               pattern_start: i64,
               pattern_width: i64,
               scale_x: f64) -> Self {
        let n = if lp.vertical {
            (lp.y2-lp.y1).abs()
        } else {
            (lp.x2-lp.x1).abs() + 1
        };
        let y1 = if lp.vertical {
            (lp.x2-lp.x1) << POLY_SUBPIXEL_SHIFT
        } else {
            (lp.y2-lp.y1) << POLY_SUBPIXEL_SHIFT
        };
        let mut m_li = LineInterpolator::new_back_adjusted_2(y1, n);
        let mut x = lp.x1 >> POLY_SUBPIXEL_SHIFT;
        let mut y = lp.y1 >> POLY_SUBPIXEL_SHIFT;
        let mut old_x = x;
        let mut old_y = y;
        let count = if lp.vertical {
            ((lp.y2 >> POLY_SUBPIXEL_SHIFT) - y).abs()
        } else {
            ((lp.x2 >> POLY_SUBPIXEL_SHIFT) - x).abs()
        };
        let width = subpixel_width;
        let max_extent = (width + POLY_SUBPIXEL_SCALE) >> POLY_SUBPIXEL_SHIFT;
        let mut step = 0;
        let start = pattern_start + (max_extent + 2) * pattern_width;
        let mut dist_pos = vec![0i64; MAX_HALF_WIDTH + 1];
        let colors = vec![Rgba8::black(); MAX_HALF_WIDTH * 2 + 4];
        let mut di = DistanceInterpolator4::new(lp.x1, lp.y1, lp.x2, lp.y2,
                                                sx, sy, ex, ey, lp.len, scale_x,
                                                lp.x1 & ! POLY_SUBPIXEL_MASK,
                                                lp.y1 & ! POLY_SUBPIXEL_MASK);
        //eprintln!("LII: sx,sy,ex,ey {} {} {} {}", sx,sy,ex,ey);
        //eprintln!("LII: WIDTH: {}", width);
        //eprintln!("LII: MAX EXTENT: {}", max_extent);
        //eprintln!("LII: START: {}", start);
        //eprintln!("LII: dist_start {}", di.dist_start);
        let dd = if lp.vertical {
            lp.dy << POLY_SUBPIXEL_SHIFT
        } else {
            lp.dx << POLY_SUBPIXEL_SHIFT
        };
        let mut li = LineInterpolator::new(0, dd, lp.len);

        let stop = width + POLY_SUBPIXEL_SCALE * 2;
        for i in 0 .. MAX_HALF_WIDTH {
            dist_pos[i] = li.y;
            if dist_pos[i] >= stop {
                break;
            }
            li.inc();
        }
        dist_pos[MAX_HALF_WIDTH] = 0x7FFF_0000;

        let mut npix = 1;

        if lp.vertical {
            loop {
                m_li.dec();
                y -= lp.inc;
                x = (lp.x1 + m_li.y) >> POLY_SUBPIXEL_SHIFT;

                if lp.inc > 0 {
                    di.dec_y_by(x - old_x);
                } else {
                    di.inc_y_by(x - old_x);
                }
                //eprintln!("LII: dist_start {}", di.dist_start);

                old_x = x;

                let mut dist1_start = di.dist_start;
                let mut dist2_start = di.dist_start;

                let mut dx = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start += di.dy_start;
                    dist2_start -= di.dy_start;
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dx += 1;
                    if dist_pos[dx] > width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }

                npix = 0;
                step -= 1;
                if step < -max_extent {
                    break;
                }
            }
        } else {
            loop {
                m_li.dec();

                x -= lp.inc;
                y = (lp.y1 + m_li.y) >> POLY_SUBPIXEL_SHIFT;

                if lp.inc > 0 {
                    di.dec_x_by(y - old_y);
                } else {
                    di.inc_x_by(y - old_y);
                }
                //eprintln!("LII: dist_start {}", di.dist_start);

                old_y = y;

                let mut dist1_start = di.dist_start;
                let mut dist2_start = di.dist_start;

                let mut dy = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start -= di.dx_start;
                    dist2_start += di.dx_start;
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dy += 1;
                    if dist_pos[dy] > width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }

                npix = 0;
                step -= 1;
                if step < -max_extent {
                    break;
                }
            }
        }
        m_li.adjust_forward();
        step -= max_extent;
        //eprintln!("LII: dist_start {}", di.dist_start);

        Self {
            lp, x, y, old_x, old_y, count, width, max_extent, step,
            dist_pos, colors, di, start,
            li: m_li,
        }
    }
    pub fn vertical(&self) -> bool {
        self.lp.vertical
    }
    pub fn step_ver<T>(&mut self, ren: &mut RendererOutlineImg<T>) -> bool
    where T: Pixel
    {
        //eprintln!("STEP_VER: di.dist_start {}", self.di.dist_start);
        self.li.inc();
        self.y += self.lp.inc;
        self.x = (self.lp.x1 + self.li.y) >> POLY_SUBPIXEL_SHIFT;

        if self.lp.inc > 0 {
            self.di.inc_y_by(self.x - self.old_x);
        } else {
            self.di.dec_y_by(self.x - self.old_x);
        }
        //eprintln!("STEP_VER: di.dist_start {}", self.di.dist_start);
        self.old_x = self.x;

        let mut s1 = self.di.dist / self.lp.len;
        let s2 = -s1;

        if self.lp.inc > 0 {
            s1 = -s1;
        }

        let mut dist_start = self.di.dist_start;
        let mut dist_pict  = self.di.dist_pict + self.start;
        let mut dist_end   = self.di.dist_end;
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;
        //eprintln!("STEP_VER: dist_pict {} start {} dist_end {} dist_start {}", self.di.dist_pict, self.start, dist_end, dist_start);
        let mut npix = 0;
        self.colors[p1].clear();
        if dist_end > 0 {
            if dist_start <= 0 {
                self.colors[p1] = ren.pixel(dist_pict, s2);
                /*
                eprintln!("STEP_VER: {:4},{:4} c {:4} {:4} {:4} {:4} dist {} s2 {}",
                          self.x, self.y, self.colors[p1].r, self.colors[p1].g,
                          self.colors[p1].b,
                          self.colors[p1].a, dist_pict, s2);
                 */
            }
            npix += 1;
        }
        p1 += 1;

        let mut dx = 1;
        let mut dist = self.dist_pos[dx];
        while dist - s1 <= self.width {
            dist_start += self.di.dy_start;
            dist_pict  += self.di.dy_pict;
            dist_end   += self.di.dy_end;
            self.colors[p1].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.colors[p1] = ren.pixel(dist_pict, s2 + dist);
                /*
                eprintln!("STEP_VER: {:4},{:4} c {:4} {:4} {:4} {:4} dist {} s2 {}",
                          self.x, self.y, self.colors[p1].r, self.colors[p1].g,
                          self.colors[p1].b,
                          self.colors[p1].a, dist_pict, s2+dist);
                 */
                npix += 1;
            }
            p1 += 1;
            dx += 1;
            dist = self.dist_pos[dx];
        }

        dx = 1;
        dist_start = self.di.dist_start;
        dist_pict  = self.di.dist_pict + self.start;
        dist_end   = self.di.dist_end;
        dist = self.dist_pos[dx];
        while dist + s1 <= self.width {
            dist_start -= self.di.dy_start;
            dist_pict  -= self.di.dy_pict;
            dist_end   -= self.di.dy_end;
            p0 -= 1;
            self.colors[p0].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.colors[p0] = ren.pixel(dist_pict, s2 - dist);
                /*
                eprintln!("STEP_VER: {:4},{:4} c {:4} {:4} {:4} {:4} dist {} s2 {}",
                          self.x, self.y, self.colors[p0].r, self.colors[p0].g,
                          self.colors[p0].b,
                          self.colors[p0].a, dist_pict, s2-dist);
                 */
                npix += 1;
            }
            dx += 1;
            dist = self.dist_pos[dx];
        }

        ren.blend_color_hspan(self.x - dx as i64 + 1,
                              self.y,
                              (p1 - p0) as i64,
                              &self.colors[p0..p1]);
        self.step += 1;

        npix != 0 && self.step < self.count

    }
    pub fn step_hor<T>(&mut self, ren: &mut RendererOutlineImg<T>) -> bool
    where T: Pixel
    {
        self.li.inc();
        self.x += self.lp.inc;
        self.y = (self.lp.y1 + self.li.y) >> POLY_SUBPIXEL_SHIFT;

        if self.lp.inc > 0 {
            self.di.inc_x_by(self.y - self.old_y);
        } else {
            self.di.dec_x_by(self.y - self.old_y);
        }

        self.old_y = self.y;

        let mut s1 = self.di.dist / self.lp.len;
        let s2 = -s1;

        if self.lp.inc < 0 {
            s1 = -s1;
        }

        let mut dist_start = self.di.dist_start;
        let mut dist_pict  = self.di.dist_pict + self.start;
        let mut dist_end   = self.di.dist_end;
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        let mut npix = 0;
        self.colors[p1].clear();
        if dist_end > 0 {
            if dist_start <= 0 {
                self.colors[p1] = ren.pixel(dist_pict, s2);
                /*
                eprintln!("STEP_HOR: {:4},{:4} c {:4} {:4} {:4} {:4}",
                          self.x, self.y, self.colors[p1].r, self.colors[p1].g,
                          self.colors[p1].b,
                          self.colors[p1].a);
                 */
            }
            npix += 1;
        }
        p1 += 1;

        let mut dy = 1;
        let mut dist = self.dist_pos[dy];
        while dist - s1 <= self.width {
            dist_start -= self.di.dx_start;
            dist_pict  -= self.di.dx_pict;
            dist_end   -= self.di.dx_end;
            self.colors[p1].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.colors[p1] = ren.pixel(dist_pict, s2 - dist);
                npix += 1;
            }
            p1 += 1;
            dy += 1;
            dist = self.dist_pos[dy];
        }

        dy = 1;
        dist_start = self.di.dist_start;
        dist_pict  = self.di.dist_pict + self.start;
        dist_end   = self.di.dist_end;
        dist = self.dist_pos[dy];
        while dist + s1 <= self.width {
            dist_start += self.di.dx_start;
            dist_pict  += self.di.dx_pict;
            dist_end   += self.di.dx_end;
            p0 -= 1;
            self.colors[p0].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.colors[p0] = ren.pixel(dist_pict, s2 + dist);
                npix += 1;
            }
            dy += 1;
            dist = self.dist_pos[dy];
        }
        ren.blend_color_vspan(self.x,
                              self.y - dy as i64 + 1,
                              (p1 - p0) as i64,
                              &self.colors[p0..p1]);
        self.step += 1;
        npix != 0 && self.step < self.count
    }
}
#[derive(Debug)]
struct DistanceInterpolator4 {
    dx: i64,
    dy: i64,
    dx_start: i64,
    dy_start: i64,
    dx_pict: i64,
    dy_pict: i64,
    dx_end: i64,
    dy_end: i64,
    dist: i64,
    dist_start: i64,
    dist_pict: i64,
    dist_end: i64,
    len: i64,
}

impl DistanceInterpolator4 {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64, sx: i64, sy: i64, ex: i64, ey: i64, len: i64, scale: f64, x: i64, y: i64) -> Self {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dx_start = line_mr(sx) - line_mr(x1);
        let dy_start = line_mr(sy) - line_mr(y1);
        let dx_end = line_mr(ex) - line_mr(x2);
        let dy_end = line_mr(ey) - line_mr(y2);

        let dist = ((x + POLY_SUBPIXEL_SCALE/2 - x2) as f64 * dy as f64 -
                    (y + POLY_SUBPIXEL_SCALE/2 - y2) as f64 * dx as f64).round() as i64;

        let dist_start =
            (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(sx)) * dy_start -
            (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(sy)) * dx_start;
        //eprintln!("TOR4: dist_start {}", dist_start);
        //eprintln!("TOR4: x,y {} {}", x,y);
        //eprintln!("TOR4: sx,sy {} {}", sx,sy);
        //eprintln!("TOR4: dx,dy {} {}", dx_start, dy_start);
        let dist_end = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(ex)) * dy_end -
            (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(ey)) * dx_end;
        let len = (len as f64 / scale).round() as i64;
        let d = len as f64 * scale;
        let tdx = (((x2 - x1) << POLY_SUBPIXEL_SHIFT) as f64 / d).round() as i64;
        let tdy = (((y2 - y1) << POLY_SUBPIXEL_SHIFT) as f64 / d).round() as i64;
        let dx_pict   = -tdy;
        let dy_pict   =  tdx;
        let dist_pict =  ((x + POLY_SUBPIXEL_SCALE/2 - (x1 - tdy)) * dy_pict -
                          (y + POLY_SUBPIXEL_SCALE/2 - (y1 + tdx)) * dx_pict)
            >>  POLY_SUBPIXEL_SHIFT;
        let dx = dx << POLY_SUBPIXEL_SHIFT;
        let dy = dy << POLY_SUBPIXEL_SHIFT;
        let dx_start = dx_start << POLY_MR_SUBPIXEL_SHIFT;
        let dy_start = dy_start << POLY_MR_SUBPIXEL_SHIFT;
        let dx_end = dx_end << POLY_MR_SUBPIXEL_SHIFT;
        let dy_end = dy_end << POLY_MR_SUBPIXEL_SHIFT;

        Self {
            dx, dy, dx_start, dx_end, dy_start, dy_end, dx_pict, dy_pict,
            dist, dist_pict, dist_start, dist_end, len
        }
    }
    // pub fn inc_x(&mut self) {
    //     self.dist += self.dy;
    //     self.dist_start += self.dy_start;
    //     self.dist_pict += self.dy_pict;
    //     self.dist_end += self.dy_end;
    // }
    // pub fn dec_x(&mut self) {
    //     self.dist -= self.dy;
    //     self.dist_start -= self.dy_start;
    //     self.dist_pict -= self.dy_pict;
    //     self.dist_end -= self.dy_end;
    // }
    // pub fn inc_y(&mut self) {
    //     self.dist -= self.dx;
    //     self.dist_start -= self.dx_start;
    //     self.dist_pict -= self.dx_pict;
    //     self.dist_end -= self.dx_end;
    // }
    // pub fn dec_y(&mut self) {
    //     self.dist += self.dx;
    //     self.dist_start += self.dx_start;
    //     self.dist_pict += self.dx_pict;
    //     self.dist_end += self.dx_end;
    // }
    pub fn inc_x_by(&mut self, dy: i64) {
        self.dist       += self.dy;
        self.dist_start += self.dy_start;
        self.dist_pict  += self.dy_pict;
        self.dist_end   += self.dy_end;
        if dy > 0 {
            self.dist       -= self.dx;
            self.dist_start -= self.dx_start;
            self.dist_pict  -= self.dx_pict;
            self.dist_end   -= self.dx_end;
        }
        if dy < 0 {
            self.dist       += self.dx;
            self.dist_start += self.dx_start;
            self.dist_pict  += self.dx_pict;
            self.dist_end   += self.dx_end;
        }
    }
    pub fn dec_x_by(&mut self, dy: i64) {
        self.dist       -= self.dy;
        self.dist_start -= self.dy_start;
        self.dist_pict  -= self.dy_pict;
        self.dist_end   -= self.dy_end;
        if dy > 0 {
            self.dist       -= self.dx;
            self.dist_start -= self.dx_start;
            self.dist_pict  -= self.dx_pict;
            self.dist_end   -= self.dx_end;
        }
        if dy < 0 {
            self.dist       += self.dx;
            self.dist_start += self.dx_start;
            self.dist_pict  += self.dx_pict;
            self.dist_end   += self.dx_end;
        }
    }
    pub fn inc_y_by(&mut self, dx: i64) {
        self.dist       -= self.dx;
        self.dist_start -= self.dx_start;
        self.dist_pict  -= self.dx_pict;
        self.dist_end   -= self.dx_end;
        if dx > 0 {
            self.dist       += self.dy;
            self.dist_start += self.dy_start;
            self.dist_pict  += self.dy_pict;
            self.dist_end   += self.dy_end;
        }
        if dx < 0 {
            self.dist       -= self.dy;
            self.dist_start -= self.dy_start;
            self.dist_pict  -= self.dy_pict;
            self.dist_end   -= self.dy_end;
        }
    }
    pub fn dec_y_by(&mut self, dx: i64) {
        self.dist       += self.dx;
        self.dist_start += self.dx_start;
        self.dist_pict  += self.dx_pict;
        self.dist_end   += self.dx_end;
        if dx > 0 {
            self.dist       += self.dy;
            self.dist_start += self.dy_start;
            self.dist_pict  += self.dy_pict;
            self.dist_end   += self.dy_end;
        }
        if dx < 0 {
            self.dist       -= self.dy;
            self.dist_start -= self.dy_start;
            self.dist_pict  -= self.dy_pict;
            self.dist_end   -= self.dy_end;
        }
    }
}
