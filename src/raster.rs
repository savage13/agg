//! Rasterizer

use crate::POLY_SUBPIXEL_SHIFT;
use crate::POLY_SUBPIXEL_SCALE;
//use crate::POLY_SUBPIXEL_MASK;

use crate::clip::Clip;
use crate::scan::ScanlineU8;
use crate::cell::RasterizerCell;
use crate::path_storage::PathCommand;
use crate::path_storage::Vertex;

//use crate::Rasterize;
use crate::VertexSource;

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
#[derive(Debug)]
pub struct RasterizerScanline {
    /// Clipping Region
    pub clipper: Clip,
    /// Collection of Rasterizing Cells
    outline: RasterizerCell,
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

impl RasterizerScanline {
    /// Reset Rasterizer
    ///
    /// Reset the RasterizerCell and set PathStatus to Initial
    pub fn reset(&mut self) {
        self.outline.reset();
        self.status = PathStatus::Initial;
    }
    /// Add a Path
    ///
    /// Walks the path from the VertexSource and rasterizes it
    pub fn add_path<VS: VertexSource>(&mut self, path: &VS) {
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
    pub fn rewind_scanlines(&mut self) -> bool {
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
    pub(crate) fn sweep_scanline(&mut self, sl: &mut ScanlineU8) -> bool {
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
    pub fn min_x(&self) -> i64 {
        self.outline.min_x
    }
    /// Return maximum x value from the RasterizerCell
    pub fn max_x(&self) -> i64 {
        self.outline.max_x
    }
}

impl RasterizerScanline {
    /// Create a new RasterizerScanline
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
    /// Create a new RasterizerScanline with a gamma function
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




pub(crate) fn len_i64(a: &Vertex<i64>, b: &Vertex<i64>) -> i64 {
    len_i64_xy(a.x, a.y, b.x, b.y)
}
pub(crate) fn len_i64_xy(x1: i64, y1: i64, x2: i64, y2: i64) -> i64 {
    let dx = x1 as f64 - x2 as f64;
    let dy = y1 as f64 - y2 as f64;
    (dx*dx + dy*dy).sqrt().round() as i64
}

// #[derive(Debug,PartialEq,Copy,Clone)]
// pub enum LineJoin {
//     Round,
//     None,
//     Miter,
//     MiterAccurate,
// }
