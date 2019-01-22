//! Transformations

use crate::paths::Vertex;
use crate::paths::Path;

use crate::VertexSource;

use std::ops::Mul;

/// Transformation
#[derive(Debug,Default,Copy,Clone,PartialEq)]
pub struct Transform {
    pub sx: f64,
    pub sy: f64,
    pub shx: f64,
    pub shy: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Transform {
    /// Creates a new Transform
    pub fn new() -> Self {
        Self { sx: 1.0,  sy: 1.0,
               shx: 0.0, shy: 0.0,
               tx: 0.0,  ty: 0.0,
        }
    }
    /// Add a translation to the transform
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.tx += dx;
        self.ty += dy;
    }
    /// Add a scaling to the transform
    pub fn scale(&mut self, sx: f64, sy: f64) {
        self.sx  *= sx;
        self.shx *= sx;
        self.tx  *= sx;
        self.sy  *= sy;
        self.shy *= sy;
        self.ty  *= sy;
    }
    /// Add a rotation to the transform
    ///
    /// angle is in radians
    pub fn rotate(&mut self, angle: f64) {
        let ca = angle.cos();
        let sa = angle.sin();
        let t0   = self.sx  * ca - self.shy * sa;
        let t2   = self.shx * ca - self.sy  * sa;
        let t4   = self.tx  * ca - self.ty  * sa;
        self.shy = self.sx  * sa + self.shy * ca;
        self.sy  = self.shx * sa + self.sy  * ca;
        self.ty  = self.tx  * sa + self.ty  * ca;
        self.sx  = t0;
        self.shx = t2;
        self.tx  = t4;
    }

    /// Perform the transform
    pub fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.sx  + y * self.shx + self.tx,
         x * self.shy + y * self.sy  + self.ty)
    }
    fn determinant(&self) -> f64 {
        self.sx * self.sy - self.shy * self.shx
    }
    pub fn invert(&mut self) {
        let d = 1.0 / self.determinant();
        let t0 = self.sy * d;
        self.sy = self.sx * d;
        self.shy = -self.shy * d;
        self.shx = -self.shx * d;
        let t4  = -self.tx * t0  - self.ty * self.shx;
        self.ty = -self.tx * self.shy - self.ty * self.sy;

        self.sx = t0;
        self.tx = t4;
    }
    pub fn mul_transform(&self, m: &Transform) -> Self {
        let t0  = self.sx  * m.sx  + self.shy * m.shx;
        let t2  = self.shx * m.sx  + self.sy  * m.shx;
        let t4  = self.tx  * m.sx  + self.ty  * m.shx + m.tx;
        let shy = self.sx  * m.shy + self.shy * m.sy;
        let sy  = self.shx * m.shy + self.sy  * m.sy;
        let ty  = self.tx  * m.shy + self.ty  * m.sy + m.ty;
        let sx  = t0;
        let shx = t2;
        let tx  = t4;
        Transform { sx, sy, tx, ty, shx, shy }
    }
    pub fn new_scale(sx: f64, sy: f64) -> Transform {
        let mut t = Self::new();
        t.scale(sx,sy);
        t
    }
    pub fn new_translate(tx: f64, ty: f64) -> Transform {
        let mut t = Self::new();
        t.translate(tx,ty);
        t
    }
    pub fn new_rotate(ang: f64) -> Transform {
        let mut t = Self::new();
        t.rotate(ang);
        t
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;
    fn mul(self, rhs: Transform) -> Self {
        self.mul_transform(&rhs)
    }
}

/// Path Transform
#[derive(Debug,Default)]
pub struct ConvTransform {
    /// Source Path to Transform
    pub source: Path,
    /// Transform to apply
    pub trans: Transform,
}

impl VertexSource for ConvTransform {
    /// Apply the Transform
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        self.transform()
    }
}


impl ConvTransform {
    /// Create a new Path Transform
    pub fn new(source: Path, trans: Transform) -> Self {
        Self { source, trans }
    }
    /// Transform the Path
    pub fn transform(&self) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        for v in &self.source.xconvert() {
            let (x,y) = self.trans.transform(v.x, v.y);
            out.push(Vertex::new(x,y,v.cmd));
        }
        out
    }
}
