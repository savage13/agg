//! Anti Grain Geometry - Rust implementation
//!
//! Originally derived from version 2.4 of [AGG](http://antigrain.com)
//!
//! This crate implments the drawing / painting 2D algorithms developed in the Anti Grain Geometry C++ library. Quoting from the author in the documentation:
//!
//! > **Anti-Grain Geometry** is not a solid graphic library and it's not very easy to use. I consider **AGG** as a **"tool to create other tools"**. It means that there's no **"Graphics"** object or something like that, instead, **AGG** consists of a number of loosely coupled algorithms that can be used together or separately. All of them have well defined interfaces and absolute minimum of implicit or explicit dependencies.
//!
//!
//! # Anti-Aliasing and Subpixel Accuracy
//!
//! One primary strenght of AGG are the combination of drawing with subpixel accuracy with anti-aliasing effects.  There are many examples within the documentation and reproduced here.
//!
//! # Drawing
//!
//! There are multiple ways to put / draw pixels including:
//!
//!   - Scanline Renderers
//!     - Antialiased or Aliased (Binary)
//!   - Outline Renderer, possibly with Images
//!   - Raw Pixel Manipulation
//!
//! # Scanline Renderer
//!
//!  The simplest scanline renderer is [`render`], but others renderers are also
//!    available with more defined capabilities include [`render_scanlines`],
//!    [`render_all_paths`], [`render_scanlines_aa_solid`] and
//!    [`render_scanlines_bin_solid`]
//!
//!        use agg::{Pixfmt,Rgba8,RenderingBase,RasterizerScanline};
//!        let pix = Pixfmt::<Rgba8>::new(10,10);
//!        let mut ren_base = agg::RenderingBase::new(pix);
//!
//!        let mut ras = RasterizerScanline::new();
//!        ras.move_to_d(1.0, 1.0);
//!        ras.line_to_d(5.0, 9.0);
//!        ras.line_to_d(9.0, 1.0);
//!
//!        agg::render(&mut ren_base, &mut ras, true);
//!
//! # Outline AntiAlias Renderer
//!
//!        use agg::{Pixfmt,Rgb8,Rgba8,RenderingBase,SetColor};
//!        use agg::{RendererOutlineAA,RasterizerOutlineAA};
//!        use agg::PixelData;
//!        let pix = Pixfmt::<Rgb8>::new(100,100);
//!        let mut ren_base = agg::RenderingBase::new(pix);
//!        ren_base.clear( Rgba8::new(255, 255, 255, 255) );
//!
//!        let mut ren = RendererOutlineAA::with_base(&mut ren_base);
//!        ren.color(agg::Rgba8::new(102,77,26,255));
//!        ren.profile.width(3.0);
//!
//!        let mut path = agg::PathStorage::new();
//!        path.move_to(10.0, 10.0);
//!        path.line_to(50.0, 90.0);
//!        path.line_to(90.0, 10.0);
//!
//!        let mut ras = RasterizerOutlineAA::with_renderer(&mut ren);
//!        ras.add_path(&path);
//!        agg::ppm::write_ppm(&ren_base.pixeldata(), 100,100,
//!              "outline_aa.ppm").unwrap();
//!
//! # Primative Renderer
//!
//! Render for primative shapes: lines, rectangles, and ellipses; filled or
//!    outlined. 
//!
//!        use agg::{Pixfmt,Rgb8,Rgba8,RenderingBase,SetColor};
//!        use agg::{RendererPrimatives,RasterizerOutline};
//!        use agg::PixelData;
//!        let pix = Pixfmt::<Rgb8>::new(100,100);
//!        let mut ren_base = agg::RenderingBase::new(pix);
//!        ren_base.clear( Rgba8::new(255, 255, 255, 255) );
//!
//!        let mut ren = RendererPrimatives::with_base(&mut ren_base);
//!        ren.line_color(agg::Rgba8::new(0,0,0,255));
//!
//!        let mut path = agg::PathStorage::new();
//!        path.move_to(10.0, 10.0);
//!        path.line_to(50.0, 90.0);
//!        path.line_to(90.0, 10.0);
//!
//!        let mut ras = RasterizerOutline::with_primative(&mut ren);
//!        ras.add_path(&path);
//!        agg::ppm::write_ppm(&ren_base.pixeldata(), 100,100,
//!              "primative.ppm").unwrap();
//!
//!
//! # Raw Pixel Manipulation
//!
//!   **Note:** Functions here are a somewhat low level interface and probably not what
//!     you want to use.
//! 
//!   Functions to set pixel color through [`Pixfmt`] are [`clear`], [`set`], [`copy_pixel`],
//!     [`copy_hline`], [`copy_vline`], [`fill`]
//!
//!   Functions to blend colors with existing pixels through [`Pixfmt`] are [`copy_or_blend_pix`], [`copy_or_blend_pix_with_cover`], [`blend_hline`], [`blend_vline`], [`blend_solid_hspan`], [`blend_solid_vspan`], [`blend_color_hspan`], [`blend_color_vspan`]
//!
//!
//! [`Pixfmt`]: pixfmt/struct.Pixfmt.html
//! [`clear`]: pixfmt/struct.Pixfmt.html#method.clear
//! [`set`]: pixfmt/struct.Pixfmt.html#method.set
//! [`copy_pixel`]: pixfmt/struct.Pixfmt.html#method.copy_pixel
//! [`copy_hline`]: pixfmt/struct.Pixfmt.html#method.copy_hline
//! [`copy_vline`]: pixfmt/struct.Pixfmt.html#method.copy_vline
//! [`fill`]: pixfmt/trait.PixelDraw.html#method.fill
//! [`copy_or_blend_pix`]: pixfmt/trait.PixelDraw.html#method.copy_or_blend_pix
//! [`copy_or_blend_pix_with_cover`]: pixfmt/trait.PixelDraw.html#method.copy_or_blend_pix_with_cover
//! [`blend_hline`]: pixfmt/trait.PixelDraw.html#method.blend_hline
//! [`blend_vline`]: pixfmt/trait.PixelDraw.html#method.blend_vline
//! [`blend_solid_hspan`]: pixfmt/trait.PixelDraw.html#method.blend_solid_hspan
//! [`blend_solid_vspan`]: pixfmt/trait.PixelDraw.html#method.blend_solid_vspan
//! [`blend_color_hspan`]: pixfmt/trait.PixelDraw.html#method.blend_color_hspan
//! [`blend_color_vspan`]: pixfmt/trait.PixelDraw.html#method.blend_color_vspan
use std::fmt::Debug;

