
extern crate agg;
use agg::RenderingBuffer;

fn draw_black_frame(rbuf: &mut RenderingBuffer) {
    let w = rbuf.width;
    let h = rbuf.height;
    let b = rbuf.bpp;
    let k = (w - 1) * 3;
    for i in 0 .. h {
        let p = rbuf.row_ptr(i);
        (0..b).for_each(|j| p[j] = 0);   // Left Side
        (0..b).for_each(|j| p[j+k] = 0); // Right Side
    }
    for k in [0,h-1].iter() {
        let p = rbuf.row_ptr(*k);
        (0..b).for_each(|i| p[i] = 0);
    }
}

fn clear(rbuf: &mut RenderingBuffer) {
    rbuf.data.iter_mut().for_each(|v| *v = 255);
}

#[test]
fn t01_rendering_buffer() {
    let mut rbuf = RenderingBuffer::new(320, 220, 3);
    clear(&mut rbuf);
    draw_black_frame(&mut rbuf);

    for i in 0 .. rbuf.height/2 {
        let p = rbuf.row_ptr(i);
        p[i*3+0] = 127; // Red
        p[i*3+1] = 200; // Green
        p[i*3+2] =  98; // Blue
    }

    agg::ppm::write_ppm(&rbuf.data, rbuf.width, rbuf.height, "agg_test_01.ppm").unwrap();
    agg::ppm::compare_ppm("agg_test_01.ppm", "tests/agg_test_01.ppm");
}

