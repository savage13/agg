agg
===

[![Documentation](https://docs.rs/agg/badge.svg)](https://docs.rs/agg)

A Rust port of [Anti-Grain Geometry](http://www.antigrain.com/)

> A High Fidelity and Quality 2D Graphics Rendering Engine

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
agg = "0.1.1"
```

## Example

![Little Black Triangle](https://github.com/savage13/agg/blob/master/tests/little_black_triangle.png)

```rust

#[test]
use agg::Render;

// Create a blank image 10x10 pixels
let pix = agg::Pixfmt::<agg::Rgb8>::new(100,100);
let mut ren_base = agg::RenderingBase::new(pix);
ren_base.clear(agg::Rgba8::white());

// Draw a polygon from (10,10) - (50,90) - (90,10)
let mut ras = agg::RasterizerScanline::new();
ras.move_to_d(10.0, 10.0);
ras.line_to_d(50.0, 90.0);
ras.line_to_d(90.0, 10.0);

// Render the line to the image
let mut ren = agg::RenderingScanlineAASolid::with_base(&mut ren_base);
ren.color(&agg::Rgba8::black());
agg::render_scanlines(&mut ras, &mut ren);

// Save the image to a file
ren_base.to_file("tests/tmp/little_black_triangle.png").unwrap();
```

## Features

  - Anti-Aliased Drawing
  - Sub-pixel Accuracy
  - Rendering of Arbitrary Polygons
  - Text/Font Rendering (through with [Freetype](https://www.freetype.org/))

  - Performance ? (to be determined)


## Complexity

Quoting the original C++ library:

> **Anti-Grain Geometry** is not a solid graphic library and it's not very easy to use. I consider **AGG** as a **"tool to create other tools"**. It means that there's no **"Graphics"** object or something like that, instead, **AGG** consists of a number of loosely coupled algorithms that can be used together or separately. All of them have well defined interfaces and absolute minimum of implicit or explicit dependencies.

## License

This version was ported from agg-2.4 (BSD 3-Clause) and is released
under the BSD 2-Clause License.


