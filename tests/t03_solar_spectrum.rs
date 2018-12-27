
extern crate agg;
use agg::PixfmtFunc;

fn draw_black_frame(pix: &mut agg::Pixfmt<agg::Rgb8>) {
    let w = pix.rbuf.width;
    let h = pix.rbuf.height;
    let black = agg::Rgb8::black();
    pix.copy_hline(0,0,  w,&black);
    pix.copy_hline(0,h-1,w,&black);

    pix.copy_vline(0,0,  h,&black);
    pix.copy_vline(w-1,0,h,&black);
}

#[test]
fn t03_solar_specturm() {
    let mut pix = agg::Pixfmt::<agg::Rgb8>::new(320, 200);
    pix.clear();
    draw_black_frame(&mut pix);

    let w = pix.rbuf.width;
    let h = pix.rbuf.height;
    let mut span = vec![agg::Rgb8::white(); w];

    for i in 0 .. w {
        span[i] = agg::Rgb8::from_wavelength_gamma(380.0 + 400.0 * i as f64 / w as f64, 0.8);
    }

    for i in 0 .. h {
        pix.blend_color_hspan(0, i as i64, w as i64, &span, &[], 255);
    }
    agg::ppm::write_ppm(&pix.rbuf.data, pix.rbuf.width, pix.rbuf.height, "agg_test_03.ppm").unwrap();
    agg::ppm::compare_ppm("agg_test_03.ppm", "tests/agg_test_03.ppm");
}
