//! Renderer

use scan::ScanlineU8;
use base::RenderingBase;
use color::Rgba8;

use PixelData;
use VertexSource;
use Render;
use Rasterize;
use Color;

/// Aliased Renderer
#[derive(Debug)]
pub struct RenderingScanlineBinSolid<'a> {
    pub base: &'a mut RenderingBase,
    pub color: Rgba8,
}
/// Anti-Aliased Renderer
#[derive(Debug)]
pub struct RenderingScanlineAASolid<'a> {
    pub base: &'a mut RenderingBase,
    pub color: Rgba8,
}

/// Render a single Scanline (y-row) without Anti-Aliasing (Binary?)
fn render_scanline_bin_solid<C: Color>(sl: &ScanlineU8,
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

/// Render a single Scanline (y-row) with Anti Aliasing
fn render_scanline_aa_solid<C: Color>(sl: &ScanlineU8,
                                          ren: &mut RenderingBase,
                                          color: &C) {
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


impl<'a> Render for RenderingScanlineAASolid<'a> {
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
impl<'a> Render for RenderingScanlineBinSolid<'a> {
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
impl<'a> RenderingScanlineBinSolid<'a> {
    /// Create a new Renderer from a Rendering Base
    pub fn with_base(base: &'a mut RenderingBase) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
}
impl<'a> RenderingScanlineAASolid<'a> {
    /// Create a new Renderer from a Rendering Base
    pub fn with_base(base: &'a mut RenderingBase) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
}
impl<'a> PixelData<'a> for RenderingScanlineAASolid<'a> {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.base.pixf.rbuf.data
    }
}
impl<'a> PixelData<'a> for RenderingScanlineBinSolid<'a> {
    fn pixeldata(&'a self) -> &'a [u8] {
        & self.base.pixf.rbuf.data
    }
}

/* pub trait Scale<T> {
    fn upscale(v: f64)   -> T;
    fn downscale(v: i64) -> T;
}*/

/// Render rasterized data to an image using a single color, Binary
pub fn render_scanlines_bin_solid<RAS,C>(_ras: &mut RAS,
                                         _sl: &mut ScanlineU8,
                                         _ren: &mut RenderingBase,
                                         _color: &C) 
    where RAS: Rasterize,
          C: Color
{
    unimplemented!();
}

/// Render rasterized data to an image using a single color, Anti-aliased
pub fn render_scanlines_aa_solid<RAS,C>(ras: &mut RAS,
                                        sl: &mut ScanlineU8,
                                        ren: &mut RenderingBase,
                                        color: &C) 
    where RAS: Rasterize,
          C: Color
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