pub use freetype as ft;

pub mod path_storage;
pub mod conv_stroke;
pub mod affine_transform;
pub mod color;
pub mod pixfmt;
pub mod buffer;
pub mod base;
pub mod clip;
pub mod cell;
pub mod raster;
pub mod scan;
pub mod ppm;
pub mod alphamask;
pub mod render;
pub mod math;
pub mod text;
pub mod outline;
pub mod outline_aa;
pub mod line_interp;

pub use crate::path_storage::*;
pub use crate::conv_stroke::*;
pub use crate::affine_transform::*;
pub use crate::color::*;
pub use crate::pixfmt::*;
pub use crate::base::*;
pub use crate::clip::*;
pub use crate::cell::*;
pub use crate::raster::*;
pub use crate::alphamask::*;
pub use crate::render::*;
pub use crate::text::*;
pub use crate::line_interp::*;
pub use crate::outline::*;
pub use crate::outline_aa::*;

const POLY_SUBPIXEL_SHIFT : i64 = 8;
const POLY_SUBPIXEL_SCALE : i64 = 1<<POLY_SUBPIXEL_SHIFT;
const POLY_SUBPIXEL_MASK  : i64 = POLY_SUBPIXEL_SCALE - 1;
const POLY_MR_SUBPIXEL_SHIFT : i64 = 4;
const MAX_HALF_WIDTH : usize = 64;


/// Access raw color component data at the pixel level
pub trait PixelData {
    fn pixeldata(&self) -> &[u8];
}
/// Source of vertex points
pub trait VertexSource {
    /// Rewind the vertex source (unused)
    fn rewind(&self) { }
    /// Get values from the source
    ///
    /// This could be turned into an iterator
    fn xconvert(&self) -> Vec<Vertex<f64>>;
}

