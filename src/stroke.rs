//! Path Stroking
//!
//! # Example
//!
//!     // Input Path
//!     let mut path = agg::Path::new();
//!     path.move_to(  0.0,   0.0);
//!     path.line_to(100.0, 100.0);
//!     path.line_to(200.0,  50.0);
//!
//!     // Stroke
//!     let mut stroke = agg::Stroke::new( path );
//!     stroke.width(2.5);
//!     stroke.line_cap(agg::LineCap::Square);
//!     stroke.line_join(agg::LineJoin::Miter);
//!     stroke.line_join(agg::LineJoin::Miter);
//!     stroke.miter_limit(5.0);
//!
//!     // Draw
//!     let mut ras = agg::RasterizerScanline::new();
//!     ras.add_path(&stroke);
//!

use crate::paths::PathCommand;
use crate::paths::Vertex;
use crate::paths::len;
use crate::paths::cross;
use crate::paths::split;

use crate::VertexSource;
use std::f64::consts::PI;

/// Line End or Cap Style
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum LineCap {
    Butt, Square, Round
}
/// Lines Join Style on the outside
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum LineJoin {
    Miter, MiterRevert, Round, Bevel, MiterRound,  MiterAccurate, None,
}
/// Lines Join Style on the inside
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InnerJoin {
    Bevel, Miter, Jag, Round
}

/// Stroke for Paths and Vertex Sources
///
/// **Missing:** fn shorten() [private]
///
#[derive(Debug)]
pub struct Stroke<T: VertexSource> {
    /// Source of Verticies
    source: T,
    /// Width of line in pixels, can be negative, 0.5
    width: f64,
    /// Absolute value of the width in pixel, 0.5
    width_abs: f64,
    /// Minimum Limit to determine if segments are almost co-linear, 0.5/1024
    width_eps: f64,
    /// Sign of the width, +1.0
    width_sign: f64,
    /// Maximum Length of miter at segment intersection, 3.0
    miter_limit: f64,
    /// Maximum Length of the inner miter at segment intersections, 1.01
    inner_miter_limit: f64,
    /// Approximation scale, 1.0
    approx_scale: f64,
    /// Line Cap Style
    line_cap: LineCap,
    /// Line Join Style
    line_join: LineJoin,
    /// Line Join Style, Inner Angle
    inner_join: InnerJoin,
}

impl<T> VertexSource for Stroke<T> where T: VertexSource {
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.stroke()
    }
}

macro_rules! prev {
    ($i:expr, $n:expr) => ( ($i + $n - 1) % $n )
}
macro_rules! curr {
    ($i:expr, $n:expr) => ( $i )
}
macro_rules! next {
    ($i:expr, $n:expr) => ( ($i + 1) % $n )
}

