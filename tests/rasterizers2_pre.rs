extern crate agg;

use agg::Pixel;
use agg::Render;
use agg::DrawOutline;
use agg::VertexSource;

pub struct Roundoff<T: VertexSource> {
    pub src: T,
}

impl<T> Roundoff<T> where T: VertexSource {
    fn new(src: T) -> Self {
        Self { src }
    }
}

impl<T> VertexSource for Roundoff<T> where T: VertexSource {
    fn xconvert(&self) -> Vec<agg::Vertex<f64>> {
        self.src.xconvert()
            .into_iter()
            .map(|v| agg::Vertex::new(v.x.floor(), v.y.floor(), v.cmd) )
            .collect()
    }
}

#[derive(Debug,Default)]
pub struct Spiral {
    x: f64,
    y: f64,
    r1: f64,
    r2: f64,
    step: f64,
    start_angle: f64,
    da: f64,
    dr: f64,
}

impl VertexSource for Spiral {
    fn xconvert(&self) -> Vec<agg::Vertex<f64>> {
        self.spin_spin_spin()
    }
}

impl Spiral {
    pub fn new(x: f64, y: f64, r1: f64, r2: f64, step: f64, start_angle: f64) -> Self {
        let da = 8.0f64.to_radians();
        let dr = step / 45.0;
        Self {x, y, r1, r2, step, start_angle, da, dr}
    }
    pub fn spin_spin_spin(&self) -> Vec<agg::Vertex<f64>> {
        let mut out = vec![];
        //let mut i = 0;
        let mut r = self.r1;
        let mut angle = self.start_angle;
        while r <= self.r2 {
            let x = self.x + angle.cos() * r;
            let y = self.y + angle.sin() * r;
            if out.is_empty() {
                out.push( agg::Vertex::move_to(x, y));
            } else {
                out.push( agg::Vertex::line_to(x, y));
            }
            //i += 1;
            r += self.dr;
            angle += self.da;
            //r = self.r1 + i as f64 * self.dr;
            //angle = self.start_angle + i as f64 * self.da;
        }
        out
    }
}


fn chain() -> agg::Pixfmt<agg::Rgba32> {
    let width  = 16;
    let height = 7;
    let mut pix = agg::Pixfmt::<agg::Rgba32>::new(width, height);
    let raw : [u32; 16*7] = [
        0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0xb4c29999, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xb4c29999, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff,
        0x00ffffff, 0x00ffffff, 0x0cfbf9f9, 0xff9a5757, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xb4c29999, 0x00ffffff, 0x00ffffff, 0x00ffffff,
        0x00ffffff, 0x5ae0cccc, 0xffa46767, 0xff660000, 0xff975252, 0x7ed4b8b8, 0x5ae0cccc, 0x5ae0cccc, 0x5ae0cccc, 0x5ae0cccc, 0xa8c6a0a0, 0xff7f2929, 0xff670202, 0x9ecaa6a6, 0x5ae0cccc, 0x00ffffff,
        0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xa4c7a2a2, 0x3affff00, 0x3affff00, 0xff975151, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000,
        0x00ffffff, 0x5ae0cccc, 0xffa46767, 0xff660000, 0xff954f4f, 0x7ed4b8b8, 0x5ae0cccc, 0x5ae0cccc, 0x5ae0cccc, 0x5ae0cccc, 0xa8c6a0a0, 0xff7f2929, 0xff670202, 0x9ecaa6a6, 0x5ae0cccc, 0x00ffffff,
        0x00ffffff, 0x00ffffff, 0x0cfbf9f9, 0xff9a5757, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xb4c29999, 0x00ffffff, 0x00ffffff, 0x00ffffff,
        0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0xb4c29999, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xb4c29999, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff];

    let mut colors = vec![];
    for v in raw.iter() {
        let r = ((v >> 16) & 0x0000_00ff_u32) as u8;
        let g = ((v >>  8) & 0x00ff_u32) as u8;
        let b = ((v      ) & 0x00ff_u32) as u8;
        let a =  (v >> 24) as u8;
        let c = agg::Rgba32::from_trait(agg::Srgba8::new(r,g,b,a));
        colors.push( c.premultiply() );
    }
    let mut k = 0;
    for j in 0 .. height {
        for i in 0 .. width {
            pix.set((i,j), colors[k]);
            k += 1;
        }
    }
    pix
}

