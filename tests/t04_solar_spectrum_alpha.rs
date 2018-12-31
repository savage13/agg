
extern crate agg;

#[test]
fn t04_solar_spectrum_alpha() {
    let mut pix = agg::Pixfmt::<agg::Rgb8>::new(320, 200);
    pix.clear();

    let mut alpha = agg::Pixfmt::<agg::Gray8>::new(320, 200);

    let w = pix.rbuf.width;
    let h = pix.rbuf.height;

    for i in 0 .. h {
        let v = (255 * i/h) as u8;
        alpha.copy_hline(0, i, w, agg::Gray8::new(v));
    }

    let mut span = vec![agg::Rgb8::white(); w];
    for i in 0 .. w {
        span[i] = agg::Rgb8::from_wavelength_gamma(380.0 + 400.0 * i as f64 / w as f64, 0.8);
    }

    let mut mix = agg::AlphaMaskAdaptor::new(pix, alpha);

    for i in 0 .. h {
        mix.blend_color_hspan(0, i, w, &span, 0);
    }
    agg::ppm::write_ppm(&mix.rgb.rbuf.data, w, h,
              "agg_test_04.ppm").unwrap();

    agg::ppm::compare_ppm("agg_test_04.ppm", "tests/agg_test_04.ppm");

}

