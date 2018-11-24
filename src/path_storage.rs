pub trait VertexSource {
    fn vertices(&self) -> &[Vertex<f64>];
    fn rewind(&mut self) {
    }
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

impl<T> Vertex<T>   {
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
        self.vertices.push( Vertex::new(x,y, PathCommand::MoveTo) );
    }
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.vertices.push( Vertex::new(x,y, PathCommand::LineTo) );
    }
}

