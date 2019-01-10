//! Gallery of images produced using agg
//!
//! - [Direct Pixel Access](#direct-pixel-access)
//! - [Copy a set of colors horizontally](#copy-a-set-of-colors-horizontally)
//! - [Copy with an Alpha Channel](#copy-with-an-alpha-channel)
//! - [Filled polygon](#filled-polygon)
//! - [Filled polygon within a clip box](#filled-polygon-within-a-clip-box)
//! - [Filled polygon within a clip box, without anti-aliasing](#filled-polygon-within-a-clip-box-without-anti-aliasing)
//! - [Filled polygon within a clip box, with a gamma value](#filled-polygon-within-a-clip-box-with-a-gamma-value)
//! - [Filled polygon with outline stroke within a clip box](#filled-polygon-with-outline-stroke-within-a-clip-box)
//! - [Filled polygon with outline stroke](#filled-polygon-with-outline-stroke)
//! - [Component Rendering](#component-rendering)
//! - [Solid Image Rendering](#solid-image-rendering)
//! - [Outline Image Rendering](#outline-image-rendering)
//! - [Render a thick line using the Outline Renderer](#render-a-thick-line-using-the-outline-renderer)
//! - [Aliased vs Anti-Aliased Drawing](#aliased-vs-anti-aliased-drawing)
//! - [Aliased vs Anit-Aliased and Subpixel vs Pixel Accuracy](#aliased-vs-anit-aliased-and-subpixel-vs-pixel-accuracy)
//!
//! ### Direct Pixel Access
//!
//! Draw a simple black frame and a green line using
//! [Pixfmt](../pixfmt/struct.Pixfmt.html) and direct pixel access
//!
//! [t01_rendering_buffer.rs](https://github.com/savage13/agg/blob/master/tests/t01_rendering_buffer.rs)
//!
//! [AGG Example](http://www.antigrain.com/doc/basic_renderers/basic_renderers.agdoc.html#toc0002)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_01.png" height="150"/>


//! ### Copy a set of colors horizontally
//!
//! Generate an array of colors, then copy those colors to the image along
//!   horizontal rows
//!
//! [t03_solar_spectrum.rs](https://github.com/savage13/agg/blob/master/tests/t03_solar_spectrum.rs)
//!
//! [AGG Example](http://www.antigrain.com/doc/basic_renderers/basic_renderers.agdoc.html#toc0007)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_03.png" height="150"/>


//! ### Copy with an Alpha Channel
//!
//! Use of a seperate layer, here a grayscale (single-channel) image to
//! *clip* the image based on the coverage level defined by the channel.
//! First with an original white background and the second with a black
//! background.
//!
//! [t04_solar_spectrum_alpha.rs](https://github.com/savage13/agg/blob/master/tests/t04_solar_spectrum_alpha.rs)
//!
//! [AGG Example](http://www.antigrain.com/doc/basic_renderers/basic_renderers.agdoc.html#toc0008)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_04.png" height="150"/>
//!
//! [t05_solar_spectrum_alpha.rs](https://github.com/savage13/agg/blob/master/tests/t05_solar_spectrum_alpha.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_05.png" height="150"/>


//! ### Filled polygon
//!
//! [t11.rs](https://github.com/savage13/agg/blob/master/tests/t11.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_11.png" height="150"/>


//! ### Filled polygon within a clip box
//!
//! [t12.rs](https://github.com/savage13/agg/blob/master/tests/t12.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_12.png" height="150"/>


//! ### Filled polygon within a clip box, without anti-aliasing
//!
//! [t13.rs](https://github.com/savage13/agg/blob/master/tests/t13.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_13.png" height="150"/>


//! ### Filled polygon within a clip box, with a gamma value
//!
//! [t14.rs](https://github.com/savage13/agg/blob/master/tests/t14.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_14.png" height="150"/>


//! ### Filled polygon with outline stroke within a clip box
//!
//! [t15.rs](https://github.com/savage13/agg/blob/master/tests/t15.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_15.png" height="150"/>
//!
//! ### Filled polygon with outline stroke
//!
//! [t16.rs](https://github.com/savage13/agg/blob/master/tests/t16.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/agg_test_16.png" height="150"/>


