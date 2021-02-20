/*

C output files from glulxtoc
============================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#include "glk.h"

// functions_safe.c
extern int VM_FUNC_IS_SAFE(glui32 addr);

// functions_unsafe.c
extern void VM_UNSAFE_FUNCS(void);

// image.c
#define GLULX_IMAGE_LENGTH IMAGE_LENGTH_VALUE
extern unsigned char *GLULX_IMAGE;

// runtime.c
extern glui32 OP_DIV(glui32 arg1, glui32 arg2);
extern glui32 OP_MOD(glui32 arg1, glui32 arg2);
extern glui32 PopStack(void);
extern void PushStack(glui32 storeval);