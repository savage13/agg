
use RenderingBuffer;
use blend;
use blend_pix;
use color::*;

#[derive(Debug,Default)]
pub struct PixfmtRgb24 {
    pub rbuf: RenderingBuffer,
}

impl PixfmtRgb24 {
    pub fn clear(&mut self) {
        self.rbuf.clear();
    }
    pub fn fill(&mut self, c: Rgb8) {
        let w = self.rbuf.width;
        let h = self.rbuf.height;
        for i in 0 .. w {
            for j in 0 .. h {
                self.set((i,j), &c);
            }
        }
    }
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        Self { rbuf: RenderingBuffer::new(width, height, bpp) }
    }
    pub fn from(rbuf: RenderingBuffer) -> Self {
        Self { rbuf }
    }
    pub fn blend_hline<C: Color>(&mut self, x: i64, y: i64, len: i64, c: &C, cover: u64) {
        if c.is_transparent() {
            return;
        }
        let (x,y,len) = (x as usize, y as usize, len as usize);
        let cover_mask = 255;
        if c.is_opaque() && cover == cover_mask {
            for i in 0 .. len {
                eprintln!("BLEND_HLINE (SET): {:3} {:3} c: {:3} {:3} {:3} cover: {:3}", x+i, y, cu8r(c), cu8g(c), cu8b(c), cover);
                self.set((x+i,y), c);
            }
        } else {
            for i in 0 .. len {
                let pix0 = self.get((x+i, y));
                //eprintln!("BLEND_HLINE (   ): {:3} {:3} c: {:3} {:3} {:3} cover: {:3} {:3} {:3} {:3}", x+i, y, cu8r(c), cu8g(c), cu8b(c), cover, cu8r(&pix), cu8g(&pix), cu8b(&pix));
                let pix = blend_pix(&pix0, c, cover);
                self.set((x+i,y), &pix);
                let pix1 = self.get((x+i, y));
                eprintln!("BLEND_HLINE (   ): {:3} {:3} c: {:3} {:3} {:3} cover: {:3} pix {:3} {:3} {:3} out {:3} {:3} {:3}", x+i, y,
                          cu8r(c), cu8g(c), cu8b(c),
                          cover,
                          cu8r(&pix0), cu8g(&pix0), cu8b(&pix0),
                          cu8r(&pix1), cu8g(&pix1), cu8b(&pix1));
            }
        }
    }
    pub fn blend_solid_hspan<C: Color>(&mut self, x: i64, y: i64, _len: i64, c: &C, covers: &[u64]) {
        eprintln!("BLEND_SOLID_HSPAN: {:3} {:3} len {:3} PIXFMT RGB", x, y, covers.len());
        if c.is_transparent() {
            return;
        }
        for (i, &cover) in covers.iter().enumerate() {
            self.blend_hline(x+i as i64,y,1,c,cover);
        }
    }
    pub fn copy_pixel(&mut self, x: usize, y: usize, c: Rgb8) {
        self.set((x,y), &c);
    }
    pub fn copy_hline(&mut self, x: usize, y: usize, n: usize, c: Rgb8) {
        for i in 0 .. n {
            self.set((x+i,y), &c);
        }
    }
    pub fn copy_vline(&mut self, x: usize, y: usize, n: usize, c: Rgb8) {
        for i in 0 .. n {
            self.set((x,y+i), &c);
        }
    }

    pub fn blend_color_hspan(&mut self, x: usize, y: usize, _n: usize,
                             c: &[Rgb8], _cover: usize) {
        for (i,ci) in c.iter().enumerate() {
            self.set((x+i,y), ci);
        }
    }

    pub fn set<C: Color>(&mut self, id: (usize, usize), c: &C) {
        self.rbuf[id][0] = (c.red()   * 255.0) as u8;
        self.rbuf[id][1] = (c.green() * 255.0) as u8;
        self.rbuf[id][2] = (c.blue()  * 255.0) as u8;
    }
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
            self.set((ypx11,  xpx11), &v1);
            self.set((ypx11+1,xpx11), &v2);
        } else {
            self.set((xpx11,  ypx11),  &v1);
            self.set((xpx11,  ypx11+1),&v2);
        }
        let mut intery = yend + gradient;
        // Handle Second Endpoint

        let (_xend, _yend, _xgap, xpx12, ypx12, v1, v2) = endpoint(x2,y2,gradient);
        let v1 = blend(c, white, v1);
        let v2 = blend(c, white, v2);
        if steep {
            self.set((ypx12,  xpx12),   &v1);
            self.set((ypx12+1,xpx12),   &v2);
        } else {
            self.set((xpx12,  ypx12),   &v1);
            self.set((xpx12,  ypx12+1), &v2);
        }
        // In Between Points
        for xp in xpx11 + 1 .. xpx12 {
            let yp = ipart(intery) as usize;
            let (p0,p1) = if steep { ((yp,xp),(yp+1,xp)) } else { ((xp,yp),(xp,yp+1)) };

            let (v1,v2) = ( rfpart(intery), fpart(intery) );
            let v0 = blend(c, self.get(p0), v1);
            let v1 = blend(c, self.get(p1), v2);
            self.set(p0,&v0);
            self.set(p1,&v1);

            intery += gradient;
        }

    }
    pub fn get(&self, id: (usize, usize)) -> Rgb8 {
        let p = &self.rbuf[id];
        Rgb8::new( [p[0], p[1], p[2]] )
    }
    pub fn line_sp(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: Rgb8) {
        println!("({}, {}) - ({}, {})", x1,y1,x2,y2);
        let x1 = (x1 * 256.0).round() as i64 / 256;
        let y1 = (y1 * 256.0).round() as i64 / 256;
        let x2 = (x2 * 256.0).round() as i64 / 256;
        let y2 = (y2 * 256.0).round() as i64 / 256;
        println!("   ({}, {}) - ({}, {})", x1,y1,x2,y2);

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
                self.set((y as usize, x as usize), &c);
            } else {
                self.set((x as usize, y as usize), &c);
            }
            xmod += rem;
            y += left;
            if xmod > 0 {
                xmod -= count;
                y += 1;
            }
        }
    }
    /// https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm#C.2B.2B
    
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
                self.set((y,x), &c);
            } else {
                self.set((x,y), &c);
            }
            error -= dy;
            if error <= 0.0 {
                y = if pos { y+1 } else { y-1 };
                error += dx;
            }
        }
    }
}

pub struct PixfmtGray8 {
    pub rbuf: RenderingBuffer
}

impl PixfmtGray8 {
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        Self{ rbuf: RenderingBuffer::new(width, height, bpp) }
    }
    pub fn copy_hline(&mut self, x: usize, y: usize, n: usize, c: Gray8) {
        for i in 0 .. n {
            self.rbuf[(x+i,y)][0] = *c;
        }
    }
}
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


fn fpart(x: f64) -> f64 {
    x - x.floor()
}
fn rfpart(x: f64) -> f64 {
    1.0 - fpart(x)
}
fn ipart(x: f64) -> f64 {
    x.floor()
}
