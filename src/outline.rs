
use crate::path_storage::PathCommand;
use crate::PixelDraw;
use crate::VertexSource;
use crate::render::RendererPrimatives;

pub struct RasterizerOutline<'a,T> where T: PixelDraw {
    pub ren: &'a mut RendererPrimatives<'a,T>,
    pub start_x: i64,
    pub start_y: i64,
    pub vertices: usize,
}
impl<'a,T> RasterizerOutline<'a,T> where T: PixelDraw {
    pub fn with_primative(ren: &'a mut RendererPrimatives<'a,T>) -> Self {
        Self { start_x: 0, start_y: 0, vertices: 0, ren}
    }
    pub fn add_path<VS: VertexSource>(&mut self, path: &VS) {
        for v in path.xconvert().iter() {
            match v.cmd {
                PathCommand::MoveTo => self.move_to_d(v.x, v.y),
                PathCommand::LineTo => self.line_to_d(v.x, v.y),
                PathCommand::Close => self.close(),
                PathCommand::Stop => unimplemented!("stop encountered"),
            }
        }
    }
    pub fn close(&mut self) {
        if self.vertices > 2 {
            let (x,y) = (self.start_x, self.start_y);
            self.line_to( x, y );
        }
        self.vertices = 0;
    }
    pub fn move_to_d(&mut self, x: f64, y: f64) {
        let x = self.ren.coord(x);
        let y = self.ren.coord(y);
        self.move_to( x, y );
    }
    pub fn line_to_d(&mut self, x: f64, y: f64) {
        let x = self.ren.coord(x);
        let y = self.ren.coord(y);
        self.line_to( x, y );
    }
    pub fn move_to(&mut self, x: i64, y: i64) {
        self.vertices = 1;
        self.start_x = x;
        self.start_y = y;
        self.ren.move_to(x, y);
    }
    pub fn line_to(&mut self, x: i64, y: i64) {
        self.vertices += 1;
        self.ren.line_to(x, y);
    }
}
