#pragma once

//	supported font sizes:
//	8x8
//	16x16
//	32x32
//	64x64

static struct font {
	u16 size;
	void *table[97];
} font;

void font_init();

//	default font is used for rendering text before custom font is loaded
static struct default_font {
	u8 table[97][8];
	u16 size;
} default_font;

#include "font.c"