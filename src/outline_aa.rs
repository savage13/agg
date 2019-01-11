
use crate::stroke::LineJoin;
use crate::paths::PathCommand;
use crate::paths::Vertex;
use crate::line_interp::LineParameters;
use crate::line_interp::DrawVars;

use crate::DrawOutline;
use crate::VertexSource;
use crate::raster::len_i64;
use crate::POLY_SUBPIXEL_SCALE;

pub struct RasterizerOutlineAA<'a,T> where T: DrawOutline {
    ren: &'a mut T,
    start_x: i64,
    start_y: i64,
    vertices: Vec<Vertex<i64>>,
    round_cap: bool,
    line_join: LineJoin,
}

impl<'a,T> RasterizerOutlineAA<'a, T> where T: DrawOutline {
    pub fn with_renderer(ren: &'a mut T) -> Self {
        let line_join = if ren.accurate_join_only() {
            LineJoin::MiterAccurate
        } else {
            LineJoin::Round
        };
        Self { ren, start_x: 0, start_y: 0, vertices: vec![],
               round_cap: false, line_join }
    }
    pub fn round_cap(&mut self, on: bool) {
        self.round_cap = on;
    }
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
    pub fn move_to_d(&mut self, x: f64, y: f64) {
        let x = self.conv(x);
        let y = self.conv(y);
        self.move_to( x, y );
    }
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
    pub fn close_path(&mut self) {
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
