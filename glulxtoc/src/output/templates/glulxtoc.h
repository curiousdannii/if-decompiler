/*

C output files from glulxtoc
============================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#include "glk.h"

// functions_safe.c
extern glui32 VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(glui32 addr, glui32 count);
extern int VM_FUNC_ARGUMENTS_COUNT(glui32 addr);

// image.c
#define GLULX_IMAGE_LENGTH IMAGE_LENGTH_VALUE
extern char *GLULX_IMAGE;

// runtime.c
extern glui32 OP_DIV(glui32 arg1, glui32 arg2);
extern glui32 OP_MOD(glui32 arg1, glui32 arg2);
extern glui32 OP_SHIFTL(glui32 arg1, glui32 arg2);
extern glui32 OP_USHIFTR(glui32 arg1, glui32 arg2);
extern glui32 OP_SSHIFTR(glui32 arg0, glui32 arg1);
extern glui32 OP_SEXS(glui32 arg0);
extern glui32 OP_SEXB(glui32 arg0);
extern glui32 OP_ALOADBIT(glui32 arg0, glui32 arg1);
extern void OP_ASTOREBIT(glui32 arg0, glui32 arg1, glui32 arg2);
extern glui32 OP_STKPEEK(glui32 arg0);
extern void OP_STKSWAP(void);
extern void OP_STKCOPY(glui32 arg0);
extern void OP_STKROLL(glui32 arg0, glui32 arg1);
extern glui32 PopStack(void);
extern void PushStack(glui32 storeval);
extern glui32 ReadLocal(glui32 addr);
extern void StoreLocal(glui32 addr, glui32 value);