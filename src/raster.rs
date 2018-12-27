//! Rasterizer

use crate::POLY_SUBPIXEL_SHIFT;
use crate::POLY_SUBPIXEL_SCALE;
use crate::POLY_SUBPIXEL_MASK;

use crate::clip::Clip;
use crate::scan::ScanlineU8;
use crate::cell::RasterizerCell;
use crate::path_storage::PathCommand;
use crate::render::RendererPrimatives;
use crate::path_storage::Vertex;
use crate::render::LineInterpolator;
use crate::render::LineInterpolatorImage;

use crate::Rasterize;
use crate::VertexSource;
use crate::PixfmtFunc;
use crate::Pixel;
use crate::SetColor;
use crate::AccurateJoins;
use crate::Lines;

use std::cmp::min;
use std::cmp::max;

struct RasConvInt {
}
impl RasConvInt {
    pub fn upscale(v: f64) -> i64 {
        (v * POLY_SUBPIXEL_SCALE as f64).round() as i64
    }
    //pub fn downscale(v: i64) -> i64 {
    //    v
    //}
}

/// Winding / Filling Rule
///
/// See (Non-Zero Filling Rule)[https://en.wikipedia.org/wiki/Nonzero-rule] and
/// (Even-Odd Filling)[https://en.wikipedia.org/wiki/Even%E2%80%93odd_rule]
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum FillingRule {
    NonZero,
    EvenOdd,
}
impl Default for FillingRule {
    fn default() -> FillingRule {
        FillingRule::NonZero
    }
}

/// Path Status
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum PathStatus {
    Initial,
    Closed,
    MoveTo,
    LineTo
}
impl Default for PathStatus {
    fn default() -> PathStatus {
        PathStatus::Initial
    }
}

/// Rasterizer Anti-Alias using Scanline
#[derive(Debug, Default)]
pub struct RasterizerScanlineAA {
    /// Clipping Region
    pub clipper: Clip,
    /// Collection of Rasterizing Cells
    pub outline: RasterizerCell,
    /// Status of Path
    pub status: PathStatus,
    /// Current x position
    pub x0: i64,
    /// Current y position
    pub y0: i64,
    /// Current y row being worked on, for output
    scan_y: i64,
    /// Filling Rule for Polygons
    filling_rule: FillingRule,
    /// Gamma Corection Values
    gamma: Vec<u64>,
}

impl Rasterize for RasterizerScanlineAA {
    /// Reset Rasterizer
    ///
    /// Reset the RasterizerCell and set PathStatus to Initial
    fn reset(&mut self) {
        self.outline.reset();
        self.status = PathStatus::Initial;
    }
    /// Add a Path
    ///
    /// Walks the path from the VertexSource and rasterizes it
    fn add_path<VS: VertexSource>(&mut self, path: &VS) {
        //path.rewind();
        if ! self.outline.sorted_y.is_empty() {
            self.reset();
        }
        for seg in path.xconvert() {
            println!("ADD_PATH: {:?}", seg);
            match seg.cmd {
                PathCommand::LineTo => self.line_to_d(seg.x, seg.y),
                PathCommand::MoveTo => self.move_to_d(seg.x, seg.y),
                PathCommand::Close  => self.close_polygon(),
                PathCommand::Stop => unimplemented!("stop encountered"),
            }
        }
    }

    /// Rewind the Scanline
    ///
    /// Close active polygon, sort the Rasterizer Cells, set the
    /// scan_y value to the minimum y value and return if any cells
    /// are present
    fn rewind_scanlines(&mut self) -> bool {
        self.close_polygon();
        self.outline.sort_cells();
        if self.outline.total_cells() == 0 {
            false
        } else {
            self.scan_y = self.outline.min_y;
            true
        }
    }

    /// Sweep the Scanline
    ///
    /// For individual y rows adding any to the input Scanline
    ///
    /// Returns true if data exists in the input Scanline
    fn sweep_scanline(&mut self, sl: &mut ScanlineU8) -> bool {
        println!("ADD_PATH: SWEEP SCANLINE: Y: {}", self.scan_y);
        loop {
            if self.scan_y < 0 {
                self.scan_y += 1;
                continue;
            }
            if self.scan_y > self.outline.max_y {
                return false;
            }
            sl.reset_spans();
            let mut num_cells = self.outline.scanline_num_cells( self.scan_y );
            let cells = self.outline.scanline_cells( self.scan_y );

            let mut cover = 0;

            let mut iter = cells.iter();

            if let Some(mut cur_cell) = iter.next() {
                while num_cells > 0 {
                    let mut x = cur_cell.x;
                    let mut area = cur_cell.area;

                    cover  += cur_cell.cover;
                    println!("ADD_PATH: SWEEP SCANLINES: x,y {} {} {} {} :: {} {} n: {}", cur_cell.x, self.scan_y, cur_cell.area, cur_cell.cover, area, cover, num_cells);
                    num_cells -= 1;
                    //accumulate all cells with the same X
                    while num_cells > 0 {
                        cur_cell = iter.next().unwrap();
                        if cur_cell.x != x {
                            break;
                        }
                        area += cur_cell.area;
                        cover += cur_cell.cover;
                        num_cells -= 1;
                        println!("ADD_PATH: SWEEP SCANLINES: x,y {} {} {} {} :: {} {}", cur_cell.x, self.scan_y, cur_cell.area, cur_cell.cover, area, cover);
                    }
                    println!("ADD_PATH: SWEEP SCANLINES: x,y {} {} {} {} :: {} {}", cur_cell.x, self.scan_y, cur_cell.area, cur_cell.cover, area, cover);
                    if area != 0 {
                        println!("ADD_PATH: SWEEP SCANLINES: ADDING CELL: x {} y {} area {} cover {}", x, self.scan_y, area, cover);
                        let alpha = self.calculate_alpha((cover << (POLY_SUBPIXEL_SHIFT + 1)) - area);
                        if alpha > 0 {
                            sl.add_cell(x, alpha);
                        }
                        x += 1;
                    }
                    if num_cells > 0 && cur_cell.x > x {
                        let alpha = self.calculate_alpha(cover << (POLY_SUBPIXEL_SHIFT + 1));
                        println!("ADD_PATH: SWEEP SCANLINES: ADDING SPAN: {} -> {} Y: {} area {} cover {}", x, cur_cell.x, self.scan_y, area, cover);
                        if alpha > 0 {
                            sl.add_span(x, cur_cell.x - x, alpha);
                        }
                    }
                }
            }
            if sl.num_spans() != 0 {
                break;
            }
            self.scan_y += 1;
        }
        sl.finalize(self.scan_y);
        self.scan_y += 1;
        true
    }
    /// Return minimum x value from the RasterizerCell
    fn min_x(&self) -> i64 {
        self.outline.min_x
    }
    /// Return maximum x value from the RasterizerCell
    fn max_x(&self) -> i64 {
        self.outline.max_x
    }
}

