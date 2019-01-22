
extern crate agg;

use agg::Render;

#[test]
fn t13_aliased() {
    let (w,h) = (100,100);

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w,h);

    let mut ren_base = agg::RenderingBase::new(pixf);

    ren_base.clear( agg::Rgba8::new(255, 255, 255, 255) );

    let mut ren = agg::RenderingScanlineBinSolid::with_base(&mut ren_base);

    ren.color( agg::Rgba8::new(255,0,0,255) );

    let mut ras = agg::RasterizerScanline::new();

    ras.clip_box(40.0, 0.0, w as f64-40.0, h as f64);

    ras.move_to(10.0, 10.0);
    ras.line_to(50.0, 90.0);
    ras.line_to(90.0, 10.0);

    agg::render_scanlines(&mut ras, &mut ren);

    ren.to_file("tests/tmp/agg_test_13.png").unwrap();

    assert_eq!(agg::ppm::img_diff("tests/tmp/agg_test_13.png", "images/agg_test_13.png").unwrap(), true);
}

