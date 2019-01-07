//! Clipping Region

//use crate::POLY_SUBPIXEL_SCALE;
use crate::cell::RasterizerCell;

/// Rectangle
#[derive(Debug,Default,Copy,Clone)]
pub struct Rectangle<T: std::cmp::PartialOrd + Copy> {
    /// Minimum x value
    pub x1: T,
    /// Minimum y value
    pub y1: T,
    /// Maximum x value
    pub x2: T,
    /// Maximum y value
    pub y2: T,
}
impl<T> Rectangle<T> where T: std::cmp::PartialOrd + Copy {
    /// Create a new Rectangle
    ///
    /// Values are sorted before storing
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        let (x1, x2) = if x1 > x2 { (x2,x1) } else { (x1,x2) };
        let (y1, y2) = if y1 > x2 { (y2,y1) } else { (y1,y2) };
        Self { x1,y1,x2,y2 }
    }
    /// Get location of point relative to rectangle
    ///
    /// Returned is an a u8 made up of the following bits:
    /// - [INSIDE](constant.INSIDE.html)
    /// - [LEFT](constant.LEFT.html)
    /// - [RIGHT](constant.RIGHT.html)
    /// - [BOTTOM](constant.BOTTOM.html)
    /// - [TOP](constant.TOP.html)
    ///
    pub fn clip_flags(&self, x: T, y: T) -> u8 {
        clip_flags(&x,&y, &self.x1, &self.y1, &self.x2, &self.y2)
    }
    /// Expand if the point (x,y) is outside
    pub fn expand(&mut self, x: T, y: T) {
        if x < self.x1 { self.x1 = x; }
        if x > self.x2 { self.x2 = x; }
        if y < self.y1 { self.y1 = y; }
        if y > self.y2 { self.y2 = y; }
    }
    /// Expand if the rectangle is outside
    pub fn expand_rect(&mut self, r: &Rectangle<T>) {
        self.expand(r.x1, r.y1);
        self.expand(r.x2, r.y2);
    }
}



/// Inside Region
///
/// See https://en.wikipedia.org/wiki/Liang-Barsky_algorithm
/// See https://en.wikipedia.org/wiki/Cyrus-Beck_algorithm
pub const INSIDE : u8 = 0b0000;
/// Left of Region
///
/// See [Liang Barsky](https://en.wikipedia.org/wiki/Liang-Barsky_algorithm)
///
/// See [Cyrus Beck](https://en.wikipedia.org/wiki/Cyrus-Beck_algorithm)
pub const LEFT   : u8 = 0b0000_0001;
/// Right of Region
///
/// See [Liang Barsky](https://en.wikipedia.org/wiki/Liang-Barsky_algorithm)
///
/// See [Cyrus Beck](https://en.wikipedia.org/wiki/Cyrus-Beck_algorithm)
pub const RIGHT  : u8 = 0b0000_0010;
/// Below Region
///
/// See [Liang Barsky](https://en.wikipedia.org/wiki/Liang-Barsky_algorithm)
///
/// See [Cyrus Beck](https://en.wikipedia.org/wiki/Cyrus-Beck_algorithm)
pub const BOTTOM : u8 = 0b0000_0100;
/// Above Region
///
/// See [Liang Barsky](https://en.wikipedia.org/wiki/Liang-Barsky_algorithm)
///
/// See [Cyrus Beck](https://en.wikipedia.org/wiki/Cyrus-Beck_algorithm)
pub const TOP    : u8 = 0b0000_1000;

/// Determine the loaiton of a point to a broken-down rectangle or range
///
/// Returned is an a u8 made up of the following bits:
/// - [INSIDE](constant.INSIDE.html)
/// - [LEFT](constant.LEFT.html)
/// - [RIGHT](constant.RIGHT.html)
/// - [BOTTOM](constant.BOTTOM.html)
/// - [TOP](constant.TOP.html)
///
fn clip_flags<T: std::cmp::PartialOrd>(x: &T, y: &T, x1: &T, y1: &T, x2: &T, y2: &T) -> u8 {
    let mut code = INSIDE;
    if x < x1 { code |= LEFT; }
    if x > x2 { code |= RIGHT; }
    if y < y1 { code |= BOTTOM; }
    if y > y2 { code |= TOP; }
    code
}

