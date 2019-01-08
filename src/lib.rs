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
//!  The multitude of renderers here include [`render_scanlines`],
//!    [`render_all_paths`], [`render_scanlines_aa_solid`] and
//!    [`render_scanlines_bin_solid`]
//!
//!       use agg::Render;
//!
//!       // Create a blank image 10x10 pixels
//!       let pix = agg::Pixfmt::<agg::Rgb8>::new(100,100);
//!       let mut ren_base = agg::RenderingBase::new(pix);
//!       ren_base.clear(agg::Rgba8::white());
//!
//!       // Draw a polygon from (10,10) - (50,90) - (90,10)
//!       let mut ras = agg::RasterizerScanline::new();
//!       ras.move_to_d(10.0, 10.0);
//!       ras.line_to_d(50.0, 90.0);
//!       ras.line_to_d(90.0, 10.0);

//!       // Render the line to the image
//!       let mut ren = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
//!       ren.color(&agg::Rgba8::black());
//!       agg::render_scanlines(&mut ras, &mut ren);

//!       // Save the image to a file
//!       agg::ppm::write_ppm(&ren_base.as_bytes(), 100,100,
//!                           "little_black_triangle.ppm").unwrap();
//!
//!
//! # Outline AntiAlias Renderer
//!
//!        use agg::{Pixfmt,Rgb8,Rgba8,RenderingBase,DrawOutline};
//!        use agg::{RendererOutlineAA,RasterizerOutlineAA};
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
//!        agg::ppm::write_ppm(&ren_base.as_bytes(), 100,100,
//!              "outline_aa.ppm").unwrap();
//!
//! # Primative Renderer
//!
//! Render for primative shapes: lines, rectangles, and ellipses; filled or
//!    outlined. 
//!
//!        use agg::{Pixfmt,Rgb8,Rgba8,RenderingBase,DrawOutline};
//!        use agg::{RendererPrimatives,RasterizerOutline};
//!
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
//!        agg::ppm::write_ppm(&ren_base.as_bytes(), 100,100,
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

#[doc(hidden)]
pub use freetype as ft;

pub mod path_storage;
pub mod conv_stroke;
pub mod affine_transform;
pub mod color;
pub mod pixfmt;
pub mod base;
pub mod clip;
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

pub(crate) mod buffer;
pub(crate) mod cell;

#[doc(hidden)]
pub use crate::path_storage::*;
#[doc(hidden)]
pub use crate::conv_stroke::*;
#[doc(hidden)]
pub use crate::affine_transform::*;
#[doc(hidden)]
pub use crate::color::*;
#[doc(hidden)]
pub use crate::pixfmt::*;
#[doc(hidden)]
pub use crate::base::*;
#[doc(hidden)]
pub use crate::clip::*;
#[doc(hidden)]
pub use crate::raster::*;
#[doc(hidden)]
pub use crate::alphamask::*;
#[doc(hidden)]
pub use crate::render::*;
#[doc(hidden)]
pub use crate::text::*;
#[doc(hidden)]
pub use crate::line_interp::*;
#[doc(hidden)]
pub use crate::outline::*;
#[doc(hidden)]
pub use crate::outline_aa::*;

const POLY_SUBPIXEL_SHIFT : i64 = 8;
const POLY_SUBPIXEL_SCALE : i64 = 1<<POLY_SUBPIXEL_SHIFT;
const POLY_SUBPIXEL_MASK  : i64 = POLY_SUBPIXEL_SCALE - 1;
const POLY_MR_SUBPIXEL_SHIFT : i64 = 4;
const MAX_HALF_WIDTH : usize = 64;


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

/// Access Pixel source color
pub trait Source {
    fn get(&self, id: (usize, usize)) -> Rgba8;
}

/// Drawing and pixel related routines
pub trait Pixel {
    fn cover_mask() -> u64;
    fn bpp() -> usize;
    fn as_bytes(&self) -> &[u8];
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set<C: Color>(&mut self, id: (usize, usize), c: C);
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64);
    /// Fill the data with the specified `color`
    fn fill<C: Color>(&mut self, color: C);
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
    ///     use agg::{Source,Pixfmt,Rgb8,Rgba8,Pixel};
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



pub(crate) trait LineInterp {
    fn init(&mut self);
    fn step_hor(&mut self);
    fn step_ver(&mut self);
}

pub trait RenderOutline {
    fn cover(&self, d: i64) -> u64;
    fn blend_solid_hspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]);
    fn blend_solid_vspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]);
}
/// Functions for Drawing Outlines
//pub trait DrawOutline: Lines + AccurateJoins + SetColor {}
pub trait DrawOutline {
/// Set the current Color
//pub trait SetColor {
    fn color<C: Color>(&mut self, color: C);
//}
/// If Line Joins are Accurate
//pub trait AccurateJoins {
    fn accurate_join_only(&self) -> bool;
//}
//pub trait Lines {
    fn line0(&mut self, lp: &LineParameters);
    fn line1(&mut self, lp: &LineParameters, sx: i64, sy: i64);
    fn line2(&mut self, lp: &LineParameters, ex: i64, ey: i64);
    fn line3(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64);
    fn semidot<F>(&mut self, cmp: F, xc1: i64, yc1: i64, xc2: i64, yc2: i64) where F: Fn(i64) -> bool;
    fn pie(&mut self, xc: i64, y: i64, x1: i64, y1: i64, x2: i64, y2: i64);
}


pub(crate) trait DistanceInterpolator {
    fn dist(&self) -> i64;
    fn inc_x(&mut self, dy: i64);
    fn inc_y(&mut self, dx: i64);
    fn dec_x(&mut self, dy: i64);
    fn dec_y(&mut self, dx: i64);
}