impl RasterizerScanlineAA {
    /// Create a new RasterizerScanlineAA
    pub fn new() -> Self {
        Self { clipper: Clip::new(), status: PathStatus::Initial,
               outline: RasterizerCell::new(),
               x0: 0, y0: 0, scan_y: 0,
               filling_rule: FillingRule::NonZero,
               gamma: (0..256).collect(),
        }
    }
    /// Set the gamma function
    ///
    /// Values are set as:
    ///```ignore
    ///      gamma = gfunc( v / mask ) * mask
    ///```
    /// where v = 0 to 255
    pub fn gamma<F>(&mut self, gfunc: F)
        where F: Fn(f64) -> f64
    {
        let aa_shift  = 8;
        let aa_scale  = 1 << aa_shift;
        let aa_mask   = f64::from(aa_scale - 1);

        self.gamma = (0..256)
            .map(|i| gfunc(f64::from(i) / aa_mask ))
            .map(|v| (v * aa_mask).round() as u64)
            .collect();
    }
    /// Create a new RasterizerScanlineAA with a gamma function
    ///
    /// See gamma() function for description
    ///
    pub fn new_with_gamma<F>(gfunc: F) -> Self
        where F: Fn(f64) -> f64
    {
        let mut new = Self::new();
        new.gamma( gfunc );
        new
    }
    /// Set Clip Box
    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.clipper.clip_box(RasConvInt::upscale(x1),
                              RasConvInt::upscale(y1),
                              RasConvInt::upscale(x2),
                              RasConvInt::upscale(y2));
    }
    /// Move to point (x,y)
    ///
    /// Sets point as the initial point
    pub fn move_to_d(&mut self, x: f64, y: f64) {
        self.x0 = RasConvInt::upscale( x );
        self.y0 = RasConvInt::upscale( y );
        self.clipper.move_to(self.x0,self.y0);
        self.status = PathStatus::MoveTo;
    }
    /// Draw line from previous point to point (x,y)
    pub fn line_to_d(&mut self, x: f64, y: f64) {
        let x = RasConvInt::upscale( x );
        let y = RasConvInt::upscale( y );
        self.clipper.line_to(&mut self.outline, x,y);
        self.status = PathStatus::LineTo;
    }
    /// Close the current polygon
    ///
    /// Draw a line from current point to initial "move to" point
    pub fn close_polygon(&mut self) {
        if self.status == PathStatus::LineTo {
            self.clipper.line_to(&mut self.outline, self.x0, self.y0);
            self.status = PathStatus::Closed;
        }
    }
    /// Calculate alpha term based on area
    ///
    ///
    pub fn calculate_alpha(&self, area: i64) -> u64 {
        let aa_shift  = 8;
        let aa_scale  = 1 << aa_shift;
        let aa_scale2 = aa_scale * 2;
        let aa_mask   = aa_scale  - 1;
        let aa_mask2  = aa_scale2 - 1;

        let mut cover = area >> (POLY_SUBPIXEL_SHIFT*2 + 1 - aa_shift);
        cover = cover.abs();
        if self.filling_rule == FillingRule::EvenOdd {
            cover *= aa_mask2;
            if cover > aa_scale {
                cover = aa_scale2 - cover;
            }
        }
        cover = max(0, min(cover, aa_mask));
        self.gamma[cover as usize]
    }
}

#[derive(Debug,Default)]
pub struct DrawVars {
    pub idx: usize,
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
    pub curr: LineParameters,
    pub next: LineParameters,
    pub lcurr: i64,
    pub lnext: i64,
    pub xb1: i64,
    pub yb1: i64,
    pub xb2: i64,
    pub yb2: i64,
    pub flags: u8,
}

impl DrawVars {
    pub fn new() -> Self {
        Self { .. Default::default() }
    }
}
#[derive(Debug,Default,Copy,Clone)]
pub struct LineParameters {
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
    pub dx: i64,
    pub dy: i64,
    pub sx: i64,
    pub sy: i64,
    pub vertical: bool,
    pub inc: i64,
    pub len: i64,
    pub octant: usize,
}

