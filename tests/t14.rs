
extern crate agg;

use agg::Render;

#[test]
fn t14_with_gamma() {
    let (w,h) = (100,100);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w,h);

    let mut ren_base = agg::RenderingBase::new(pixf);

    ren_base.clear( agg::Rgba8::new(255, 255, 255, 255) );

    let mut ren = agg::RenderingScanlineAASolid::with_base(&mut ren_base);

    ren.color( &agg::Rgba8::new(255,0,0,255) );

    let mut ras = agg::RasterizerScanline::new();

    ras.clip_box(40.0, 0.0, w as f64-40.0, h as f64);

    ras.move_to_d(10.0, 10.0);
    ras.line_to_d(50.0, 90.0);
    ras.line_to_d(90.0, 10.0);

    agg::render_scanlines(&mut ras, &mut ren);

    agg::ppm::write_ppm(&ren.as_bytes(), w, h, "agg_test_14.ppm").unwrap();

    agg::ppm::compare_ppm("agg_test_14.ppm", "tests/agg_test_14.ppm");
}

