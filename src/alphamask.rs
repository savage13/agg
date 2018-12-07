//! Alphamask Adapator

use math::blend_pix;
use color::Rgb8;
use pixfmt::PixfmtGray8;
use pixfmt::PixfmtRgb24;

/// Alpha Mask Adaptor
pub struct AlphaMaskAdaptor {
    pub rgb: PixfmtRgb24,
    pub alpha: PixfmtGray8,
}


impl AlphaMaskAdaptor {
    /// Create a new Alpha Mask Adapator from a two PixelFormats
    pub fn new(rgb: PixfmtRgb24, alpha: PixfmtGray8) -> Self {
        Self { rgb, alpha }
    }
    /// Blend a set of colors starting at (x,y) with a length
    ///
    /// Background color is from the rgb image and
    /// alpha form the gray scale
    ///
    /// Calls blend_pix
    //
    // From https://stackoverflow.com/a/746937 :
    // out = alpha * new + (1 - alpha) * old
    //   p[j]  = out
    //   alpha = alpha
    //   new   = c
    //   old   = p[j]
    pub fn blend_color_hspan(&mut self, x: usize, y: usize, n: usize,
                             colors: &[Rgb8], _cover: usize) {
        //for i in 0 .. n {
        //assert!(1==2);
        assert_eq!(n, colors.len());
        for (i, color) in colors.iter().enumerate() {
            let pix = &mut self.rgb.get((x+i,y));
            let alpha = u64::from(self.alpha.rbuf[(x+i,y)][0]);
            let pix = blend_pix(pix, color, alpha);
            self.rgb.set((x+i,y), &pix);
        }
    }
}
