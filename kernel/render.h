
#pragma once

#define FONT_PLACE_SUB 31

static struct output {
	u64 line;
	u64 column;

	u64 w;
	u64 h;

	u8 space_between_lines;
	
	volatile u32* ptr;

	u32 color;
} output;




void printc(const char c) {
	if (((c >= ' ') && (c <= '~')) || ((c == '\t') || (c == '\n'))) {
		switch (c) {
			case '\n': {
				output.column = 0;
				output.line++;
				break;
			}
			case ' ': {
				output.column++;
				if (output.column >= output.w) {
					output.column = 0;
					output.line++;
				}
				break;
			}
			case '\t': {
				//	:(
				break;
			}
			default: {
				u8 actual = c * (c > FONT_PLACE_SUB) - (FONT_PLACE_SUB * (c > FONT_PLACE_SUB));
				volatile u32 *ptr = output.ptr + ((output.line * output.w * (font.size + output.space_between_lines))) + (output.column * font.size);
                u8 *fnt;
                for (u16 i = 0; i < font.size; i++) {
                    fnt = font.table[actual];
                    for (u16 ii = 0; ii < font.size; ii++) {
                        *(ptr + (i * output.w) + (font.size - ii)) = output.color * ((fnt[i] >> ii) & 1);
                    }
                }
            }
        }
    }
}

void print(const char* s) {
	for (;*s != '\0'; s++) {
		printc(*s);
	}
}
                    