impl LineParameters {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64, len: i64) -> Self {
        let dx = (x2-x1).abs();
        let dy = (y2-y1).abs();
        let vertical = dy >= dx;
        let sx = if x2 > x1 { 1 } else { -1 };
        let sy = if y2 > y1 { 1 } else { -1 };
        let inc = if vertical { sy } else { sx };
        let octant = (sy & 4) as usize | (sx & 2) as usize | vertical as usize;
        Self {x1,y1,x2,y2,len,dx,dy,vertical,sx,sy,inc,octant}
    }
    pub fn diagonal_quadrant(&self) -> u8 {
        let quads = [0,1,2,1,0,3,2,3];
        quads[ self.octant ]
    }
    pub fn divide(&self) -> (LineParameters, LineParameters) {
        let xmid = (self.x1+self.x2) / 2;
        let ymid = (self.y1+self.y2) / 2;
        let len2  = self.len / 2;

        let lp1 = LineParameters::new(self.x1, self.y1, xmid, ymid, len2);
        let lp2 = LineParameters::new(xmid, ymid, self.x2, self.y2, len2);

        (lp1, lp2)
    }
    fn fix_degenerate_bisectrix_setup(&self, x: i64, y: i64) -> i64 {
        let dx = (self.x2 - self.x1) as f64;
        let dy = (self.y2 - self.y1) as f64;
        let dx0 = (x - self.x2) as f64;
        let dy0 = (y - self.y2) as f64;
        let len = self.len as f64;
        let d = ((dx0 * dy - dy0 * dx) / len).round() as i64;
        d
    }
    pub fn fix_degenerate_bisectrix_end(&self, x: i64, y: i64) -> (i64, i64) {
        let d = self.fix_degenerate_bisectrix_setup(x,y);
        if d < POLY_SUBPIXEL_SCALE / 2 {
            (self.x2 + (self.y2 - self.y1), self.y2 - (self.x2 - self.x1))
        } else {
            (x,y)
        }
    }
    pub fn fix_degenerate_bisectrix_start(&self, x: i64, y: i64) -> (i64, i64) {
        let d = self.fix_degenerate_bisectrix_setup(x,y);
        if d < POLY_SUBPIXEL_SCALE / 2 {
            (self.x1 + (self.y2 - self.y1), self.y1 - (self.x2 - self.x1))
        } else {
            (x,y)
        }
    }
    pub fn interp0(&self, subpixel_width: i64) -> AA0 {
        AA0::new(*self, subpixel_width)
    }
    pub fn interp1(&self, sx: i64, sy: i64, subpixel_width: i64) -> AA1 {
        AA1::new(*self, sx, sy, subpixel_width)
    }
    pub fn interp2(&self, ex: i64, ey: i64, subpixel_width: i64) -> AA2 {
        AA2::new(*self, ex, ey, subpixel_width)
    }
    pub fn interp3(&self, sx: i64, sy: i64, ex: i64, ey: i64, subpixel_width: i64) -> AA3 {
        AA3::new(*self, sx, sy, ex, ey, subpixel_width)
    }
    pub fn interp_image(&self, sx: i64, sy: i64, ex: i64, ey: i64, subpixel_width: i64, pattern_start: i64, pattern_width: i64, scale_x: f64) -> LineInterpolatorImage {
        LineInterpolatorImage::new(*self, sx, sy, ex, ey,
                                   subpixel_width, pattern_start,
                                   pattern_width, scale_x)
    }
}

pub trait DistanceInterpolator {
    fn dist(&self) -> i64;
    fn inc_x(&mut self, dy: i64);
    fn inc_y(&mut self, dx: i64);
    fn dec_x(&mut self, dy: i64);
    fn dec_y(&mut self, dx: i64);
}

pub struct DistanceInterpolator00 {
    pub dx1: i64,
    pub dy1: i64,
    pub dx2: i64,
    pub dy2: i64,
    pub dist1: i64,
    pub dist2: i64,
}

impl DistanceInterpolator00 {
    pub fn new(xc: i64, yc: i64, x1: i64, y1: i64, x2: i64, y2: i64, x: i64, y: i64) -> Self {
        let dx1 = line_mr(x1) - line_mr(xc);
        let dy1 = line_mr(y1) - line_mr(yc);
        let dx2 = line_mr(x2) - line_mr(xc);
        let dy2 = line_mr(y2) - line_mr(yc);
        let dist1 = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(x1)) * dy1 -
                    (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(y1)) * dx1;
        let dist2 = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(x2)) * dy2 -
                    (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(y2)) * dx2;
        let dx1 = dx1 << POLY_MR_SUBPIXEL_SHIFT;
        let dy1 = dy1 << POLY_MR_SUBPIXEL_SHIFT;
        let dx2 = dx2 << POLY_MR_SUBPIXEL_SHIFT;
        let dy2 = dy2 << POLY_MR_SUBPIXEL_SHIFT;

        Self { dx1, dy1, dx2, dy2, dist1, dist2 }
    }
    pub fn inc_x(&mut self) {
        self.dist1 += self.dy1;
        self.dist2 += self.dy2;
    }
}

pub struct DistanceInterpolator0 {
    pub dx: i64,
    pub dy: i64,
    pub dist: i64,
}

impl DistanceInterpolator0 {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64, x: i64, y: i64) -> Self {
        let dx = line_mr(x2) - line_mr(x1);
        let dy = line_mr(y2) - line_mr(y1);
        let dist = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(x2)) * dy -
                   (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(y2)) * dx;
        let dx = dx << POLY_MR_SUBPIXEL_SHIFT;
        let dy = dy << POLY_MR_SUBPIXEL_SHIFT;
        Self { dx, dy, dist }
    }
    pub fn inc_x(&mut self) {
        self.dist += self.dy;
    }
}

pub struct DistanceInterpolator1 {
    pub dx: i64,
    pub dy: i64,
    pub dist: i64
}
pub struct DistanceInterpolator2 {
    pub dx: i64,
    pub dy: i64,
    pub dx_start: i64,
    pub dy_start: i64,
    pub dist: i64,
    pub dist_start: i64,
}
pub struct DistanceInterpolator3 {
    pub dx: i64,
    pub dy: i64,
    pub dx_start: i64,
    pub dy_start: i64,
    pub dx_end: i64,
    pub dy_end: i64,
    pub dist: i64,
    pub dist_start: i64,
    pub dist_end: i64,
}
impl DistanceInterpolator1 {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64, x: i64, y: i64) -> Self {
        let dx = x2-x1;
        let dy = y2-y1;
        let dist_fp = (x + POLY_SUBPIXEL_SCALE/2 - x2) as f64 * dy as f64 -
            (y + POLY_SUBPIXEL_SCALE/2 - y2) as f64 * dx as f64;
        let dist = dist_fp.round() as i64;
        let dx = dx << POLY_SUBPIXEL_SHIFT;
        let dy = dy << POLY_SUBPIXEL_SHIFT;
        Self { dist, dx, dy }
    }
}
impl DistanceInterpolator for DistanceInterpolator1 {
    fn dist(&self) -> i64 {
        self.dist
    }
    fn inc_x(&mut self, dy: i64) {
        self.dist += self.dy;
        if dy > 0 {
            self.dist -= self.dx;
        }
        if dy < 0 {
            self.dist += self.dx;
        }
    }
    fn dec_x(&mut self, dy: i64) {
        self.dist -= self.dy;
        if dy > 0 {
            self.dist -= self.dx;
        }
        if dy < 0 {
            self.dist += self.dx;
        }
    }
    fn inc_y(&mut self, dx: i64) {
        self.dist -= self.dx;
        if dx > 0 {
            self.dist += self.dy;
        }
        if dx < 0 {
            self.dist -= self.dy;
        }
    }
    fn dec_y(&mut self, dx: i64) {
        self.dist += self.dx;
        if dx > 0 {
            self.dist += self.dy;
        }
        if dx < 0 {
            self.dist -= self.dy;
        }
    }
}

