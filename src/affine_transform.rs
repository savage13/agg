

use path_storage::Vertex;
use path_storage::PathStorage;

use VertexSource;

#[derive(Debug,Default,Copy,Clone)]
pub struct AffineTransform {
    pub sx: f64,
    pub sy: f64,
    pub shx: f64,
    pub shy: f64,
    pub tx: f64,
    pub ty: f64,
}

impl AffineTransform {
    pub fn new() -> Self {
        Self { sx: 1.0,  sy: 1.0,
               shx: 0.0, shy: 0.0,
               tx: 0.0,  ty: 0.0,
        }
    }
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.tx += dx;
        self.ty += dy;
    }
    pub fn scale(&mut self, sx: f64, sy: f64) {
        self.sx  *= sx;
        self.shx *= sx;
        self.tx  *= sx;
        self.sy  *= sy;
        self.shy *= sy;
        self.ty  *= sy;
    }
    pub fn rotate(&mut self, angle: f64) {
        let ca = angle.cos();
        let sa = angle.sin();
        let t0 = self.sx  * ca - self.shy * sa;
        let t2 = self.shx * ca - self.sy * sa;
        let t4 = self.tx  * ca - self.ty * sa;
        self.shy = self.sx  * sa + self.shy * ca;
        self.sy  = self.shx * sa + self.sy * ca;
        self.ty  = self.tx  * sa + self.ty * ca;
        self.sx  = t0;
        self.shx = t2;
        self.tx  = t4;
    }
    pub fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.sx  + y * self.shy + self.tx,
         x * self.shy + y * self.sy  + self.ty)
    }
}

#[derive(Debug,Default)]
pub struct ConvTransform {
    pub source: PathStorage,
    pub trans: AffineTransform,
}

impl VertexSource for ConvTransform {
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.transform()
    }
}

impl ConvTransform {
    pub fn new(source: PathStorage, trans: AffineTransform) -> Self {
        Self { source, trans }
    }
    pub fn transform(&self) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        for v in &self.source.xconvert() {
            let (x,y) = self.trans.transform(v.x, v.y);
            out.push(Vertex::new(x,y,v.cmd));
        }
        out
    }
}
