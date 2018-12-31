
extern crate agg;
use agg::PixelData;
use agg::Render;
use agg::Rasterize;
#[test]
fn t16_path_stroke_no_clip() {
    let (w,h) = (100,100);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w,h);

    let mut ren_base = agg::RenderingBase::with_rgb24(pixf);

    ren_base.clear( agg::Rgba8::new(255, 255, 255, 255) );

    let mut ren = agg::RenderingScanlineAASolid::with_base(&mut ren_base);

    ren.color( &agg::Rgba8::new(255,0,0,255) );

    let mut ras = agg::RasterizerScanlineAA::new();
    let mut sl = agg::ScanlineU8::new();

    //ras.clip_box(40.0, 0.0, w as f64-40.0, h as f64);

    ras.reset();
    ras.move_to_d(10.0, 10.0);
    ras.line_to_d(50.0, 90.0);
    ras.line_to_d(90.0, 10.0);

    agg::render_scanlines(&mut ras, &mut sl, &mut ren);

    let ps = agg::PathStorage::new();
    let mut pg = agg::ConvStroke::new(ps);

    pg.width(2.0);
    pg.source.remove_all();
    pg.source.move_to(10.0, 10.0);
    pg.source.line_to(50.0, 90.0);
    pg.source.line_to(90.0, 10.0);
    pg.source.line_to(10.0, 10.0);
    ras.add_path(&mut pg);

    agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut ren.base,
                                   agg::Rgba8::new(0,0,0,255));

    agg::ppm::write_ppm(&ren.pixeldata(), w, h, "agg_test_16.ppm").unwrap();

    agg::ppm::compare_ppm("agg_test_16.ppm", "tests/agg_test_16.ppm");
}

