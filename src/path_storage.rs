pub trait VertexSource {
    fn vertices(&self) -> &[Vertex<f64>];
    fn rewind(&self) { }
    fn convert(&self) -> Vec<Vertex<f64>>;
}

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

pub trait Zero<T> {
    fn zero() -> T;
}
impl Zero<f64> for f64 {
    fn zero() -> f64 {
        0.0
    }
}

impl<T> Vertex<T> where T: Zero<T>  {
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
    fn vertices(&self) -> &[Vertex<f64>] {
        &self.vertices
    }
    fn convert(&self) -> Vec<Vertex<f64>> {
        self.vertices().to_vec()
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
        if self.vertices.len() == 0 {
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

fn arrange_orientations(path: &mut PathStorage, dir: PathOrientation) {
    let (mut start, mut end) = (None, None);
    let mut pairs = vec![];
    for (i,v) in path.vertices.iter().enumerate() {
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
    for (s,e) in pairs {
        let pdir = preceive_polygon_orientation(&path.vertices[s..e+1]);
        //println!("FIND: {:?} {:?} {} {} LAST {} DIR {:?}", path.vertices[s], path.vertices[e], s, e, path.vertices.len(), pdir);
        if pdir != dir {
            invert_polygon(&mut path.vertices[s..e+1]);
            //let pdir = preceive_polygon_orientation(&path.vertices[s..e+1]);
            //println!("FIND: {:?} {:?} {} {} LAST {} DIR {:?}", path.vertices[s], path.vertices[e], s, e, path.vertices.len(), pdir);
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

use Rectangle;
pub fn bounding_rect<VS: VertexSource>(path: &VS) -> Option<Rectangle<f64>> {
    let pts = path.vertices();
    if pts.len() == 0 {
        None
    } else {
        eprintln!("dx,dy: {:?}", pts[0]);
        let mut r = Rectangle::new(pts[0].x, pts[0].y,
                                   pts[0].x, pts[0].y);
        for p in pts {
            r.expand(p.x, p.y);
        }
        Some(r)
    }
}
