//! Rasterizer for Outlines with Anti-Aliasing
//!
//! # Example
//!
//!
//!     use agg::{Pixfmt,Rgb8,Rgba8,DrawOutline};
//!     use agg::{RendererOutlineAA,RasterizerOutlineAA};
//!
//!     // Create Image and Rendering Base
//!     let pix = Pixfmt::<Rgb8>::new(100,100);
//!     let mut ren_base = agg::RenderingBase::new(pix);
//!     ren_base.clear( Rgba8::new(255, 255, 255, 255) );
//!
//!     // Create Outline Rendering, set color and width
//!     let mut ren = RendererOutlineAA::with_base(&mut ren_base);
//!     ren.color(agg::Rgba8::new(0,0,0,255));
//!     ren.width(20.0);
//!
//!     // Create a Path
//!     let mut path = agg::Path::new();
//!     path.move_to(10.0, 10.0);
//!     path.line_to(50.0, 90.0);
//!     path.line_to(90.0, 10.0);
//!
//!     // Create Outline Rasterizer and add path
//!     let mut ras = RasterizerOutlineAA::with_renderer(&mut ren);
//!     ras.round_cap(true);
//!     ras.add_path(&path);
//!     ren_base.to_file("outline_aa.png").unwrap();
//!
//! The above code will produce:
//!
//! ![Output](https://raw.githubusercontent.com/savage13/agg/master/images/outline_aa.png)
//!
use crate::stroke::LineJoin;
use crate::paths::PathCommand;
use crate::paths::Vertex;
use crate::line_interp::LineParameters;
use crate::line_interp::DrawVars;
use crate::line_interp::DistanceInterpolator00;
use crate::line_interp::DistanceInterpolator0;
use crate::base::RenderingBase;
use crate::color::Rgba8;
use crate::clip::Rectangle;
use crate::render::clip_line_segment;
use crate::raster::len_i64_xy;
use crate::Pixel;
use crate::Color;
use crate::RenderOutline;
use crate::render::LINE_MAX_LENGTH;
use crate::MAX_HALF_WIDTH;
use crate::POLY_SUBPIXEL_SHIFT;
use crate::POLY_SUBPIXEL_MASK;

use crate::DrawOutline;
use crate::VertexSource;
use crate::raster::len_i64;
use crate::POLY_SUBPIXEL_SCALE;

/// Outline Rasterizer with Anti-Aliasing
pub struct RasterizerOutlineAA<'a,T> where T: DrawOutline {
    ren: &'a mut T,
    start_x: i64,
    start_y: i64,
    vertices: Vec<Vertex<i64>>,
    round_cap: bool,
    line_join: LineJoin,
}