/// Clip Region
///
/// Clipping for Rasterizers
#[derive(Debug,Default)]
pub struct Clip {
    /// Current x Point
    x1: i64,
    /// Current y Point
    y1: i64,
    /// Rectangle to clip on
    clip_box: Option<Rectangle<i64>>,
    /// Current clip flag for point (x1,y1)
    clip_flag: u8,
}

fn mul_div(a: i64, b: i64, c: i64) -> i64 {
    let (a,b,c) = (a as f64, b as f64, c as f64);
    (a * b / c).round() as i64
}
impl Clip {
    /// Create new Clipping region
    pub fn new() -> Self {
        Self {x1: 0, y1: 0,
              clip_box: None,
              clip_flag: INSIDE }
    }
    /// Clip a line along the top and bottom of the regon
    fn line_clip_y(&self, ras: &mut RasterizerCell,
                   x1: i64, y1: i64,
                   x2: i64, y2: i64,
                   f1: u8, f2: u8) {
        let b = match self.clip_box {
            None => return,
            Some(ref b) => b,
        };
        let f1 = f1 & (TOP|BOTTOM);
        let f2 = f2 & (TOP|BOTTOM);
        // Fully Visible in y
        if f1 == INSIDE && f2 == INSIDE {
            println!("ras.line_to_d({:.2} , {:.2});//1", x1>>8,y1>>8);
            println!("ras.line_to_d({:.2} , {:.2});//2", x2>>8,y2>>8);
            ras.line(x1,y1,x2,y2);
        } else {
            // Both points above or below clip box
            if f1 == f2 {
                return;
            }
            let (mut tx1, mut ty1, mut tx2, mut ty2) = (x1,y1,x2,y2);
            if f1 == BOTTOM {
                tx1 = x1 + mul_div(b.y1-y1, x2-x1, y2-y1);
                ty1 = b.y1;
            }
            if f1 == TOP {
                tx1 = x1 + mul_div(b.y2-y1, x2-x1, y2-y1);
                ty1 = b.y2;
            }
            if f2 == BOTTOM {
                tx2 = x1 + mul_div(b.y1-y1, x2-x1, y2-y1);
                ty2 = b.y1;
            }
            if f2 == TOP {
                tx2 = x1 + mul_div(b.y2-y1, x2-x1, y2-y1);
                ty2 = b.y2;
            }
            println!("ras.line_to_d({:.2} , {:.2}); //3", tx1>>8,ty1>>8);
            println!("ras.line_to_d({:.2} , {:.2}); //4", tx2>>8,ty2>>8);
            ras.line(tx1,tx2,ty1,ty2);
        }
    }

