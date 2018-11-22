
extern crate agg;

#[test]
fn t06_simple_line() {
    let mut pix = agg::PixfmtRgb24::new(320,200,3);
    let black = agg::Rgb8::black();
    let w = pix.rbuf.width;
    let h = pix.rbuf.height;
    pix.fill(agg::Rgb8::white());
    let r = h as f64/2.0;
    let (x0,y0) = (w as f64/2.0, h as f64/2.0);
    for i in (0 .. 360).step_by(1) {
        let x1 = x0 + r * (i as f64).to_radians().cos();
        let y1 = y0 + r * (i as f64).to_radians().sin();
        //println!("angle: {} {} {}", i, x1,y1);
        pix.line(x0,y0, x1, y1, black);
    }
    agg::write_ppm(&pix.rbuf.data, w, h,
                   "agg_test_06.ppm").unwrap();
    agg::compare_ppm("agg_test_06.ppm", "tests/agg_test_06.ppm");

}
