use crate::color::Rgba8;
use crate::Color;

/// Interpolate a value between two end points using fixed point math
///
/// See agg_color_rgba.h:454 of agg version 2.4
///
pub fn lerp_u8(p: u8, q: u8, a: u8) -> u8 {
    let base_shift = 8;
    let base_msb = 1 << (base_shift - 1);
    let v = if p > q { 1 } else { 0 };
    let (q,p,a) = (i32::from(q), i32::from(p), i32::from(a));
    let t0 : i32  = (q - p) * a + base_msb - v; // Signed multiplication
    let t1 : i32 = ((t0>>base_shift) + t0) >> base_shift;
    (p + t1) as u8
}

pub fn prelerp_u8(p: u8, q: u8, a: u8) -> u8 {
    p.wrapping_add(q).wrapping_sub(multiply_u8(p,a))
}

/// Multiply two u8 values using fixed point math
///
/// See agg_color_rgba.h:395
/// https://sestevenson.wordpress.com/2009/08/19/rounding-in-fixed-point-number-conversions/
/// https://stackoverflow.com/questions/10067510/fixed-point-arithmetic-in-c-programming
/// http://x86asm.net/articles/fixed-point-arithmetic-and-tricks/
/// Still not sure where the value is added and shifted multiple times
pub fn multiply_u8(a: u8, b: u8) -> u8 {
    let base_shift = 8;
    let base_msb = 1 << (base_shift - 1);
    let (a,b) = (u32::from(a), u32::from(b));
    let t : u32  = a * b + base_msb;
    let tt : u32 = ((t >> base_shift) + t) >> base_shift;
    tt as u8
}

/// Blend foreground and background pixels with an cover value
///
/// Color components are computed by:
///
/// out = (alpha * cover) * (c - p)
///
/// Computations are conducted using fixed point math
///
/// see [Alpha Compositing](https://en.wikipedia.org/wiki/Alpha_compositing)

pub fn blend_pix<C1: Color, C2: Color>(p: &C1, c: &C2, cover: u64) -> Rgba8 {

    assert!(c.alpha() >= 0.0);
    assert!(c.alpha() <= 1.0);

    let alpha = multiply_u8(c.alpha8(), cover as u8);
    eprintln!("BLEND PIX: ALPHA COVER {} {} => {}", c.alpha8(), cover, alpha);
    eprintln!("BLEND PIX: {:?}", p);
    eprintln!("BLEND PIX: {:?}", c);

    let red   = lerp_u8(p.red8(),   c.red8(),   alpha);
    let green = lerp_u8(p.green8(), c.green8(), alpha);
    let blue  = lerp_u8(p.blue8(),  c.blue8(),  alpha);
    let alpha = lerp_u8(p.alpha8(), c.alpha8(), alpha);
    eprintln!("BLEND PIX: r,g,b,a {:.3} {:.3} {:.3} {:.3}",
              red, green, blue, alpha);
    Rgba8::new(red, green, blue, alpha)
}