pub const POLY_MR_SUBPIXEL_SHIFT : i64 = 4;

pub fn line_mr(x: i64) -> i64 {
    x >> (POLY_SUBPIXEL_SHIFT - POLY_MR_SUBPIXEL_SHIFT)
}

impl DistanceInterpolator2 {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64,
               sx: i64, sy: i64, x: i64, y: i64, start: bool) -> Self {
        let dx = x2-x1;
        let dy = y2-y1;
        let (dx_start, dy_start) = if start {
            (line_mr(sx) - line_mr(x1), line_mr(sy) - line_mr(y1))
        } else {
            (line_mr(sx) - line_mr(x2), line_mr(sy) - line_mr(y2))
        };
        let dist = (x + POLY_SUBPIXEL_SCALE/2 - x2) as f64 * dy as f64 -
                   (y + POLY_SUBPIXEL_SCALE/2 - y2) as f64 * dx as f64;
        let dist = dist.round() as i64;
        let dist_start = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(sx)) * dy_start -
                         (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(sy)) * dx_start;
        let dx = dx << POLY_SUBPIXEL_SHIFT;
        let dy = dy << POLY_SUBPIXEL_SHIFT;
        let dx_start = dx_start << POLY_MR_SUBPIXEL_SHIFT;
        let dy_start = dy_start << POLY_MR_SUBPIXEL_SHIFT;

        Self { dx, dy, dx_start, dy_start, dist, dist_start }
    }
}

impl DistanceInterpolator for DistanceInterpolator2 {
    fn dist(&self) -> i64 {
        self.dist
    }
    fn inc_x(&mut self, dy: i64) {
        self.dist       += self.dy;
        self.dist_start += self.dy_start;
        if dy > 0 {
            self.dist       -= self.dx;
            self.dist_start -= self.dx_start;
        }
        if dy < 0 {
            self.dist       += self.dx;
            self.dist_start += self.dx_start;
        }
    }
    fn inc_y(&mut self, dx: i64) {
        self.dist       -= self.dx;
        self.dist_start -= self.dx_start;
        if dx > 0 {
            self.dist       += self.dy;
            self.dist_start += self.dy_start;
        }
        if dx < 0 {
            self.dist       -= self.dy;
            self.dist_start -= self.dy_start;
        }
    }
    fn dec_x(&mut self, dy: i64) {
        self.dist       -= self.dy;
        self.dist_start -= self.dy_start;
        if dy > 0 {
            self.dist       -= self.dx;
            self.dist_start -= self.dx_start;
        }
        if dy < 0 {
            self.dist       += self.dx;
            self.dist_start += self.dx_start;
        }
    }
    fn dec_y(&mut self, dx: i64) {
        self.dist       += self.dx;
        self.dist_start += self.dx_start;
        if dx > 0 {
            self.dist       += self.dy;
            self.dist_start += self.dy_start;
        }
        if dx < 0 {
            self.dist       -= self.dy;
            self.dist_start -= self.dy_start;
        }
    }
}

impl DistanceInterpolator3 {
    pub fn new(x1: i64, y1: i64, x2: i64, y2: i64,
               sx: i64, sy: i64, ex: i64, ey: i64,
               x: i64, y: i64) -> Self {
        let dx = x2-x1;
        let dy = y2-y1;
        let dx_start = line_mr(sx) - line_mr(x1);
        let dy_start = line_mr(sy) - line_mr(y1);
        let dx_end   = line_mr(sx) - line_mr(x2);
        let dy_end   = line_mr(sy) - line_mr(y2);

        let dist = (x + POLY_SUBPIXEL_SHIFT/2 - x2) as f64 * dy as f64 -
                   (y + POLY_SUBPIXEL_SHIFT/2 - y2) as f64 * dx as f64;
        let dist = dist.round() as i64;
        let dist_start = line_mr(x + POLY_SUBPIXEL_SHIFT/2) - line_mr(sx) * dy_start -
                         line_mr(y + POLY_SUBPIXEL_SHIFT/2) - line_mr(sy) * dx_start;
        let dist_end   = line_mr(x + POLY_SUBPIXEL_SHIFT/2) - line_mr(ex) * dy_end -
                         line_mr(y + POLY_SUBPIXEL_SHIFT/2) - line_mr(ey) * dx_end;

        let dx = dx << POLY_SUBPIXEL_SHIFT;
        let dy = dy << POLY_SUBPIXEL_SHIFT;
        let dx_start = dx_start << POLY_MR_SUBPIXEL_SHIFT;
        let dy_start = dy_start << POLY_MR_SUBPIXEL_SHIFT;
        let dx_end   = dx_start << POLY_MR_SUBPIXEL_SHIFT;
        let dy_end   = dy_start << POLY_MR_SUBPIXEL_SHIFT;
        Self {
            dx, dy, dx_start, dy_start, dx_end, dy_end, dist_start, dist_end, dist
        }
    }
}

