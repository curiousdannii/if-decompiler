/*

Runtime functions - mostly things that used to be in exec.c
===========================================================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#include "glk.h"
#include "glulxe.h"
#include "glulxtoc.h"

glui32 OP_DIV(glui32 arg1, glui32 arg2) {
    glsi32 dividend = (glsi32) arg1;
    glsi32 divisor = (glsi32) arg2;
    if (divisor == 0) {
        fatal_error("Division by zero.");
    }
    /* Since C doesn't guarantee the results of division of negative
        numbers, we carefully convert everything to positive values
        first. They have to be unsigned values, too, otherwise the
        0x80000000 case goes wonky. */
    glui32 value, val0, val1;
    if (dividend < 0) {
        val0 = (-dividend);
        if (divisor < 0) {
            val1 = (-divisor);
            value = val0 / val1;
        }
        else {
            val1 = divisor;
            value = -(val0 / val1);
        }
    } else {
        val0 = dividend;
        if (divisor < 0) {
            val1 = (-divisor);
            value = -(val0 / val1);
        }
        else {
            val1 = divisor;
            value = val0 / val1;
        }
    }
    return value;
}

glui32 OP_MOD(glui32 arg1, glui32 arg2) {
    glsi32 dividend = (glsi32) arg1;
    glsi32 divisor = (glsi32) arg2;
    glui32 value, val0, val1;
    if (divisor == 0) {
        fatal_error("Division by zero doing remainder.");
    }
    if (divisor < 0) {
        val1 = -divisor;
    }
    else {
        val1 = divisor;
    }
    if (dividend < 0) {
        val0 = (-dividend);
        value = -(val0 % val1);
    }
    else {
        val0 = dividend;
        value = val0 % val1;
    }
    return value;
}