/// Access Color properties and compoents
pub trait Color: Debug + Copy {
    /// Get red value [0,1] as f64
    fn red(&self) -> f64;
    /// Get green value [0,1] as f64 
    fn green(&self) -> f64;
    /// Get blue value [0,1] as f64
    fn blue(&self) -> f64;
    /// Get alpha value [0,1] as f64
    fn alpha(&self) -> f64;
    /// Get red value [0,255] as u8
    fn red8(&self) -> u8;
    /// Get green value [0,255] as u8
    fn green8(&self) -> u8;
    /// Get blue value [0,255] as u8
    fn blue8(&self) -> u8;
    /// Get alpha value [0,255] as u8
    fn alpha8(&self) -> u8;
    /// Return if the color is completely transparent, alpha = 0.0
    fn is_transparent(&self) -> bool { self.alpha() == 0.0 }
    /// Return if the color is completely opaque, alpha = 1.0
    fn is_opaque(&self) -> bool { self.alpha() >= 1.0 }
    /// Return if the color has been premultiplied
    fn is_premultiplied(&self) -> bool;
}
/// Render scanlines to Image
pub trait Render {
    /// Render a single scanlines to the image
    fn render(&mut self, data: &RenderData);
    /// Set the Color of the Renderer
    fn color<C: Color>(&mut self, color: &C);
    /// Prepare the Renderer
    fn prepare(&self) { }
}
/*
/// Rasterize lines, path, and other things to scanlines
pub trait Rasterize {
    /// Setup Rasterizer, returns if data is available
    fn rewind_scanlines(&mut self) -> bool;
    /// Sweeps cells in a scanline for data, returns if data is available
    fn sweep_scanline(&mut self, sl: &mut ScanlineU8) -> bool;
    /// Return maximum x value of rasterizer
    fn min_x(&self) -> i64;
    /// Return maximum x value of rasterizer
    fn max_x(&self) -> i64;
    /// Resets the rasterizer, clearing content
    fn reset(&mut self);
    /// Rasterize a path 
    fn add_path<VS: VertexSource>(&mut self, path: &VS);
}
*/

pub trait SetColor {
    fn color<C: Color>(&mut self, color: C);
}
pub trait AccurateJoins {
    fn accurate_join_only(&self) -> bool;
}

pub trait Source {
    fn get(&self, id: (usize, usize)) -> Rgba8;
}

