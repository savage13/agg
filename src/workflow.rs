
// How does this work / Data Flow
//    ren = RenAA( RenBase( Pixfmt( data ) ) )
//    ras = Raster()
//    sl  = Scanline()
//  Raster Operations
//    line, move, add_path
//    clip.line()
//       clip.line_clip_y()
//        line()
//         render_hline()    -- 'INCR[0,1,2,3]'
//          set_curr_cell()
//         set_curr_cell()
//     Output: Cells with X, Cover, and Area
//  Render to Image
//   render_scanlines(ras, sl, ren)
//     rewind_scanline
//       close_polygon()
//       sort_cells() -- 'SORT_CELLS: SORTING'
//     scanline_reset
//     sweep_scanlines()
//       render_scanline - Individual horizontal (y) lines
//         blend_solid_hspan
//         blend_hline
//           blend_hline (pixfmt)

// When difference occur:
//   - Check Input Path (ADD_PATH) in rasterizer
//   - Check Scanlines (SWEEP SCANLINES) in rasterizer
//   - Check Pixels    (BLEND_HLINE)

