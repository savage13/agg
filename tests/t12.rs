
extern crate agg;
use agg::PixelData;

#[test]
fn t12_clip_box() {
    let (w,h,bpp) = (100,100,3);

    let pixf = agg::PixfmtRgb24::new(w,h,bpp);

    let mut ren_base = agg::RenderingBase::with_rgb24(pixf);

    ren_base.clear( agg::Rgba8::new([255, 255, 255, 255]) );

    let mut ren = agg::RenderingScanlineAASolid::with_base(ren_base);

    ren.color( agg::Rgba8::new([255,0,0,255]) );

    let mut ras = agg::RasterizerScanlineAA::new();
    let mut sl = agg::ScanlineU8::new();

    ras.clip_box(40.0, 0.0, w as f64-40.0, h as f64);

    ras.move_to_d(10.0, 10.0);
    ras.line_to_d(50.0, 90.0);
    ras.line_to_d(90.0, 10.0);

    agg::render_scanlines(&mut ras, &mut sl, &mut ren);

    agg::write_ppm(&ren.pixeldata(), w, h, "agg_test_12.ppm").unwrap();

    agg::compare_ppm("agg_test_12.ppm", "tests/agg_test_12.ppm");
}

