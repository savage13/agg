

use path_storage::PathStorage;
use path_storage::PathCommand;
use path_storage::Vertex;
use path_storage::VertexSource;
use path_storage::len;
use path_storage::cross;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum LineCap {
    Butt, Square, Round
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum LineJoin {
    Miter, MiterRevert, Round, Bevel, MiterRound
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InnerJoin {
    Bevel, Miter, Jag, Round
}

impl Default for LineCap   { fn default() -> LineCap   { LineCap::Butt    } }
impl Default for LineJoin  { fn default() -> LineJoin  { LineJoin::Miter  } }
impl Default for InnerJoin { fn default() -> InnerJoin { InnerJoin::Miter } }

#[derive(Debug,Default)]
pub struct ConvStroke {
    pub source: PathStorage,
    pub width: f64,
    pub width_abs: f64,
    pub width_eps: f64,
    pub width_sign: f64,
    pub miter_limit: f64,
    pub inner_miter_limit: f64,
    pub approx_scale: f64,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub inner_join: InnerJoin,
}

impl VertexSource for ConvStroke {
    fn vertices(&self) -> &[Vertex<f64>] {
        self.source.vertices()
    }
    fn convert(&self) -> Vec<Vertex<f64>> {
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

impl ConvStroke {
    pub fn new(source: PathStorage) -> Self {
        Self {
            source: source,
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
        eprintln!("SET WIDTH");
    }
    pub fn calc_cap(&self, v0: &Vertex<f64>, v1: &Vertex<f64>) -> Vec<Vertex<f64>> {
        eprintln!("JOIN: CAP: v0 {} {}", v0.x, v0.y);
        eprintln!("JOIN: CAP: v1 {} {}", v1.x, v1.y);
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
                out.push(Vertex::xy(v0.x - dx1 - dx2, v0.y + dy1 - dy2));
                out.push(Vertex::xy(v0.x + dx1 - dx2, v0.y - dy1 - dy2));
            },
            LineCap::Butt => {
                out.push(Vertex::xy(v0.x - dx1, v0.y + dy1));
                out.push(Vertex::xy(v0.x + dx1, v0.y - dy1));
            },
            LineCap::Round => {

            }
        }
        out
    }
    pub fn calc_arc(&self, x: f64, y: f64, dx1: f64, dy1: f64, dx2: f64, dy2: f64) -> Vec<Vertex<f64>> {
        unimplemented!("calc_arc");
    }
    pub fn calc_miter(&self,
                      p0: &Vertex<f64>,
                      p1: &Vertex<f64>,
                      p2: &Vertex<f64>,
                      dx1: f64, dy1: f64,dx2: f64, dy2: f64,
                      join: LineJoin, mlimit: f64, dbevel: f64)
                      -> Vec<Vertex<f64>>{
        let mut out = vec![];
        let xi  = p1.x;
        let yi  = p1.y;
        let di  = 1.0;
        let lim = self.width_abs * mlimit;
        let mut miter_limit_exceeded = true; // Assume the worst
        let mut intersection_failed  = true; // Assume the worst

        if let Some((xi,yi)) = self.calc_intersection(p0.x + dx1, p0.y - dy1,
                                                      p1.x + dx1, p1.y - dy1,
                                                      p1.x + dx2, p1.y - dy2,
                                                      p2.x + dx2, p2.y - dy2) {
                                               //&xi, &yi))
            // Calculation of the intersection succeeded
            //---------------------
            let pz = Vertex::xy(xi,yi);
            let di = len(p1,&pz);
            if di <= lim {
                // Inside the miter limit
                //---------------------
                out.push(Vertex::xy(xi, yi));
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
            let pz = Vertex::xy(x2,y2);
            if (cross(&p0, &p1, &pz) < 0.0) ==
               (cross(&p1, &p2, &pz) < 0.0) {
                // This case means that the next segment continues
                // the previous one (straight line)
                //-----------------
                out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
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
                    //-------------------
                    out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::xy(p1.x + dx2, p1.y - dy2));
                },
                LineJoin::Round => out.extend( self.calc_arc(p1.x, p1.y, dx1, -dy1, dx2, -dy2)),
                _ => {
                    //default:
                    // If no miter-revert, calculate new dx1, dy1, dx2, dy2
                    //----------------
                    if intersection_failed {
                        let mlimit = mlimit * self.width_sign;
                        out.push(Vertex::xy(p1.x + dx1 + dy1 * mlimit,
                                            p1.y - dy1 + dx1 * mlimit));
                        out.push(Vertex::xy(p1.x + dx2 - dy2 * mlimit,
                                            p1.y - dy2 - dx2 * mlimit));
                    } else {
                        let x1 = p1.x + dx1;
                        let y1 = p1.y - dy1;
                        let x2 = p1.x + dx2;
                        let y2 = p1.y - dy2;
                        let di = (lim - dbevel) / (di - dbevel);
                        out.push(Vertex::xy(x1 + (xi - x1) * di,
                                            y1 + (yi - y1) * di));
                        out.push(Vertex::xy(x2 + (xi - x2) * di,
                                            y2 + (yi - y2) * di));
                    }
                }
            }
        }
        out
    }
    pub fn calc_intersection(&self,
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
    pub fn calc_join(&self,
                     p0: &Vertex<f64>,
                     p1: &Vertex<f64>,
                     p2: &Vertex<f64>) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let dx01 = p1.x - p0.x;
        let dx12 = p2.x - p1.x;
        let dy01 = p1.y - p0.y;
        let dy12 = p2.y - p1.y;
        let len1 = len(p1,p0);
        let len2 = len(p2,p1);
        //eprintln!("LINE: JOIN: V0: {} {}", p0.x, p0.y);
        //eprintln!("JOIN: V1: {} {}", p1.x, p1.y);
        //eprintln!("JOIN: V2: {} {}", p2.x, p2.y);
        //eprintln!("JOIN: LEN1,LEN2: {} {} {} {}", len1,len2, self.width, self.width_abs);
        let dx1 = self.width * (p1.y-p0.y) / len1;
        let dy1 = self.width * (p1.x-p0.x) / len1;
        let dx2 = self.width * (p2.y-p1.y) / len2;
        let dy2 = self.width * (p2.x-p1.x) / len2;
        //eprintln!("JOIN: {} {} {} {}", dx1, dy1, dx2, dy2);

        let cp = cross(p0, p1, p2);
        if cp != 0.0 && cp.is_sign_positive() == self.width.is_sign_positive() {
            //println!("LINE: INNER JOIN");
            // Inner Join
            let mut limit = if len1 < len2 {
                len1 / self.width_abs
            } else {
                len2 / self.width_abs
            };
            if limit < self.inner_miter_limit {
                limit = self.inner_miter_limit;
            }
            match self.inner_join {
                InnerJoin::Bevel => {
                    out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::xy(p1.x + dx2, p1.y - dy2));
                },
                InnerJoin::Miter => {
                    eprintln!("JOIN: MITER: {} {} {} {} {}", dx1,dy1,dx2,dy2,limit);
                    out.extend(self.calc_miter(p0, p1, p2, dx1, dy1, dx2, dy2, LineJoin::MiterRevert, limit, 0.0));
                }
                InnerJoin::Jag |
                InnerJoin::Round => {
                    let cp = (dx1-dx2).powi(2) + (dy1-dy2).powi(2);
                    if cp < len1.powi(2) && cp < len2.powi(2) {
                        out.extend(self.calc_miter(p0,p1,p2, dx1, dy1, dx2, dy2, LineJoin::MiterRevert, limit, 0.0));
                    } else {
                        if self.inner_join == InnerJoin::Jag {
                            out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
                            out.push(Vertex::xy(p1.x,       p1.y      ));
                            out.push(Vertex::xy(p1.x + dx2, p1.y - dy2));
                        }
                        if self.inner_join == InnerJoin::Round {
                            out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
                            out.push(Vertex::xy(p1.x,       p1.y      ));
                            out.extend(self.calc_arc(p1.x, p1.y, dx2, -dy2, dx1, -dy1));
                            out.push(Vertex::xy(p1.x,       p1.y      ));
                            out.push(Vertex::xy(p1.x + dx2, p1.y - dy2));
                        }
                    }
                }
            }
        } else {
            //println!("LINE: OUTER JOIN");
            // Outer Join
            let dx = (dx1 + dx2) / 2.0;
            let dy = (dy1 + dy2) / 2.0;
            let dbevel = (dx*dx + dy*dy).sqrt();

            if self.line_join == LineJoin::Round || self.line_join == LineJoin::Bevel {
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
                if self.approx_scale * (self.width_abs - dbevel) < self.width_eps {
                    if let Some((dx,dy)) =
                        self.calc_intersection(p0.x + dx1, p0.y - dy1,
                                               p1.x + dx1, p1.y - dy1,
                                               p1.x + dx2, p1.y - dy2,
                                               p2.x + dx2, p2.y - dy2) {
                            out.push(Vertex::xy(dx, dy));
                        } else {
                            out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
                        }
                    //eprintln!("LINE: RETURN APPROX");
                    return out ;
                }
            }
            //eprintln!("LINE: RETURN NON APPROX {:?}", self.line_join);
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
                    out.push(Vertex::xy(p1.x + dx1, p1.y - dy1));
                    out.push(Vertex::xy(p1.x + dx2, p1.y - dy2));
                },
            }
        }
        out
    }
    pub fn stroke(&self) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let v = self.source.vertices();
        let n = v.len();
        //eprintln!("LINE: JOIN: BEGIN CAP {}", n);
        out.extend( self.calc_cap(&v[0], &v[1]) ); // Begin Cap
        //eprintln!("LINE: JOIN: FORWARD PATH");
        for i in 1 .. n-1 { // Forward Path
            out.extend(
                self.calc_join(&v[prev!(i,n)], &v[curr!(i,n)], &v[next!(i,n)])
            );
        }
        //eprintln!("LINE: JOIN: END CAP");
        out.extend( self.calc_cap(&v[n-1], &v[n-2]) ); // End Cap
        //eprintln!("LINE: JOIN: BACKWARD PATH");
        for i in (1 .. n-1).rev() { // Backward Path
            out.extend(
                self.calc_join(&v[next!(i,n)], &v[curr!(i,n)], &v[prev!(i,n)])
            );
            //eprintln!("LINE: BACKWARDS: {}/{}", i,out.len());
        }
        
        out.iter_mut().for_each(|x| x.cmd = PathCommand::LineTo);
        out[0].cmd = PathCommand::MoveTo;
        let v = Vertex::new(out[0].x, out[0].y, PathCommand::Close);
        out.push(v);
        out
    }
}
