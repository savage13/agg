
extern crate agg;
use agg::Pixel;

fn draw_black_frame(pix: &mut agg::Pixfmt<agg::Rgb8>) {
    let w = pix.width();
    let h = pix.height();
    for i in 0 .. h {
        pix.set((0,i),agg::Rgb8::black()); // Left Side
        pix.set((w-1,i),agg::Rgb8::black()); // Right Side
    }
    for i in 0 .. w {
        pix.set((i,0), agg::Rgb8::black()); // Top Side
        pix.set((i,h-1), agg::Rgb8::black()); // Bottom Side
    }
}


#[test]
fn t01_rendering_buffer() {
    //let mut rbuf = RenderingBuffer::new(320, 220, 3);
    let mut pix = agg::Pixfmt::<agg::Rgb8>::new(320, 220);
    for i in 0 .. pix.width() {
        for j in 0 .. pix.height() {
            pix.set((i,j), agg::Rgb8::white());
        }
    }
    draw_black_frame(&mut pix);

    for i in 0 .. pix.height()/2 {
        //let p = rbuf.row_ptr(i);
        pix.set((i,i), agg::Rgb8::new(127,200,98));
    }

    pix.to_file("tests/tmp/agg_test_01.png").unwrap();
    assert_eq!(agg::ppm::img_diff("tests/tmp/agg_test_01.png", "images/agg_test_01.png").unwrap(), true);
}

