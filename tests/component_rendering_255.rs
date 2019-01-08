
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
fn component_rendering_255() {
    let alpha = 255;
    let (w, h) = (320, 320);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w, h);
    let mut ren_base = agg::RenderingBase::new(pixf);
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

    let mut ras = agg::RasterizerScanline::new();

    {
        let pfr = PixfmtAlphaBlend::<PixRgb8,Gray8>::new(&mut ren_base, 0);
        let mut rbr = agg::RenderingBase::new(pfr);
        ras.add_path(&er);
        agg::render_scanlines_aa_solid(&mut ras, &mut rbr, g8);
    }
    {
        let pfg = PixfmtAlphaBlend::<PixRgb8,Gray8>::new(&mut ren_base, 1);
        let mut rbg = agg::RenderingBase::new(pfg);
        ras.add_path(&eg);
        agg::render_scanlines_aa_solid(&mut ras, &mut rbg, g8);
    }
    {
        let pfb = PixfmtAlphaBlend::<PixRgb8,Gray8>::new(&mut ren_base, 2);
        let mut rbb = agg::RenderingBase::new(pfb);
        ras.add_path(&eb);
        agg::render_scanlines_aa_solid(&mut ras, &mut rbb, g8);
    }

    let (ppm, test) = ppm_names();

    agg::ppm::write_ppm(&ren_base.as_bytes(), w, h, ppm.clone()).unwrap();
    agg::ppm::compare_ppm(ppm, test);
}
