
extern crate agg;

use agg::PixfmtFunc;

#[test]
fn t09_spiral() {
    let mut pix = agg::Pixfmt::<agg::Rgb8>::new(320,200);
    let black = agg::Rgb8::black();
    let w = pix.rbuf.width;
    let h = pix.rbuf.height;
    pix.fill(agg::Rgb8::white());

    let (mut x0, mut y0) = (w as f64/2., h as f64/2.);
    let mut r = 0.0;
    let n = 10 * 360;
    for i in (0 .. n).step_by(1) {
        r += h as f64/ 2.0 /  (10. * 360.0);
        let x1 = w as f64/2. + r * (i as f64).to_radians().cos();
        let y1 = h as f64/2. + r * (i as f64).to_radians().sin();
        pix.line(x0,y0, x1, y1, black);
        x0 = x1;
        y0 = y1;
    }
    agg::ppm::write_ppm(&pix.rbuf.data, w, h,
                   "agg_test_09.ppm").unwrap();
    agg::ppm::compare_ppm("agg_test_09.ppm", "tests/agg_test_09.ppm");

}