impl<T> Stroke<T> where T: VertexSource {
    /// Create a new Stroke from a Vertex Source
    pub fn new(source: T) -> Self {
        Self {
            source,
            width: 0.5,
            width_abs: 0.5,
            width_eps: 0.5/1024.0,
            width_sign: 1.0,
            miter_limit: 4.0,
            inner_miter_limit: 1.01,
            approx_scale: 1.0,
            inner_join: InnerJoin::Miter,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
        }
    }
    /// Set the Stroke Width
    pub fn width(&mut self, width: f64) {
        self.width = width / 2.0;
        self.width_abs = self.width.abs();
        self.width_sign = if self.width < 0.0 { -1.0 } else { 1.0 };
    }
    /// Set Line cap style
    ///
    /// Available options are
    ///   - `Butt`
    ///   - `Square`
    ///   - `Round`
    pub fn line_cap(&mut self, line_cap: LineCap) {
        self.line_cap = line_cap;
    }
    /// Set Line Join style
    ///
    /// Available options are
    ///   - `Miter`
    ///   - `MiterRevert`
    ///   - `RoundJoin`
    ///   - `Bevel`
    ///   - `MiterRound`
    ///
    /// Variants of `MiterAccurate` and `None` are not available and will
    /// be reset to `Miter`
    ///
    pub fn line_join(&mut self, line_join: LineJoin) {
        self.line_join = line_join;
        if self.line_join == LineJoin::MiterAccurate {
            self.line_join = LineJoin::Miter;
        }
        if self.line_join == LineJoin::None {
            self.line_join = LineJoin::Miter;
        }
    }
    /// Set Inner Join style
    ///
    /// Available options are
    ///   - `Bevel`
    ///   - `Miter`
    ///   - `Jag`
    ///   - `Round`
    pub fn inner_join(&mut self, inner_join: InnerJoin) {
        self.inner_join = inner_join;
    }
    /// Set miter limit
    pub fn miter_limit(&mut self, miter_limit: f64) {
        self.miter_limit = miter_limit;
    }
    // Set miter limit theta
    //pub fn miter_limit_theta(&mut self, miter_limit_theta: f64) {
    //    self.miter_limit_theta = miter_limit_theta;
    //}
    /// Set inner miter limit
    pub fn inner_miter_limit(&mut self, inner_miter_limit: f64) {
        self.inner_miter_limit = inner_miter_limit;
    }
    /// Set approximation scale
    pub fn approximation_scale(&mut self, scale: f64) {
        self.approx_scale = scale;
    }
    /// Calculate Line End Cap
    ///
    fn calc_cap(&self, v0: &Vertex<f64>, v1: &Vertex<f64>) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let dx = v1.x-v0.x;
        let dy = v1.y-v0.y;
        let len = (dx*dx + dy*dy).sqrt();
        let dx1 = self.width * dy / len;
        let dy1 = self.width * dx / len;

