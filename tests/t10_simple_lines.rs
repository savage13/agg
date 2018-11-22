
extern crate agg;

#[test]
fn t10_simple_lines() {
    let mut pix = agg::PixfmtRgb24::new(320,200,3);
    let black = agg::Rgb8::black();
    let w = pix.rbuf.width;
    let h = pix.rbuf.height;
    pix.fill(agg::Rgb8::white());
    let r = h as f64 *0.48;
    let (x0,y0) = (w as f64/2.0, h as f64/2.0);
    for i in (0 .. 360).step_by(10) {
        let x1 = x0 + r * (i as f64).to_radians().cos();
        let y1 = y0 + r * (i as f64).to_radians().sin();
        pix.line_sp_aa(x0,y0, x1, y1, black);
    }
    agg::write_ppm(&pix.rbuf.data, w, h, "agg_test_10.ppm").unwrap();
}
