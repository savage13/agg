
use POLY_SUBPIXEL_SCALE;
use POLY_SUBPIXEL_SHIFT;
use POLY_SUBPIXEL_MASK;

use std::cmp::min;
use std::cmp::max;

#[derive(Debug,Copy,Clone,PartialEq, Default)]
pub struct Cell { // cell_aa
    pub x: i64,
    pub y: i64,
    pub cover: i64,
    pub area: i64,
}

impl Cell {
    pub fn new() -> Self {
        Cell { x: std::i64::MAX,
               y: std::i64::MAX,
               cover: 0,
               area: 0
        }
    }
    pub fn at(x: i64, y: i64) -> Self {
        let mut c = Cell::new();
        c.x = x;
        c.y = y;
        c
    }
    pub fn equal(&self, x: i64, y: i64) -> bool {
        self.x - x == 0 && self.y - y == 0
    }
    pub fn is_empty(&self) -> bool {
        self.cover == 0 && self.area == 0
    }
}

#[derive(Debug,Default)]
pub struct RasterizerCell {
    pub cells: Vec<Cell>,
    pub min_x: i64,
    pub max_x: i64,
    pub min_y: i64,
    pub max_y: i64,
    pub sorted_y: Vec<Vec<Cell>>,
}


impl RasterizerCell {
    pub fn new() -> Self {
        Self { cells: vec![],
               min_x: std::i64::MAX,
               min_y: std::i64::MAX,
               max_x: std::i64::MIN,
               max_y: std::i64::MIN,
               sorted_y: vec![],
        }
    }
    pub fn reset(&mut self) {
        self.max_x = std::i64::MIN;
        self.max_y = std::i64::MIN;
        self.min_x = std::i64::MAX;
        self.min_y = std::i64::MAX;
        self.sorted_y.clear(); // Not sure if this should be cleared
        self.cells.clear();    // Not sure if this should be cleared
    }

    pub fn total_cells(&self) -> usize {
        self.cells.len()
    }
    pub fn sort_cells(&mut self) {
        eprintln!("SORT_CELLS MAX_Y: {} N: {} MIN_Y: {}", self.max_y, self.cells.len(), self.min_y);
        if ! self.sorted_y.is_empty() || self.max_y < 0 {
            return;
        }
        // Distribute into 
        self.sorted_y = vec![vec![]; (self.max_y+1) as usize];
        for c in self.cells.iter() {
            if c.y >= 0 {
                let y = c.y as usize;
                eprintln!("SORT_CELLS SORTING {:?}", c);
                self.sorted_y[y].push(c.clone());
            }
        }
        // Sort by the x value
        for i in 0 .. self.sorted_y.len() {
            eprintln!("SORT_CELLS: y: {} len: {}", i, self.sorted_y[i].len());
            self.sorted_y[i].sort_by(|a,b| (a.x).cmp(&b.x));
        }
    }
    pub fn scanline_num_cells(&self, y: i64) -> usize {
        self.sorted_y[y as usize].len()
    }
    pub fn scanline_cells(&self, y: i64) -> &[Cell] {
        & self.sorted_y[y as usize]
    }

    //pub fn add_curr_cell(&mut self, new_cell: Cell) {
    //    self.cells.push( new_cell );
    //}
    pub fn curr_cell_is_set(&self, x: i64, y: i64) -> bool {
        match self.cells.last() {
            None      => true,
            Some(cur) => {
                //eprintln!("SET_CURR_CELL: {} {} EQUAL: {} EMPTY: {}", x, y, cur.equal(x,y), !cur.is_empty());
                ! cur.equal(x,y) && ! cur.is_empty()
            }
        }
    }
    pub fn curr_cell_not_equal(&self, x: i64, y: i64) -> bool {
        match self.cells.last() {
            None      => true,
            Some(cur) => {
                //eprintln!("XXXX {:?}", cur);
                ! cur.equal(x,y)

            }
        }
    }

    pub fn pop_last_cell_if_empty(&mut self) {
        let n = self.cells.len();
        if n == 0 {
            return;
        }
        if self.cells[n-1].area == 0 && self.cells[n-1].cover == 0 {
            self.cells.pop();
        } else {
            self.show_last_cell();
        }
    }
    pub fn show_last_cell(&self) {
        if let Some(c) = self.cells.last() {
            eprintln!("ADD_CURR_CELL: {} {} area {} cover {} len {}", c.x,c.y,c.area,c.cover, self.cells.len());
        }
    }
    pub fn set_curr_cell(&mut self, x: i64, y: i64)  {
        //eprintln!("SET_CURR_CELL: {} {}", x,y);
        if self.curr_cell_not_equal(x, y) {
            //eprintln!("ADD_CURR_CELL: {} {} {} ", x,y, self.cells.len()+1);
            self.pop_last_cell_if_empty();
            self.cells.push( Cell::at(x,y) );
        }
    }

