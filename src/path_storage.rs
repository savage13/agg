
use clip::Rectangle;
use VertexSource;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PathCommand {
    Stop,
    MoveTo,
    LineTo,
    Close,
    //Curve3,
    //Curve4,
    //CurveN,
    //Catrom,
    //UBSpline,
    //EndPoly,
}
impl Default for PathCommand {
    fn default() -> PathCommand {
        PathCommand::MoveTo
    }
}

#[derive(Debug,Default,Copy,Clone)]
pub struct Vertex<T> {
    pub x: T,
    pub y: T,
    pub cmd: PathCommand
}

impl<T> Vertex<T> {
    pub fn new(x: T, y: T, cmd: PathCommand) -> Self {
        Self { x, y, cmd }
    }
    pub fn xy(x: T, y:T) -> Self {
        Self { x, y, cmd: PathCommand::Stop }
    }
    pub fn move_to(x: T, y:T) -> Self {
        Self { x, y, cmd: PathCommand::MoveTo }
    }
    pub fn line_to(x: T, y:T) -> Self {
        Self { x, y, cmd: PathCommand::LineTo }
    }
    pub fn close_polygon(x: T, y: T) -> Self {
        Self { x, y, cmd: PathCommand::Close }
    }
}

pub fn len(a: &Vertex<f64>, b: &Vertex<f64>) -> f64 {
    ((a.x-b.x).powi(2) + (a.y-b.y).powi(2)).sqrt()
}
pub fn cross(p1: &Vertex<f64>, p2: &Vertex<f64>, p: &Vertex<f64>) -> f64 {
    (p.x - p2.x) * (p2.y - p1.y) - (p.y - p2.y) * (p2.x - p1.x)
}

//  typedef path_base<vertex_block_storage<double> > path_storage;
#[derive(Debug,Default)]
pub struct PathStorage {
    pub vertices: Vec<Vertex<f64>>,
}

impl VertexSource for PathStorage {
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.vertices.clone()
    }
}

