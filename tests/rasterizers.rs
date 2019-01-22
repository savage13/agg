
extern crate agg;

use agg::Render;

fn rgb64(r: f64, g: f64,b: f64,a: f64) -> agg::Rgba8 {
    agg::Rgba8::new((r * 255.0).round() as u8,
                    (g * 255.0).round() as u8,
                    (b * 255.0).round() as u8,
                    (a * 255.0).round() as u8)
}

#[test]
fn rasterizers() {
    let (w,h) = (500,330);

    let m_x = [100.+120., 369.+120., 143.+120.];
    let m_y = [60.,       170.,      310.0];

    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w,h);
    let mut ren_base = agg::RenderingBase::new(pixf);
    ren_base.clear( agg::Rgba8::new(255, 255, 255, 255) );

    //let gamma = 1.0;
    let alpha = 0.5;

    let mut ras = agg::RasterizerScanline::new();

    // Anti-Aliased
    {
        let mut ren_aa = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
        let mut path = agg::Path::new();

        path.move_to(m_x[0], m_y[0]);
        path.line_to(m_x[1], m_y[1]);
        path.line_to(m_x[2], m_y[2]);
        path.close_polygon();
        ren_aa.color( rgb64(0.7, 0.5, 0.1, alpha));
        ras.add_path(&path);
        agg::render_scanlines(&mut ras, &mut ren_aa);
    }

    // Aliased
    {
        let mut ren_bin = agg::RenderingScanlineBinSolid::with_base(&mut ren_base);
        let mut path = agg::Path::new();

        path.move_to(m_x[0] - 200., m_y[0]);
        path.line_to(m_x[1] - 200., m_y[1]);
        path.line_to(m_x[2] - 200., m_y[2]);
        path.close_polygon();
        ren_bin.color( rgb64(0.1, 0.5, 0.7, alpha) );
        ras.add_path(&path);
        //ras.
        agg::render_scanlines(&mut ras, &mut ren_bin);
    }
    ren_base.to_file("tests/tmp/rasterizers.png").unwrap();
    assert!(agg::ppm::img_diff("tests/tmp/rasterizers.png", "images/rasterizers.png").unwrap());
}