impl<'a,T> RasterizerOutlineAA<'a, T> where T: DrawOutline {
    /// Create and connect an Outline Rasterizer to a Renderer
    pub fn with_renderer(ren: &'a mut T) -> Self {
        let line_join = if ren.accurate_join_only() {
            LineJoin::MiterAccurate
        } else {
            LineJoin::Round
        };
        Self { ren, start_x: 0, start_y: 0, vertices: vec![],
               round_cap: false, line_join }
    }
    /// Set Rounded End Caps
    pub fn round_cap(&mut self, on: bool) {
        self.round_cap = on;
    }
    /// Add and Render a path
    pub fn add_path<VS: VertexSource>(&mut self, path: &VS) {
        for v in path.xconvert().iter() {
            match v.cmd {
                PathCommand::MoveTo => self.move_to_d(v.x, v.y),
                PathCommand::LineTo => self.line_to_d(v.x, v.y),
                PathCommand::Close => self.close_path(),
                PathCommand::Stop => unimplemented!("stop encountered"),
            }
        }
        self.render(false);
    }
    fn conv(&self, v: f64) -> i64 {
        (v * POLY_SUBPIXEL_SCALE as f64).round() as i64
    }
    /// Move the current point to (`x`,`y`)
    pub fn move_to_d(&mut self, x: f64, y: f64) {
        let x = self.conv(x);
        let y = self.conv(y);
        self.move_to( x, y );
    }
    /// Draw a line from the current point to (`x`,`y`)
    pub fn line_to_d(&mut self, x: f64, y: f64) {
        let x = self.conv(x);
        let y = self.conv(y);
        self.line_to( x, y );
    }
    fn move_to(&mut self, x: i64, y: i64) {
        self.start_x = x;
        self.start_y = y;
        self.vertices.push( Vertex::move_to(x, y) );
    }
    fn line_to(&mut self, x: i64, y: i64) {
        let n = self.vertices.len();
        if n > 1 {
            let v0 = self.vertices[n-1];
            let v1 = self.vertices[n-2];
            let len = len_i64(&v0,&v1);
            if len < POLY_SUBPIXEL_SCALE + POLY_SUBPIXEL_SCALE / 2 {
                self.vertices.pop();
            }

        }
        self.vertices.push( Vertex::line_to(x, y) );
    }
    /// Close the current path
    pub fn close_path(&mut self) {
        self.line_to(self.start_x, self.start_y);
    }
    fn cmp_dist_start(d: i64) -> bool { d > 0 }
    fn cmp_dist_end  (d: i64) -> bool { d <= 0 }
    fn draw_two_points(&mut self) {
        debug_assert!(self.vertices.len() == 2);
        let p1 = self.vertices.first().unwrap();
        let p2 = self.vertices.last().unwrap();
        let (x1,y1) = (p1.x, p1.y);
        let (x2,y2) = (p2.x, p2.y);
        let lprev = len_i64(p1,p2);
        let lp = LineParameters::new(x1,y1, x2,y2, lprev);
        if self.round_cap {
            self.ren.semidot(Self::cmp_dist_start,
                             x1, y1,
                             x1 + (y2-y1),
                             y1 - (x2-x1));
        }
        self.ren.line3(&lp,
                       x1 + (y2-y1), y1 - (x2-x1),
                       x2 + (y2-y1), y2 - (x2-x1));
        if self.round_cap {
            self.ren.semidot(Self::cmp_dist_end,
                             x2, y2,
                             x2 + (y2-y1),
                             y2 - (x2-x1));
        }
    }
    fn draw_three_points(&mut self) {
        debug_assert!(self.vertices.len() == 3);
        let mut v = self.vertices.iter();
        let p1 = v.next().unwrap();
        let p2 = v.next().unwrap();
        let p3 = v.next().unwrap();
        let (x1,y1) = (p1.x, p1.y);
        let (x2,y2) = (p2.x, p2.y);
        let (x3,y3) = (p3.x, p3.y);
        let lprev = len_i64(p1,p2);
        let lnext = len_i64(p2,p3);
        let lp1 = LineParameters::new(x1, y1, x2, y2, lprev);
        let lp2 = LineParameters::new(x2, y2, x3, y3, lnext);
        if self.round_cap {
            self.ren.semidot(Self::cmp_dist_start,
                             x1, y1,
                             x1 + (y2-y1),
                             y1 - (x2-x1));
        }
        if self.line_join == LineJoin::Round {
            self.ren.line3(&lp1,
                           x1 + (y2-y1), y1 - (x2-x1),
                           x2 + (y2-y1), y2 - (x2-x1));
            self.ren.pie(x2, y2,
                         x2 + (y2-y1), y2 - (x2-x1),
                         x2 + (y3-y2), y2 - (x3-x2));
            self.ren.line3(&lp2,
                           x2 + (y3-y2), y2 - (x3-x2),
                           x3 + (y3-y2), y3 - (x3-x2));
        } else {
            let (xb1, yb1) = Self::bisectrix(&lp1, &lp2);
            self.ren.line3(&lp1, x1 + (y2-y1), y1 - (x2-x1), xb1, yb1);
            self.ren.line3(&lp2, xb1, yb1, x3 + (y3-y2), y3 - (x3-x2));
        }
        if self.round_cap {
            self.ren.semidot(Self::cmp_dist_end,
                             x3, y3,
                             x3 + (y3-y2),
                             y3 - (x3-x2));
        }
    }
    fn draw_many_points(&mut self) {
        debug_assert!(self.vertices.len() > 3);
        let v1 = self.vertices[0];
        let x1 = v1.x;
        let y1 = v1.y;

        let v2 = self.vertices[1];
        let x2 = v2.x;
        let y2 = v2.y;

        let v3 = self.vertices[2];
        let v4 = self.vertices[3];

        let mut dv = DrawVars::new();
        dv.idx = 3;
        let lprev = len_i64(&v1,&v2);
        dv.lcurr  = len_i64(&v2,&v3);
        dv.lnext  = len_i64(&v3,&v4);
        let prev = LineParameters::new(x1,y1, x2, y2, lprev); // pt1 -> pt2
        dv.x1 = v3.x;
        dv.y1 = v3.y;
        dv.curr = LineParameters::new(x2,y2, dv.x1, dv.y1, dv.lcurr); // pt2 -> pt3
        dv.x2 = v4.x;
        dv.y2 = v4.y;
        dv.next = LineParameters::new(dv.x1,dv.y1, dv.x2, dv.y2, dv.lnext); // pt3 -> pt4
        dv.xb1 = 0;
        dv.xb2 = 0;
        dv.yb1 = 0;
        dv.yb2 = 0;
        dv.flags = match self.line_join {
            LineJoin::MiterRevert | LineJoin::Bevel | LineJoin::MiterRound => { 3 },
            LineJoin::None => 3,
            LineJoin::MiterAccurate => 0,
            LineJoin::Miter | LineJoin::Round => {
                let mut v = 0;
                if prev.diagonal_quadrant() == dv.curr.diagonal_quadrant() {
                    v |= 1;
                }
                if dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant() {
                    v |= 2;
                }
                v
            }
        };
        if self.round_cap {
            self.ren.semidot(Self::cmp_dist_start, x1,y1, x1 + (y2-y1), y1 - (x2-x1));
        }
        if (dv.flags & 1) == 0 {
            if self.line_join == LineJoin::Round {
                self.ren.line3(&prev,
                               x1 + (y2-y1), y1 - (x2-x1),
                               x2 + (y2-y1), y2 - (x2-x1));
                self.ren.pie(prev.x2, prev.y2,
                             x2 + (y2-y1), y2 - (x2-x1),
                             dv.curr.x1 + (dv.curr.y2-dv.curr.y1),
                             dv.curr.y1 + (dv.curr.x2-dv.curr.x1));
            } else {
                let(xb1, yb1) = Self::bisectrix(&prev, &dv.curr);
                self.ren.line3(&prev,
                               x1 + (y2-y1), y1 - (x2-x1), xb1, yb1);
                dv.xb1 = xb1;
                dv.yb1 = yb1;
            }
        } else {
            self.ren.line1(&prev, x1 + (y2-y1), y1-(x2-x1));
        }
        if (dv.flags & 2) == 0 && self.line_join != LineJoin::Round {
            let (xb2, yb2) = Self::bisectrix(&dv.curr, &dv.next);
            dv.xb2 = xb2;
            dv.yb2 = yb2;
        }
        self.draw(&mut dv, 1, self.vertices.len()-2);
        if (dv.flags & 1) == 0 {
            if self.line_join == LineJoin::Round {
                self.ren.line3(&dv.curr,
                               dv.curr.x1 + (dv.curr.y2-dv.curr.y1),
                               dv.curr.y1 - (dv.curr.x2 - dv.curr.x1),
                               dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                               dv.curr.y2 - (dv.curr.x2 - dv.curr.x1));
            } else {
                self.ren.line3(&dv.curr, dv.xb1, dv.yb1,
                               dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                               dv.curr.y2 - (dv.curr.x2 - dv.curr.x1));
            }
        } else {
            self.ren.line2(&dv.curr,
                         dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                         dv.curr.y2 - (dv.curr.x2 - dv.curr.x1));
        }
        if self.round_cap {
            self.ren.semidot(Self::cmp_dist_end, dv.curr.x2, dv.curr.y2,
                             dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                             dv.curr.y2 - (dv.curr.x2 - dv.curr.x1));
        }
    }
    /// Render the current path
    ///
    /// Use only if drawing a path with [`move_to_d`](#method.move_to_d) and
    ///  [`line_to_d`](#method.line_to_d).  Paths drawn with [`add_path`](#method.add_path)
    ///  are automatically rendered
    ///
    pub fn render(&mut self, close_polygon: bool) {
        if close_polygon {
            unimplemented!("no closed polygons yet");
        } else {
            match self.vertices.len() {
                0 | 1 => return,
                2 => self.draw_two_points(),
                3 => self.draw_three_points(),
                _ => self.draw_many_points(),
            }
        }
        self.vertices.clear();
    }
    fn draw(&mut self, dv: &mut DrawVars, start: usize, end: usize) {
        for _i in start .. end {
            if self.line_join == LineJoin::Round {
                dv.xb1 = dv.curr.x1 + (dv.curr.y2 - dv.curr.y1);
                dv.yb1 = dv.curr.y1 - (dv.curr.x2 - dv.curr.x1);
                dv.xb2 = dv.curr.x2 + (dv.curr.y2 - dv.curr.y1);
                dv.yb2 = dv.curr.y2 - (dv.curr.x2 - dv.curr.x1);
            }
            match dv.flags {
                0 => self.ren.line3(&dv.curr, dv.xb1, dv.yb1, dv.xb2, dv.yb2),
                1 => self.ren.line2(&dv.curr, dv.xb2, dv.yb2),
                2 => self.ren.line1(&dv.curr, dv.xb1, dv.yb1),
                3 => self.ren.line0(&dv.curr),
                _ => unreachable!("flag value not covered")
            }
            if self.line_join == LineJoin::Round && (dv.flags & 2) == 0 {
                self.ren.pie(dv.curr.x2, dv.curr.y2,
                             dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                             dv.curr.y2 - (dv.curr.x2 - dv.curr.x1),
                             dv.curr.x2 + (dv.next.y2 - dv.next.y1),
                             dv.curr.y2 - (dv.next.x2 - dv.next.x1));
            }
            // Increment to next segment
            dv.x1 = dv.x2;
            dv.y1 = dv.y2;
            dv.lcurr = dv.lnext;
            //dv.lnext = self.vertices[dv.idx].len;
            let v0 = self.vertices[dv.idx];
            dv.idx += 1;
            if dv.idx >= self.vertices.len() {
                dv.idx = 0;
            }

            let v = self.vertices[dv.idx];
            dv.x2 = v.x;
            dv.y2 = v.y;
            dv.lnext = len_i64(&v0,&v);

            dv.curr = dv.next;
            dv.next = LineParameters::new(dv.x1, dv.y1, dv.x2, dv.y2, dv.lnext);
            dv.xb1 = dv.xb2;
            dv.yb1 = dv.yb2;

            match self.line_join {
                LineJoin::Bevel | LineJoin::MiterRevert | LineJoin::MiterRound => dv.flags = 3,
                LineJoin::None => dv.flags = 3,
                LineJoin::Miter => {
                    dv.flags >>= 1;
                    if dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant() {
                        dv.flags |= 1 << 1;
                    }
                    if (dv.flags & 2) == 0 {
                        let (xb2,yb2) = Self::bisectrix(&dv.curr, &dv.next);
                        dv.xb2 = xb2;
                        dv.yb2 = yb2;
                    }
                },
                LineJoin::Round => {
                    dv.flags >>= 1;
                    if dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant() {
                        dv.flags |= 1 << 1;
                    }
                },
                LineJoin::MiterAccurate => {
                    dv.flags = 0;
                    let (xb2,yb2) = Self::bisectrix(&dv.curr, &dv.next);
                    dv.xb2 = xb2;
                    dv.yb2 = yb2;
                }
            }
        }
    }
    fn bisectrix(l1: &LineParameters, l2: &LineParameters) -> (i64, i64) {
        let k = l2.len as f64 / l1.len as f64;
        let mut tx = l2.x2 as f64 - (l2.x1 - l1.x1) as f64 * k;
        let mut ty = l2.y2 as f64 - (l2.y1 - l1.y1) as f64 * k;

        //All bisectrices must be on the right of the line
        //If the next point is on the left (l1 => l2.2)
        //then the bisectix should be rotated by 180 degrees.
        if ((l2.x2 - l2.x1) as f64 * (l2.y1 - l1.y1) as f64) <
            ((l2.y2 - l2.y1) as f64 * (l2.x1 - l1.x1) as f64 + 100.0) {
            tx -= (tx - l2.x1 as f64) * 2.0;
            ty -= (ty - l2.y1 as f64) * 2.0;
        }

        // Check if the bisectrix is too short
        let dx = tx - l2.x1 as f64;
        let dy = ty - l2.y1 as f64;
        if ((dx * dx + dy * dy).sqrt() as i64) < POLY_SUBPIXEL_SCALE {
            let x = (l2.x1 + l2.x1 + (l2.y1 - l1.y1) + (l2.y2 - l2.y1)) >> 1;
            let y = (l2.y1 + l2.y1 - (l2.x1 - l1.x1) - (l2.x2 - l2.x1)) >> 1;
            (x,y)
        } else {
            (tx.round() as i64,ty.round() as i64)
        }
    }
}


