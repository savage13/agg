//! Pixel Format

use crate::buffer::RenderingBuffer;
use crate::blend;
use crate::color::*;
use crate::math::*;

use crate::Color;
use crate::Source;
use crate::Pixel;
use crate::PixelDraw;
use crate::PixelData;

use std::marker::PhantomData;

/// Pixel Format Wrapper around raw pixel component data
///
pub struct Pixfmt<T> {
    rbuf: RenderingBuffer,
    phantom: PhantomData<T>,
}

impl<T> Pixfmt<T> where Pixfmt<T>: PixelDraw {
    /// Create new Pixel Format of width * height * bpp
    ///
    /// Allocates memory of width * height * bpp
    pub fn new(width: usize, height: usize) -> Self {
        if width == 0 || height == 0 {
            panic!("Cannot create pixfmt with 0 width or height");
        }
        Self { rbuf: RenderingBuffer::new(width, height, Self::bpp()),
               phantom: PhantomData
        }
    }
    /// Size of Rendering Buffer in bytes; width * height * bpp
    pub fn size(&self) -> usize {
        self.rbuf.len()
    }
    /// Clear the Image
    ///
    /// All color components are set to 255, including `alpha` if present
    ///
    ///     use agg::{Source,Pixfmt,Rgb8,Rgba8};
    ///
    ///     // Pixfmt with Rgb8, not Alpha Component
    ///     let mut pix = Pixfmt::<Rgb8>::new(2,2);
    ///     pix.clear();
    ///     let empty = Rgba8 { r:255, g:255, b:255, a:255};
    ///     assert_eq!(pix.get((0,0)), empty);
    ///     assert_eq!(pix.get((0,1)), empty);
    ///     assert_eq!(pix.get((1,0)), empty);
    ///     assert_eq!(pix.get((1,1)), empty);
    ///
    ///     // Pixfmt with Rgba8, including Alpha Component
    ///     let mut pix = Pixfmt::<Rgb8>::new(2,2);
    ///     pix.clear();
    ///     let empty = Rgba8 { r:255, g:255, b:255, a:255};
    ///     assert_eq!(pix.get((0,0)), empty);
    ///     assert_eq!(pix.get((0,1)), empty);
    ///     assert_eq!(pix.get((1,0)), empty);
    ///     assert_eq!(pix.get((1,1)), empty);
    ///
    pub fn clear(&mut self) {
        self.rbuf.clear();
    }
    //pub fn from(rbuf: RenderingBuffer) -> Self {
    //    Self { rbuf, phantom: PhantomData }
    //}
    /// Copies the [Color] `c` to pixel at (`x`,`y`)
    ///
    /// Locations outside of the region are igorned
    ///
    ///     use agg::{Source,Pixfmt,Rgba8};
    ///
    ///     let mut pix = Pixfmt::<Rgba8>::new(1,2);
    ///     let black = Rgba8::black();
    ///     pix.copy_pixel(0,1, black);
    ///     assert_eq!(pix.get((0,0)), Rgba8{r:0, g:0, b:0, a:0});
    ///     assert_eq!(pix.get((0,1)), black);
    ///
    ///     pix.copy_pixel(10,10, black); // Ignored, outside of range
    ///
    /// [Color]: ../trait.Color.html
    pub fn copy_pixel<C: Color>(&mut self, x: usize, y: usize, c: C) {
        if x >= self.rbuf.width || y >= self.rbuf.height {
            return;
        }
        self.set((x,y), c);
    }
    /// Copies the [Color] `c` to pixels from (`x`,`y`) to (`x+n-1`,y)
    ///
    /// Locations outside of the region are ignored
    ///
    ///     use agg::{Source,Pixfmt,Rgb8,Rgba8};
    ///
    ///     let mut pix = Pixfmt::<Rgb8>::new(10,1);
    ///     let black = Rgba8::black();
    ///     pix.copy_hline(0,0,10, black);
    ///     assert_eq!(pix.get((0,0)), black);
    ///     assert_eq!(pix.get((1,0)), black);
    ///     assert_eq!(pix.get((9,0)), black);
    ///
    ///     pix.copy_hline(1,1,10, black); // Ignored, outside of range
    ///
    /// [Color]: ../trait.Color.html
    pub fn copy_hline<C: Color>(&mut self, x: usize, y: usize, n: usize, c: C) {
        if y >= self.rbuf.height || x >= self.rbuf.width || n == 0 {
            return;
        }
        let n = if x+n >= self.rbuf.width { self.rbuf.width - x } else { n };
        for i in 0 .. n {
            self.set((x+i,y), c);
        }
    }
    /// Copies the [Color] `c` to pixels from (`x`,`y`) to (`x`,`y+n-1`)
    ///
    /// Locations outside of the region are ignored
    ///
    ///     use agg::{Source,Pixfmt,Rgba8,Rgba32};
    ///
    ///     let mut pix = Pixfmt::<Rgba32>::new(1,10);
    ///     let black  = Rgba32::new(0.,0.,0.,1.);
    ///     pix.copy_vline(0,0,10, black);
    ///
    ///     let black8 = Rgba8::from_trait(black); // pix.get() returns Rgba8
    ///     assert_eq!(pix.get((0,0)), black8);
    ///     assert_eq!(pix.get((0,1)), black8);
    ///     assert_eq!(pix.get((0,9)), black8);
    ///
    ///     pix.copy_vline(1,1,10, black); // Ignored, outside of range
    ///
    /// [Color]: ../trait.Color.html
    /// [Rgba8]: ../Color/struct.Rgba8.html
    pub fn copy_vline<C: Color>(&mut self, x: usize, y: usize, n: usize, c: C) {
        if y >= self.rbuf.height || x >= self.rbuf.width || n == 0 {
            return;
        }
        let n = if y+n >= self.rbuf.height { self.rbuf.height - y } else { n };
        for i in 0 .. n {
            self.set((x,y+i), c);
        }
    }
    /// Draw a line from `(x1,y1)` to `(x2,y2)` of color `c`
    ///
    /// Uses [Xiaolin Wu's line drawing algorithm](https://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm) which include Anti-Aliasing
    ///
    #[deprecated(since="0.1.0", note="please use `path_storage` and friends instead")]
    pub fn line_sp_aa(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        let steep = (x2-x1).abs() < (y2-y1).abs();
        let (x1,y1,x2,y2) = if steep   { (y1,x1,y2,x2) } else { (x1,y1,x2,y2) };
        let (x1,y1,x2,y2) = if x2 < x1 { (x2,y2,x1,y1) } else { (x1,y1,x2,y2) };
        let dx = x2-x1;
        let dy = y2-y1;
        let gradient = if dx.abs() <= 1e-6 { 1.0 } else { dy/dx };

        let white = Rgb8::white();
        // Handle First Endpoint
        let (_xend, yend, _xgap, xpx11, ypx11, v1, v2) = endpoint(x1,y1,gradient);
        let v1 = blend(c, white, v1);
        let v2 = blend(c, white, v2);
        if steep {
            self.set((ypx11,  xpx11), v1);
            self.set((ypx11+1,xpx11), v2);
        } else {
            self.set((xpx11,  ypx11),  v1);
            self.set((xpx11,  ypx11+1),v2);
        }
        let mut intery = yend + gradient;
        // Handle Second Endpoint

        let (_xend, _yend, _xgap, xpx12, ypx12, v1, v2) = endpoint(x2,y2,gradient);
        let v1 = blend(c, white, v1);
        let v2 = blend(c, white, v2);
        if steep {
            self.set((ypx12,  xpx12),   v1);
            self.set((ypx12+1,xpx12),   v2);
        } else {
            self.set((xpx12,  ypx12),   v1);
            self.set((xpx12,  ypx12+1), v2);
        }
        // In Between Points
        for xp in xpx11 + 1 .. xpx12 {
            let yp = ipart(intery) as usize;
            let (p0,p1) = if steep { ((yp,xp),(yp+1,xp)) } else { ((xp,yp),(xp,yp+1)) };

            let (v1,v2) = ( rfpart(intery), fpart(intery) );
            //let v0 = blend(c, self.get(p0), v1);
            //let v1 = blend(c, self.get(p1), v2);
            //self.set(p0,&v0);
            //self.set(p1,&v1);
            self.blend_pix(p0, c, (v1*255.) as u64);
            self.blend_pix(p1, c, (v2*255.) as u64);

            intery += gradient;
        }
    }

