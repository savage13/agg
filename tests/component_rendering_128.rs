use agg::PixelData;
use agg::Rasterize;

use std::path::PathBuf;
use std::path::Path;
use std::env;


type PixRgb8 = agg::Pixfmt<agg::Rgb8>;
use agg::Gray8;
use agg::PixfmtAlphaBlend;

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


#[test]
fn component_rendering_128() {
    let alpha = 128;
    let (w, h) = (320, 320);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w, h);
    let mut ren_base = agg::RenderingBase::with_rgb24(pixf);
    ren_base.clear(agg::Rgba8::new(255,255,255,255));
    let g8 = Gray8::new_with_alpha(0,alpha);

    let mut er = agg::Ellipse::new();
    let mut eb = agg::Ellipse::new();
    let mut eg = agg::Ellipse::new();
    let w2 = (w/2) as f64;
    let h2 = (h/2) as f64;
    er.init(w2 - 0.87*50.0, h2 - 0.5*50., 100., 100., 100);
    eg.init(w2 + 0.87*50.0, h2 - 0.5*50., 100., 100., 100);
    eb.init(w2,             h2 + 50., 100., 100., 100);

    let mut ras = agg::RasterizerScanlineAA::new();
    let mut sl  = agg::ScanlineU8::new();

    {
        let pfr = PixfmtAlphaBlend::<PixRgb8,Gray8>::new(&mut ren_base, 0);
        let mut rbr = agg::RenderingBase::with_rgb24(pfr);
        ras.add_path(&er);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rbr, &g8);
    }
    {
        let pfg = PixfmtAlphaBlend::<PixRgb8,Gray8>::new(&mut ren_base, 1);
        let mut rbg = agg::RenderingBase::with_rgb24(pfg);
        ras.add_path(&eg);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rbg, &g8);
    }
    {
        let pfb = PixfmtAlphaBlend::<PixRgb8,Gray8>::new(&mut ren_base, 2);
        let mut rbb = agg::RenderingBase::with_rgb24(pfb);
        ras.add_path(&eb);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rbb, &g8);
    }

    let (ppm, test) = ppm_names();

    agg::ppm::write_ppm(&ren_base.pixeldata(), w, h, ppm.clone()).unwrap();
    agg::ppm::compare_ppm(ppm, test);
}
