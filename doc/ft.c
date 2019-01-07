
#include <ft2build.h>
#include <freetype/ftglyph.h>
#include FT_FREETYPE_H

int
main() {

    FT_Library  library;
    FT_Face     face;      /* handle to face object */
    
    
    FT_Init_FreeType(&library);

    int error = FT_New_Face( library,
                             "/System/Library/Fonts/Helvetica.ttc",
                             0,
                             &face );

    error = FT_Set_Char_Size(face,    /* handle to face object           */
                             0,       /* char_width in 1/64th of points  */
                             13*64,   /* char_height in 1/64th of points */
                             72,     /* horizontal device resolution    */
                             72 );   /* vertical device resolution      */
    unsigned int glyph_index = FT_Get_Char_Index( face, 'H' );

    error = FT_Load_Glyph(face,          /* handle to face object */
                          glyph_index,   /* glyph index           */
                          FT_LOAD_DEFAULT );  /* load flags, see below */
    if(error) {
        printf("error loading glyph\n");
        exit(1);
    }
    FT_Glyph glyph;
    FT_Get_Glyph(face->glyph, &glyph);

    printf("library: %p\n", glyph->library);
    printf("clazz: %p\n", glyph->clazz);
    printf("format: %d bitmap: %d\n", glyph->format, ft_glyph_format_bitmap);
    printf("format: %d outline: %d\n", glyph->format, ft_glyph_format_outline);
    printf("advance_x: %ld\n", glyph->advance.x / 65536);
    printf("advance_y: %ld\n", glyph->advance.y / 65536);

    error = FT_Glyph_To_Bitmap(&glyph, FT_RENDER_MODE_NORMAL, 0, 1);
    if(error) {
        printf("error converting glyph to bitmap\n");
        exit(1);
    }
    FT_BitmapGlyph  bit = (FT_BitmapGlyph)glyph;
    printf("left: %d\n", bit->left);
    printf("top: %d\n", bit->top);
    printf("rows,width,pitch: %d %d %d\n", bit->bitmap.rows, bit->bitmap.width, bit->bitmap.pitch);
    
    return 1;
}
