

//use path_storage::PathStorage;
use crate::path_storage::PathCommand;
use crate::path_storage::Vertex;
use crate::path_storage::len;
use crate::path_storage::cross;
use crate::path_storage::split;

use crate::VertexSource;
use std::f64::consts::PI;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum LineCap {
    Butt, Square, Round
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum LineJoin {
    Miter, MiterRevert, Round, Bevel, MiterRound,  MiterAccurate, None,
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InnerJoin {
    Bevel, Miter, Jag, Round
}

impl Default for LineCap   { fn default() -> LineCap   { LineCap::Butt    } }
impl Default for LineJoin  { fn default() -> LineJoin  { LineJoin::Miter  } }
impl Default for InnerJoin { fn default() -> InnerJoin { InnerJoin::Miter } }

#[derive(Debug,Default)]
pub struct ConvStroke<T: VertexSource + Default> {
    source: T,//PathStorage,
    width: f64,
    width_abs: f64,
    width_eps: f64,
    width_sign: f64,
    miter_limit: f64,
    inner_miter_limit: f64,
    approx_scale: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    inner_join: InnerJoin,
}

impl<T> VertexSource for ConvStroke<T> where T: VertexSource + Default {
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

impl<T> ConvStroke<T> where T: VertexSource + Default {
    pub fn new(source: T) -> Self {
        Self {
            source,
            width: 0.5, width_abs: 0.5, width_eps: 0.5/1024.0, width_sign: 1.0,
            miter_limit: 4.0, inner_miter_limit: 1.01,
            approx_scale: 1.0,
            .. Default::default()
        }
    }
    pub fn width(&mut self, width: f64) {
        self.width = width / 2.0;
        self.width_abs = self.width.abs();
        self.width_sign = if self.width < 0.0 { -1.0 } else { 1.0 };
        //eprintln!("SET WIDTH");
    }
    fn calc_cap(&self, v0: &Vertex<f64>, v1: &Vertex<f64>) -> Vec<Vertex<f64>> {
        //eprintln!("JOIN: CAP: v0 {} {}", v0.x, v0.y);
        //eprintln!("JOIN: CAP: v1 {} {}", v1.x, v1.y);
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

            }
        }
        out
    }
    fn calc_arc(&self, x: f64, y: f64, dx1: f64, dy1: f64, dx2: f64, dy2: f64) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let mut a1 = (dy1 * self.width_sign).atan2(dx1 * self.width_sign);
        let mut a2 = (dy2 * self.width_sign).atan2(dx2 * self.width_sign);
        //let da = a1 - a2;
        //int i, n;

        let mut da = 2.0 * (self.width_abs / (self.width_abs + 0.125 / self.approx_scale)).acos();
        out.push(Vertex::line_to(x + dx1, y + dy1));
        if self.width_sign > 0.0 {
            if a1 > a2 {
                a2 += 2.0 * PI;
            }
            let n = ((a2 - a1) / da) as i64;
            da = (a2 - a1) / (n + 1) as f64;
            a1 += da;
            for _ in 0 .. n {
                out.push(Vertex::line_to(x + a1.cos() * self.width,
                                         y + a1.sin() * self.width));
                a1 += da;
            }
        } else {
            if a1 < a2 {
                a2 -= 2.0 * PI;
            }
            let n = ((a1 - a2) / da) as i64;
            da = (a1 - a2) / (n + 1) as f64;
            a1 -= da;
            for _ in 0 .. n {
                out.push(Vertex::line_to(x + a1.cos() * self.width,
                                         y + a1.sin() * self.width));
                a1 -= da;
            }
        }
        out.push(Vertex::line_to(x + dx2, y + dy2));
        out
    }
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
        //eprintln!("LINEOUT: calc_miter");
        //eprintln!("LINEOUT: dx,dy {} {} {} {}", dx1, dx2, dy1, dy2);
        //eprintln!("LINEOUT: mlimit {} width_abs {}", mlimit, self.width_abs);
        if let Some((xit,yit)) = self.calc_intersection(p0.x + dx1, p0.y - dy1,
                                                      p1.x + dx1, p1.y - dy1,
                                                      p1.x + dx2, p1.y - dy2,
                                                      p2.x + dx2, p2.y - dy2) {
                                               //&xi, &yi))
            // Calculation of the intersection succeeded
            //---------------------
            xi = xit;
            yi = yit;
            let pz = Vertex::line_to(xi,yi);
            //eprintln!("LINEOUT: intersection ok {:?}", pz);
            di = len(p1,&pz);
            if di <= lim {
                // Inside the miter limit
                //---------------------
                //eprintln!("LINEOUT: Inside the miter limit");
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
            //eprintln!("LINEOUT: intersection failed");
            //----------------
            let x2 = p1.x + dx1;
            let y2 = p1.y - dy1;
            let pz = Vertex::line_to(x2,y2);
            if (cross(&p0, &p1, &pz) < 0.0) ==
               (cross(&p1, &p2, &pz) < 0.0) {
                // This case means that the next segment continues
                // the previous one (straight line)
                //-----------------
                out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                miter_limit_exceeded = false;
            }
        }

        if miter_limit_exceeded {
            //println!("LINEOUT: miter_limit_exceeded");
            // Miter limit exceeded
            //------------------------
            match join {
                LineJoin::MiterRevert => {
                    // For the compatibility with SVG, PDF, etc, 
                    // we use a simple bevel join instead of
                    // "smart" bevel
                    //-------------------
                    //println!("LINEOUT: Doing miter_revert: {:?}", join);
                    out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                },
                LineJoin::Round => out.extend( self.calc_arc(p1.x, p1.y, dx1, -dy1, dx2, -dy2)),
                _ => {
                    //default:
                    //println!("LINEOUT: Doing miter default");
                    // If no miter-revert, calculate new dx1, dy1, dx2, dy2
                    //----------------
                    if intersection_failed {
                        //println!("LINEOUT: intersection failed arm");
                        let mlimit = mlimit * self.width_sign;
                        out.push(Vertex::line_to(p1.x + dx1 + dy1 * mlimit,
                                            p1.y - dy1 + dx1 * mlimit));
                        out.push(Vertex::line_to(p1.x + dx2 - dy2 * mlimit,
                                            p1.y - dy2 - dx2 * mlimit));
                    } else {
                        //println!("LINEOUT: intersection ok arm");
                        let x1 = p1.x + dx1;
                        let y1 = p1.y - dy1;
                        let x2 = p1.x + dx2;
                        let y2 = p1.y - dy2;
                        //println!("LINEOUT: OK V0: {:?}", p0);
                        //println!("LINEOUT: OK V1: {:?}", p1);
                        //println!("LINEOUT: OK V2: {:?}", p2);
                        //println!("LINEOUT: OK di {} lim {} dbevel {}\n", di, lim, dbevel);
                        //println!("LINEOUT: OK {} {} {} {}", x1,y1,x2,y2);
                        //println!("LINEOUT: OK {} {} INTERSECTION", xi,yi);
                        let di = (lim - dbevel) / (di - dbevel);
                        //println!("LINEOUT: OK di {} \n", di);
                        out.push(Vertex::line_to(x1 + (xi - x1) * di,
                                                 y1 + (yi - y1) * di));
                        out.push(Vertex::line_to(x2 + (xi - x2) * di,
                                                 y2 + (yi - y2) * di));
                    }
                }
            }
        }
        //for v in &out {
            //eprintln!("LINEOUT: CALC_MITER: {:?}", v);
        //}
        out
    }
    fn calc_intersection(&self,
                             ax: f64, ay: f64, bx: f64, by: f64,
                             cx: f64, cy: f64, dx: f64, dy: f64)
                             -> Option<(f64, f64)> {
        let intersection_epsilon = 1.0e-30;
        let num = (ay-cy) * (dx-cx) - (ax-cx) * (dy-cy);
        let den = (bx-ax) * (dy-cy) - (by-ay) * (dx-cx);
        if den.abs() < intersection_epsilon {
            return None;
        }
        let r = num / den;
        let x = ax + r * (bx-ax);
        let y = ay + r * (by-ay);
        Some((x,y))
    }
    fn calc_join(&self,
                     p0: &Vertex<f64>,
                     p1: &Vertex<f64>,
                     p2: &Vertex<f64>) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let len1 = len(p1,p0);
        let len2 = len(p2,p1);

        //eprintln!("LINEOUT: V0: {:?} {}", p0, len1);
        //eprintln!("LINEOUT: V1: {:?} {}", p1, len2);
        //eprintln!("LINEOUT: V2: {:?}", p2);
        if len1 == 0.0 {
            panic!("Same point between p0,p1 {:?} {:?}", p0,p1);
        }
        if len2 == 0.0 {
            panic!("Same point between p1,p2 {:?} {:?}", p1,p2);
        }
        let dx1 = self.width * (p1.y-p0.y) / len1;
        let dy1 = self.width * (p1.x-p0.x) / len1;
        let dx2 = self.width * (p2.y-p1.y) / len2;
        let dy2 = self.width * (p2.x-p1.x) / len2;
        //eprintln!("LINEOUT: {} {} {} {}", dx1, dy1, dx2, dy2);
        let cp = cross(p0, p1, p2);
        if cp != 0.0 && cp.is_sign_positive() == self.width.is_sign_positive() {
            println!("LINE: INNER JOIN");
            // Inner Join
            let mut limit = if len1 < len2 {
                len1 / self.width_abs
            } else {
                len2 / self.width_abs
            };
            if limit < self.inner_miter_limit {
                limit = self.inner_miter_limit;
            }
            println!("INNER_JOIN {:?}", self.inner_join);
            match self.inner_join {
                InnerJoin::Bevel => {
                    println!("INNER_JOIN BEVEL {} {} -> {} {}",
                             p1.x + dx1, p1.y - dy1,
                             p1.x + dx2, p1.y - dy2);
                    out.push(Vertex::line_to(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::line_to(p1.x + dx2, p1.y - dy2));
                },
                InnerJoin::Miter => {
                    //eprintln!("LINEOUT: MITER: {} {} {} {} {}", dx1,dy1,dx2,dy2,limit);
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
            eprintln!("LINEOUT: OUTER JOIN");
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
                //eprintln!("LINEOUT: RETURN APPROX");
                return out ;
            }
            //eprintln!("LINEOUT: RETURN NON APPROX {:?} {}", self.line_join, dbevel);
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
    /// Set Line cap style to [LineCap](enum.LineCap.html)
    ///
    pub fn line_cap(&mut self, line_cap: LineCap) {
        self.line_cap = line_cap;
    }
    /// Set Line Join style to [LineJoin](enum.LineJoin.html)
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
    /// Set Inner Join style to [InnerJoin](enum.InnerJoin.html)
    pub fn inner_join(&mut self, inner_join: InnerJoin) {
        self.inner_join = inner_join;
    }
    // miter_limit
    //     miter_limit_theta
    //     inner_miter_limit
    //     approximation_scale
    //     shorten
    fn stroke(&self) -> Vec<Vertex<f64>> {
        //println!("LINEOUT: STROKE PATH");
        let mut all_out = vec![];
        let v0 = &self.source.xconvert();
        let pairs = split(&v0);
        // println!("STROKE PATH: pathlen {} segments {}",
        //          v0.len(), pairs.len());
        for (m1,m2) in pairs {
            //eprintln!("SPLIT {:?}", &v0[m1..=m2]);
            let mut outf = vec![];
            let v = clean_path(&v0[m1..=m2]);
            // Has Closed Path Element
            let closed = is_path_closed(&v);
            // Ignore Closed Tag Element
            let n = if closed { v.len() - 1 } else { v.len() };
            let (n1,n2) = if closed { (0, n) } else { (1,n-1) };
            if ! closed {
                outf.extend( self.calc_cap(&v[0], &v[1]) ); // Begin Cap
            }
            for i in n1 .. n2 { // Forward Path
                outf.extend(
                    self.calc_join(&v[prev!(i,n)], &v[curr!(i,n)], &v[next!(i,n)])
                );
            }
            if closed {
                let n = outf.len();
                let last = outf[n-1];
                outf.push( Vertex::close_polygon(last.x, last.y) );
            }
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
                outb[0].cmd = PathCommand::MoveTo;
                let n = outb.len();
                let last = outb[n-1];
                outb.push( Vertex::close_polygon(last.x, last.y) );
            } else {
                let n = outb.len();
                let last = outb[n-1];
                outb.push( Vertex::close_polygon(last.x, last.y) );
            }
            
            outf[0].cmd = PathCommand::MoveTo;
            outf.extend(outb);

            //println!("COMPLETE: closed? {}", closed);
            //for v in &outf {
            //    println!("COMPLETE: {:?} {:.6} {:.6}", v.cmd, v.x,v.y);
            //}
            all_out.extend(outf);
        }
        all_out
    }
}

fn is_path_closed(verts: &[Vertex<f64>]) -> bool {
    for v in verts {
        if v.cmd == PathCommand::Close {
            return true;
        }
    }
    false
}
/// Remove repeated vertices, defined with a distance <= 1e-6
fn clean_path(v: &[Vertex<f64>]) -> Vec<Vertex<f64>>{
    let mut mark = vec![];
    if ! v.is_empty() {
        mark.push(0);
    }
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
    let mut out : Vec<_> = mark.into_iter().map(|i| v[i]).collect();
    if ! is_path_closed(&out) {
        return out;
    }

    let first = out[0];
    loop {
        let i = match last_line_to(&out) {
            Some(i) => i,
            None => panic!("Missing Last Line To"),
        };
        let last = out[i];
        if len(&first, &last) >= 1e-6 {
            break;
        }
        //eprintln!("REMOVING POINT {:?} {:?}", first, last);
        out.remove(i);
    }
    out
}
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
