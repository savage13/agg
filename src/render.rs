use scan::ScanlineU8;
use path_storage::VertexSource;
use raster::RasterizerScanline;
use color::Color;
use base::RenderingBase;
use PixelData;
use color::Rgba8;
#[derive(Debug)]
pub struct RenderingScanlineBinSolid<'a> {
    pub base: &'a mut RenderingBase,
    pub color: Rgba8,
}

#[derive(Debug)]
pub struct RenderingScanlineAASolid<'a> {
    pub base: &'a mut RenderingBase,
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
            // eprintln!("RENDER SCANLINE AA SOLID: {} {}",
            //           span.len, span.covers.len());
            ren.blend_solid_hspan(x, y, span.len, color, &span.covers);
        } else {
            ren.blend_hline(x, y, x-span.len-1, color, span.covers[0]);
        }
    }
}

pub trait RenderingScanline {
    fn render(&mut self, sl: &ScanlineU8);
    fn prepare(&self) { }
    fn color<C: Color>(&mut self, color: &C);
}

impl<'a> RenderingScanline for RenderingScanlineAASolid<'a> {
    fn render(&mut self, sl: &ScanlineU8) {
        render_scanline_aa_solid(sl, &mut self.base, &self.color);
    }
    fn color<C: Color>(&mut self, color: &C) {
        self.color = Rgba8::new((color.red() *255.0) as u8,
                                (color.green() *255.0) as u8,
                                (color.blue() *255.0) as u8,
                                (color.alpha() *255.0) as u8);
        
    }

}
impl<'a> RenderingScanline for RenderingScanlineBinSolid<'a> {
    fn render(&mut self, sl: &ScanlineU8) {
        render_scanline_bin_solid(sl, &mut self.base, &self.color);
    }
    fn color<C: Color>(&mut self, color: &C) {
        self.color = Rgba8::new((color.red() *255.0) as u8,
                                (color.green() *255.0) as u8,
                                (color.blue() *255.0) as u8,
                                (color.alpha() *255.0) as u8);
    }
}
impl<'a> RenderingScanlineBinSolid<'a> {
    pub fn with_base(base: &'a mut RenderingBase) -> Self {
        let color = Rgba8::black();
        Self { base, color }
    }
}
impl<'a> RenderingScanlineAASolid<'a> {
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


/*
pub trait Scale<T> {
    fn upscale(v: f64)   -> T;
    fn downscale(v: i64) -> T;

}
*/


pub fn render_scanlines_aa_solid<RAS,C>(ras: &mut RAS,
                                            sl: &mut ScanlineU8,
                                            ren: &mut RenderingBase,
                                            color: &C) 
    where RAS: RasterizerScanline,
          C: Color
{
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(sl) {
            render_scanline_aa_solid(sl, ren, color);
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

pub fn render_all_paths<REN,RAS,VS,C>(ras: &mut RAS,
                                      sl: &mut ScanlineU8,
                                      ren: &mut REN,
                                      paths: &[VS],
                                      colors: &[C])
    where C: Color,
          REN: RenderingScanline,
          RAS: RasterizerScanline,
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
