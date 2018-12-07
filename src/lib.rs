
// How does this work / Data Flow
//    ren = RenAA( RenBase( Pixfmt( data ) ) )
//    ras = Raster()
//    sl  = Scanline()
//  Raster Operations
//    line, move, add_path
//    clip.line()
//       clip.line_clip_y()
//        line()
//         render_hline()    -- 'INCR[0,1,2,3]'
//          set_curr_cell()
//         set_curr_cell()
//     Output: Cells with X, Cover, and Area
//  Render to Image
//   render_scanlines(ras, sl, ren)
//     rewind_scanline
//       close_polygon()
//       sort_cells() -- 'SORT_CELLS: SORTING'
//     scanline_reset
//     sweep_scanlines()
//       render_scanline - Individual horizontal (y) lines
//         blend_solid_hspan
//         blend_hline
//           blend_hline (pixfmt)

// When difference occur:
//   - Check Input Path (ADD_PATH) in rasterizer
//   - Check Scanlines (SWEEP SCANLINES) in rasterizer
//   - Check Pixels    (BLEND_HLINE)

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

pub use path_storage::*;
pub use conv_stroke::*;
pub use affine_transform::*;
pub use color::*;
pub use pixfmt::*;
pub use buffer::*;
pub use base::*;
pub use clip::*;
pub use cell::*;
pub use raster::*;
pub use scan::*;
pub use alphamask::*;
pub use render::*;

const POLY_SUBPIXEL_SHIFT : i64 = 8;
const POLY_SUBPIXEL_SCALE : i64 = 1<<POLY_SUBPIXEL_SHIFT;
const POLY_SUBPIXEL_MASK  : i64 = POLY_SUBPIXEL_SCALE - 1;

/// Access raw color component data at the pixel level
pub trait PixelData<'a> {
    fn pixeldata(&'a self) -> &'a [u8];
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
pub trait Color: std::fmt::Debug {
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
}
/// Render scanlines to Image
pub trait Render {
    /// Render a single scanlines to the image
    fn render(&mut self, sl: &ScanlineU8);
    /// Set the Color of the Renderer
    fn color<C: Color>(&mut self, color: &C);
    /// Prepare the Renderer
    fn prepare(&self) { }
}
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

pub trait Pixel {
    fn set<C: Color>(&mut self, id: (usize, usize), c: &C);
    fn cover_mask() -> u64;
    fn bpp() -> usize;
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: &C, cover: u64);
}
pub trait PixfmtFunc {
    fn fill<C: Color>(&mut self, color: &C);
    fn rbuf(&self) -> &RenderingBuffer;
    fn blend_hline<C: Color>(&mut self, x: i64, y: i64, len: i64, c: &C, cover: u64);
    fn blend_solid_hspan<C: Color>(&mut self, x: i64, y: i64, len: i64, c: &C, covers: &[u64]);
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


