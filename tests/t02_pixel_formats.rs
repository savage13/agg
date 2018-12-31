
extern crate agg;

fn draw_black_frame(pix: &mut agg::Pixfmt<agg::Rgb8>) {
    let w = pix.rbuf.width;
    let h = pix.rbuf.height;
    println!("w,h: {} {}", w,h);
    let black = agg::Rgb8::black();
    for i in 0 .. h {
        pix.copy_pixel(0,   i, black);
        pix.copy_pixel(w-1, i, black);
    }
    for &k in [0,h-1].iter() {
        for i in 0 .. w {
            pix.copy_pixel(i, k, black);
        }
    }
}

#[test]
fn t02_pixel_formats() {
    let rbuf = agg::RenderingBuffer::new(320, 220, 3);
    let mut pix = agg::Pixfmt::<agg::Rgb8>::from(rbuf);
    pix.clear();
    draw_black_frame(&mut pix);

    for i in 0 .. pix.rbuf.height/2 {
        let c = agg::Rgb8::new(127,200,98);
        pix.copy_pixel(i, i, c);
    }

    agg::ppm::write_ppm(&pix.rbuf.data, pix.rbuf.width, pix.rbuf.height, "agg_test_02.ppm").unwrap();
    agg::ppm::compare_ppm("agg_test_02.ppm", "tests/agg_test_02.ppm");
}