impl DistanceInterpolator for DistanceInterpolator3 {
    fn dist(&self) -> i64 {
        self.dist
    }
    fn inc_x(&mut self, dy: i64) {
        self.dist       += self.dy; 
        self.dist_start += self.dy_start; 
        self.dist_end   += self.dy_end;
        if dy > 0 {
            self.dist       -= self.dx; 
            self.dist_start -= self.dx_start; 
            self.dist_end   -= self.dx_end;
        }
        if dy < 0 {
            self.dist       += self.dx; 
            self.dist_start += self.dx_start; 
            self.dist_end   += self.dx_end;
        }
    }
    fn inc_y(&mut self, dx: i64) {
        self.dist       -= self.dx; 
        self.dist_start -= self.dx_start; 
        self.dist_end   -= self.dx_end;
        if dx > 0 {
            self.dist       += self.dy; 
            self.dist_start += self.dy_start; 
            self.dist_end   += self.dy_end;
        }
        if dx < 0 {
            self.dist       -= self.dy; 
            self.dist_start -= self.dy_start; 
            self.dist_end   -= self.dy_end;
        }
    }
    fn dec_x(&mut self, dy: i64) {
        self.dist       -= self.dy; 
        self.dist_start -= self.dy_start; 
        self.dist_end   -= self.dy_end;
        if dy > 0 {
            self.dist       -= self.dx; 
            self.dist_start -= self.dx_start; 
            self.dist_end   -= self.dx_end;
        }
        if dy < 0 {
            self.dist       += self.dx; 
            self.dist_start += self.dx_start; 
            self.dist_end   += self.dx_end;
        }
    }
    fn dec_y(&mut self, dx: i64) {
        self.dist       += self.dx; 
        self.dist_start += self.dx_start; 
        self.dist_end   += self.dx_end;
        if dx > 0 {
            self.dist       += self.dy; 
            self.dist_start += self.dy_start; 
            self.dist_end   += self.dy_end;
        }
        if dx < 0 {
            self.dist       -= self.dy; 
            self.dist_start -= self.dy_start; 
            self.dist_end   -= self.dy_end;
        }
    }
}

pub struct AA0 {
    pub di: DistanceInterpolator1,
    pub li: LineInterpolatorAA,
}
impl AA0 {
    pub fn new(lp: LineParameters, subpixel_width: i64) -> Self {
        let mut li = LineInterpolatorAA::new(lp, subpixel_width);
        li.li.adjust_forward();
        Self { li, di: DistanceInterpolator1::new(lp.x1,lp.y1,lp.x2,lp.y2,
                                                  lp.x1 & ! POLY_SUBPIXEL_MASK,
                                                  lp.y1 & ! POLY_SUBPIXEL_MASK)
        }
    }
    pub fn count(&self) -> i64 {     self.li.count    }
    pub fn vertical(&self) -> bool { self.li.lp.vertical    }
    pub fn step_hor<R>(&mut self, ren: &mut R) -> bool
        where R: RenderOutline
    {
        let s1 = self.li.step_hor_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        self.li.covers[p1] = ren.cover(s1);

        p1 += 1;
        let mut dy = 1;
        let mut dist = self.li.dist[dy] - s1;
        while dist <= self.li.width {
            self.li.covers[p1] = ren.cover(dist);
            p1 += 1;
            dy += 1;
            dist = self.li.dist[dy] - s1;
        }

        let mut dy = 1;
        dist = self.li.dist[dy] + s1;
        while dist <= self.li.width {
            p0 -= 1;
            self.li.covers[p0] = ren.cover(dist);
            dy += 1;
            dist = self.li.dist[dy] + s1;
        }
        ren.blend_solid_vspan(self.li.x,
                              self.li.y - dy as i64 + 1,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step += 1;
        self.li.step < self.li.count
    }
    pub fn step_ver<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_ver_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;
        self.li.covers[p1] = ren.cover(s1);
        p1 += 1;
        let mut dx = 1;
        let mut dist = self.li.dist[dx] - s1;
        while dist <= self.li.width {
            self.li.covers[p1] = ren.cover(dist);
            p1 += 1;
            dx += 1;
            dist = self.li.dist[dx] - s1;
        }

        dx = 1;
        dist = self.li.dist[dx] + s1;
        while dist  <= self.li.width {
            p0 -= 1;
            self.li.covers[p0] = ren.cover(dist);
            dx += 1;
            dist = self.li.dist[dx] + s1;
        }
        ren.blend_solid_hspan(self.li.x - dx as i64 + 1,
                              self.li.y,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step += 1;
        self.li.step < self.li.count
    }
}

pub struct AA1 {
    pub di: DistanceInterpolator2,
    pub li: LineInterpolatorAA,
}
impl AA1 {
    pub fn new(lp: LineParameters, sx: i64, sy: i64, subpixel_width: i64) -> Self {
        let mut li = LineInterpolatorAA::new(lp, subpixel_width);
        let mut di =  DistanceInterpolator2::new(lp.x1,lp.y1,lp.x2,lp.y2, sx, sy,
                                                 lp.x1 & ! POLY_SUBPIXEL_MASK,
                                                 lp.y1 & ! POLY_SUBPIXEL_MASK,
                                                 true);
        let mut npix = 1;
        if lp.vertical {
            loop {
                li.li.dec();
                li.y -= lp.inc;
                li.x = (li.lp.x1 + li.li.y) >> POLY_SUBPIXEL_SHIFT;

                if lp.inc > 0 {
                    di.dec_y(li.x - li.old_x);
                } else {
                    di.inc_y(li.x - li.old_x);
                }
                li.old_x = li.x;

                let mut dist1_start = di.dist_start;
                let mut dist2_start = di.dist_start;

                let mut dx = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start += di.dy_start;
                    dist2_start -= di.dy_start;
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1
                    }
                    dx += 1;
                    if li.dist[dx] > li.width {
                        break;
                    }
                }
                li.step -= 1;
                if npix == 0 {
                    break;
                }
                npix = 0;
                if li.step < -li.max_extent {
                    break;
                }
            }
        } else {
            loop {
                li.li.dec();
                li.x -= lp.inc;
                li.y = (li.lp.y1 + li.li.y) >> POLY_SUBPIXEL_SHIFT;
                if lp.inc > 0 {
                    di.dec_x(li.y - li.old_y);
                } else {
                    di.inc_x(li.y - li.old_y);
                }
                li.old_y = li.y;

                let mut dist1_start = di.dist_start;
                let mut dist2_start = di.dist_start;

                let mut dy = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start -= di.dx_start;
                    dist2_start += di.dx_start;
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dy += 1;
                    if li.dist[dy] > li.width {
                        break;
                    }
                }
                li.step -= 1;
                if npix == 0 {
                    break;
                }
                npix = 0;
                if li.step < -li.max_extent {
                    break;
                }
            }
        }
        li.li.adjust_forward();
        Self { li, di }
    }
    pub fn count(&self) -> i64 {        self.li.count    }
    pub fn vertical(&self) -> bool {        self.li.lp.vertical    }
    pub fn step_hor<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_hor_base(&mut self.di);

        let mut dist_start = self.di.dist_start;
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;
        self.li.covers[p1] = 0;
        if dist_start <= 0 {
            self.li.covers[p1] = ren.cover(s1);
        }
        p1 += 1;
        let mut dy = 1;
        let mut dist = self.li.dist[dy] - s1;
        while dist <= self.li.width {
            dist_start -= self.di.dx_start;
            self.li.covers[p1] = 0;
            if dist_start <= 0  {
                self.li.covers[p1] = ren.cover(dist);
            }
            p1 += 1;
            dy += 1;
            dist = self.li.dist[dy] - s1;
        }

        dy = 1;
        dist_start = self.di.dist_start;
        dist = self.li.dist[dy] + s1;
        while dist <= self.li.width {
            dist_start += self.di.dx_start;
            p0 -= 1;
            self.li.covers[p0] = 0;
            if dist_start <= 0 {
                self.li.covers[p0] = ren.cover(dist);
            }
            dy += 1;
            dist = self.li.dist[dy] + s1;
        }
        ren.blend_solid_vspan(self.li.x,
                              self.li.y - dy as i64 + 1,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step += 1;
        self.li.step < self.li.count

    }
    pub fn step_ver<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_ver_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        let mut dist_start = self.di.dist_start;
        self.li.covers[p1] = 0;
        if dist_start <= 0 {
            self.li.covers[p1] = ren.cover(s1);
        }
        p1 += 1;
        let mut dx = 1;
        let mut dist = self.li.dist[dx] - s1;
        while dist <= self.li.width {
            dist_start += self.di.dy_start;
            self.li.covers[p1] = 0;
            if dist_start <= 0 {
                self.li.covers[p1] = ren.cover(dist);
            }
            p1 += 1;
            dx += 1;
            dist = self.li.dist[dx] - s1;
        }
        dx = 1;
        dist_start = self.di.dist_start;
        dist = self.li.dist[dx] + s1;
        while dist <= self.li.width {
            dist_start -= self.di.dy_start;
            p0 -= 1;
            self.li.covers[p0] = 0;
            if dist_start <= 0 {
                self.li.covers[p0] = ren.cover(dist);
            }
            dx += 1;
            dist = self.li.dist[dx] + s1;
        }
        ren.blend_solid_hspan(self.li.x - dx as i64 + 1,
                              self.li.y,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step += 1;
        self.li.step < self.li.count
    }
}
pub struct AA2 {
    pub di: DistanceInterpolator2,
    pub li: LineInterpolatorAA,
}
impl AA2 {
    pub fn new(lp: LineParameters, ex: i64, ey: i64, subpixel_width: i64) -> Self {
        let mut li = LineInterpolatorAA::new(lp, subpixel_width);
        let di = DistanceInterpolator2::new(lp.x1,lp.y1,lp.x2,lp.y2, ex, ey,
                                            lp.x1 & ! POLY_SUBPIXEL_MASK,
                                            lp.y1 & ! POLY_SUBPIXEL_MASK,
                                            false);
        li.li.adjust_forward();
        li.step -= li.max_extent;
        Self {  li, di }
    }
    pub fn count(&self) -> i64 {        self.li.count    }
    pub fn vertical(&self) -> bool {        self.li.lp.vertical    }
    pub fn step_hor<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_hor_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        let mut dist_end = self.di.dist_start;

