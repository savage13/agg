
use std::ops::Deref;

pub trait Color: std::fmt::Debug {
    fn red(&self) -> f64;
    fn green(&self) -> f64;
    fn blue(&self) -> f64;
    fn alpha(&self) -> f64;
    fn is_transparent(&self) -> bool { self.alpha() == 0.0 }
    fn is_opaque(&self) -> bool { self.alpha() >= 1.0 }
    fn alpha8(&self) -> u8;
    fn red8(&self) -> u8;
    fn green8(&self) -> u8;
    fn blue8(&self) -> u8;
}

pub fn cu8(v: f64) -> u8 {
    (v * 255.0).round() as u8
}

pub fn cu8r<C: Color>(c: &C) -> u8 { cu8(c.red())   }
pub fn cu8g<C: Color>(c: &C) -> u8 { cu8(c.green()) }
pub fn cu8b<C: Color>(c: &C) -> u8 { cu8(c.blue())  }

pub fn srgb_to_rgb(x: f64) -> f64 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
pub fn rgb_to_srgb(x: f64) -> f64 {
    if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0/2.4) - 0.055
    }
}



#[derive(Debug,Default,Copy,Clone)]
pub struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba8 {
    pub fn white() -> Self {
        Self::new(255,255,255,255)
    }
    pub fn black() -> Self {
        Self::new(0,0,0,255)
    }
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Rgba8 { r, g, b, a }
    }
    pub fn from_wavelength_gamma(w: f64, gamma: f64) -> Self {
        let c = &*Rgb8::from_wavelength_gamma(w, gamma);
        Self::new(c[0],c[1],c[2],255)
    }
}

impl Color for Rgba8 {
    fn   red(&self) -> f64 { color_u8_to_f64(self.r) }
    fn green(&self) -> f64 { color_u8_to_f64(self.g) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.b) }
    fn alpha(&self) -> f64 { color_u8_to_f64(self.a) }
    fn alpha8(&self) -> u8 { self.a }
    fn red8(&self) -> u8 { self.r }
    fn green8(&self) -> u8 { self.g }
    fn blue8(&self) -> u8 { self.b }
}

impl From<Rgba8> for Rgb8 {
    fn from(c: Rgba8) -> Rgb8 {
        Rgb8::new( [c.r, c.g, c.b] )
    }
}

#[derive(Debug,Copy,Clone)]
pub struct Gray8(u8);
impl Deref for Gray8 {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}
impl Gray8 {
    pub fn new(g: u8) -> Self {
        Gray8( g )
    }
}


impl Rgb8 {
    pub fn white() -> Self {
        Self::new([255,255,255])
    }
    pub fn black() -> Self {
        Self::new([0,0,0])
    }
    pub fn new(rgb: [u8; 3]) -> Self {
        Rgb8 ( rgb )
    }
    pub fn gray(g: u8) -> Self {
        Self::new([g,g,g])
    }
    pub fn from_wavelength_gamma(w: f64, gamma: f64) -> Self {
        let (r,g,b) =
            if w >= 380.0 && w <= 440.0 {
                (-1.0 * (w-440.0) / (440.0-380.0), 0.0, 1.0)
            } else if w >= 440.0 && w <= 490.0 {
                (0.0, (w-440.0)/(490.0-440.0), 1.0)
            } else if w >= 490.0 && w <= 510.0 {
                (0.0, 1.0, -1.0 * (w-510.0)/(510.0-490.0))
            } else if w >= 510.0 && w <= 580.0 {
                ((w-510.0)/(580.0-510.0), 1.0, 0.0)
            } else if w >= 580.0 && w <= 645.0 {
                (1.0, -1.0*(w-645.0)/(645.0-580.0), 0.0)
            } else if w >= 645.0 && w <= 780.0 {
                (1.0, 0.0, 0.0)
            } else {
                (0.,0.,0.)
            };
        let scale =
            if w > 700.0 {
                0.3 + 0.7 * (780.0-w)/(780.0-700.0)
            } else if w < 420.0 {
                0.3 + 0.7 * (w-380.0)/(420.0-380.0)
            } else {
                1.0
            };
        let r = (r * scale).powf(gamma) * 255.0;
        let g = (g * scale).powf(gamma) * 255.0;
        let b = (b * scale).powf(gamma) * 255.0;
        Self::new ( [r as u8, g as u8, b as u8] )
    }
}

fn color_u8_to_f64(x: u8) -> f64 {
    f64::from(x) / 255.0
}

impl Color for Rgb8 {
    fn   red(&self) -> f64 { color_u8_to_f64(self.0[0]) }
    fn green(&self) -> f64 { color_u8_to_f64(self.0[1]) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.0[2]) }
    fn alpha(&self) -> f64 { 1.0 }
    fn alpha8(&self) -> u8 { 255 }
    fn red8(&self) -> u8 { self.0[0] }
    fn green8(&self) -> u8 { self.0[1] }
    fn blue8(&self) -> u8 { self.0[2] }
}


#[derive(Debug,Default,Copy,Clone)]
pub struct Rgb8([u8;3]);
impl Deref for Rgb8 {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug,Default,Copy,Clone)]
pub struct Srgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Srgba8 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
impl Color for Srgba8 {
    fn   red(&self) -> f64 { color_u8_to_f64(self.r) }
    fn green(&self) -> f64 { color_u8_to_f64(self.g) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.b) }
    fn alpha(&self) -> f64 { color_u8_to_f64(self.a) }
    fn alpha8(&self) -> u8 { self.a }
    fn red8(&self) -> u8 { self.r }
    fn green8(&self) -> u8 { self.g }
    fn blue8(&self) -> u8 { self.b }
}


impl From<Rgba8> for Srgba8 {
    fn from(c: Rgba8) -> Self {
        let r = cu8(rgb_to_srgb(c.red()));
        let g = cu8(rgb_to_srgb(c.green()));
        let b = cu8(rgb_to_srgb(c.blue()));
        Self::new(r,g,b,cu8(c.alpha()))
    }
}
impl<'a> From<&'a Rgba8> for Srgba8 {
    fn from(c: &Rgba8) -> Self {
        let r = cu8(rgb_to_srgb(c.red()));
        let g = cu8(rgb_to_srgb(c.green()));
        let b = cu8(rgb_to_srgb(c.blue()));
        Self::new(r,g,b,c.a)
    }
}
impl<'a> From<&'a Srgba8> for Rgba8 {
    fn from(c: &Srgba8) -> Self {
        let r = cu8(srgb_to_rgb(c.red()));
        let g = cu8(srgb_to_rgb(c.green()));
        let b = cu8(srgb_to_rgb(c.blue()));
        Self::new(r,g,b,c.a)
    }
}
impl From<Srgba8> for Rgba8 {
    fn from(c: Srgba8) -> Self {
        let r = cu8(srgb_to_rgb(c.red()));
        let g = cu8(srgb_to_rgb(c.green()));
        let b = cu8(srgb_to_rgb(c.blue()));
        Self::new(r,g,b,c.a)
    }
}