impl PathStorage {
    pub fn new() -> Self {
        Self { vertices: vec![] }
    }
    pub fn remove_all(&mut self) {
        self.vertices.clear();
    }
    pub fn move_to(&mut self, x: f64, y: f64) {
        //self.vertices.push( Vertex::new(x,y, PathCommand::MoveTo) );
        self.vertices.push( Vertex::move_to(x,y) );
    }
    pub fn line_to(&mut self, x: f64, y: f64) {
        //self.vertices.push( Vertex::new(x,y, PathCommand::LineTo) );
        self.vertices.push( Vertex::line_to(x,y) );
    }
    pub fn close_polygon(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let n = self.vertices.len();
        let last = self.vertices[n-1];
        if last.cmd == PathCommand::LineTo {
            self.vertices.push( Vertex::close_polygon(last.x, last.y) );
        }
    }
    pub fn arrange_orientations(&mut self, dir: PathOrientation) {
        arrange_orientations(self, dir);
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PathOrientation {
    Clockwise,
    CounterClockwise
}

pub fn split(path: &[Vertex<f64>]) -> Vec<(usize, usize)> {
    let (mut start, mut end) = (None, None);
    let mut pairs = vec![];
    println!("SPLIT PATH");
    for (i,v) in path.iter().enumerate() {
        println!("SPLIT: item[{}] {:?}  {:?} {:?}", i, v, start, end);
        match (start, end) {
            (None, None) => {
                match v.cmd {
                    PathCommand::MoveTo => {
                        start = Some(i);
                    },
                    PathCommand::LineTo |
                    PathCommand::Close  |
                    PathCommand::Stop => { },
                }
            },
            (Some(_),None) => {
                match v.cmd {
                    PathCommand::MoveTo => { start = Some(i); },
                    PathCommand::LineTo => { end = Some(i); },
                    PathCommand::Close |
                    PathCommand::Stop => { end = Some(i) },
                }
            }
            (Some(s),Some(e)) => {
                match v.cmd {
                    PathCommand::MoveTo => {
                        pairs.push((s,e));
                        start = Some(i);
                        end = None;
                    },
                    PathCommand::LineTo  |
                    PathCommand::Close   |
                    PathCommand::Stop => { end = Some(i) },
                }
            }
            (None, Some(_)) => unreachable!("oh on bad state!"),
        }
    }
    if let (Some(s), Some(e)) = (start, end) {
        pairs.push((s,e));
    }
    println!("SPLIT PATH segments: {} {:?} {:?}", pairs.len(), start, end);
    pairs
}

fn arrange_orientations(path: &mut PathStorage, dir: PathOrientation) {
    let pairs = split(&path.vertices);
    for (s,e) in pairs {
        let pdir = preceive_polygon_orientation(&path.vertices[s..=e]);
        if pdir != dir {
            invert_polygon(&mut path.vertices[s..=e]);
        }
    }
}
pub fn invert_polygon(v: &mut [Vertex<f64>]) {
    let n = v.len();
    v.reverse();
    let tmp  = v[0].cmd;
    v[0].cmd = v[n-1].cmd;
    v[n-1].cmd = tmp;
}

pub fn preceive_polygon_orientation(vertices: &[Vertex<f64>]) -> PathOrientation {

    let n = vertices.len();
    let p0 = vertices[0];
    let mut area = 0.0;
    for (i,p1) in vertices.iter().enumerate() {
        let p2 = vertices[(i+1) % n];
        let (x1,y1) = if p1.cmd == PathCommand::Close {
            (p0.x, p0.y)
        } else {
            (p1.x, p1.y)
        };
        let (x2,y2) = if p2.cmd == PathCommand::Close {
            (p0.x, p0.y)
        } else {
            (p2.x, p2.y)
        };
        area += x1 * y2 - y1 * x2;
        //eprintln!("FIND AREA: {} <- {} {} {} {} ", area, x1,y1,x2,y2);
    }
    //eprintln!("FIND AREA: {}", area);
    if area < 0.0 {
        PathOrientation::Clockwise
    } else {
        PathOrientation::CounterClockwise
    }
}

pub fn bounding_rect<VS: VertexSource>(path: &VS) -> Option<Rectangle<f64>> {
    let pts = path.xconvert();
    if pts.is_empty() {
        None
    } else {
        let mut r = Rectangle::new(pts[0].x, pts[0].y, pts[0].x, pts[0].y);
        for p in pts {
            r.expand(p.x, p.y);
        }
        Some(r)
    }
}
#[derive(Debug,Default)]
pub struct Ellipse {
    x: f64,
    y: f64,
    rx: f64,
    ry: f64,
    scale: f64,
    num: usize,
    //step: usize,
    cw: bool,
    vertices: Vec<Vertex<f64>>,
}

impl VertexSource for Ellipse {
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.vertices.clone()
    }
}

use std::f64::consts::PI;
impl Ellipse {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0, rx: 1.0, ry: 1.0, scale: 1.0,
               num: 4, cw: false, vertices: vec![] }
    }
    pub fn init(&mut self, x: f64, y: f64, rx: f64, ry: f64, num: usize) {
        self.x   = x;
        self.y   = y;
        self.rx  = rx;
        self.ry  = ry;
        self.num = num;
        self.cw  = false;
        if num == 0 {
            self.calc_num_steps();
        }
        self.calc();
    }
    pub fn calc_num_steps(&mut self) {
        let ra = (self.rx.abs() + self.ry.abs()) / 2.0;
        let da = (ra / (ra + 0.125 / self.scale)).acos() * 2.0;
        self.num = (2.0 * PI / da).round() as usize;
    }
    pub fn calc(&mut self) {
        self.vertices = vec![];
        for i in 0 .. self.num {
            let angle = i as f64 / self.num as f64 * 2.0 * PI;
            let angle = if self.cw {
                2.0 * PI - angle
            } else {
                angle
            };
            let x = self.x + angle.cos() * self.rx;
            let y = self.y + angle.sin() * self.ry;
            let v = if i == 0 {
                Vertex::move_to(x, y)
            } else {
                Vertex::line_to(x, y)
            };
            self.vertices.push( v );
        }
        let v = self.vertices[0];
        self.vertices.push( Vertex::close_polygon(v.x, v.y) );
    }
}
#[derive(Debug,Default)]
pub struct RoundedRect {
    x: [f64;2],
    y: [f64;2],
    rx: [f64; 4],
    ry: [f64; 4],
    vertices: Vec<Vertex<f64>>,
}

impl VertexSource for RoundedRect {
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.vertices.clone()
    }
}

