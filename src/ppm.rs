//! Writing of PPM (Portable Pixmap Format) files
//!
//! See <https://en.wikipedia.org/wiki/Netpbm_format#PPM_example>
//!
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

/// Compare two PPM files, panic'ing on a difference
pub fn compare_ppm<P: AsRef<Path>>(f1: P, f2: P) {
    let d1 = fs::read(f1).expect("Unable to read file");
    let d2 = fs::read(f2).expect("Unable to read file");
    for (i,(v1,v2)) in d1.iter().zip(d2.iter()).enumerate() {
        if v1 != v2 {
            eprintln!("{}: {} {}", i, v1, v2);
            assert_eq!(v1,v2);
        }
    }
}
/// Write a PPM file
///
/// P6 - Binary Portable Pixmap (0-255 RGB data)
///
/// PPM uses 24 bits per pixel: 8 for red, 8 for green, 8 for blue.
///
/// Width and Height are specified, then the maximum value for any color
///
/// Then the data in row-major order (C-format)
///
pub fn write_ppm<P: AsRef<Path>>(buf: &[u8], width: usize, height: usize, filename: P) -> Result<(),std::io::Error> {
    let mut fd = File::create(filename)?;
    write!(fd, "P6 {} {} 255 ", width, height)?;
    fd.write_all(buf)?;
    Ok(())
}

pub fn read_file<P: AsRef<Path>>(filename: P) -> Result<(Vec<u8>,usize,usize),image::ImageError> {
    use image::GenericImageView;
    let img = image::open(filename)?;
    let (w, h) = img.dimensions();
    let buf = img.raw_pixels();
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
    for (i,(v1,v2)) in d1.iter().zip(d2.iter()).enumerate() {
        if v1 != v2 {
            eprintln!("{}: {} {}", i, v1, v2);
            return Ok(false);
        }
    }
    Ok(true)
}