    /// Draw a line from `(x1,y1)` to `(x2,y2)` of color `c` using a
    ///   subpixel algorithm
    ///
    /// Line is Aliased (not-anti-aliased)
    ///
    #[deprecated(since="0.1.0", note="please use `path_storage` and friends instead")]
    pub fn line_sp(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        //println!("({}, {}) - ({}, {})", x1,y1,x2,y2);
        let x1 = (x1 * 256.0).round() as i64 / 256;
        let y1 = (y1 * 256.0).round() as i64 / 256;
        let x2 = (x2 * 256.0).round() as i64 / 256;
        let y2 = (y2 * 256.0).round() as i64 / 256;
        //println!("   ({}, {}) - ({}, {})", x1,y1,x2,y2);

        let steep = (x2-x1).abs() < (y2-y1).abs();
        let (x1,y1,x2,y2) = if steep   { (y1,x1,y2,x2) } else { (x1,y1,x2,y2) };
        let (x1,y1,x2,y2) = if x2 < x1 { (x2,y2,x1,y1) } else { (x1,y1,x2,y2) };

        let count = (x2-x1).abs();
        let count = std::cmp::max(count, 1);
        let dy = y2-y1;

        let mut left = dy / count;
        let mut rem  = dy % count;
        let mut xmod = rem;
        let mut y = y1;
        //println!("   count, left, rem, dy: {} {} {} {}", count, left, rem, dy);
        if xmod <= 0 {
            xmod += count;
            rem  += count;
            left -= 1;
        }
        xmod -= count;

        for x in x1..x2 {
            if steep {
                self.set((y as usize, x as usize), c);
            } else {
                self.set((x as usize, y as usize), c);
            }
            xmod += rem;
            y += left;
            if xmod > 0 {
                xmod -= count;
                y += 1;
            }
        }
    }

