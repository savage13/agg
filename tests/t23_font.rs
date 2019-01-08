
#[test]
fn t23_font() {
    //let ft = agg::FreeType::init();
    let lib = agg::ft::Library::init().unwrap();
    let font = lib.new_face("/System/Library/Fonts/Helvetica.ttc", 0).unwrap();
    font.set_char_size(13 * 64, 0, 72, 0).unwrap();

    let pix = agg::Pixfmt::<agg::Rgb8>::new(100,100);
    let mut ren_base = agg::RenderingBase::new(pix);
    ren_base.clear( agg::Rgba8::new(255, 255, 255, 255) );

    agg::draw_text("Hello World!!!", 50, 45, &font, &mut ren_base);

    let mut label = agg::Label::new("Hello World!!!", 50., 57., &font)
        .xalign(agg::XAlign::Center)
        .yalign(agg::YAlign::Center);
    label.draw(&mut ren_base);

    ren_base.blend_hline(50,57,50,agg::Rgba8::new(255,0,0,255),255);

    agg::ppm::write_ppm(&ren_base.as_bytes(), 100,100,
                        "font.ppm").unwrap();

}