    pub fn render_hline(&mut self, ey: i64, x1: i64, y1: i64, x2: i64, y2: i64) {
        let ex1 = x1 >> POLY_SUBPIXEL_SHIFT;
        let ex2 = x2 >> POLY_SUBPIXEL_SHIFT;
        let fx1 = x1  & POLY_SUBPIXEL_MASK;
        let fx2 = x2  & POLY_SUBPIXEL_MASK;

        // Horizontal Line
        if y1 == y2 {
            self.set_curr_cell(ex2, ey);
            return;
        }

        // Single Cell
        if ex1 == ex2 {
            //eprintln!("RENDER_HLINE LEN: {}", self.cells.len());
            let m_curr_cell = self.cells.last_mut().unwrap();
            m_curr_cell.cover += y2-y1;
            m_curr_cell.area  += (fx1 + fx2) * (y2-y1);
            eprintln!("INCR0 cover {} area {} dcover {} darea {} x,y {} {}",
                      m_curr_cell.cover,
                      m_curr_cell.area,
                      y2-y1,
                      (fx1 + fx2) * (y2-y1), m_curr_cell.x, m_curr_cell.y);
            return;
        }
        //eprintln!("RENDER_HLINE ADJCENT CELLS SAME LINE {} {}", x1,x2);
        // Adjacent Cells on Same Line
        let (mut p, first, incr, dx) = if x2-x1 < 0 {
            (fx1 * (y2-y1), 0,-1, x1-x2)
        } else {
            ((POLY_SUBPIXEL_SCALE - fx1) * (y2-y1), POLY_SUBPIXEL_SCALE, 1, x2-x1)
        };
        let mut delta = p / dx;
        let mut xmod =  p % dx;

        if xmod < 0 {
            delta -= 1;
            xmod += dx;
        }
        {
            let m_curr_cell = self.cells.last_mut().unwrap();
            m_curr_cell.cover += delta;
            m_curr_cell.area  += (fx1 + first) * delta;
            eprintln!("INCR1 cover {} area {} dcover {} darea {} x,y {} {}",
                      m_curr_cell.cover,
                      m_curr_cell.area,
                      delta,
                      (fx1 + first) * delta, m_curr_cell.x, m_curr_cell.y);

        }
        let mut ex1 = ex1 + incr;
        self.set_curr_cell(ex1, ey);
        let mut y1 = y1 + delta;

        if ex1 != ex2 {
            p = POLY_SUBPIXEL_SCALE * (y2 - y1 + delta);
            let mut lift = p / dx;
            let mut rem = p % dx;
            if rem < 0 {
                lift -= 1;
                rem += dx;
            }
            xmod -= dx;

            while ex1 != ex2 {
                delta = lift;
                xmod += rem;
                if xmod >= 0 {
                    xmod -= dx;
                    delta += 1;
                }
                {
                    let m_curr_cell = self.cells.last_mut().unwrap();
                    m_curr_cell.cover += delta;
                    m_curr_cell.area  += POLY_SUBPIXEL_SCALE * delta;
                    eprintln!("INCR2 cover {} area {} dcover {} darea {} x,y {} {}",
                              m_curr_cell.cover,
                              m_curr_cell.area,
                              delta,
                              POLY_SUBPIXEL_SCALE * delta, m_curr_cell.x, m_curr_cell.y);
                }
                y1 += delta;
                ex1 += incr;
                self.set_curr_cell(ex1, ey);
            }
        }
        delta = y2-y1;
        {
            let m_curr_cell = self.cells.last_mut().unwrap();
            m_curr_cell.cover += delta;
            m_curr_cell.area  += (fx2 + POLY_SUBPIXEL_SCALE - first) * delta;
            eprintln!("INCR3 cover {} area {} dcover {} darea {} x,y {} {}",
                      m_curr_cell.cover,
                      m_curr_cell.area,
                      delta,
                      (fx2 + POLY_SUBPIXEL_SCALE - first) * delta, m_curr_cell.x, m_curr_cell.y);
        }
    }