    /// Draw a line from `(x1,y1)` to `(x2,y2)` of color `c`
    ///
    /// Uses [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
    ///
    ///
    #[deprecated(since="0.1.0", note="please use `path_storage` and friends instead")]
    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        let steep = (y2-y1).abs() > (x2-x1).abs();

        let (x1,y1,x2,y2) = if steep { (y1,x1,y2,x2) } else { (x1,y1,x2,y2) };
        let (x1,y1,x2,y2) = if x1>x2 { (x2,y2,x1,y1) } else { (x1,y1,x2,y2) };
        let dx = x2-x1;
        let dy = (y2-y1).abs();
        let mut error = dx / 2.0;

        let pos   = y1<y2;
        let mut y = y1.floor() as usize;
        let x1    = x1.floor() as usize;
        let x2    = x2.floor() as usize;
        for x in x1 .. x2 {
            if steep {
                self.set((y,x), c);
            } else {
                self.set((x,y), c);
            }
            error -= dy;
            if error <= 0.0 {
                y = if pos { y+1 } else { y-1 };
                error += dx;
            }
        }
    }
}

/// Access Pixeldata from a Pixfmt<T>
///
impl<T> PixelData for Pixfmt<T> {
    fn pixeldata(&self) -> &[u8] {
        & self.rbuf.data
    }
}

impl<T> PixelDraw for Pixfmt<T> where Pixfmt<T> : Pixel { }

impl Source for Pixfmt<Rgba8> {
    fn get(&self, id: (usize, usize)) -> Rgba8 {
        let p = &self.rbuf[id];
        Rgba8::new(p[0],p[1],p[2],p[3])
    }
}
impl Source for Pixfmt<Rgba8pre> {
    fn get(&self, id: (usize, usize)) -> Rgba8 {
        let p = &self.rbuf[id];
        //eprintln!("COLOR ({},{}) : {} {} {})", id.0,id.1,p[0],p[1],p[2]);
        Rgba8::new(p[0],p[1],p[2],p[3])
    }
}
impl Source for Pixfmt<Rgb8> {
    fn get(&self, id: (usize, usize)) -> Rgba8 {
        let p = &self.rbuf[id];
        Rgba8::new(p[0],p[1],p[2],255)
    }
}
impl Source for Pixfmt<Rgba32> {
    fn get(&self, id: (usize, usize)) -> Rgba8 {
        //let n = (id.0 + id.1 * self.rbuf.width) * Pixfmt::<Rgba32>::bpp();
        let p = &self.rbuf[id];
        //eprintln!("GET {:?}", &p[..16]);
        let red   : f32 = unsafe { std::mem::transmute::<[u8;4],f32>([p[0],p[1],p[2],p[3]]) };
        let green : f32 = unsafe { std::mem::transmute::<[u8;4],f32>([p[4],p[5],p[6],p[7]]) };
        let blue  : f32 = unsafe { std::mem::transmute::<[u8;4],f32>([p[8],p[9],p[10],p[11]]) };
        let alpha : f32 = unsafe { std::mem::transmute::<[u8;4],f32>([p[12],p[13],p[14],p[15]]) };
        //eprintln!("GET: {} {} {} {}", r,g,b,a);
        //eprintln!("GET {:?}", Rgba32::new(r,g,b,a));
        let c = Rgba32::new(red,green,blue,alpha);
        Rgba8::from_trait(c)
    }
}