        match self.line_cap {
            LineCap::Square => {
                let dx2 = dy1 * self.width_sign;
                let dy2 = dx1 * self.width_sign;
                out.push(Vertex::line_to(v0.x - dx1 - dx2, v0.y + dy1 - dy2));
                out.push(Vertex::line_to(v0.x + dx1 - dx2, v0.y - dy1 - dy2));
            },
            LineCap::Butt => {
                out.push(Vertex::line_to(v0.x - dx1, v0.y + dy1));
                out.push(Vertex::line_to(v0.x + dx1, v0.y - dy1));
            },
            LineCap::Round => {
                let da = 2.0 * (self.width_abs / (self.width_abs + 0.125 / self.approx_scale)).acos();
                let n = (PI / da).round() as usize;

                let da = PI / (n + 1) as f64;
                out.push(Vertex::line_to(v0.x - dx1, v0.y + dy1));
                if self.width_sign > 0.0 {
                    let mut a1 = dy1.atan2(-dx1);
                    a1 += da;
                    for _ in 0 .. n {
                        out.push(Vertex::line_to(v0.x + a1.cos() * self.width,
                                                 v0.y + a1.sin() * self.width));
                        a1 += da;
                    }
                } else {
                    let mut a1 = (-dy1).atan2(dx1);
                    a1 -= da;
                    for _ in 0 .. n {
                        out.push(Vertex::line_to(v0.x + a1.cos() * self.width,
                                                 v0.y + a1.sin() * self.width));
                        a1 -= da;
                    }
                }
                out.push(Vertex::line_to(v0.x + dx1, v0.y - dy1));
            }
        }
        out
    }

    /// Calculate an Arc
    ///
    /// Returns Vertices represening the Arc
    ///
    fn calc_arc(&self, x: f64, y: f64, dx1: f64, dy1: f64, dx2: f64, dy2: f64) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        // Starting and Ending Angle
        let mut a1 = (dy1 * self.width_sign).atan2(dx1 * self.width_sign);
        let mut a2 = (dy2 * self.width_sign).atan2(dx2 * self.width_sign);

        // 
        let mut da = 2.0 * (self.width_abs / (self.width_abs + 0.125 / self.approx_scale)).acos();
        out.push(Vertex::line_to(x + dx1, y + dy1));
        // Positive Line Width
        if self.width_sign > 0.0 {
            // Require a1 > a2
            if a1 > a2 {
                a2 += 2.0 * PI;
            }
            // Number of points in Arc
            let n = ((a2 - a1) / da) as i64;
            // Arc Increment in radians
            da = (a2 - a1) / (n + 1) as f64;
            // Increment from original angle as a1 is at initial point
            a1 += da;
            // Create Arc
            for _ in 0 .. n {
                out.push(Vertex::line_to(x + a1.cos() * self.width,
                                         y + a1.sin() * self.width));
                a1 += da;
            }
        } else {
            // Negative Line Width
            // Require: a2 < a1
            if a1 < a2 {
                a2 -= 2.0 * PI;
            }
            // Number of point in Arc
            let n = ((a1 - a2) / da) as i64;
            // Arc Increment in radians
            da = (a1 - a2) / (n + 1) as f64;
            // Decrement from original angle as a1 is at initial point
            a1 -= da;
            // Create Arc
            for _ in 0 .. n {
                out.push(Vertex::line_to(x + a1.cos() * self.width,
                                         y + a1.sin() * self.width));
                a1 -= da;
            }
        }
        // Add Last Point
        out.push(Vertex::line_to(x + dx2, y + dy2));
        out
    }
    /// Calculate a Miter Join
    ///
    /// Return the Miter Join for 3 points
    ///
    fn calc_miter(&self,
                  p0: &Vertex<f64>,
                  p1: &Vertex<f64>,
                  p2: &Vertex<f64>,
                  dx1: f64, dy1: f64, dx2: f64, dy2: f64,
                  join: LineJoin, mlimit: f64, dbevel: f64)
                  -> Vec<Vertex<f64>>{
        let mut out = vec![];
        let mut xi  = p1.x;
        let mut yi  = p1.y;
        let mut di  = 1.0;
        let lim = self.width_abs * mlimit;
        let mut miter_limit_exceeded = true; // Assume the worst
        let mut intersection_failed  = true; // Assume the worst
        // Find the Intersection between the two points
        //
        // a--b-p
        // 0   1 c
        // -----  \
        //      \  \
        //       \2 d
        if let Some((xit,yit)) = self.calc_intersection(p0.x + dx1, p0.y - dy1,   // a
                                                        p1.x + dx1, p1.y - dy1,   // b
                                                        p1.x + dx2, p1.y - dy2,   // c
                                                        p2.x + dx2, p2.y - dy2) { // d
            // Calculation of the intersection succeeded
            xi = xit;
            yi = yit;
            let pz = Vertex::line_to(xi,yi); // Intersection point
            di = len(p1,&pz); // Distance from p1 to p
            if di <= lim {
                // Inside the miter limit - Simplest case
                out.push(Vertex::line_to(xi, yi));
                miter_limit_exceeded = false;
            }
            intersection_failed = false;
        } else {
            // Calculation of the intersection failed, most probably
            // the three points lie one straight line. 
            // First check if v0 and v2 lie on the opposite sides of vector: 
            // (v1.x, v1.y) -> (v1.x+dx1, v1.y-dy1), that is, the perpendicular
            // to the line determined by vertices v0 and v1.
            // This condition determines whether the next line segments continues
            // the previous one or goes back.
            //----------------
            let x2 = p1.x + dx1;
            let y2 = p1.y - dy1;
            let pz = Vertex::line_to(x2,y2);
            if (cross(&p0, &p1, &pz) < 0.0) == (cross(&p1, &p2, &pz) < 0.0) {
                // This case means that the next segment continues
                // the previous one (straight line)
                //-----------------
                out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                miter_limit_exceeded = false;
            }
        }

        if miter_limit_exceeded {
            // Miter limit exceeded
            //------------------------
            match join {
                LineJoin::MiterRevert => {
                    // For the compatibility with SVG, PDF, etc,
                    // we use a simple bevel join instead of
                    // "smart" bevel
                    out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                },
                LineJoin::Round => out.extend( self.calc_arc(p1.x, p1.y, dx1, -dy1, dx2, -dy2)),
                _ => { // default
                    // If no miter-revert, calculate new dx1, dy1, dx2, dy2
                    //----------------
                    if intersection_failed {
                        let mlimit = mlimit * self.width_sign;
                        out.push(Vertex::line_to(p1.x + dx1 + dy1 * mlimit,
                                                 p1.y - dy1 + dx1 * mlimit));
                        out.push(Vertex::line_to(p1.x + dx2 - dy2 * mlimit,
                                                 p1.y - dy2 - dx2 * mlimit));
                    } else {
                        let x1 = p1.x + dx1;
                        let y1 = p1.y - dy1;
                        let x2 = p1.x + dx2;
                        let y2 = p1.y - dy2;
                        let di = (lim - dbevel) / (di - dbevel);
                        out.push(Vertex::line_to(x1 + (xi - x1) * di,
                                                 y1 + (yi - y1) * di));
                        out.push(Vertex::line_to(x2 + (xi - x2) * di,
                                                 y2 + (yi - y2) * di));
                    }
                }
            }
        }
        out
    }
    /// Calculate Intersection of two lines
    ///
    /// Parallel Line are return as `None` otherwise the Intersection
    ///    (`px`,`py`) is returned
    ///
    /// [Line-Line Intersection at Wikipedia](https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line)
    ///
    /// Lines are specified to pairs of points
    ///   - (`ax`, `ay`) -> (`bx`, `by`)
    ///   - (`cv`, `cy`) -> (`dx`, `dy`)
    ///
    /// The intersection is defined at
    ///```text
    ///     px = ax + t (bx-ax)
    ///     py = ay + y (by-ay)
    ///```
    ///   where
    ///```text
    ///          (ay-cy)(dx-cx) - (ax-cx)(dy-cy)
    ///     t = ----------------------------------
    ///          (bx-ax)(dy-cy) - (by-ay)(dx-cx)
    ///```
    fn calc_intersection(&self,
                         ax: f64, ay: f64, bx: f64, by: f64,
                         cx: f64, cy: f64, dx: f64, dy: f64)
                         -> Option<(f64, f64)> {
        let intersection_epsilon = 1.0e-30;
        // Numerator
        let num = (ay-cy) * (dx-cx) - (ax-cx) * (dy-cy);
        // Denominator
        let den = (bx-ax) * (dy-cy) - (by-ay) * (dx-cx);
        // Denominator == 0 :: Lines are Parallel or Co-Linear
        if den.abs() < intersection_epsilon {
            return None;
        }
        // Compute Intersection
        let r = num / den;
        let x = ax + r * (bx-ax);
        let y = ay + r * (by-ay);
        Some((x,y))
    }
    /// Calculate the Join of Two Line Segments
    ///
    /// [SVG Line Joins](https://www.w3.org/TR/SVG/painting.html#LineJoin)
    ///
    fn calc_join(&self,
                     p0: &Vertex<f64>,
                     p1: &Vertex<f64>,
                     p2: &Vertex<f64>) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let len1 = len(p1,p0);
        let len2 = len(p2,p1);

        if len1 == 0.0 {
            panic!("Same point between p0,p1 {:?} {:?}", p0,p1);
        }
        if len2 == 0.0 {
            panic!("Same point between p1,p2 {:?} {:?}", p1,p2);
        }
        // Distance, perpendidular from line
        let dx1 = self.width * (p1.y-p0.y) / len1;
        let dy1 = self.width * (p1.x-p0.x) / len1;
        let dx2 = self.width * (p2.y-p1.y) / len2;
        let dy2 = self.width * (p2.x-p1.x) / len2;
        // Cross Product of the three points
        let cp = cross(p0, p1, p2);

        if cp != 0.0 && cp.is_sign_positive() == self.width.is_sign_positive() {
            // Inner Join
            let mut limit = if len1 < len2 {
                len1 / self.width_abs
            } else {
                len2 / self.width_abs
            };
            // Enforce Minimum Miter Limit
            if limit < self.inner_miter_limit {
                limit = self.inner_miter_limit;
            }
            // Construct Joins
            match self.inner_join {
                // Simple Bevel Join
                InnerJoin::Bevel => {
                    out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                },
                InnerJoin::Miter => {
                    out.extend(self.calc_miter(p0, p1, p2, dx1, dy1, dx2, dy2, LineJoin::MiterRevert, limit, 0.0));
                }
                InnerJoin::Jag |
                InnerJoin::Round => {
                    let cp = (dx1-dx2).powi(2) + (dy1-dy2).powi(2);
                    if cp < len1.powi(2) && cp < len2.powi(2) {
                        out.extend(self.calc_miter(p0,p1,p2, dx1, dy1, dx2, dy2, LineJoin::MiterRevert, limit, 0.0));
                    } else {
                        if self.inner_join == InnerJoin::Jag {
                            out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                            out.push(Vertex::line_to(p1.x,       p1.y      ));
                            out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                        }
                        if self.inner_join == InnerJoin::Round {
                            out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                            out.push(Vertex::line_to(p1.x,       p1.y      ));
                            out.extend(self.calc_arc(p1.x, p1.y, dx2, -dy2, dx1, -dy1));
                            out.push(Vertex::line_to(p1.x,       p1.y      ));
                            out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                        }
                    }
                }
            }
        } else {
            // Outer Join
            let dx = (dx1 + dx2) / 2.0;
            let dy = (dy1 + dy2) / 2.0;
            let dbevel = (dx*dx + dy*dy).sqrt();

            if (self.line_join == LineJoin::Round || self.line_join == LineJoin::Bevel) &&  self.approx_scale * (self.width_abs - dbevel) < self.width_eps {
                // This is an optimization that reduces the number of points 
                // in cases of almost collinear segments. If there's no
                // visible difference between bevel and miter joins we'd rather
                // use miter join because it adds only one point instead of two. 
                //
                // Here we calculate the middle point between the bevel points 
                // and then, the distance between v1 and this middle point. 
                // At outer joins this distance always less than stroke width, 
                // because it's actually the height of an isosceles triangle of
                // v1 and its two bevel points. If the difference between this
                // width and this value is small (no visible bevel) we can 
                // add just one point. 
                //
                // The constant in the expression makes the result approximately 
                // the same as in round joins and caps. You can safely comment 
                // out this entire "if".
                //-------------------

                if let Some((dx,dy)) =
                    self.calc_intersection(p0.x + dx1, p0.y - dy1,
                                           p1.x + dx1, p1.y - dy1,
                                           p1.x + dx2, p1.y - dy2,
                                           p2.x + dx2, p2.y - dy2) {
                        out.push(Vertex::line_to(dx, dy));
                    } else {
                        out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                    }
                return out ;
            }
            match self.line_join {
                LineJoin::Miter |
                LineJoin::MiterRevert |
                LineJoin::MiterRound => 
                    out.extend(self.calc_miter(p0,p1,p2, dx1,dy1,dx2,dy2,
                                               self.line_join,
                                               self.miter_limit,
                                               dbevel)),
                LineJoin::Round => out.extend(
                    self.calc_arc(p1.x, p1.y, dx1, -dy1, dx2, -dy2)
                ),
                LineJoin::Bevel => {
                    out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                },
                LineJoin::None | LineJoin::MiterAccurate => {},
            }
        }
        out
    }
    /// Stroke the Vertex Source
    ///
    /// There is lots of logic here and probably overly complex
    ///
    fn stroke(&self) -> Vec<Vertex<f64>> {
        let mut all_out = vec![];
        // Get verticies from Vertex Source
        let v0 = &self.source.xconvert();
        // Split and loop along unique paths, ended by MoveTo's
        let pairs = split(&v0);
        for (m1,m2) in pairs {
            let mut outf = vec![];
            // Clean the current path, return new path
            let v = clean_path(&v0[m1..=m2]);
            // Check for Closed Path Element
            let closed = is_path_closed(&v);
            // Ignore Closed Tag Element
            let n = if closed { v.len() - 1 } else { v.len() };
            let (n1,n2) = if closed { (0, n) } else { (1,n-1) };

            // Forward Path
            if ! closed {
                outf.extend( self.calc_cap(&v[0], &v[1]) );
            }
            for i in n1 .. n2 { // Forward Path
                outf.extend(
                    self.calc_join(&v[prev!(i,n)], &v[curr!(i,n)], &v[next!(i,n)])
                );
            }
            if closed {
                // Close the polygon
                let n = outf.len();
                let last = outf[n-1];
                outf.push( Vertex::close_polygon(last.x, last.y) );
            }

            // Backward Path
            let mut outb = vec![];
            if ! closed {
                outb.extend( self.calc_cap(&v[n-1], &v[n-2]) ); // End Cap
            }
            for i in (n1 .. n2).rev() { // Backward Path
                outb.extend(
                    self.calc_join(&v[next!(i,n)], &v[curr!(i,n)], &v[prev!(i,n)])
                );
            }
            if closed {
                // Set first point as a MoveTo
                outb[0].cmd = PathCommand::MoveTo;
                // Close the polygon, using the last point
                let n = outb.len();
                let last = outb[n-1];
                outb.push( Vertex::close_polygon(last.x, last.y) );
            } else {
                // Close the polygon, using the last point
                let n = outb.len();
                let last = outb[n-1];
                outb.push( Vertex::close_polygon(last.x, last.y) );
            }

            // Set First point as MoveTo
            outf[0].cmd = PathCommand::MoveTo;
            // Combine Forward and Backward Paths
            outf.extend(outb);

            // Add to Path Collection
            all_out.extend(outf);
        }
        all_out
    }
}