    /// Draw a line from (x1,y1) to (x2,y2) into a RasterizerCell
    ///
    /// Final point (x2,y2) is saved internally as (x1,y1))
    pub fn line_to(&mut self, ras: &mut RasterizerCell, x2: i64, y2: i64) {
        //eprintln!("ras.line_to_d({}, {}); // LINE TO: {} {}",
        //          x2 / POLY_SUBPIXEL_SCALE, y2 / POLY_SUBPIXEL_SCALE,
        //          x2, y2);
        if let Some(ref b) = self.clip_box {
            println!("LINE CLIPPING ON");
            let f2 = b.clip_flags(x2,y2);
            // Both points above or below clip box
            let fy1 = (TOP | BOTTOM) & self.clip_flag;
            let fy2 = (TOP | BOTTOM) & f2;
            if fy1 != INSIDE && fy1 == fy2 {
                println!("LINE OUTSIDE CLIP BOX {:?}", b);
                println!("LINE xlim {} {} x1 {} x2 {} f {:04b}", b.x1,b.x2,self.x1,x2, self.clip_flag);
                println!("LINE ylim {} {} y1 {} y2 {} f {:04b}", b.y1,b.y2,self.y1,y2,f2);
                self.x1 = x2;
                self.y1 = y2;
                self.clip_flag = f2;
                return;
            }
            let (x1,y1,f1) = (self.x1, self.y1, self.clip_flag);
            println!("LINE CLIP: L {} R {} T {} B {} -- {} {}", f1 & LEFT, f1 & RIGHT, f1 & TOP, f1 & BOTTOM, x1>>8, y1>>8);
            println!("LINE CLIP: L {} R {} T {} B {} -- {} {}", f2 & LEFT, f2 & RIGHT, f2 & TOP, f2 & BOTTOM, x2>>8, y2>>8);
            match (f1 & (LEFT|RIGHT), f2 & (LEFT|RIGHT)) {
                (INSIDE,INSIDE) => self.line_clip_y(ras, x1,y1,x2,y2,f1,f2),
                (INSIDE,RIGHT) => {
                    let y3 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x2, y3);
                    self.line_clip_y(ras, x1,   y1, b.x2, y3, f1, f3);
                    self.line_clip_y(ras, b.x2, y3, b.x2, y2, f3, f2);
                },
                (RIGHT,INSIDE) => {
                    let y3 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x2, y3);
                    self.line_clip_y(ras, b.x2, y1, b.x2, y3, f1, f3);
                    self.line_clip_y(ras, b.x2, y3,   x2, y2, f3, f2);
                },
                (INSIDE,LEFT) => {
                    let y3 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x1, y3);
                    self.line_clip_y(ras, x1,   y1, b.x1, y3, f1, f3);
                    self.line_clip_y(ras, b.x1, y3, b.x1, y2, f3, f2);
                },
                (RIGHT,LEFT) => {
                    let y3 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let y4 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x2, y3);
                    let f4 = b.clip_flags(b.x1, y4);
                    self.line_clip_y(ras, b.x2, y1, b.x2, y3, f1, f3);
                    self.line_clip_y(ras, b.x2, y3, b.x1, y4, f3, f4);
                    self.line_clip_y(ras, b.x1, y4, b.x1, y2, f4, f2);
                },
                (LEFT,INSIDE) => {
                    let y3 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x1, y3);
                    self.line_clip_y(ras, b.x1, y1, b.x1, y3, f1, f3);
                    self.line_clip_y(ras, b.x1, y3,   x2, y2, f3, f2);
                },
                (LEFT,RIGHT) => {
                    let y3 = y1 + mul_div(b.x1-x1, y2-y1, x2-x1);
                    let y4 = y1 + mul_div(b.x2-x1, y2-y1, x2-x1);
                    let f3 = b.clip_flags(b.x1, y3);
                    let f4 = b.clip_flags(b.x2, y4);
                    self.line_clip_y(ras, b.x1, y1, b.x1, y3, f1, f3);
                    self.line_clip_y(ras, b.x1, y3, b.x2, y4, f3, f4);
                    self.line_clip_y(ras, b.x2, y4, b.x2, y2, f4, f2);
                },
                (LEFT,LEFT)   => self.line_clip_y(ras, b.x1,y1,b.x1,y2,f1,f2),
                (RIGHT,RIGHT) => self.line_clip_y(ras, b.x2,y1,b.x2,y2,f1,f2),

                (_,_) => unreachable!("f1,f2 {:?} {:?}", f1,f2),
            }
            self.clip_flag = f2;
        } else {
            ras.line(self.x1, self.y1, x2, y2);
        }
        self.x1 = x2;
        self.y1 = y2;
    }
    /// Move to point (x2,y2)
    ///
    /// Point is saved internally as (x1,y1)
    pub fn move_to(&mut self, x2: i64, y2: i64) {
        //eprintln!("//ras.move_to_d({}, {}); // MOVE TO: {} {}", x2/POLY_SUBPIXEL_SCALE, y2/POLY_SUBPIXEL_SCALE, x2, y2);
        self.x1 = x2;
        self.y1 = y2;
        if let Some(ref b) = self.clip_box {
            self.clip_flag = clip_flags(&x2,&y2,
                                        &b.x1,&b.y1,
                                        &b.x2,&b.y2);
        }
    }
    /// Define the clipping region
    pub fn clip_box(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) {
        self.clip_box = Some( Rectangle::new(x1, y1, x2, y2) );
    }
}