impl RoundedRect {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> Self {
        let (x1,x2) = if x1 > x2 { (x2,x1) } else { (x1,x2) };
        let (y1,y2) = if y1 > y2 { (y2,y1) } else { (y1,y2) };
        Self { x:  [x1,x2], y:  [y1,y2],
               rx: [r; 4],  ry: [r; 4],
               vertices: vec![]
        }
    }
    pub fn calc(&mut self) {
        let vx = [1.0, -1.0, -1.0,  1.0];
        let vy = [1.0,  1.0, -1.0, -1.0];
        let x  = [self.x[0], self.x[1], self.x[1], self.x[0]];
        let y  = [self.y[0], self.y[0], self.y[1], self.y[1]];
        let a = [PI,        PI+PI*0.5, 0.0,    0.5*PI];
        let b = [PI+PI*0.5, 0.0,       PI*0.5, PI];
        for i in 0 .. 4 {
            let arc = Arc::init(x[i] + self.rx[i] * vx[i],
                                y[i] + self.ry[i] * vy[i],
                                self.rx[i], self.ry[i],
                                a[i], b[i]);
            let mut verts = arc.xconvert();
            for vi in verts.iter_mut() {
                vi.cmd = PathCommand::LineTo;
            }
            self.vertices.extend( verts );
        }
        if let Some(first) = self.vertices.first_mut() {
            first.cmd = PathCommand::MoveTo;
        }
        let first = self.vertices[0];
        self.vertices.push(Vertex::close_polygon(first.x, first.y));

        for v in &self.vertices {
            println!("RECT: {:?}", v);
        }
    }
    pub fn normalize_radius(&mut self) {
        let dx = (self.y[1] - self.y[0]).abs();
        let dy = (self.x[1] - self.x[0]).abs();

        let mut k = 1.0f64;
        let ts = [dx / (self.rx[0] + self.rx[1]),
                  dx / (self.rx[2] + self.rx[3]),
                  dy / (self.rx[0] + self.rx[1]),
                  dy / (self.rx[2] + self.rx[3])];
        for &t in ts.iter() {
            if t < k {
                k = t;
            }
        }
        if k < 1.0 {
            for v in &mut self.rx {
                *v *= k;
            }
            for v in &mut self.ry {
                *v *= k;
            }
        }
    }
}

pub struct Arc {
    x: f64,
    y: f64,
    rx: f64,
    ry: f64,
    start: f64,
    end: f64,
    scale: f64,
    ccw: bool,
    da: f64,
    vertices: Vec<Vertex<f64>>,
}
impl VertexSource for Arc {
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.vertices.clone()
    }
}

impl Arc {
    pub fn init(x: f64, y: f64, rx: f64, ry: f64, a1: f64, a2: f64) -> Self {
        let mut a = Self { x, y, rx, ry, scale: 1.0,
                           ccw: true,
                           start: 0.0, end: 0.0, da: 0.0,
                           vertices: vec![]
        };
        a.normalize(a1, a2, true);
        a.calc();
        a
    }
    pub fn calc(&mut self) {
        let mut angle : Vec<_> = (0..)
            .map(|i| self.start + self.da * f64::from(i))
            .take_while(|x|
                        if self.da > 0.0 {
                            x < &self.end
                        } else {
                            x > &self.end
                        })
            .collect();
        angle.push(self.end);
        for a in &angle {
            eprintln!("ARC: {}", a);
            let x = self.x + a.cos() * self.rx;
            let y = self.y + a.sin() * self.ry;
            self.vertices.push( Vertex::line_to(x,y) );
        }
        if let Some(first) = self.vertices.first_mut() {
            first.cmd = PathCommand::MoveTo;
        }
        if let Some(last) = self.vertices.last_mut() {
            last.cmd = PathCommand::Close;
        }
        //repeat_last_point(&mut self.vertices);
    }
    pub fn normalize(&mut self, a1: f64, a2: f64, ccw: bool) {
        let ra = (self.rx.abs() + self.ry.abs()) / 2.0;
        self.da = (ra / (ra + 0.125 / self.scale)).acos() * 2.0;
        let mut a1 = a1;
        let mut a2 = a2;
        if ccw {
            while a2 < a1 {
                a2 += 2.0 * PI;
            }
        } else {
            while a1 < a2 {
                a1 += 2.0 * PI;
            }
            self.da = -self.da;
        }
        self.ccw   = ccw;
        self.start = a1;
        self.end   = a2;
    }
}
