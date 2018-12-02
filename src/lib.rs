
/// How does this work
///    ren = RenAA( RenBase( Pixfmt( data ) ) )
///    ras = Raster()
///    sl  = Scanline()
///  Raster Operations
///    line, move, add_path
///    clip.line()
///       clip.line_clip_y()
///        line()
///         render_hline()    -- 'INCR[0,1,2,3]'
///          set_curr_cell()
///         set_curr_cell()
///     Output: Cells with X, Cover, and Area
///  Render to Image
///   render_scanlines(ras, sl, ren)
///     rewind_scanline
///       close_polygon()
///       sort_cells() -- 'SORT_CELLS: SORTING'
///     scanline_reset
///     sweep_scanlines()
///       render_scanline - Individual horizontal (y) lines
///         blend_solid_hspan
///         blend_hline
///           blend_hline (pixfmt)

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
pub use ppm::*;
pub use alphamask::*;
pub use render::*;

const POLY_SUBPIXEL_SHIFT : i64 = 8;
const POLY_SUBPIXEL_SCALE : i64 = 1<<POLY_SUBPIXEL_SHIFT;
const POLY_SUBPIXEL_MASK  : i64 = POLY_SUBPIXEL_SCALE - 1;

pub trait PixelData<'a> {
    fn pixeldata(&'a self) -> &'a [u8];
}


fn blend(fg: Rgb8, bg: Rgb8, alpha: f64) -> Rgb8 {
    let v : Vec<_> = fg.iter().zip(bg.iter())
        .map(|(&fg,&bg)| (f64::from(fg), f64::from(bg)) )
        .map(|(fg,bg)| alpha * fg + (1.0 - alpha) * bg)
        .map(|v| v as u8)
        .collect();
    Rgb8::new([v[0],v[1],v[2]])
}

pub fn prelerp(a: f64, b: f64, t: f64)  {
    let (_a,_b,_t) = (a as f64, b as f64, t as f64);
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64{
    let mut v = (b-a) * t + a;
    //eprintln!("BLEND PIX: {} {} {} => {}", a, b, t, v);
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

// See agg_color_rgba.h:454 
pub fn lerp_u8(p: u8, q: u8, a: u8) -> u8 {
    let base_shift = 8;
    let base_msb = 1 << (base_shift - 1);
    let v = if p > q { 1 } else { 0 };
    let (q,p,a) = (i32::from(q), i32::from(p), i32::from(a));
    let t0 : i32  = (q - p) * a + base_msb - v; // Signed multiplication
    let t1 : i32 = ((t0>>base_shift) + t0) >> base_shift;
    (p + t1) as u8
}
// See agg_color_rgba.h:395
// https://sestevenson.wordpress.com/2009/08/19/rounding-in-fixed-point-number-conversions/
// https://stackoverflow.com/questions/10067510/fixed-point-arithmetic-in-c-programming
// http://x86asm.net/articles/fixed-point-arithmetic-and-tricks/
// Still not sure where the value is added and shifted multiple times
pub fn multiply_u8(a: u8, b: u8) -> u8 {
    let base_shift = 8;
    let base_msb = 1 << (base_shift - 1);
    let (a,b) = (u32::from(a), u32::from(b));
    let t : u32  = a * b + base_msb;
    let tt : u32 = ((t >> base_shift) + t) >> base_shift;
    tt as u8
}

pub fn blend_pix<C1: Color, C2: Color>(p: &C1, c: &C2, cover: u64) -> Rgba8 {

    assert!(c.alpha() >= 0.0);
    assert!(c.alpha() <= 1.0);

    let alpha = multiply_u8(c.alpha8(), cover as u8);
    eprintln!("BLEND PIX: ALPHA COVER {} {} => {}", c.alpha8(), cover, alpha);
    eprintln!("BLEND PIX: {:?}", p);
    eprintln!("BLEND PIX: {:?}", c);

    let red   = lerp_u8(p.red8(),   c.red8(),   alpha);
    let green = lerp_u8(p.green8(), c.green8(), alpha);
    let blue  = lerp_u8(p.blue8(),  c.blue8(),  alpha);
    let alpha = lerp_u8(p.alpha8(), c.alpha8(), alpha);
    eprintln!("BLEND PIX: r,g,b,a {:.3} {:.3} {:.3} {:.3}", red, green, blue, alpha);
    Rgba8::new(red, green, blue,alpha)
}