pub trait Pixel {
    fn cover_mask() -> u64;
    fn bpp() -> usize;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set<C: Color>(&mut self, id: (usize, usize), c: C);
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64);
}
pub trait PixelDraw: Pixel  {
    /// Fill the data with the specified `color`
    fn fill<C: Color>(&mut self, color: C) {
        let w = self.width();
        let h = self.height();
        for i in 0 .. w {
            for j in 0 .. h {
                self.set((i,j), color);
            }
        }
    }
    /// Copy or blend a pixel at `id` with `color`
    ///
    /// If `color` [`is_opaque`], the color is copied directly to the pixel,
    ///   otherwise the color is blended with the pixel at `id`
    ///
    /// If `color` [`is_transparent`] nothing is done
    ///
    /// [`is_opaque`]: ../trait.Color.html#method.is_opaque
    /// [`is_transparent`]: ../trait.Color.html#method.is_transparent
    fn copy_or_blend_pix<C: Color>(&mut self, id: (usize,usize), color: C) {
        if ! color.is_transparent() {
            if color.is_opaque() {
                self.set(id, color);
            } else {
                self.blend_pix(id, color, 255);
            }
        }
    }
    /// Copy or blend a pixel at `id` with `color` and a `cover`
    ///
    /// If `color` [`is_opaque`] *and* `cover` equals [`cover_mask`] then
    ///   the color is copied to the pixel at `id', otherwise the `color`
    ///   is blended with the pixel at `id' considering the amount of `cover`
    ///
    /// If `color` [`is_transparent`] nothing is done
    ///
    ///     use agg::{Source,Pixfmt,Rgb8,Rgba8,PixelDraw};
    ///
    ///     let mut pix = Pixfmt::<Rgb8>::new(1,1);
    ///     let black  = Rgba8::black();
    ///     let white  = Rgba8::white();
    ///     pix.copy_pixel(0,0,black);
    ///     assert_eq!(pix.get((0,0)), black);
    ///
    ///     let (alpha, cover) = (255, 255); // Copy Pixel
    ///     let color = Rgba8::new(255,255,255,alpha);
    ///     pix.copy_or_blend_pix_with_cover((0,0), color, cover);
    ///     assert_eq!(pix.get((0,0)), white);
    ///
    ///     let (alpha, cover) = (255, 128); // Partial Coverage, Blend
    ///     let color = Rgba8::new(255,255,255,alpha);
    ///     pix.copy_pixel(0,0,black);
    ///     pix.copy_or_blend_pix_with_cover((0,0), color, cover);
    ///     assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));
    ///
    ///     let (alpha, cover) = (128, 255); // Partial Coverage, Blend
    ///     let color = Rgba8::new(255,255,255,alpha);
    ///     pix.copy_pixel(0,0,black);
    ///     pix.copy_or_blend_pix_with_cover((0,0), color, cover);
    ///     assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));
    ///
    /// [`is_opaque`]: ../trait.Color.html#method.is_opaque
    /// [`is_transparent`]: ../trait.Color.html#method.is_transparent
    /// [`cover_mask`]: ../trait.Pixel.html#method.cover_mask
    ///
    fn copy_or_blend_pix_with_cover<C: Color>(&mut self, id: (usize,usize), color: C, cover: u64) {
        if ! color.is_transparent() {
            if color.is_opaque() && cover == Self::cover_mask() {
                self.set(id, color);
            } else {
                self.blend_pix(id, color, cover);
            }
        }
    }
    /// Copy or Blend a single `color` from (`x`,`y`) to (`x+len-1`,`y`)
    ///    with `cover`
    ///
    fn blend_hline<C: Color>(&mut self, x: i64, y: i64, len: i64, color: C, cover: u64) {
        if color.is_transparent() {
            return;
        }
        let (x,y,len) = (x as usize, y as usize, len as usize);
        if color.is_opaque() && cover == Self::cover_mask() {
            for i in 0 .. len {
                self.set((x+i,y),color);
            }
        } else {
            for i in 0 .. len {
                self.blend_pix((x+i,y),color,cover);
            }
        }
    }
    /// Blend a single `color` from (`x`,`y`) to (`x+len-1`,`y`) with collection
    ///   of `covers`
    fn blend_solid_hspan<C: Color>(&mut self, x: i64, y: i64, len: i64, color: C, covers: &[u64]) {
        assert_eq!(len as usize, covers.len());
        for (i, &cover) in covers.iter().enumerate() {
            self.blend_hline(x+i as i64,y,1,color,cover);
        }
    }
    /// Copy or Blend a single `color` from (`x`,`y`) to (`x`,`y+len-1`)
    ///    with `cover`
    ///
    fn blend_vline<C: Color>(&mut self, x: i64, y: i64, len: i64, c: C, cover: u64) {
        if c.is_transparent() {
            return;
        }
        let (x,y,len) = (x as usize, y as usize, len as usize);
        if c.is_opaque() && cover == Self::cover_mask() {
            for i in 0 .. len {
                self.set((x,y+i),c);
            }
        } else {
            for i in 0 .. len {
                self.blend_pix((x,y+i),c,cover);
            }
        }
    }
    /// Blend a single `color` from (`x`,`y`) to (`x`,`y+len-1`) with collection
    ///   of `covers`
    fn blend_solid_vspan<C: Color>(&mut self, x: i64, y: i64, len: i64, c: C, covers: &[u64]){
        assert_eq!(len as usize, covers.len());
        for (i, &cover) in covers.iter().enumerate() {
            self.blend_vline(x,y+i as i64,1,c,cover);
        }
    }
    /// Blend a collection of `colors` from (`x`,`y`) to (`x+len-1`,`y`) with
    ///   either a collection of `covers` or a single `cover`
    ///
    /// A collection of `covers` takes precedance over a single `cover`
    fn blend_color_hspan<C: Color>(&mut self, x: i64, y: i64, len: i64, colors: &[C], covers: &[u64], cover: u64) {

        assert_eq!(len as usize, colors.len());
        let (x,y) = (x as usize, y as usize);
        if ! covers.is_empty() {
            assert_eq!(colors.len(), covers.len());
            for (i,(&color,&cover)) in colors.iter().zip(covers.iter()).enumerate() {
                self.copy_or_blend_pix_with_cover((x+i,y), color, cover);
            }
        } else if cover == 255 {
            for (i,&color) in colors.iter().enumerate() {
                self.copy_or_blend_pix((x+i,y), color);
            }
        } else {
            for (i,&color) in colors.iter().enumerate() {
                self.copy_or_blend_pix_with_cover((x+i,y), color, cover);
            }
        }
    }
    /// Blend a collection of `colors` from (`x`,`y`) to (`x`,`y+len-1`) with
    ///   either a collection of `covers` or a single `cover`
    ///
    /// A collection of `covers` takes precedance over a single `cover`
    fn blend_color_vspan<C: Color>(&mut self, x: i64, y: i64, len: i64, colors: &[C], covers: &[u64], cover: u64) {
        assert_eq!(len as usize, colors.len());
        let (x,y) = (x as usize, y as usize);
        if ! covers.is_empty() {
            assert_eq!(colors.len(), covers.len());
            for (i,(&color,&cover)) in colors.iter().zip(covers.iter()).enumerate() {
                self.copy_or_blend_pix_with_cover((x,y+i), color, cover);
            }
        } else if cover == 255 {
            for (i,&color) in colors.iter().enumerate() {
                self.copy_or_blend_pix((x,y+i), color);
            }
        } else {
            for (i,&color) in colors.iter().enumerate() {
                self.copy_or_blend_pix_with_cover((x,y+i), color, cover);
            }
        }
    }
}