/// Check if Path is Closed
///
/// Path is considered close if the any of the verticies have
///   a PathCommand::Close vertex
///
fn is_path_closed(verts: &[Vertex<f64>]) -> bool {
    for v in verts {
        if v.cmd == PathCommand::Close {
            return true;
        }
    }
    false
}
/// Remove repeated vertices
///
/// Repeated verticies are defined with a distance <= 1e-6
///
fn clean_path(v: &[Vertex<f64>]) -> Vec<Vertex<f64>>{
    let mut mark = vec![];
    if ! v.is_empty() {
        mark.push(0);
    }
    // Find indicies of LineTo verticies far enough away from last point
    //  All other vertices are included
    for i in 1 .. v.len() {
        match v[i].cmd {
            PathCommand::LineTo => {
                if len(&v[i-1],&v[i]) >= 1e-6 {
                    mark.push(i);
                }
            },
            _ => mark.push(i),
        }
    }
    if mark.is_empty() {
        return vec![]
    }
    // Collect only "ok" verticies
    let mut out : Vec<_> = mark.into_iter().map(|i| v[i]).collect();

    // Return if path is not closeda
    if ! is_path_closed(&out) {
        return out;
    }
    // Path is closed
    let first = out[0];
    loop {
        // Get Last LineTo Command
        let i = match last_line_to(&out) {
            Some(i) => i,
            None => panic!("Missing Last Line To"),
        };
        let last = out[i];
        // If last point and first are **NOT** the same, done
        if len(&first, &last) >= 1e-6 {
            break;
        }
        // If **SAME** point, remove last Vertex and continue
        out.remove(i);
    }
    out
}

/// Return index of the last LineTo Vertex in the array
fn last_line_to(v: &[Vertex<f64>]) -> Option<usize> {
    let mut i = v.len()-1;
    while i > 0 {
        if v[i].cmd == PathCommand::LineTo {
            return Some(i);
        }
        i -= 1;
    }
    None
}