impl Pixel for Pixfmt<Rgba8> {
    fn bpp() -> usize { 4 }
    fn cover_mask() -> u64 { 255 }
    /// Height of rendering buffer in pixels
    fn height(&self) -> usize {
        self.rbuf.height
    }
    /// Width of rendering buffer in pixels
    fn width(&self) -> usize {
        self.rbuf.width
    }
    fn set<C: Color>(&mut self, id: (usize, usize), c: C) {
        let c = Rgba8::from_trait(c);
        assert!(! self.rbuf.data.is_empty() );
        self.rbuf[id][0] = c.red8();
        self.rbuf[id][1] = c.green8();
        self.rbuf[id][2] = c.blue8();
        self.rbuf[id][3] = c.alpha8();
    }
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64) {
        println!("blend_pix: {:?} {:?} {}", self.get(id), c, cover);
        let alpha = multiply_u8(c.alpha8(), cover as u8);
        let pix0 = self.get(id); // Rgba8
        let pix  = self.mix_pix(pix0, Rgba8::from_trait(c), alpha);
        self.set(id, pix);
    }
}

impl Pixel for Pixfmt<Rgb8> {
    fn set<C: Color>(&mut self, id: (usize, usize), c: C) {
        let c = Rgb8::from_trait(c);
        self.rbuf[id][0] = c.red8();
        self.rbuf[id][1] = c.green8();
        self.rbuf[id][2] = c.blue8();
    }
    fn bpp() -> usize { 3 }
    fn cover_mask() -> u64 { 255 }
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64) {
        //eprintln!("BLEND PIX rgb8 in  {:?} cover {}", c, cover);
        let pix0 = self.raw(id);
        //eprintln!("BLEND PIX rgb8 cur {:?}", c);
        let pix  = self.mix_pix(pix0, Rgb8::from_trait(c), c.alpha8(), cover);
        self.set(id, pix);
    }
    /// Height of rendering buffer in pixels
    fn height(&self) -> usize {
        self.rbuf.height
    }
    /// Width of rendering buffer in pixels
    fn width(&self) -> usize {
        self.rbuf.width
    }
}
impl Pixfmt<Gray8> {
    fn mix_pix(&mut self, id: (usize,usize), c: Gray8, alpha: u8) -> Gray8 {
        let p = Gray8::from_slice( &self.rbuf[id] );
        Gray8::new_with_alpha(lerp_u8(p.value, c.value, alpha), alpha)
    }
    pub fn raw(&self, id: (usize,usize)) -> Gray8 {
        Gray8::from_slice( &self.rbuf[id] )
    }
}