pub trait Lines {
    fn line0(&mut self, lp: &LineParameters);
    fn line1(&mut self, lp: &LineParameters, sx: i64, sy: i64);
    fn line2(&mut self, lp: &LineParameters, ex: i64, ey: i64);
    fn line3(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64);
    fn semidot<F>(&mut self, cmp: F, xc1: i64, yc1: i64, xc2: i64, yc2: i64) where F: Fn(i64) -> bool;
    fn pie(&mut self, xc: i64, y: i64, x1: i64, y1: i64, x2: i64, y2: i64);
}

pub trait LineInterp {
    fn init(&mut self);
    fn step_hor(&mut self);
    fn step_ver(&mut self);
}

pub trait RenderOutline {
    fn cover(&self, d: i64) -> u64;
    fn blend_solid_hspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]);
    fn blend_solid_vspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]);
}

pub trait DistanceInterpolator {
    fn dist(&self) -> i64;
    fn inc_x(&mut self, dy: i64);
    fn inc_y(&mut self, dx: i64);
    fn dec_x(&mut self, dy: i64);
    fn dec_y(&mut self, dx: i64);
}

/// Blend a Foreground, Background and Alpha Components
fn blend(fg: Rgb8, bg: Rgb8, alpha: f64) -> Rgb8 {
    let r = alpha * f64::from(fg.r) + (1.0 - alpha) * f64::from(bg.r);
    let g = alpha * f64::from(fg.g) + (1.0 - alpha) * f64::from(bg.g);
    let b = alpha * f64::from(fg.b) + (1.0 - alpha) * f64::from(bg.b);
    Rgb8::new(r as u8 ,g as u8 ,b as u8)
}

// fn prelerp(a: f64, b: f64, t: f64)  {
//     let (_a,_b,_t) = (a as f64, b as f64, t as f64);
// }


// fn lerp(a: f64, b: f64, t: f64) -> f64{
//     let mut v = (b-a) * t + a;
//     if v < 0.0 {
//         v = 0.0;
//     }
//     if v >= 1.0 {
//         v = 1.0;
//     }
//     v
// }

// fn mult_cover(alpha: f64, cover: f64) -> f64 {
//     alpha * cover
// }

pub fn render<T>(base: &mut RenderingBase<T>, ras: &mut RasterizerScanline, antialias: bool)
      where T: PixelDraw
{
    if antialias {
        let mut ren = RenderingScanlineAASolid::with_base(base);
        render_scanlines(ras, &mut ren);
    } else {
        let mut ren = RenderingScanlineBinSolid::with_base(base);
        render_scanlines(ras, &mut ren);
    };
}


fn string_width(txt: &str, font: &ft::Face) -> f64 {
    let mut width = 0.0;
    for c in txt.chars() {
        let glyph_index = font.get_char_index(c as usize);
        font.load_glyph(glyph_index, ft::face::LoadFlag::DEFAULT).unwrap();
        let glyph = font.glyph();
        glyph.render_glyph(ft::RenderMode::Normal).unwrap();
        let adv = glyph.advance();
        width += adv.x as f64
    }
    width / 64.0
}

pub fn line_height(font: &ft::Face) -> f64 {
    let met = font.size_metrics().unwrap();
    (met.ascender - met.descender) as f64 / 64.0
}

