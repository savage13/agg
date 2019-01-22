
type PixRgb8 = agg::Pixfmt<agg::Rgb8>;
use agg::Gray8;
use agg::PixfmtAlphaBlend;


#[test]
fn component_rendering_128() {
    let alpha = 128;
    let (w, h) = (320, 320);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w, h);
    let mut ren_base = agg::RenderingBase::new(pixf);
    ren_base.clear(agg::Rgba8::new(255,255,255,255));
    let g8 = Gray8::new_with_alpha(0,alpha);

    let w2 = (w/2) as f64;
    let h2 = (h/2) as f64;
    let er = agg::Ellipse::new(w2 - 0.87*50.0, h2 - 0.5*50., 100., 100., 100);
    let eg = agg::Ellipse::new(w2 + 0.87*50.0, h2 - 0.5*50., 100., 100., 100);
    let eb = agg::Ellipse::new(w2,             h2 + 50., 100., 100., 100);

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

    ren_base.to_file("tests/tmp/component_rendering_128.png").unwrap();
    assert_eq!(agg::ppm::img_diff("tests/tmp/component_rendering_128.png", "images/component_rendering_128.png").unwrap(), true);
}
