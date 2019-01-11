//! Writing of PPM (Portable Pixmap Format) files
//!
//! See <https://en.wikipedia.org/wiki/Netpbm_format#PPM_example>
//!
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(filename: P) -> Result<(Vec<u8>,usize,usize),image::ImageError> {
    let img = image::open(filename)?.to_rgb(); // This should be changed
    let (w, h) = img.dimensions();
    let buf = img.into_raw();
    Ok((buf, w as usize, h as usize))
}
pub fn write_file<P: AsRef<Path>>(buf: &[u8], width: usize, height: usize, filename: P) -> Result<(), std::io::Error> {
    image::save_buffer(filename, buf, width as u32, height as u32, image::RGB(8))
}

pub fn img_diff<P: AsRef<Path>>(f1: P, f2: P) -> Result<bool,image::ImageError> {
    let (d1,w1,h1) = read_file(f1)?;
    let (d2,w2,h2) = read_file(f2)?;
    if w1 != w2 || h1 != h2 {
        return Ok(false);
    }
    if d1.len() != d2.len() {
        println!("files not equal length");
        return Ok(false);
    }
    let mut flag = true;
    for (i,(v1,v2)) in d1.iter().zip(d2.iter()).enumerate() {
        if v1 != v2 {
            println!("{} [{},{},{}]: {} {}", i, (i/3)%w1,(i/3)/w1,i%3, v1,v2);
            flag = false;
        }
    }
    Ok(flag)
}