impl Pixfmt<Rgba8> {
    fn mix_pix(&mut self, p: Rgba8, c: Rgba8, alpha: u8) -> Rgba8 {
        let red   =    lerp_u8(p.r, c.r, alpha);
        let green =    lerp_u8(p.g, c.g, alpha);
        let blue  =    lerp_u8(p.b, c.b, alpha);
        let alpha =    prelerp_u8(p.a, alpha, alpha);
        Rgba8::new(red, green, blue, alpha)
    }
    fn _blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64) {
        let alpha = multiply_u8(c.alpha8(), cover as u8);
        let pix0 = self.get(id);
        let pix  = self.mix_pix(pix0, Rgba8::from_trait(c), alpha);
        self.set(id, pix);
    }
}
impl Pixel for Pixfmt<Rgba8pre> {
    fn set<C: Color>(&mut self, id: (usize, usize), c: C) {
        //let c = Rgba8pre::from(c);
        self.rbuf[id][0] = c.red8();
        self.rbuf[id][1] = c.green8();
        self.rbuf[id][2] = c.blue8();
        self.rbuf[id][3] = c.alpha8();
    }
    fn bpp() -> usize { 4 }
    fn cover_mask() -> u64 { 255 }
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64) {
        let p = self.get(id);
        let p0 = Rgba8pre::new(p.red8(), p.green8(), p.blue8(), p.alpha8());
        let c0 = Rgba8pre::new(c.red8(), c.green8(), c.blue8(), c.alpha8());
        let p  = self.mix_pix(p0, c0, c.alpha8(), cover);
        self.set(id, p);
    }
    /// Height of rendering buffer in pixels
    fn height(&self) -> usize {
        self.rbuf.height
    }
    /// Width of rendering buffer in pixels
    fn width(&self) -> usize {
        self.rbuf.width
    }

}

impl Pixfmt<Rgb8> {
    pub fn raw(&self, id: (usize, usize)) -> Rgb8 {
         let p = &self.rbuf[id];
        Rgb8::new(p[0],p[1],p[2])
    }
    fn mix_pix(&mut self, p: Rgb8, c: Rgb8, alpha: u8, cover: u64) -> Rgb8 {
        let alpha = multiply_u8(alpha, cover as u8);
        let red   = lerp_u8(p.r, c.r, alpha);
        let green = lerp_u8(p.g, c.g, alpha);
        let blue  = lerp_u8(p.b, c.b, alpha);
        Rgb8::new(red, green, blue)
    }
}
impl Pixfmt<Rgba8pre> {
    fn mix_pix(&mut self, p: Rgba8pre, c: Rgba8pre, alpha: u8, cover: u64) -> Rgba8pre {
        let alpha = multiply_u8(alpha, cover as u8);
        let red   = multiply_u8(c.r, cover as u8);
        let green = multiply_u8(c.g, cover as u8);
        let blue  = multiply_u8(c.b, cover as u8);

        let red   = prelerp_u8(p.r, red,   alpha);
        let green = prelerp_u8(p.g, green, alpha);
        let blue  = prelerp_u8(p.b, blue,  alpha);
        let alpha = prelerp_u8(p.a, alpha,  alpha);
        Rgba8pre::new(red, green, blue, alpha)
    }
}


/// Compute endpoint values of a line in Xiaolin Wu's line algorithm
fn endpoint(x: f64, y: f64, gradient: f64) -> (f64,f64,f64,usize,usize,f64,f64) {
    let xend = x.round();
    let yend = y + gradient * (xend - x);
    let xgap = rfpart(x + 0.5);
    let v1 = xgap * rfpart(yend);
    let v2 = xgap *  fpart(yend);

    (xend,yend,xgap,
     xend as usize,
     ipart(yend) as usize,
     v1, v2)
}

/// Compute fractional part of an f64 number
fn fpart(x: f64) -> f64 {
    x - x.floor()
}
/// Compute 1.0 - fractional part of an f64 number (remainder)
fn rfpart(x: f64) -> f64 {
    1.0 - fpart(x)
}
/// Compute integral part of an f64 number
fn ipart(x: f64) -> f64 {
    x.floor()
}

impl Pixel for Pixfmt<Rgba32> {
    fn set<C: Color>(&mut self, id: (usize, usize), c: C) {
        let c = Rgba32::from_trait(c);
        assert!(self.rbuf.data.len() > 0);
        let red   : [u8;4] = unsafe { std::mem::transmute(c.r) };
        let green : [u8;4] = unsafe { std::mem::transmute(c.g) };
        let blue  : [u8;4] = unsafe { std::mem::transmute(c.b) };
        let alpha : [u8;4] = unsafe { std::mem::transmute(c.a) };
        //eprintln!("SET: {:?} {:?} {:?} {:?}", r,g,b,a);
        for i in 0 .. 4 {
            self.rbuf[id][i]    = red[i];
            self.rbuf[id][i+4]  = green[i];
            self.rbuf[id][i+8]  = blue[i];
            self.rbuf[id][i+12] = alpha[i];
        }
        //self.rbuf[id][ 4.. 8] = unsafe { std::mem::transmute(c.g) };
        //self.rbuf[id][ 8..12] = unsafe { std::mem::transmute(c.b) };
        //self.rbuf[id][12..16] = unsafe { std::mem::transmute(c.a) };
    }
    fn bpp() -> usize { 4*4 }
    fn cover_mask() -> u64 { unimplemented!("no cover mask") }
    fn blend_pix<C: Color>(&mut self, _id: (usize, usize), _c: C, _cover: u64) {
        unimplemented!("no blending");
        /*
        let alpha = multiply_u8(c.alpha8(), cover as u8);
        let pix0 = self.get(id); // Rgba8
        let pix  = self.mix_pix(&pix0, &Rgba8::from(c), alpha);
        self.set(id, &pix);
         */
    }
    /// Height of rendering buffer in pixels
    fn height(&self) -> usize {
        self.rbuf.height
    }
    /// Width of rendering buffer in pixels
    fn width(&self) -> usize {
        self.rbuf.width
    }

}