pub fn draw_text<T: PixelDraw>(txt: &str, x: i64, y: i64, font: &ft::Face, ren_base: &mut RenderingBase<T>) {
    let color = Rgba8::new(0,0,0,255);
    let (mut x, mut y) = (x,y);
    let width  = string_width(txt, font);
    let height = line_height(font);
    // Shift to center justification, x and y
    let dx = (width / 2.0).round() as i64;
    let dy = (height / 2.0).round() as i64;
    x -= dx;
    y += dy;
    for c in txt.chars() {
        let glyph_index = font.get_char_index(c as usize);
        font.load_glyph(glyph_index, ft::face::LoadFlag::DEFAULT).unwrap();
        font.glyph().render_glyph(ft::RenderMode::Normal).unwrap();
        let g = font.glyph().bitmap();
        let left = font.glyph().bitmap_left() as i64;
        let top  = font.glyph().bitmap_top() as i64;
        let buf : Vec<_> = g.buffer().iter().map(|&x| x as u64).collect();
        let rows = g.rows() as i64;
        let pitch = g.pitch().abs() as usize;
        let width = g.width() as i64;
        for i in 0 .. rows {
            ren_base.blend_solid_hspan(x + left, y-top+i, width,
                                       color, &buf[pitch*i as usize..]);
        }
        let adv = font.glyph().advance();
        x += (adv.x as f64 / 64.0).round() as i64;
        y += (adv.y as f64 / 64.0).round() as i64;
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum XAlign {
    Left, Center, Right
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum YAlign {
    Top, Center, Bottom
}

pub struct Label<'a> {
    txt: String,
    x: f64,
    y: f64,
    xa: XAlign,
    ya: YAlign,
    color: Rgba8,
    font: &'a ft::Face,
}

impl<'a> Label<'a> {
    pub fn new(txt: &str, x: f64, y: f64, font: &'a ft::Face) -> Self {
        Self {
            txt: txt.to_string(), x, y,
            xa: XAlign::Left,
            ya: YAlign::Bottom,
            color: Rgba8::black(),
            font
        }
    }
    pub fn xalign(mut self, xalign: XAlign) -> Self {
        self.xa = xalign;
        self
    }
    pub fn yalign(mut self, yalign: YAlign) -> Self {
        self.ya = yalign;
        self
    }
    pub fn color(mut self, color: Rgba8) -> Self {
        self.color = color;
        self
    }
    pub fn draw<T: PixelDraw>(&mut self, ren: &mut RenderingBase<T>) {
        draw_text_subpixel(&self.txt, self.x, self.y,
                           self.xa, self.ya, self.color,
                           self.font, ren);
    }
}

// https://www.freetype.org/freetype2/docs/glyphs/glyphs-5.html
// 2. Subpixel positioning
fn draw_text_subpixel<T: PixelDraw>(txt: &str, x: f64, y: f64,
                                    xalign: XAlign,
                                    yalign: YAlign,
                                    color: Rgba8,
                                    font: &ft::Face,
                                    ren_base: &mut RenderingBase<T>) {
    let (mut x, mut y) = (x,y);
    let width  = string_width(txt, font);

    let asc = font.size_metrics().unwrap().ascender as f64 / 64.0;
    x += match xalign {
        XAlign::Left => 0.0,
        XAlign::Right => -width,
        XAlign::Center => -width/2.0,
    };
    y += match yalign {
        YAlign::Top => asc,
        YAlign::Bottom => 0.0,
        YAlign::Center => asc / 2.0,
    };

    for c in txt.chars() {
        let glyph_index = font.get_char_index(c as usize);
        font.load_glyph(glyph_index, ft::face::LoadFlag::DEFAULT).unwrap();

        let glyph = font.glyph().get_glyph().unwrap();
        let dt = ft::Vector {
            x: ((x - x.floor()) * 64.0).round() as i64,
            y: ((y - y.floor()) * 64.0).round() as i64
        };
        glyph.transform(None, Some(dt)).unwrap();
        let g = glyph.to_bitmap(ft::RenderMode::Normal, None).unwrap();
        let left = g.left() as i64;
        let top  = g.top() as i64;
        let bit  = g.bitmap();
        let buf : Vec<_> = bit.buffer().iter().map(|&x| x as u64).collect();
        let rows  = bit.rows() as i64;
        let width = bit.width() as i64;
        let pitch = bit.pitch().abs() as usize;
        for i in 0 .. rows {
            ren_base.blend_solid_hspan(x.floor() as i64 + left,
                                       y.floor() as i64 + i - top,
                                       width,
                                       color, &buf[pitch*i as usize..]);
        }

        x += glyph.advance_x() as f64 / 65536.0;
        y += glyph.advance_y() as f64 / 65536.0;
    }
}
