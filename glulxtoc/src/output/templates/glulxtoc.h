/*

C output files from glulxtoc
============================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

// functions_safe.c
extern int VM_FUNC_IS_SAFE(glui32 addr);

// functions_unsafe.c
extern void VM_UNSAFE_FUNCS(void);

// image.c
#define GLULX_IMAGE_LENGTH IMAGE_LENGTH_VALUE
extern unsigned char *GLULX_IMAGE;

#define TODORAM 0
#define TODOSTACK 0