impl Pixel for Pixfmt<Gray8> {
    fn set<C: Color>(&mut self, id: (usize, usize), c: C) {
        let c = Gray8::from_trait(c);
        self.rbuf[id][0] = c.value;
        self.rbuf[id][1] = c.alpha;
    }
    fn cover_mask() -> u64 {  255  }
    fn bpp() -> usize { 2 }
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64) {
        let alpha = multiply_u8(c.alpha8(), cover as u8);
        let p0 = self.mix_pix(id, Gray8::from_trait(c), alpha);
        self.set(id, p0);
    }
    /// Height of rendering buffer in pixels
    fn height(&self) -> usize {
        self.rbuf.height
    }
    /// Width of rendering buffer in pixels
    fn width(&self) -> usize {
        self.rbuf.width
    }

}

use crate::base::RenderingBase;

pub struct PixfmtAlphaBlend<'a,T,C> where T: PixelDraw {
    ren: &'a mut RenderingBase<T>,
    offset: usize,
    //step: usize,
    phantom: PhantomData<C>,
}

impl<'a,T,C> PixfmtAlphaBlend<'a,T,C> where T: PixelDraw {
    pub fn new(ren: &'a mut RenderingBase<T>, offset: usize) -> Self {
        //let step = T::bpp();
        Self { ren, offset, phantom: PhantomData }
    }
}
impl PixfmtAlphaBlend<'_,Pixfmt<Rgb8>,Gray8> {
    fn component(&self, c: Rgb8) -> Gray8 {
        match self.offset {
            0 => Gray8::new(c.r),
            1 => Gray8::new(c.g),
            2 => Gray8::new(c.b),
            _ => unreachable!("incorrect offset for Rgb8"),
        }
    }
    fn mix_pix(&mut self, id: (usize,usize), c: Gray8, alpha: u8) -> Gray8 {
        let p = self.component( Rgb8::from_slice( &self.ren.pixf.rbuf[id] ) );
        Gray8::new_with_alpha(lerp_u8(p.value, c.value, alpha), alpha)
    }
}

