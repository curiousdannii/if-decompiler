/*

C output files from glulxtoc
============================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#include "glk.h"

#define ReadLocal(addr) (Stk4(addr + localsbase))

// functions_safe.c
extern int VM_FUNC_IS_SAFE(glui32 addr);
extern int VM_FUNC_IS_SAFE_VARARGS(glui32 addr);
extern glui32 VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(glui32 addr, glui32 count);

// image.c
#define GLULX_IMAGE_LENGTH IMAGE_LENGTH_VALUE
extern char GLULX_IMAGE[];

// runtime.c
extern glui32 OP_DIV(glui32 arg1, glui32 arg2);
extern glui32 OP_MOD(glui32 arg1, glui32 arg2);
extern glui32 OP_SHIFTL(glui32 arg1, glui32 arg2);
extern glui32 OP_USHIFTR(glui32 arg1, glui32 arg2);
extern glui32 OP_SSHIFTR(glui32 arg0, glui32 arg1);
extern int OP_CATCH(glui32 storetype, glui32 storeval, glui32 offset, glui32 next);
extern glui32 OP_SEXS(glui32 arg0);
extern glui32 OP_SEXB(glui32 arg0);
extern glui32 OP_ALOADBIT(glui32 arg0, glui32 arg1);
extern void OP_ASTOREBIT(glui32 arg0, glui32 arg1, glui32 arg2);
extern glui32 OP_STKPEEK(glui32 arg0);
extern void OP_STKSWAP(void);
extern void OP_STKCOPY(glui32 arg0);
extern void OP_STKROLL(glui32 arg0, glui32 arg1);
extern int OP_STREAMX(int mode, glui32 val, glui32 next);
extern glui32 OP_RANDOM(glui32 arg0);
extern void OP_PROTECT(glui32 arg0, glui32 arg1);
extern void OP_MZERO(glui32 arg0, glui32 arg1);
extern void OP_SAVE(glui32 arg0, glui32 next, glui32 storetype, glui32 storeval);
extern int OP_RESTORE(glui32 arg0, glui32 storetype, glui32 storeval);
extern void OP_SAVEUNDO(glui32 next, glui32 storetype, glui32 storeval);
extern int OP_RESTOREUNDO(glui32 storetype, glui32 storeval);
extern int OP_CALLFI(glui32 addr, glui32 arg0, glui32 storetype, glui32 storeval, glui32 next);
extern int OP_CALLFII(glui32 addr, glui32 arg0, glui32 arg1, glui32 storetype, glui32 storeval, glui32 next);
extern int OP_CALLFIII(glui32 addr, glui32 arg0, glui32 arg1, glui32 arg2, glui32 storetype, glui32 storeval, glui32 next);
extern void OP_MCOPY(glui32 arg0, glui32 arg1, glui32 arg2);
extern glsi32 OP_FTONUMZ(glui32 arg0);
extern glsi32 OP_FTONUMN(glui32 arg0);
extern void OP_FMOD(glui32 arg0, glui32 arg1, glui32 *dest0, glui32 *dest1);
extern glui32 OP_CEIL(glui32 arg0);
extern glui32 OP_JFEQ(glui32 arg0, glui32 arg1, glui32 arg2);
extern glui32 PopStack(void);
extern void PushStack(glui32 storeval);
extern int VM_BRANCH(glui32 offset, glui32 next);
extern int VM_CALL_FUNCTION(glui32 addr, glui32 count, glui32 storetype, glui32 storeval, glui32 next);
extern int VM_JUMP_CALL(glui32 pc);
extern void VM_TAILCALL_FUNCTION(glui32 addr, glui32 count);