//! ### Component Rendering
//!
//! The first is with a alpha of 0.5, the second is with an alpha of 1.0.
//!
//! [component_rendering_128.rs](https://github.com/savage13/agg/blob/master/tests/component_rendering_128.rs)
//! [component_rendering_255.rs](https://github.com/savage13/agg/blob/master/tests/component_rendering_255.rs)
//!
//! [AGG Demo](http://www.antigrain.com/demo/component_rendering.cpp.html) and
//! [AGG Output](http://www.antigrain.com/demo/component_rendering_s.gif)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/component_rendering_128.png" height="150"/>
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/component_rendering_255.png" height="150"/>


//! ### Solid Image Rendering
//!
//! Rendering [lion.rs](https://github.com/savage13/agg/blob/master/tests/lion.rs)
//!
//! Rendering, correcting polygon orientation to be clockwise:
//! [lion_cw.rs](https://github.com/savage13/agg/blob/master/tests/lion_cw.rs)
//!
//! Rendering, clockwise orientation, with anti-aliasing:
//! [lion_cw_aa.rs](https://github.com/savage13/agg/blob/master/tests/lion_cw_aa.rs)
//!
//! Rendering, clockwise orientation, with anti-aliasing, and reading from SRGBA format:
//! [lion_cw_aa_srgba.rs](https://github.com/savage13/agg/blob/master/tests/lion_cw_aa_srgba.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/lion.png" height="150"/>
//!

//! ### Outline Image Rendering
//! Rendering using outline only, thick width
//! [lion_outline.rs](https://github.com/savage13/agg/blob/master/tests/lion_outline.rs)
//!
//! Rendering using outline only, width = 1
//! [lion_outline_width1.rs](https://github.com/savage13/agg/blob/master/tests/lion_outline_width1.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/lion_outline.png" height="150"/>
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/lion_outline_width1.png" height="150"/>

//! ## Little Black Triangle
//!
//! Basic Solid Polygon Rendering
//!
//! [t00_example.rs](https://github.com/savage13/agg/blob/master/tests/t00_example.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/little_black_triangle.png" height="150"/>

//! ### Render a thick line using the Outline Renderer
//!
//! [outline_aa.rs](https://github.com/savage13/agg/blob/master/tests/outline_aa.rs)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/outline_aa.png" height="150"/>

//! ### Aliased vs Anti-Aliased Drawing
//!
//! Comparison between Aliased and Anti-Aliased Rendering. Also contructed with a different gamma value
//!
//! [rasterizers.rs](https://github.com/savage13/agg/blob/master/tests/rasterizers.rs)
//!
//! [rasterizers_gamma.rs](https://github.com/savage13/agg/blob/master/tests/rasterizers_gamma.rs)
//!
//! [AGG Source](http://www.antigrain.com/demo/rasterizers.cpp.html)
//! 
//! [AGG Output](http://www.antigrain.com/demo/rasterizers.gif)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/rasterizers.png" height="150"/>

//! ### Aliased vs Anit-Aliased and Subpixel vs Pixel Accuracy
//!
//! Demostration of the **utility** of using both Anti-Aliasing and Subpixel Accuracy
//!
//! The original AGG Documentation has a [discussion](http://www.antigrain.com/doc/introduction/introduction.agdoc.html#toc0005)
//! about the result of using both these features together to produce natural looking graphics.
//!
//! [rasterizers2_pre.rs](https://github.com/savage13/agg/blob/master/tests/rasterizers2_pre.rs)
//!
//! [rasterizers2.rs](https://github.com/savage13/agg/blob/master/tests/rasterizers2.rs)
//!
//! [AGG Source](http://www.antigrain.com/demo/rasterizers2.cpp.html)
//!
//! [AGG Output](http://www.antigrain.com/demo/rasterizers2_s.gif)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/rasterizers2_pre.png" height="150"/>


//! ### Rounded Rectangle
//!
//! Construction of a Rounded Rectangle
//!
//! [AGG Source](http://www.antigrain.com/demo/rounded_rect.cpp.html)
//!
//! [AGG Output](http://www.antigrain.com/demo/rounded_rect_s.gif)
//!
//! <img src="https://raw.githubusercontent.com/savage13/agg/master/images/rounded_rect.png" height="150"/>
//!