#[test]
fn rasterizers2_pre() {
    let (w,h) = (500, 450);

    let pixf = agg::Pixfmt::<agg::Rgba8pre>::new(w, h);
    let mut ren_base = agg::RenderingBase::new(pixf);

    ren_base.clear( agg::Rgba8::new(255, 255, 242, 255) );

    let start_angle = 0.0;
    let line_width = 3.0;
    let _width  = w as f64;
    let height = h as f64;
    let (r1, r2) = (5.0, 70.0);
    let step = 16.0;
    // Anti-aliased Scanline Spiral
    {
        let x = (w / 2) as f64;
        let y = (h - h / 4 + 20) as f64;
        let spiral = Spiral::new(x, y, r1, r2, step, start_angle);

        let mut ras_aa = agg::RasterizerScanline::new();
        let mut ren_aa = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
        let mut stroke = agg::ConvStroke::new(spiral);
        stroke.width(line_width);
        //stroke.cap(round_cap);
        ren_aa.color( &agg::Rgba8::new(102, 77, 26, 255));
        ras_aa.add_path(&stroke);
        agg::render_scanlines(&mut ras_aa, &mut ren_aa);

    }
    // Aliased Pixel Accuracy
    {
        let x = (w / 5) as f64;
        let y = (h / 4 + 50) as f64;
        let spiral = Spiral::new(x, y, r1, r2, step, start_angle);

        let mut ren_prim = agg::RendererPrimatives::with_base(&mut ren_base);
        ren_prim.line_color(agg::Rgba8::new(102, 77, 26, 255));
        let mut ras_al = agg::RasterizerOutline::with_primative(&mut ren_prim);
        let trans = Roundoff::new(spiral);
        ras_al.add_path(&trans);
    }
    // Aliased Subpixel Accuracy
    {
        let x = (w / 2) as f64;
        let y = (h / 4 + 50) as f64;
        eprintln!("DDA SPIRAL: {} {} h {} h/4 {}", x, y, height, height/4.0);
        let spiral = Spiral::new(x, y, r1, r2, step, start_angle);

        let mut ren_prim = agg::RendererPrimatives::with_base(&mut ren_base);
        ren_prim.line_color(agg::Rgba8::new(102, 77, 26, 255));
        let mut ras_al = agg::RasterizerOutline::with_primative(&mut ren_prim);
        ras_al.add_path(&spiral);
    }
    // Anti-Aliased Outline
    {
        let x = (w/5) as f64;
        let y = (h - h/4 + 20) as f64;
        let spiral = Spiral::new(x, y, r1, r2, step, start_angle);

        let mut ren_oaa = agg::RendererOutlineAA::with_base(&mut ren_base);
        ren_oaa.color(agg::Rgba8::new(102,77,26,255));
        ren_oaa.width(3.0);
        let mut ras_oaa = agg::RasterizerOutlineAA::with_renderer(&mut ren_oaa);
        ras_oaa.round_cap(true);
        ras_oaa.add_path(&spiral);
    }
    // Anti-Aliased Outline Image
    {
        let x = (w - w/5) as f64;
        let y = (h - h/4 + 20) as f64;
        let spiral = Spiral::new(x, y, r1, r2, step, start_angle);

        //let ren_oaa = agg::RendererOutlineAA::with_base(&mut ren_base);

        let filter  = agg::PatternFilterBilinear::new();
        let mut pattern = agg::LineImagePatternPow2::new(filter);
        let ch = chain();
        pattern.create( &ch );
        let mut ren_img = agg::RendererOutlineImg::with_base_and_pattern(&mut ren_base, pattern);
        let mut ras_img = agg::RasterizerOutlineAA::with_renderer(&mut ren_img);
        //ren_oaa.color(&agg::Rgba8::new(102,77,26,255));
        ras_img.round_cap(true);
        ras_img.add_path(&spiral);
    }

    {
        let mut ras_aa = agg::RasterizerScanline::new();
        let mut ren_aa = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
        text(&mut ras_aa, &mut ren_aa, 50.0, 75.0,
             "Bresenham lines,\n\nregular accuracy");
        text(&mut ras_aa, &mut ren_aa, (w/2-50) as f64, 75.0,
             "Bresenham lines,\n\nsubpixel accuracy");
        text(&mut ras_aa, &mut ren_aa, 50., (h/2+50) as f64,
             "Anti-aliased lines");
        text(&mut ras_aa, &mut ren_aa, (w/2-50) as f64, (h/2+50) as f64,
             "Scanline rasterizer");
        text(&mut ras_aa, &mut ren_aa, (w - w/5 - 50) as f64, (h/2+50) as f64,
             "Arbitrary Image Pattern");

    }

    // Revove alpha channel from data
    let data = ren_base.as_bytes();
    let mut out = vec![];
    for i in 0 .. data.len() {
        if i%4 < 3 {
            out.push(data[i]);
        }
    }
    ren_base.pixf.drop_alpha().to_file("tests/tmp/rasterizers2_pre.png").unwrap();
    assert!(agg::ppm::img_diff("tests/tmp/rasterizers2_pre.png", "images/rasterizers2_pre.png",).unwrap());

}

fn text<T>(ras: &mut agg::RasterizerScanline,
           ren: &mut agg::RenderingScanlineAASolid<T>,
           x: f64, y: f64, txt: &str)
    where T: agg::Pixel
{
    let mut t = agg::GsvText::new();
    t.size(8.0, 0.0);
    t.text(txt);
    t.start_point(x,y);
    t.flip(true);
    let mut stroke = agg::ConvStroke::new(t);
    stroke.width(0.7);
    ras.add_path(&stroke);
    ren.color(&agg::Rgba8::new(0,0,0,255));
    agg::render_scanlines(ras, ren);

}
