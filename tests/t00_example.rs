
#[test]
fn t00_example() {
    use agg::Render;

    // Create a blank image 10x10 pixels
    let pix = agg::Pixfmt::<agg::Rgb8>::new(100,100);
    let mut ren_base = agg::RenderingBase::new(pix);
    ren_base.clear(agg::Rgba8::white());

    // Draw a polygon from (10,10) - (50,90) - (90,10)
    let mut ras = agg::RasterizerScanline::new();
    ras.move_to_d(10.0, 10.0);
    ras.line_to_d(50.0, 90.0);
    ras.line_to_d(90.0, 10.0);

    // Render the line to the image
    let mut ren = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
    ren.color(&agg::Rgba8::black());
    agg::render_scanlines(&mut ras, &mut ren);

    // Save the image to a file
    agg::ppm::write_ppm(&ren_base.as_bytes(), 100,100,
                        "little_black_triangle.ppm").unwrap();
}