        let mut npix = 0;
        self.li.covers[p1] = 0;
        if dist_end > 0 {
            self.li.covers[p1] = ren.cover(s1);
            npix += 1;
        }
        p1 += 1;

        let mut dy = 1;
        let mut dist = self.li.dist[dy] - s1;
        while dist <= self.li.width {
            dist_end -= self.di.dx_start;
            self.li.covers[p1] = 0;
            if dist_end > 0 {
                self.li.covers[p1] = ren.cover(dist);
                npix += 1;
            }
            p1 += 1;
            dy += 1;
            dist = self.li.dist[dy] - s1;
        }

        dy = 1;
        dist_end = self.di.dist_start;
        dist = self.li.dist[dy] + s1;
        while dist <= self.li.width {
            dist_end += self.di.dx_start;
            p0 -= 1;
            self.li.covers[p0] = 0;
            if dist_end > 0 {
                self.li.covers[p0] = ren.cover(dist);
                npix += 1;
            }
            dy += 1;
            dist = self.li.dist[dy] + s1;
        }
        ren.blend_solid_vspan(self.li.x,
                              self.li.y - dy as i64 + 1,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step += 1;
        npix != 0 && self.li.step < self.li.count
    }
    pub fn step_ver<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_ver_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        let mut dist_end = self.di.dist_start; // Really dist_end

        let mut npix = 0;
        self.li.covers[p1] = 0;
        if dist_end > 0 {
            self.li.covers[p1] = ren.cover(s1);
            npix += 1;
        }
        p1 += 1;

        let mut dx = 1;
        let mut dist = self.li.dist[dx] - s1;
        while dist <= self.li.width {
            dist_end += self.di.dy_start;
            self.li.covers[p1] = 0;
            if dist_end > 0  {
                self.li.covers[p1] = ren.cover(dist);
                npix += 1;
            }
            p1 += 1;
            dx += 1;
            dist = self.li.dist[dx] - s1;
        }

