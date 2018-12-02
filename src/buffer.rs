
#[derive(Debug,Default)]
pub struct RenderingBuffer {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
}

impl RenderingBuffer {
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        RenderingBuffer {
            width, height, bpp, data: vec![0u8; width * height * bpp]
        }
    }
    pub fn row_ptr(&mut self, i: usize) -> &mut [u8] {
        let row = i * self.width * self.bpp;
        &mut self.data[row .. ]
    }
    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|v| *v = 255);
    }
    pub fn fill(&mut self) {

    }
}

use std::ops::Index;
use std::ops::IndexMut;

impl Index<(usize,usize)> for RenderingBuffer {
    type Output = [u8];
    fn index(&self, index: (usize, usize)) -> &[u8] {
        let i = ((index.1 * self.width) + index.0) * self.bpp;
        &self.data[i..]
    }
}
impl IndexMut<(usize,usize)> for RenderingBuffer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut [u8] {
        let i = ((index.1 * self.width) + index.0) * self.bpp;
        &mut self.data[i..]
    }
}
