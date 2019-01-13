#[test]
fn t24_outline_basic_render() {
    use agg::{Pixfmt,Rgb8,Rgba8};
    use agg::{RendererPrimatives,RasterizerOutline};
    let pix = Pixfmt::<Rgb8>::new(100,100);
    let mut ren_base = agg::RenderingBase::new(pix);
    ren_base.clear( Rgba8::new(255, 255, 255, 255) );

    let mut ren = RendererPrimatives::with_base(&mut ren_base);
    ren.line_color(agg::Rgba8::new(0,0,0,255));

    let mut path = agg::Path::new();
    path.move_to(10.0, 10.0);
    path.line_to(50.0, 90.0);
    path.line_to(90.0, 10.0);

    let mut ras = RasterizerOutline::with_primative(&mut ren);
    ras.add_path(&path);
    ren_base.to_file("tests/tmp/primative.png").unwrap();

    //assert!(agg::ppm::img_diff("tests/tmp/primative.png",
    //                           "images/primative.png").unwrap());
}