        dx = 1;
        dist_end = self.di.dist_start;
        dist = self.li.dist[dx] + s1;
        while dist <= self.li.width {
            dist_end -= self.di.dy_start;
            p0 -= 1;
            self.li.covers[p0] = 0;
            if dist_end > 0 {
                self.li.covers[p0] = ren.cover(dist);
                npix += 1;
            }
            dx += 1;
            dist = self.li.dist[dx] + s1;
        }
        ren.blend_solid_hspan(self.li.x - dx as i64 + 1,
                              self.li.y,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step += 1;
        npix != 0 && self.li.step < self.li.count
    }
}
pub struct AA3 {
    pub di: DistanceInterpolator3,
    pub li: LineInterpolatorAA,
}
impl AA3 {
    pub fn new(lp: LineParameters, sx: i64, sy: i64, ex: i64, ey: i64, subpixel_width: i64) -> Self {
        let mut li = LineInterpolatorAA::new(lp, subpixel_width);
        let mut di = DistanceInterpolator3::new(lp.x1, lp.y1, lp.x2, lp.y2,
                                            sx, sy, ex, ey,
                                            lp.x1 & ! POLY_SUBPIXEL_MASK,
                                            lp.y1 & ! POLY_SUBPIXEL_MASK);
        let mut npix = 1;
        if lp.vertical {
            loop {
                li.li.dec();
                li.y -= lp.inc;
                li.x = (li.lp.x1 + li.li.y) >> POLY_SUBPIXEL_SHIFT;

                if lp.inc > 0 {
                    di.dec_y(li.x - li.old_x);
                } else {
                    di.inc_y(li.x - li.old_x);
                }

                li.old_x = li.x;

                let mut dist1_start = di.dist_start;
                let mut dist2_start = di.dist_start;

                let mut dx = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start += di.dy_start;
                    dist2_start -= di.dy_start;
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dx += 1;
                    if li.dist[dx] > li.width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }
                npix = 0;
                li.step -= 1;
                if li.step < -li.max_extent {
                    break;
                }
            }
        } else {
            loop {
                li.li.dec();
                li.x -= lp.inc;
                li.y = (li.lp.y1 + li.li.y) >> POLY_SUBPIXEL_SHIFT;

                if lp.inc > 0 {
                    di.dec_x(li.y - li.old_y);
                } else {
                    di.inc_x(li.y - li.old_y);
                }

                li.old_y = li.y;

                let mut dist1_start = di.dist_start;
                let mut dist2_start = di.dist_start;

                let mut dy = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start -= di.dx_start;
                    dist2_start += di.dx_start;
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dy += 1;
                    if li.dist[dy] > li.width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }
                npix = 0;
                li.step -= 1;
                if li.step < -li.max_extent {
                    break;
                }
            }
        }
        li.li.adjust_forward();
        li.step -= li.max_extent;
        Self { li, di }
    }
    pub fn count(&self) -> i64 {        self.li.count    }
    pub fn vertical(&self) -> bool {        self.li.lp.vertical    }
    pub fn step_hor<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_hor_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        let mut dist_start = self.di.dist_start;
        let mut dist_end   = self.di.dist_end;

        let mut npix = 0;
        self.li.covers[p1] = 0;
        if dist_end > 0 {
            if dist_start <= 0 {
                self.li.covers[p1] = ren.cover(s1);
            }
            npix += 1;
        }
        p1 += 1;

        let mut dy = 1;
        let mut dist = self.li.dist[dy] - s1;
        while dist <= self.li.width {
            dist_start -= self.di.dx_start;
            dist_end   -= self.di.dx_end;
            self.li.covers[p1] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.li.covers[p1] = ren.cover(dist);
                npix += 1;
            }
            p1 += 1;
            dy += 1;
            dist = self.li.dist[dy] - s1;
        }

        dy = 1;
        dist_start = self.di.dist_start;
        dist_end   = self.di.dist_end;
        dist = self.li.dist[dy] + s1;
        while dist <= self.li.width {
            dist_start += self.di.dx_start;
            dist_end   += self.di.dx_end;
            p0 -= 1;
            self.li.covers[p0] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.li.covers[p0] = ren.cover(dist);
                npix += 1;
            }
            dy += 1;
        }
        ren.blend_solid_vspan(self.li.x,
                              self.li.y - dy as i64 + 1,
                              (p1 - p0) as i64,
                              &self.li.covers[p0..]);
        self.li.step -= 1;
        npix != 0 && self.li.step < self.li.count

    }
    pub fn step_ver<R: RenderOutline>(&mut self, ren: &mut R) -> bool {
        let s1 = self.li.step_ver_base(&mut self.di);
        let mut p0 = MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        let mut dist_start = self.di.dist_start;
        let mut dist_end   = self.di.dist_end;

        let mut npix = 0;
        self.li.covers[p1] = 0;
        if dist_end > 0 {
            if dist_start <= 0 {
                self.li.covers[p1] = ren.cover(s1);
            }
            npix += 1;
        }
        p1 += 1;

        let mut dx = 1;
        let mut dist = self.li.dist[dx] - s1;
        while dist <= self.li.width {
            dist_start += self.di.dy_start;
            dist_end   += self.di.dy_end;
            self.li.covers[p1] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.li.covers[p1] = ren.cover(dist);
                npix += 1;
            }
            p1 += 1;
            dx += 1;
            dist = self.li.dist[dx] - s1;
        }

        dx = 1;
        dist_start = self.di.dist_start;
        dist_end   = self.di.dist_end;
        dist = self.li.dist[dx] + s1;
        while dist <= self.li.width {
            dist_start -= self.di.dy_start;
            dist_end   -= self.di.dy_end;
            p0 -= 1;
            self.li.covers[p0] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.li.covers[p0] = ren.cover(dist);
                npix += 1;
            }
            dx += 1;
            dist = self.li.dist[dx] + s1;
        }
        ren.blend_solid_hspan(self.li.x - dx as i64 + 1,
                                      self.li.y,
                                      (p1 - p0) as i64,
                                      &self.li.covers[p0..]);
        self.li.step -= 1;
        npix != 0&& self.li.step < self.li.count

    }
}
pub trait RenderOutline {
    fn cover(&self, d: i64) -> u64;
    fn blend_solid_hspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]);
    fn blend_solid_vspan(&mut self, x: i64, y: i64, len: i64, covers: &[u64]);
}

pub trait LineInterp {
    fn init(&mut self);
    fn step_hor(&mut self);
    fn step_ver(&mut self);
}

