
use crate::render::LineInterpolator;
use crate::render::LineInterpolatorImage;
//use crate::render::RendererPrimatives;

use crate::MAX_HALF_WIDTH;
use crate::POLY_MR_SUBPIXEL_SHIFT;
use crate::POLY_SUBPIXEL_SHIFT;
use crate::POLY_SUBPIXEL_MASK;
use crate::POLY_SUBPIXEL_SCALE;
use crate::DistanceInterpolator;
use crate::RenderOutline;

#[derive(Debug)]
pub struct LineInterpolatorAA {
    lp: LineParameters,
    li: LineInterpolator,
    len: i64,
    x: i64,
    y: i64,
    old_x: i64,
    old_y: i64,
    count: i64,
    width: i64,
    max_extent: i64,
    step: i64,
    //pub dist: [i64; MAX_HALF_WIDTH + 1],
    dist: Vec<i64>,
    //pub covers: [u64; MAX_HALF_WIDTH * 2 + 4],
    covers: Vec<u64>,
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
        dd <<= POLY_SUBPIXEL_SHIFT;
        let mut li = LineInterpolator::new_foward_adjusted(0, dd, lp.len);

        let mut dist = vec![0i64; MAX_HALF_WIDTH + 1];
        let stop = width + POLY_SUBPIXEL_SCALE * 2;
        for i in 0 .. MAX_HALF_WIDTH {
            dist[i] = li.y;
            if li.y >= stop {
                break;
            }
            li.inc();
        }
        dist[MAX_HALF_WIDTH] = 0x7FFF_0000 ;
        let covers = vec![0u64; MAX_HALF_WIDTH * 2 + 4];
        Self { lp, li: m_li, len, x, y, old_x, old_y, count,
               width, max_extent, step,
               dist, covers }
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

#[derive(Debug)]
pub(crate) struct AA0 {
    di: DistanceInterpolator1,
    li: LineInterpolatorAA,
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
#[derive(Debug)]
pub(crate) struct AA1 {
    di: DistanceInterpolator2,
    li: LineInterpolatorAA,
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
    //pub fn count(&self) -> i64 {        self.li.count    }
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
#[derive(Debug)]
pub(crate) struct AA2 {
    di: DistanceInterpolator2,
    li: LineInterpolatorAA,
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
    //pub fn count(&self) -> i64 {        self.li.count    }
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
#[derive(Debug)]
pub(crate) struct AA3 {
    di: DistanceInterpolator3,
    li: LineInterpolatorAA,
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
    //pub fn count(&self) -> i64 {        self.li.count    }
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
                              &self.li.covers[p0..p1]);
        self.li.step -= 1;
        npix != 0&& self.li.step < self.li.count

    }
}

#[derive(Debug)]
pub(crate) struct DistanceInterpolator00 {
    dx1: i64,
    dy1: i64,
    dx2: i64,
    dy2: i64,
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
#[derive(Debug)]
pub(crate) struct DistanceInterpolator0 {
    dx: i64,
    dy: i64,
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
#[derive(Debug)]
struct DistanceInterpolator1 {
    dx: i64,
    dy: i64,
    pub dist: i64
}
#[derive(Debug)]
struct DistanceInterpolator2 {
    dx: i64,
    dy: i64,
    dx_start: i64,
    dy_start: i64,
    dist: i64,
    dist_start: i64,
}
#[derive(Debug)]
struct DistanceInterpolator3 {
    dx: i64,
    dy: i64,
    dx_start: i64,
    dy_start: i64,
    dx_end: i64,
    dy_end: i64,
    dist: i64,
    dist_start: i64,
    dist_end: i64,
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


pub(crate) fn line_mr(x: i64) -> i64 {
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
        let dx_end   = line_mr(ex) - line_mr(x2);
        let dy_end   = line_mr(ey) - line_mr(y2);

        let dist = (x + POLY_SUBPIXEL_SCALE/2 - x2) as f64 * dy as f64 -
                   (y + POLY_SUBPIXEL_SCALE/2 - y2) as f64 * dx as f64;
        let dist = dist.round() as i64;
        let dist_start = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(sx)) * dy_start -
                         (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(sy)) * dx_start;
        let dist_end   = (line_mr(x + POLY_SUBPIXEL_SCALE/2) - line_mr(ex)) * dy_end -
                         (line_mr(y + POLY_SUBPIXEL_SCALE/2) - line_mr(ey)) * dx_end;


        let dx = dx << POLY_SUBPIXEL_SHIFT;
        let dy = dy << POLY_SUBPIXEL_SHIFT;
        let dx_start = dx_start << POLY_MR_SUBPIXEL_SHIFT;
        let dy_start = dy_start << POLY_MR_SUBPIXEL_SHIFT;
        let dx_end   = dx_end << POLY_MR_SUBPIXEL_SHIFT;
        let dy_end   = dy_end << POLY_MR_SUBPIXEL_SHIFT;
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
        ((dx0 * dy - dy0 * dx) / len).round() as i64
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
    pub(crate) fn interp0(&self, subpixel_width: i64) -> AA0 {
        AA0::new(*self, subpixel_width)
    }
    pub(crate) fn interp1(&self, sx: i64, sy: i64, subpixel_width: i64) -> AA1 {
        AA1::new(*self, sx, sy, subpixel_width)
    }
    pub(crate) fn interp2(&self, ex: i64, ey: i64, subpixel_width: i64) -> AA2 {
        AA2::new(*self, ex, ey, subpixel_width)
    }
    pub(crate) fn interp3(&self, sx: i64, sy: i64, ex: i64, ey: i64, subpixel_width: i64) -> AA3 {
        AA3::new(*self, sx, sy, ex, ey, subpixel_width)
    }
    pub fn interp_image(&self, sx: i64, sy: i64, ex: i64, ey: i64, subpixel_width: i64, pattern_start: i64, pattern_width: i64, scale_x: f64) -> LineInterpolatorImage {
        LineInterpolatorImage::new(*self, sx, sy, ex, ey,
                                   subpixel_width, pattern_start,
                                   pattern_width, scale_x)
    }
}