impl Pixel for PixfmtAlphaBlend<'_,Pixfmt<Rgb8>,Gray8> {
    fn set<C: Color>(&mut self, id: (usize, usize), c: C) {
        let c = Rgb8::from_trait(c);
        self.ren.pixf.rbuf[id][self.offset] = self.component(c).value;
    }
    fn cover_mask() -> u64 { Pixfmt::<Rgb8>::cover_mask() }
    fn bpp() -> usize { Pixfmt::<Rgb8>::bpp() }
    fn blend_pix<C: Color>(&mut self, id: (usize, usize), c: C, cover: u64) {
        let alpha = multiply_u8(c.alpha8(), cover as u8);
        //println!("blend_pix color {:?} cover {} alpha {} {}", c, cover, alpha, c.alpha8());
        let c = Rgb8::from_trait(c);
        //println!("          color {:?}", c);
        let c0 = self.component(c);
        //println!("          color {:?}", c);
        let p0 = self.mix_pix(id, c0, alpha);
        //println!("          color {:?}", c);
        self.set(id, p0);
    }
    /// Height of rendering buffer in pixels
    fn height(&self) -> usize {
        self.ren.pixf.height()
    }
    /// Width of rendering buffer in pixels
    fn width(&self) -> usize {
        self.ren.pixf.width()
    }

}
impl PixelDraw for PixfmtAlphaBlend<'_,Pixfmt<Rgb8>,Gray8> {
    fn fill<C: Color>(&mut self, color: C) {
        self.ren.pixf.fill(color);
    }
    fn blend_color_vspan<C: Color>(&mut self, x: i64, y: i64, len: i64, colors: &[C], covers: &[u64], cover: u64) {
        assert_eq!(len as usize, colors.len());
        let (x,y) = (x as usize, y as usize);
        if ! covers.is_empty() {
            assert_eq!(colors.len(), covers.len());
            for (i,(&color,&cover)) in colors.iter().zip(covers.iter()).enumerate() {
                self.copy_or_blend_pix_with_cover((x,y+i), color, cover);
            }
        } else if cover == Self::cover_mask() {
            for (i,&color) in colors.iter().enumerate() {
                self.copy_or_blend_pix((x,y+i), color);
            }
        } else {
            for (i,&color) in colors.iter().enumerate() {
                self.copy_or_blend_pix_with_cover((x,y+i), color, cover);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Pixfmt;
    use crate::PixelDraw;
    use crate::Source;
    use crate::Rgb8;
    use crate::Rgba8;
    use crate::Rgba8pre;
    use crate::Srgba8;
    use crate::Rgba32;
    #[test]
    fn pixfmt_test() {
        let mut p = Pixfmt::<Rgb8>::new(10,10);
        assert_eq!(p.rbuf.data.len(),300);

        p.copy_pixel(0,0, Rgb8::black());
        assert_eq!(p.get((0,0)), Rgba8::black());

        assert_ne!(p.get((1,0)), Rgba8::white());
        p.copy_pixel(1,0, Rgb8::white());
        assert_eq!(p.get((1,0)), Rgba8::white());

        let red = Rgba8::new(255,0,0,128);
        p.copy_hline(0,1,10,red);
        for i in 0 .. 10 {
            assert_eq!(p.get((i,1)), Rgba8::new(255,0,0,255));
        }
        let yellow = Srgba8::new(128,255,0,128);
        p.copy_hline(0,2,10,yellow);
        for i in 0 .. 10 {
            assert_eq!(p.get((i,2)), Rgba8::new(55,255,0,255));
        }
        let fuchsia = Rgba32::new(0.0,1.0,1.0,0.5);
        p.copy_hline(0,3,10,fuchsia);
        for i in 0 .. 10 {
            assert_eq!(p.get((i,3)), Rgba8::new(0,255,255,255));
        }
        p.clear();
        assert_eq!(p.get((0,3)), Rgba8::new(255,255,255,255));

        let red = Rgba8::new(255,0,0,128);
        p.copy_vline(1,0,10,red);
        for i in 0 .. 10 {
            assert_eq!(p.get((1,i)), Rgba8::new(255,0,0,255));
        }
        let yellow = Srgba8::new(128,255,0,128);
        p.copy_vline(2,0,10,yellow);
        for i in 0 .. 10 {
            assert_eq!(p.get((2,i)), Rgba8::new(55,255,0,255));
        }
        let fuchsia = Rgba32::new(0.0,1.0,1.0,0.5);
        p.copy_vline(3,0,10,fuchsia);
        for i in 0 .. 10 {
            assert_eq!(p.get((3,i)), Rgba8::new(0,255,255,255));
        }

        p.clear();
        p.copy_pixel(11,11,Rgb8::black());
        for i in 0 .. 10 {
            for j in 0 .. 10 {
                assert_eq!(p.get((i,j)), Rgba8::white());
            }
        }
        p.copy_hline(0,0,20,Rgb8::black());
        for i in 0 .. 10 {
            assert_eq!(p.get((i,0)), Rgba8::black());
        }
        p.copy_hline(5,1,20,Rgb8::black());
        for i in 5 .. 10 {
            assert_eq!(p.get((i,1)), Rgba8::black());
        }

        p.clear();
        p.copy_vline(0,0,20,Rgb8::black());
        for i in 0 .. 10 {
            assert_eq!(p.get((0,i)), Rgba8::black());
        }

        println!("copy_vline: y inside but too long");
        p.clear();
        p.copy_vline(1,5,20,Rgb8::black());
        for i in 0 .. 5 {
            assert_eq!(p.get((1,i)), Rgba8::white(),"pix({},{}): {:?}",1,i,p.get((1,i)));
        }
        for i in 5 .. 10 {
            assert_eq!(p.get((1,i)), Rgba8::black(),"pix({},{}): {:?}",1,i,p.get((1,i)));
        }
        println!("copy_vline: y inside and short");
        p.copy_vline(2,3,5,Rgb8::black());
        for i in 0 .. 3 {
            assert_eq!(p.get((2,i)), Rgba8::white(),"pix({},{}): {:?}",2,i,p.get((2,i)));
        }
        for i in 3 .. 8 {
            assert_eq!(p.get((2,i)), Rgba8::black(),"pix({},{}): {:?}",2,i,p.get((2,i)));
        }
        for i in 8 .. 10 {
            assert_eq!(p.get((2,i)), Rgba8::white(),"pix({},{}): {:?}",2,i,p.get((2,i)));
        }
    }

    #[test]
    fn pixfmt_rgb8_test() {

        let mut pix = Pixfmt::<Rgb8>::new(1,1);
        let black  = Rgba8::black();
        let white  = Rgba8::white();

        pix.copy_pixel(0,0,Rgba8::new(0,0,0,255));
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 255, 255); // Copy Pixel
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), white);

        let (alpha, beta, cover) = (255, 255, 0); // Do Nothing, No Coverage
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 0, 255); // Do Nothing, Transparent
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 255, 128); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));

        let (alpha, beta, cover) = (255, 128, 255); // Full Coverage, Alpha Color
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));

        let (alpha, beta, cover) = (128, 128, 255); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(255,255,255,alpha));
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,255)); // Alpha channel is ignored
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(0,0,0,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(127,127,127,255));

        let (alpha, beta, cover) = (128, 128, 128); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(255,255,255,alpha));
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,255)); // Alpha channel is ignored
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(0,0,0,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(191,191,191,255));
    }

    #[test]
    fn pixfmt_rgba8_test() {

        let mut pix = Pixfmt::<Rgba8>::new(1,1);
        let black  = Rgba8::black();
        let white  = Rgba8::white();

        pix.copy_pixel(0,0,Rgba8::new(0,0,0,255));
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 255, 255); // Copy Pixel
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), white);

        let (alpha, beta, cover) = (255, 255, 0); // Do Nothing, No Coverage
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 0, 255); // Do Nothing, Transparent
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 255, 128); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));

        let (alpha, beta, cover) = (255, 128, 255); // Full Coverage, Alpha Color
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));

        let (alpha, beta, cover) = (128, 128, 255); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(255,255,255,alpha));
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,128));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(0,0,0,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(127,127,127,192));

        let (alpha, beta, cover) = (128, 128, 128); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(255,255,255,alpha));
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,128)); // Alpha channel is ignored
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(0,0,0,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(191,191,191,160));
    }

    #[test]
    fn pixfmt_rgba8pre_test() {

        let mut pix = Pixfmt::<Rgba8pre>::new(1,1);
        let black  = Rgba8::black();
        let white  = Rgba8::white();

        pix.copy_pixel(0,0,Rgba8::new(0,0,0,255));
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 255, 255); // Copy Pixel
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), white);

        let (alpha, beta, cover) = (255, 255, 0); // Do Nothing, No Coverage
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 0, 255); // Do Nothing, Transparent
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), black);

        let (alpha, beta, cover) = (255, 255, 128); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(128,128,128,255));

        let (alpha, beta, cover) = (255, 128, 255); // Full Coverage, Alpha Color
        pix.copy_pixel(0,0,Rgba8::new(0,0,0,alpha));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(255,255,255,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,255));

        let (alpha, beta, cover) = (128, 128, 255); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(255,255,255,alpha));
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,128));
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(0,0,0,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(127,127,127,192));

        let (alpha, beta, cover) = (128, 128, 128); // Partial Coverage, Blend
        pix.copy_pixel(0,0,Rgba8::new(255,255,255,alpha));
        assert_eq!(pix.get((0,0)), Rgba8::new(255,255,255,128)); // Alpha channel is ignored
        pix.copy_or_blend_pix_with_cover((0,0), Rgba8::new(0,0,0,beta), cover);
        assert_eq!(pix.get((0,0)), Rgba8::new(191,191,191,160));
    }
}
