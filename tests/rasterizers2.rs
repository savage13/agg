
extern crate agg;

use agg::PixelData;
use agg::Render;
use agg::VertexSource;
//use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::env;

fn ppm_names() -> (PathBuf,PathBuf) {
    let progname = env::args().next().unwrap();
    let progname = Path::new(&progname);
    let mut base = progname.file_stem().unwrap().to_string_lossy().into_owned();
    let n = base.rfind("-").unwrap();
    base.truncate(n);
    let ppm = Path::new(&base).with_extension("ppm");
    let test = Path::new("tests").join(ppm.clone());
    (ppm, test)
}

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

#[test]
fn rasterizers2() {
    let (w,h) = (500, 450);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w, h);
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

    let (ppm, test) = ppm_names();

    agg::ppm::write_ppm(&ren_base.pixeldata(), w, h, ppm.clone()).unwrap();
    agg::ppm::compare_ppm(ppm, test);

}
