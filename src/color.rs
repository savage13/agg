//! Colors

use std::ops::Deref;

use crate::Color;
use crate::math::multiply_u8;

/// Convert an f64 [0,1] component to a u8 [0,255] component
pub fn cu8(v: f64) -> u8 {
    (v * 255.0 + 0.5) as u8
}

pub fn cu8r<C: Color>(c: &C) -> u8 { cu8(c.red())   }
pub fn cu8g<C: Color>(c: &C) -> u8 { cu8(c.green()) }
pub fn cu8b<C: Color>(c: &C) -> u8 { cu8(c.blue())  }

/// Convert from sRGB to RGB for a single component
pub fn srgb_to_rgb(x: f64) -> f64 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
/// Convert from RGB to sRGB for a single component
pub fn rgb_to_srgb(x: f64) -> f64 {
    if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0/2.4) - 0.055
    }
}


/// Color as Red, Green, Blue, and Alpha
#[derive(Debug,Default,Copy,Clone)]
pub struct Rgba8 {
    /// Red
    pub r: u8,
    /// Green
    pub g: u8,
    /// Blue
    pub b: u8,
    /// Alpha
    pub a: u8,
}

impl Rgba8 {
    /// White Color (255,255,255,255)
    pub fn white() -> Self {
        Self::new(255,255,255,255)
    }
    /// Black Color (0,0,0,255)
    pub fn black() -> Self {
        Self::new(0,0,0,255)
    }
    /// Create new color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Rgba8 { r, g, b, a }
    }
    /// Crate new color from a wavelength and gamma 
    pub fn from_wavelength_gamma(w: f64, gamma: f64) -> Self {
        Rgb8::from_wavelength_gamma(w, gamma).into()
    }
    pub fn clear(&mut self) {
        self.r = 0;
        self.g = 0;
        self.b = 0;
        self.a = 0;
    }
    pub fn premultiply(&self) -> Rgba8pre {
        //if self.a == 158 {
        //    eprint!("COLOR {} {} {} {} => ", self.r, self.g, self.b, self.a);
        //}
        match self.a {
            255 => {
                //eprintln!("{} {} {} {} PREMULTIPLY", self.r, self.g, self.b, self.a);
                Rgba8pre::new(self.r, self.g, self.b, self.a)
            },
            0   => {
                //eprintln!("{} {} {} {} PREMULTIPLY", 0,0,0, self.a);
                Rgba8pre::new(0, 0, 0, self.a)
            },
            _   => {
                let r = multiply_u8(self.r, self.a);
                let g = multiply_u8(self.g, self.a);
                let b = multiply_u8(self.b, self.a);
                //if self.a == 158 {
                //    eprintln!("{} {} {} PREMULTIPLY ", r, g, b);
                //}
                Rgba8pre::new(r, g, b, self.a)
            }
        }
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
    fn is_premultiplied(&self) -> bool { false }
}

impl From<Rgba8> for Rgb8 {
    fn from(c: Rgba8) -> Rgb8 {
        Rgb8::new( c.r, c.g, c.b )
    }
}
impl From<Rgb8> for Rgba8 {
    fn from(c: Rgb8) -> Rgba8 {
        Rgba8::new( c.r, c.g, c.b, 255 )
    }
}

/// Gray scale
#[derive(Debug,Copy,Clone)]
pub struct Gray8(u8);
impl Deref for Gray8 {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}
impl Gray8 {
    /// Create a new gray scale value
    pub fn new(g: u8) -> Self {
        Gray8( g )
    }
}


impl Rgb8 {
    pub fn white() -> Self {
        Self::new(255,255,255)
    }
    pub fn black() -> Self {
        Self::new(0,0,0)
    }
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb8 { r, g, b }
    }
    pub fn gray(g: u8) -> Self {
        Self::new(g,g,g)
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
        Self::new ( r as u8, g as u8, b as u8 )
    }
}

fn color_u8_to_f64(x: u8) -> f64 {
    f64::from(x) / 255.0
}

