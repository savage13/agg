
extern crate agg;
use agg::PixelData;
use agg::Render;

use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::env;

fn parse_lion() -> (Vec<agg::PathStorage>, Vec<agg::Srgba8>){
    let txt = fs::read_to_string("tests/lion.txt").unwrap();
    let mut paths = vec![];
    let mut colors = vec![];
    let mut path = agg::PathStorage::new();
    //let mut color = agg::Srgba8::black();
    let mut color = agg::Srgba8::new(0,0,0,255);
    let mut cmd = agg::PathCommand::Stop;

    for line in txt.lines() {
        let v : Vec<_> = line.split_whitespace().collect();
        if v.len() == 1 {
            let n = 0;
            let hex = v[0];
            let r = u8::from_str_radix(&hex[n+0..n+2],16).unwrap();
            let g = u8::from_str_radix(&hex[n+2..n+4],16).unwrap();
            let b = u8::from_str_radix(&hex[n+4..n+6],16).unwrap();
            if path.vertices.len() > 0 {
                path.close_polygon();
                paths.push(path);
                colors.push(color);
            }
            path = agg::PathStorage::new();
            let rgb = agg::Rgba8::new(r,g,b,255);
            color =  agg::Srgba8::from_rgb(rgb);
            //color =  agg::Rgba8::new(r,g,b,255);
        } else {
            for val in v {
                if val == "M" {
                    cmd = agg::PathCommand::MoveTo;
                } else if val == "L" {
                    cmd = agg::PathCommand::LineTo;
                } else {
                    let pts : Vec<_> = val.split(",")
                        .map(|x| x.parse::<f64>().unwrap())
                        .collect();

                    match cmd {
                        agg::PathCommand::LineTo =>
                            path.line_to(pts[0], pts[1]),
                        agg::PathCommand::MoveTo => {
                            path.close_polygon();
                            path.move_to(pts[0], pts[1]);
                        }
                        _ => unreachable!("oh no !!!"),
                    }
                }
            }
        }
    }
    if path.vertices.len() > 0 {
        colors.push(color);
        path.close_polygon();
        paths.push(path);
    }
    assert_eq!(paths.len(), colors.len());
    paths.iter_mut().for_each(|p| p.arrange_orientations(agg::PathOrientation::Clockwise));
    (paths, colors)
}

fn ppm_names() -> (PathBuf,PathBuf) {
    let progname = env::args().next().unwrap();
    let progname = Path::new(&progname);
    let mut base = progname.file_stem().unwrap().to_string_lossy().into_owned();
    let n = base.rfind("-").unwrap();
    base.truncate(n);
    let ppm = Path::new(&base).with_extension("ppm");
    let test = Path::new("tests").join(ppm.clone());
    (ppm, test)
}


#[test]
fn lion_outline() {
    let (w,h) = (400,400);

    let (paths, colors) = parse_lion();
    let pixf = agg::Pixfmt::<agg::Rgb8>::new(w,h);
    let mut ren_base = agg::RenderingBase::new(pixf);
    //ren_base.clear( agg::Srgba8::new([255, 255, 255, 255]) );
    ren_base.clear( agg::Rgba8::new(255, 255, 255, 255) );
    let mut ren = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
    //ren.color( &agg::Srgba8::new([255,0,0,255]) );
    ren.color( &agg::Rgba8::new(255,0,0,255) );

    let mut ras = agg::RasterizerScanline::new();

    if paths.len() == 0 {
        return;
    }
    let p = paths[0].vertices[0];
    let mut r = agg::Rectangle::new(p.x,p.y,p.x,p.y);
    for p in &paths {
        if let Some(rp) = agg::bounding_rect(p) {
            //eprintln!("dx,dy: {:?}", rp);
            r.expand_rect(&rp);
        }
    }
    //eprintln!("dx,dy: {:?}", r);
    let g_base_dx = (r.x2 - r.x1)/2.0;
    let g_base_dy = (r.y2 - r.y1)/2.0;
    let mut mtx = agg::AffineTransform::new();
    eprintln!("dx,dy: {} {}", -g_base_dx, -g_base_dy);
    eprintln!("dx,dy: {} {}", (w/2) as f64, (h/2) as f64);
    mtx.translate(-g_base_dx, -g_base_dy);
    mtx.translate((w/2) as f64, (h/2) as f64);
    //mtx.translate(0.0, 0.0);
    let t : Vec<_> = paths.into_iter()
        .map(|p| agg::ConvTransform::new(p, mtx.clone()))
        .collect();
    println!("polygons: {}", t.len());

    let mut stroke : Vec<_> = t.into_iter()
        .map(|p| agg::ConvStroke::new( p ))
        .collect();
    stroke.iter_mut().for_each(|p| p.width(7.0));
    agg::render_all_paths(&mut ras, &mut ren, &stroke, &colors);

    let (ppm, test) = ppm_names();

    agg::ppm::write_ppm(&ren.pixeldata(), w, h, ppm.clone()).unwrap();
    agg::ppm::compare_ppm(ppm, test);

}
// compare -verbose -metric AE lion.ppm ./tests/lion.ppm diff.ppm
