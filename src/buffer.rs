//! Rendering buffer

/// Rendering Buffer
///
/// Data is stored as row-major order (C-format)
#[derive(Debug,Default)]
pub struct RenderingBuffer {
    /// Pixel / Component level data of Image
    pub data: Vec<u8>,
    /// Image Width in pixels
    pub width: usize,
    /// Image Height in pixels
    pub height: usize,
    /// Bytes per pixel or number of color components 
    pub bpp: usize,
}


impl RenderingBuffer {
    /// Create a new buffer of width, height, and bpp
    ///
    /// Data for the Image is allocated 
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        RenderingBuffer {
            width, height, bpp, data: vec![0u8; width * height * bpp]
        }
    }
    /// Size of underlying Rendering Buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Return slice starting a beginning of a row
    pub fn row_ptr(&mut self, i: usize) -> &mut [u8] {
        debug_assert!(i < self.height);
        let row = i * self.width * self.bpp;
        &mut self.data[row .. ]
    }
    /// Clear an image
    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|v| *v = 255);
    }
}

use std::ops::Index;
use std::ops::IndexMut;

impl Index<(usize,usize)> for RenderingBuffer {
    type Output = [u8];
    fn index(&self, index: (usize, usize)) -> &[u8] {
        assert!(index.0 < self.width, "request {} >= {} width :: index", index.0, self.width);
        assert!(index.1 < self.height, "request {} >= {} height :: index", index.1, self.height);
        let i = ((index.1 * self.width) + index.0) * self.bpp;
        assert!(i < self.data.len());
        &self.data[i..]
    }
}
impl IndexMut<(usize,usize)> for RenderingBuffer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut [u8] {
        assert!(index.0 < self.width, "request {} >= {} width :: index_mut", index.0, self.width);
        assert!(index.1 < self.height, "request {} >= {} height :: index_mut", index.1, self.height);
        let i = ((index.1 * self.width) + index.0) * self.bpp;
        assert!(i < self.data.len());
        &mut self.data[i..]
    }
}