#[derive(Debug)]
/// Outline Renderer with Anti-Aliasing
pub struct RendererOutlineAA<'a,T>  {
    ren: &'a mut RenderingBase<T>,
    color: Rgba8,
    clip_box: Option<Rectangle<i64>>,
    profile: LineProfileAA,
}

impl<'a,T> RendererOutlineAA<'a,T> where T: Pixel {
    /// Create Outline Renderer with a [`RenderingBase`](../base/struct.RenderingBase.html)
    pub fn with_base(ren: &'a mut RenderingBase<T>) -> Self {
        let profile = LineProfileAA::new();
        Self { ren, color: Rgba8::black(), clip_box: None, profile }
    }
    /// Set width of the line
    pub fn width(&mut self, width: f64) {
        self.profile.width(width);
    }
    /// Set minimum with of the line
    ///
    /// Use [`width`](#method.width) for this to take effect
    pub fn min_width(&mut self, width: f64) {
        self.profile.min_width(width);
    }
    /// Set smoother width of the line
    ///
    /// Use [`width`](#method.width) for this to take effect
    pub fn smoother_width(&mut self, width: f64) {
        self.profile.smoother_width(width);
    }

    fn subpixel_width(&self) -> i64 {
        self.profile.subpixel_width
    }

    /// Draw a Line Segment
    ///
    /// If line to "too long", divide it by two and draw both segments
    /// otherwise, interpolate along the line to draw
    /// 
    fn line0_no_clip(&mut self, lp: &LineParameters) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            self.line0_no_clip(&lp1);
            self.line0_no_clip(&lp2);
            return;
        }
        let mut li = lp.interp0(self.subpixel_width());
        if li.count() > 0 {
            if li.vertical() {
                while li.step_ver(self) {
                }
            } else {
                while li.step_hor(self) {
                }
            }
        }
    }
    fn line1_no_clip(&mut self, lp: &LineParameters, sx: i64, sy: i64) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            self.line1_no_clip(&lp1, (lp.x1 + sx)>>1, (lp.y1+sy)>>1);
            self.line1_no_clip(&lp2, lp.x2 + (lp.y1 + lp1.y1), lp1.y2 - (lp1.x2-lp1.x1));
            return;
        }
        let (sx, sy) = lp.fix_degenerate_bisectrix_start(sx, sy);
        let mut li = lp.interp1(sx, sy, self.subpixel_width());
        if li.vertical() {
            while li.step_ver(self) {
            }
        } else {
            while li.step_hor(self) {
            }
        }
    }
    fn line2_no_clip(&mut self, lp: &LineParameters, ex: i64, ey: i64) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1,lp2) = lp.divide();
            self.line2_no_clip(&lp1, lp1.x2 + (lp1.y2 - lp1.y1), lp1.y2 - (lp1.x2 - lp1.x1));
            self.line2_no_clip(&lp2, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }
        let (ex, ey) = lp.fix_degenerate_bisectrix_end(ex, ey);
        let mut li = lp.interp2(ex, ey, self.subpixel_width());
        if li.vertical() {
            while li.step_ver(self) {
            }
        } else {
            while li.step_hor(self) {
            }
        }
    }
    fn line3_no_clip(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64) {
        if lp.len > LINE_MAX_LENGTH {
            let (lp1, lp2) = lp.divide();
            let mx = lp1.x2 + (lp1.y2 - lp1.y1);
            let my = lp1.y2 - (lp1.x2 - lp1.x1);
            self.line3_no_clip(&lp1, (lp.x1 + sx) >> 1, (lp.y1 + sy) >> 1, mx, my);
            self.line3_no_clip(&lp2, mx, my, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }
        let (sx, sy) = lp.fix_degenerate_bisectrix_start(sx, sy);
        let (ex, ey) = lp.fix_degenerate_bisectrix_end(ex, ey);
        let mut li = lp.interp3(sx, sy, ex, ey, self.subpixel_width());
        if li.vertical() {
            while li.step_ver(self) {
            }
        } else {
            while li.step_hor(self) {
            }
        }
    }

    fn semidot_hline<F>(&mut self, cmp: F, xc1: i64, yc1: i64, xc2: i64, yc2: i64, x1: i64, y1: i64, x2: i64)
    where F: Fn(i64) -> bool
    {
        let mut x1 = x1;
        let mut covers = [0u64; MAX_HALF_WIDTH * 2 + 4];
        let p0 = 0;
        let mut p1 = 0;
        let mut x = x1 << POLY_SUBPIXEL_SHIFT;
        let mut y = y1 << POLY_SUBPIXEL_SHIFT;
        let w = self.subpixel_width();

        let mut di = DistanceInterpolator0::new(xc1, yc1, xc2, yc2, x, y);
        x += POLY_SUBPIXEL_SCALE/2;
        y += POLY_SUBPIXEL_SCALE/2;

        let x0 = x1;
        let mut dx = x - xc1;
        let dy = y - yc1;
        loop {
            let d = ((dx*dx + dy*dy) as f64).sqrt() as i64;
            covers[p1] = 0;
            if cmp(di.dist) && d <= w {
                covers[p1] = self.cover(d);
            }
            p1 += 1;
            dx += POLY_SUBPIXEL_SCALE;
            di.inc_x();
            x1 += 1;
            if x1 > x2 {
                break;
            }
        }
        self.ren.blend_solid_hspan(x0, y1,
                                   (p1 - p0) as i64,
                                   self.color,
                                   &covers);
    }

    fn pie_hline(&mut self, xc: i64, yc: i64, xp1: i64, yp1: i64, xp2: i64, yp2: i64, xh1: i64, yh1: i64, xh2: i64) {
        if let Some(clip_box) = self.clip_box {
            if clip_box.clip_flags(xc, yc) != 0 {
                return;
            }
        }
        let mut xh1 = xh1;
        let mut covers = [0u64; MAX_HALF_WIDTH * 2 + 4];

        let p0 = 0;
        let mut p1 = 0;
        let mut x = xh1 << POLY_SUBPIXEL_SHIFT;
        let mut y = yh1 << POLY_SUBPIXEL_SHIFT;
        let w = self.subpixel_width();

        let mut di = DistanceInterpolator00::new(xc, yc, xp1, yp1, xp2, yp2,
                                                 x, y);
        x += POLY_SUBPIXEL_SCALE/2;
        y += POLY_SUBPIXEL_SCALE/2;

        let xh0 = xh1;
        let mut dx = x - xc;
        let dy = y - yc;
        loop {
            let d = ((dx*dx + dy*dy) as f64).sqrt() as i64;
            covers[p1] = 0;
            if di.dist1 <= 0 && di.dist2 > 0 && d <= w {
                covers[p1] = self.cover(d);
            }
            p1 += 1;
            dx += POLY_SUBPIXEL_SCALE;
            di.inc_x();
            xh1 += 1;
            if xh1 > xh2 {
                break;
            }
        }
        self.ren.blend_solid_hspan(xh0, yh1,
                                   (p1 - p0) as i64,
                                   self.color,
                                   &covers);

    }

}