    pub fn line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) {
        eprintln!("LINE: {} {} -> {} {}", x1,y1, x2,y2);
        let dx_limit = 16384 << POLY_SUBPIXEL_SHIFT;
        let dx = x2 - x1;
        // Split long lines in half
        if dx >= dx_limit || dx <= -dx_limit {
            let cx = (x1 + x2) / 2;
            let cy = (y1 + y2) / 2;
            self.line(x1, y1, cx, cy);
            self.line(cx, cy, x2, y2);
        }
        let dy = y2-y1;
        // Downshift
        let ex1 = x1 >> POLY_SUBPIXEL_SHIFT;
        let ex2 = x2 >> POLY_SUBPIXEL_SHIFT;
        let ey1 = y1 >> POLY_SUBPIXEL_SHIFT;
        let ey2 = y2 >> POLY_SUBPIXEL_SHIFT;
        let fy1 = y1 &  POLY_SUBPIXEL_MASK;
        let fy2 = y2 &  POLY_SUBPIXEL_MASK;

        self.min_x = min(ex2, min(ex1, self.min_x));
        self.min_y = min(ey2, min(ey1, self.min_y));
        self.max_x = max(ex2, max(ex1, self.max_x));
        self.max_y = max(ey2, max(ey1, self.max_y));

        self.set_curr_cell(ex1, ey1);
        //eprintln!("EY1, EY2: {} {}", ey1, ey2);
        // Horizontal Line
        if ey1 == ey2 {
            //eprintln!("LINE EY1 = EY2");
            self.render_hline(ey1, x1, fy1, x2, fy2);
            let n = self.cells.len();
            if self.cells[n-1].area == 0 && self.cells[n-1].cover == 0 {
                self.cells.pop();
            }
            return;
        }

        if dx == 0 {
            eprintln!("LINE DX = 0");
            let ex = x1 >> POLY_SUBPIXEL_SHIFT;
            let two_fx = (x1 - (ex << POLY_SUBPIXEL_SHIFT)) << 1;

            let (first, incr) = if dy < 0 {
                (0, -1)
            } else {
                (POLY_SUBPIXEL_SCALE, 1)
            };
            //let x_from = x1;
            let delta = first - fy1;
            {
                let m_curr_cell = self.cells.last_mut().unwrap();
                m_curr_cell.cover += delta;
                m_curr_cell.area  += two_fx * delta;
            }

            let mut ey1 = ey1 + incr;
            self.set_curr_cell(ex, ey1);
            let delta = first + first - POLY_SUBPIXEL_SCALE;
            let area = two_fx * delta;
            while ey1 != ey2 {
                {
                    let m_curr_cell = self.cells.last_mut().unwrap();
                    m_curr_cell.cover = delta;
                    m_curr_cell.area = area;
                }
                ey1 += incr;
                self.set_curr_cell(ex, ey1);
            }
            let delta = fy2 - POLY_SUBPIXEL_SCALE + first;
            {
                let m_curr_cell = self.cells.last_mut().unwrap();
                m_curr_cell.cover += delta;
                m_curr_cell.area += two_fx * delta;
            }
            return;
        }
        eprintln!("LINE RENDER MULTPLE LINES {} {}", dx, dy);
        // Render Multiple Lines
        let (p,first,incr, dy) = if dy < 0 {
            (fy1 * dx, 0, -1, -dy)
        } else {
            ((POLY_SUBPIXEL_SCALE - fy1) * dx, POLY_SUBPIXEL_SCALE, 1, dy)
        };
        let mut delta = p / dy;
        let mut xmod  = p % dy;
        if xmod < 0 {
            delta -= 1;
            xmod += dy;
        }
        let mut x_from = x1 + delta;
        self.render_hline(ey1, x1, fy1, x_from, first);
        let mut ey1 = ey1 + incr;
        self.set_curr_cell(x_from >> POLY_SUBPIXEL_SHIFT, ey1);
        if ey1 != ey2 {
            let p = POLY_SUBPIXEL_SCALE * dx;
            let mut lift = p / dy;
            let mut rem  = p % dy;
            if rem < 0 {
                lift -= 1;
                rem += dy;
            }
            xmod -= dy;
            while ey1 != ey2 {
                delta = lift;
                xmod += rem;
                if xmod >= 0 {
                    xmod -= dy;
                    delta += 1;
                }
                let x_to = x_from + delta;
                self.render_hline(ey1, x_from, POLY_SUBPIXEL_SCALE - first, x_to, first);
                x_from = x_to;
                ey1 += incr;
                self.set_curr_cell(x_from >> POLY_SUBPIXEL_SHIFT, ey1);
            }
        }
        self.render_hline(ey1, x_from, POLY_SUBPIXEL_SCALE - first, x2, fy2);
        self.pop_last_cell_if_empty();
    }
}