impl Color for Rgb8 {
    fn   red(&self) -> f64 { color_u8_to_f64(self.r) }
    fn green(&self) -> f64 { color_u8_to_f64(self.g) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.b) }
    fn alpha(&self) -> f64 { 1.0 }
    fn alpha8(&self) -> u8 { 255 }
    fn red8(&self) -> u8   { self.r }
    fn green8(&self) -> u8 { self.g }
    fn blue8(&self) -> u8  { self.b }
    fn is_premultiplied(&self) -> bool { false }
}
impl Color for Rgba8pre {
    fn   red(&self) -> f64 { color_u8_to_f64(self.r) }
    fn green(&self) -> f64 { color_u8_to_f64(self.g) }
    fn  blue(&self) -> f64 { color_u8_to_f64(self.b) }
    fn alpha(&self) -> f64 { color_u8_to_f64(self.a) }
    fn alpha8(&self) -> u8 { self.a }
    fn red8(&self) -> u8   { self.r }
    fn green8(&self) -> u8 { self.g }
    fn blue8(&self) -> u8  { self.b }
    fn is_premultiplied(&self) -> bool { true }
    fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

/// Color as Red, Green, Blue
#[derive(Debug,Default,Copy,Clone)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
/// Color as Red, Green, Blue, and Alpha with pre-multiplied components
#[derive(Debug,Default,Copy,Clone)]
pub struct Rgba8pre {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8pre {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {r, g, b, a}
    }
}

/// Color as standard Red, Green, Blue, Alpha
///
/// See <https://en.wikipedia.org/wiki/SRGB>
///
#[derive(Debug,Default,Copy,Clone)]
pub struct Srgba8 {
    /// Red
    r: u8,
    /// Green
    g: u8,
    /// Blue
    b: u8,
    /// Alpha
    a: u8,
}

impl Srgba8 {
    /// Create a new Srgba8 color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
impl Color for Srgba8 {
    fn   red(&self)  -> f64 { srgb_to_rgb(color_u8_to_f64(self.r)) }
    fn green(&self)  -> f64 { srgb_to_rgb(color_u8_to_f64(self.g)) }
    fn  blue(&self)  -> f64 { srgb_to_rgb(color_u8_to_f64(self.b)) }
    fn alpha(&self)  -> f64 { color_u8_to_f64(self.a) }

    fn alpha8(&self) -> u8  { cu8(self.alpha()) }
    fn red8(&self)   -> u8  { cu8(self.red()) }
    fn green8(&self) -> u8  { cu8(self.green()) }
    fn blue8(&self)  -> u8  { cu8(self.blue()) }

    fn is_premultiplied(&self) -> bool { false }
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
/*impl<'a> From<&'a Srgba8> for Rgba8 {
    fn from(c: &Srgba8) -> Self {
        let r = cu8(srgb_to_rgb(c.red()));
        let g = cu8(srgb_to_rgb(c.green()));
        let b = cu8(srgb_to_rgb(c.blue()));
        Self::new(r,g,b,c.a)
    }
}*/
impl From<Srgba8> for Rgba8 {
    fn from(c: Srgba8) -> Self {
        let r = c.red8();
        let g = c.green8();
        let b = c.blue8();
        Self::new(r,g,b,c.a)
    }
}
impl From<Srgba8> for Rgba32 {
    fn from(c: Srgba8) -> Self {
        let r = c.red() as f32;
        let g = c.green() as f32;
        let b = c.blue() as f32;
        let a = c.alpha() as f32;
        Self::new(r,g,b,a)
    }
}

impl<'a, C> From<&'a C> for Rgba8 where C: Color {
    fn from(c: &C) -> Self {
        //eprintln!("YCOLOR: Convert from color to rgba8: {:?}", c);
        Self::new(c.red8(), c.green8(), c.blue8(), c.alpha8() )
    }
}
impl<'a, C> From<&'a C> for Rgb8 where C: Color {
    fn from(c: &C) -> Self {
        Self::new(c.red8(), c.green8(), c.blue8())
    }
}
impl<'a, C> From<&'a C> for Rgba32 where C: Color {
    fn from(c: &C) -> Self {
        Self::new(c.red() as f32, c.green() as f32, c.blue() as f32, c.alpha() as f32 )
    }
}
/*
impl<'a, C> From<&'a C> for Rgba8pre where C: Color {
    fn from(c: &C) -> Self {
        //eprintln!("YCOLOR Convert from color to rgba8pre {}", c.alpha8());
        if c.is_premultiplied() {
            Self::new(c.red8(), c.green8(), c.blue8(), c.alpha8())
        } else {
            let r = multiply_u8(c.red8(),   c.alpha8());
            let g = multiply_u8(c.green8(), c.alpha8());
            let b = multiply_u8(c.blue8(),  c.alpha8());
            let v = Self::new(r,g,b, c.alpha8());
            //eprintln!("YCOLOR Convert from color to rgba8pre {:?}", v);
            v
        }
    }
}
*/

#[derive(Debug)]
pub struct Rgba32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba32 {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn premultiply(&self) -> Self {
        if self.a == 1.0 {
            Rgba32::new(self.r, self.g, self.b, self.a)
        } else if self.a == 0.0 {
            Rgba32::new(0., 0., 0., self.a)
        } else {
            let r = self.r * self.a;
            let g = self.g * self.a;
            let b = self.b * self.a;
            Rgba32::new(r, g, b, self.a)
        }
    }
}

impl Color for Rgba32 {
    fn   red(&self) -> f64 { self.r as f64 }
    fn green(&self) -> f64 { self.g as f64 }
    fn  blue(&self) -> f64 { self.b as f64 }
    fn alpha(&self) -> f64 { self.a as f64 }
    fn alpha8(&self) -> u8 { cu8(self.a as f64) }
    fn red8(&self) -> u8 { cu8(self.r as f64) }
    fn green8(&self) -> u8 { cu8(self.g as f64) }
    fn blue8(&self) -> u8 { cu8(self.b as f64) }
    fn is_premultiplied(&self) -> bool { false }
}
