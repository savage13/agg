use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;


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

pub fn write_ppm<P: AsRef<Path>>(buf: &[u8], width: usize, height: usize, filename: P) -> Result<(),std::io::Error> {
    let mut fd = File::create(filename)?;
    write!(fd, "P6 {} {} 255 ", width, height);
    fd.write_all(buf)?;
    Ok(())
}