impl<T> RenderOutline for RendererOutlineAA<'_, T> where T: Pixel {
    fn cover(&self, d: i64) -> u64 {
        let subpixel_shift = POLY_SUBPIXEL_SHIFT;
        let subpixel_scale = 1 << subpixel_shift;
        let index = d + i64::from(subpixel_scale) * 2;
        assert!(index >= 0);
        u64::from( self.profile.profile[index as usize] )
    }
    fn blend_solid_hspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]) {
        self.ren.blend_solid_hspan(x, y, len,  self.color, covers);
    }
    fn blend_solid_vspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]) {
        self.ren.blend_solid_vspan(x, y, len,  self.color, covers);
    }
}

impl<T> DrawOutline for RendererOutlineAA<'_, T> where T: Pixel {
    fn line3(&mut self, lp: &LineParameters, sx: i64, sy: i64, ex: i64, ey: i64) {
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1, lp.y1, lp.x2, lp.y2, clip_box);
            if (flags & 4) == 0 {
                let (mut sx, mut sy, mut ex, mut ey) = (sx,sy,ex,ey);
                if flags != 0{
                    let lp2 = LineParameters::new(x1,y1,x2,y2,
                                                  len_i64_xy(x1, y1, x2, y2));
                    if flags & 1 != 0{
                        sx = x1 + (y2 - y1);
                        sy = y1 - (x2 - x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy - lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    if flags & 2 != 0{
                        ex = x2 + (y2 - y1);
                        ey = y2 - (x2 - x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line3_no_clip(&lp2, sx, sy, ex, ey);
                } else {
                    self.line3_no_clip(&lp, sx, sy, ex, ey);
                }
            }
        } else {
            self.line3_no_clip(&lp, sx, sy, ex, ey);
        }
    }
    fn semidot<F>(&mut self, cmp: F, xc1: i64, yc1: i64, xc2: i64, yc2: i64)
    where F: Fn(i64) -> bool
    {
        if let Some(clip_box) = self.clip_box {
            if clip_box.clip_flags(xc1, yc1) != 0 {
                return;
            }
        }

        let mut r = (self.subpixel_width() + POLY_SUBPIXEL_MASK) >> POLY_SUBPIXEL_SHIFT;
        if r < 1 {
            r = 1;
        }
        let mut ei = EllipseInterpolator::new(r, r);
        let mut dx = 0;
        let mut dy = -r;
        let mut dy0 = dy;
        let mut dx0 = dx;
        let x = xc1 >> POLY_SUBPIXEL_SHIFT;
        let y = yc1 >> POLY_SUBPIXEL_SHIFT;

        loop {
            dx += ei.dx;
            dy += ei.dy;

            if dy != dy0 {
                self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x-dx0, y+dy0, x+dx0);
                self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x-dx0, y-dy0, x+dx0);
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
        self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x-dx0, y+dy0, x+dx0);
    }

    fn pie(&mut self, xc: i64, yc: i64, x1: i64, y1: i64, x2: i64, y2: i64) {
        let mut r = (self.subpixel_width() + POLY_SUBPIXEL_MASK) >> POLY_SUBPIXEL_SHIFT;
        if r < 1 {
            r = 1;
        }
        let mut ei = EllipseInterpolator::new(r, r);
        let mut dx = 0;
        let mut dy = -r;
        let mut dy0 = dy;
        let mut dx0 = dx;
        let x = xc >> POLY_SUBPIXEL_SHIFT;
        let y = yc >> POLY_SUBPIXEL_SHIFT;

        loop {
            dx += ei.dx;
            dy += ei.dy;

            if dy != dy0 {
                self.pie_hline(xc, yc, x1, y1, x2, y2, x-dx0, y+dy0, x+dx0);
                self.pie_hline(xc, yc, x1, y1, x2, y2, x-dx0, y-dy0, x+dx0);
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
        self.pie_hline(xc, yc, x1, y1, x2, y2, x-dx0, y+dy0, x+dx0);

    }
    /// Draw a Line Segment, clipping if necessary
    ///
    fn line0(&mut self, lp: &LineParameters) {
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1,lp.y1,lp.x2,lp.y2,clip_box);
            if flags & 4 == 0 { // Line in Visible
                if flags != 0 { // Line is Clipped
                    // Create new Line from clipped lines and draw
                    let lp2 = LineParameters::new(x1, y1, x2, y2,
                                                  len_i64_xy(x1,y1,x2,y2));
                    self.line0_no_clip(&lp2);
                } else {
                    // Line is not Clipped
                    self.line0_no_clip(&lp)
                }
            }
        } else {
            // No clip box defined
            self.line0_no_clip(&lp);
        }
    }
    fn line1(&mut self, lp: &LineParameters, sx: i64, sy: i64) {
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1,lp.y1,lp.x2,lp.y2, clip_box);
            if flags & 4 == 0 {
                if flags != 0{
                    let (mut sx, mut sy) = (sx,sy);
                    let lp2 = LineParameters::new(x1,y1,x2,y2, len_i64_xy(x1,y1,x2,y2));
                    if flags & 1 == 0 {
                        sx = x1 + (y2-y1);
                        sy = y1 - (x2-x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy-lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    self.line1_no_clip(&lp2, sx, sy);
                } else {
                    self.line1_no_clip(&lp, sx, sy);
                }
            }
        } else {
            self.line1_no_clip(&lp, sx, sy);
        }
    }
    fn line2(&mut self, lp: &LineParameters, ex: i64, ey: i64) {
        if let Some(clip_box) = self.clip_box {
            let (x1,y1,x2,y2,flags) = clip_line_segment(lp.x1,lp.y1,lp.x2,lp.y2, clip_box);
            if flags & 4 == 0 {
                if flags != 0 {
                    let (mut ex,mut ey) = (ex,ey);
                    let lp2 = LineParameters::new(x1,y1,x2,y2, len_i64_xy(x1,y1,x2,y2));
                    if flags & 2 != 0{
                        ex = x2 + (y2-y1);
                        ey = y2 + (x2-x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line2_no_clip(&lp2, ex, ey);
                } else {
                    self.line2_no_clip(&lp, ex, ey);
                }
            }
        } else {
            self.line2_no_clip(&lp, ex, ey);
        }
    }
    fn color<C: Color>(&mut self, color: C) {
        self.color = Rgba8::from_trait(color);
    }

    fn accurate_join_only(&self) -> bool{
        false
    }
}

#[derive(Debug,Default)]
/// Profile of a Line
struct LineProfileAA {
    min_width: f64,
    smoother_width: f64,
    subpixel_width: i64,
    gamma: Vec<u8>,
    profile: Vec<u8>,
}

impl LineProfileAA {
    /// Create new LineProfile
    ///
    /// Width is initialized to 0.0
    pub fn new() -> Self {
        let gamma : Vec<_> = (0..POLY_SUBPIXEL_SCALE).map(|x| x as u8).collect();
        let mut s = Self { min_width: 1.0,
                           smoother_width: 1.0,
                           subpixel_width: 0,
                           profile: vec![], gamma };
        s.width(0.0);
        s
    }
    /// Set minimum width
    ///
    /// For this to take effect, the width needs to be set
    pub fn min_width(&mut self, width: f64) {
        self.min_width = width;
    }
    /// Set smoother width
    ///
    /// For this to take effect, the width needs to be set
    pub fn smoother_width(&mut self, width: f64) {
        self.smoother_width = width;
    }
    /// Set width
    ///
    /// Negative widths are set to 0.0
    ///
    /// Width less than smoother width are doubled, otherwise the smoother width is added
    ///  to the with
    /// Widths are then divied by 2 and the smoother width is removed.
    ///
    /// The line profile is then constructed and saved to `profile`
    pub fn width(&mut self, w: f64) {
        let mut w = w;
        if w < 0.0 {
            w = 0.0;
        }
        if w < self.smoother_width {
            w += w;
        } else {
            w += self.smoother_width;
        }
        w *= 0.5;
        w -= self.smoother_width;
        let mut s = self.smoother_width;
        if w < 0.0 {
            s += w;
            w = 0.0;
        }
        self.set(w, s);
    }
    fn profile(&mut self, w: f64) {
        let subpixel_shift = POLY_SUBPIXEL_SHIFT;
        let subpixel_scale = 1 << subpixel_shift;
        self.subpixel_width = (w * subpixel_scale as f64).round() as i64;
        let size = (self.subpixel_width + subpixel_scale * 6) as usize;
        if size > self.profile.capacity() {
            self.profile.resize(size, 0);
        }
    }
    /// Create the Line Profile
    ///
    /// 
    fn set(&mut self, center_width: f64, smoother_width: f64) {
        let subpixel_shift = POLY_SUBPIXEL_SHIFT;
        let subpixel_scale = 1 << subpixel_shift;
        let aa_shift = POLY_SUBPIXEL_SHIFT;
        let aa_scale = 1 << aa_shift;
        let aa_mask = aa_scale - 1;

        let mut base_val = 1.0;
        let mut center_width = center_width;
        let mut smoother_width = smoother_width;

        // Set minimum values for the center and smoother widths
        if center_width == 0.0 {
            center_width = 1.0 / subpixel_scale as f64;
        }
        if smoother_width == 0.0 {
            smoother_width = 1.0 / subpixel_scale as f64;
        }
        // Full width
        let width = center_width + smoother_width;

        // Scale widths so they equal the minimum width
        if width < self.min_width {
            let k = width / self.min_width;
            base_val *= k;
            center_width /= k;
            smoother_width /= k;
        }

        // Allocate space for the line profile
        self.profile(center_width + smoother_width);

        // Width in Subpixel scales
        let subpixel_center_width : usize   = (center_width * subpixel_scale as f64) as usize;
        let subpixel_smoother_width : usize = (smoother_width * subpixel_scale as f64) as usize;
        // 
        let n_smoother = self.profile.len() -
            subpixel_smoother_width -
            subpixel_center_width -
            subpixel_scale*2;

        // Center and Smoother Width Offsets
        let ch_center   = subpixel_scale*2;
        let ch_smoother = ch_center + subpixel_center_width;

        // Fill center portion of the profile (on one side) base_val
        let val = self.gamma[(base_val * f64::from(aa_mask)) as usize];
        for i in 0 .. subpixel_center_width {
            self.profile[ch_center + i] = val;
        }
        // Fill smoother portion of the profile with value decreasing linearly
        for i  in 0 .. subpixel_smoother_width {
            let k = ((base_val - base_val * (i as f64 / subpixel_smoother_width as f64)) * f64::from(aa_mask)) as usize;
            self.profile[ch_smoother + i] = self.gamma[k];
        }

        // Remainder is essentially 0.0
        let val = self.gamma[0];
        for i in 0 .. n_smoother  {
            self.profile[ch_smoother + subpixel_smoother_width + i] = val;
        }
        // Copy to other side
        for i in 0 .. subpixel_scale*2 {
            self.profile[ch_center - 1 - i] = self.profile[ch_center + i]
        }
    }
}

/// Ellipse Interpolator
#[derive(Debug)]
struct EllipseInterpolator {
    rx2: i64,
    ry2: i64,
    two_rx2: i64,
    two_ry2: i64,
    dx: i64,
    dy: i64,
    inc_x: i64,
    inc_y: i64,
    cur_f: i64,
}

impl EllipseInterpolator {
    /// Create new Ellipse Interpolator with axes lenghts `rx` and `ry`
    pub fn new(rx: i64, ry: i64) -> Self {
        let rx2 = rx * rx;
        let ry2 = ry * ry;
        let two_rx2 = rx2 * 2;
        let two_ry2 = ry2 * 2;
        let dx = 0;
        let dy = 0;
        let inc_x = 0;
        let inc_y = -ry * two_rx2;
        let cur_f = 0;

        Self { rx2, ry2, two_rx2, two_ry2, dx, dy, inc_x, inc_y, cur_f }
    }

    /// Increment the Interpolator
    fn inc(&mut self) {
        // 
        let mut mx = self.cur_f + self.inc_x + self.ry2;
        let fx = mx;
        if mx < 0 {
            mx = -mx;
        }

        let mut my = self.cur_f + self.inc_y + self.rx2;
        let fy = my;
        if my < 0  {
            my = -my;
        }

        let mut mxy = self.cur_f + self.inc_x + self.ry2 + self.inc_y + self.rx2;
        let fxy = mxy;
        if mxy < 0 {
            mxy = -mxy;
        }

        let mut min_m = mx;

        let flag = if min_m > my {
            min_m = my;
            false
        } else {
            true
        };

        self.dx = 0;
        self.dy = 0;
        if min_m > mxy {
            self.inc_x += self.two_ry2;
            self.inc_y += self.two_rx2;
            self.cur_f = fxy;
            self.dx = 1;
            self.dy = 1;
            return;
        }

        if flag {
            self.inc_x += self.two_ry2;
            self.cur_f = fx;
            self.dx = 1;
            return;
        }

        self.inc_y += self.two_rx2;
        self.cur_f = fy;
        self.dy = 1;
    }
}