pub const MAX_HALF_WIDTH : usize = 64;

pub struct LineInterpolatorAA {
    pub lp: LineParameters,
    pub li: LineInterpolator,
    pub len: i64,
    pub x: i64,
    pub y: i64,
    pub old_x: i64,
    pub old_y: i64,
    pub count: i64,
    pub width: i64,
    pub max_extent: i64,
    pub step: i64,
    pub dist: [i64; MAX_HALF_WIDTH + 1],
    pub covers: [u64; MAX_HALF_WIDTH * 2 + 4],
}

impl LineInterpolatorAA {
    fn new(lp: LineParameters, subpixel_width: i64) -> Self {
        let len = if lp.vertical == (lp.inc > 0) { -lp.len } else { lp.len };
        let x = lp.x1 >> POLY_SUBPIXEL_SHIFT;
        let y = lp.y1 >> POLY_SUBPIXEL_SHIFT;
        let old_x = x;
        let old_y = y;
        let count = if lp.vertical {
            ((lp.y2 >> POLY_SUBPIXEL_SHIFT) - y).abs()
        } else {
            ((lp.x2 >> POLY_SUBPIXEL_SHIFT) - x).abs()
        };
        let width = subpixel_width;
        let max_extent = (width + POLY_SUBPIXEL_MASK) >> POLY_SUBPIXEL_SHIFT;
        let step = 0;
        let y1 = if lp.vertical {
            (lp.x2-lp.x1) << POLY_SUBPIXEL_SHIFT
        } else {
            (lp.y2-lp.y1) << POLY_SUBPIXEL_SHIFT
        };
        let n = if lp.vertical {
            (lp.y2-lp.y1).abs()
        } else {
            (lp.x2-lp.x1).abs() + 1
        };

        let m_li = LineInterpolator::new_back_adjusted_2(y1, n);

        let mut dd = if lp.vertical { lp.dy } else { lp.dx };
        dd = dd << POLY_SUBPIXEL_SHIFT;
        let mut li = LineInterpolator::new_foward_adjusted(0, dd, lp.len);

        let mut dist = [0i64; MAX_HALF_WIDTH + 1];
        let stop = width + POLY_SUBPIXEL_SCALE * 2;
        for i in 0 .. MAX_HALF_WIDTH {
            dist[i] = li.y;
            if li.y >= stop {
                break;
            }
            li.inc();
        }
        dist[MAX_HALF_WIDTH] = 0x7FFF_0000 ;
        let covers = [0u64; MAX_HALF_WIDTH * 2 + 4];
        let li = Self { lp, li: m_li, len, x, y, old_x, old_y, count,
                        width, max_extent, step,
                        dist, covers };
        li
    }
    pub fn step_hor_base<DI>(&mut self, di: &mut DI) -> i64
        where DI: DistanceInterpolator
    {
        self.li.inc();
        self.x += self.lp.inc;
        self.y = (self.lp.y1 + self.li.y) >> POLY_SUBPIXEL_SHIFT;
        if self.lp.inc > 0 {
            di.inc_x(self.y - self.old_y);
        } else {
            di.dec_x(self.y - self.old_y);
        }
        self.old_y = self.y;
        di.dist() / self.len
    }
    pub fn step_ver_base<DI>(&mut self, di: &mut DI) -> i64
        where DI: DistanceInterpolator
    {
        self.li.inc();
        self.y += self.lp.inc;
        self.x = (self.lp.x1 + self.li.y) >> POLY_SUBPIXEL_SHIFT;

        if self.lp.inc > 0 {
            di.inc_y(self.x - self.old_x);
        } else {
            di.dec_y(self.x - self.old_x);
        }

        self.old_x = self.x;
        di.dist() / self.len
    }
}


pub struct RasterizerOutline<'a,T> where T: PixfmtFunc + Pixel  {
    pub ren: &'a mut RendererPrimatives<'a,T>,
    pub start_x: i64,
    pub start_y: i64,
    pub vertices: usize,
}
impl<'a,T> RasterizerOutline<'a,T> where T: PixfmtFunc + Pixel {
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

pub fn len_i64(a: &Vertex<i64>, b: &Vertex<i64>) -> i64 {
    len_i64_xy(a.x, a.y, b.x, b.y)
}
pub fn len_i64_xy(x1: i64, y1: i64, x2: i64, y2: i64) -> i64 {
    let dx = x1 as f64 - x2 as f64;
    let dy = y1 as f64 - y2 as f64;
    let v = (dx*dx + dy*dy).sqrt().round() as i64;
    v
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum LineJoin {
    Round,
    None,
    Miter,
    MiterAccurate,
}

pub struct RasterizerOutlineAA<'a,T> where T: SetColor + AccurateJoins + Lines {
    pub ren: &'a mut T,
    pub start_x: i64,
    pub start_y: i64,
    pub vertices: Vec<Vertex<i64>>,
    pub round_cap: bool,
    pub line_join: LineJoin,
}

impl<'a,T> RasterizerOutlineAA<'a, T> where T: SetColor + AccurateJoins + Lines {
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
                PathCommand::Close => self.close(),
                PathCommand::Stop => unimplemented!("stop encountered"),
            }
        }
        self.render(false);
    }
    pub fn conv(&self, v: f64) -> i64 {
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
    pub fn move_to(&mut self, x: i64, y: i64) {
        self.start_x = x;
        self.start_y = y;
        self.vertices.push( Vertex::move_to(x, y) );
    }
    pub fn line_to(&mut self, x: i64, y: i64) {
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
    pub fn close(&mut self) {
    }
    pub fn cmp_dist_start(d: i64) -> bool { d > 0 }
    pub fn cmp_dist_end  (d: i64) -> bool { d <= 0 }
    pub fn draw_two_points(&mut self) {
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
    pub fn draw_three_points(&mut self) {
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
    pub fn draw_many_points(&mut self) {
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
    pub fn draw(&mut self, dv: &mut DrawVars, start: usize, end: usize) {
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
    pub fn bisectrix(l1: &LineParameters, l2: &LineParameters) -> (i64, i64) {
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
