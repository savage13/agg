
extern crate agg;

use agg::PixelDraw;
use agg::PixelData;
use agg::Pixel;

#[test]
fn t07_subpixel_line() {
    let mut pix = agg::Pixfmt::<agg::Rgb8>::new(320,200);
    let black = agg::Rgb8::black();
    let w = pix.width();
    let h = pix.height();
    pix.fill(agg::Rgb8::white());
    let r = h as f64/2.0;
    let (x0,y0) = (w as f64/2.0, h as f64/2.0);
    for i in (0 .. 360).step_by(1) {
        let x1 = x0 + r * (i as f64).to_radians().cos();
        let y1 = y0 + r * (i as f64).to_radians().sin();
        println!("angle: {} {} {}", i, x1,y1);
        pix.line_sp(x0,y0, x1, y1, black);
    }
    agg::ppm::write_ppm(&pix.pixeldata(), w, h,
                   "agg_test_07.ppm").unwrap();
    agg::ppm::compare_ppm("agg_test_07.ppm", "tests/agg_test_07.ppm");

}
