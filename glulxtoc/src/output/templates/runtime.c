/*

Runtime functions - mostly things that used to be in exec.c
===========================================================

Copyright (c) 2021 Dannii Willis
Copyright (c) 1999-2016, Andrew Plotkin
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#include "glk.h"
#include "glulxe.h"
#include "glulxtoc.h"

glui32 OP_DIV(glui32 arg0, glui32 arg1) {
    glsi32 dividend = (glsi32) arg0;
    glsi32 divisor = (glsi32) arg1;
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

glui32 OP_MOD(glui32 arg0, glui32 arg1) {
    glsi32 dividend = (glsi32) arg0;
    glsi32 divisor = (glsi32) arg1;
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

glui32 OP_SHIFTL(glui32 arg0, glui32 arg1) {
    glsi32 vals0 = (glsi32) arg1;
    if (vals0 < 0 || vals0 >= 32) {
        return 0;
    }
    return (glui32) arg0 << (glui32) vals0;
}

glui32 OP_USHIFTR(glui32 arg0, glui32 arg1) {
    glsi32 vals0 = (glsi32) arg1;
    if (vals0 < 0 || vals0 >= 32) {
        return 0;
    }
    return (glui32) arg0 >> (glui32) vals0;
}

glui32 OP_SSHIFTR(glui32 arg0, glui32 arg1) {
    glsi32 vals0 = (glsi32) arg1;
    if (vals0 < 0 || vals0 >= 32) {
        if (arg0 & 0x80000000)
        {
            return 0xFFFFFFFF;
        } else {
            return 0;
        }
    }
    /* This is somewhat foolhardy -- C doesn't guarantee that
        right-shifting a signed value replicates the sign bit.
        We'll assume it for now. */
    return (glsi32) arg0 >> (glsi32) vals0;
}

glui32 OP_SEXS(glui32 arg0) {
    if (arg0 & 0x8000)
    {
        return arg0 |= 0xFFFF0000;
    }
    return arg0 &= 0x0000FFFF;
}

glui32 OP_SEXB(glui32 arg0) {
    if (arg0 & 0x80)
    {
        return arg0 |= 0xFFFFFF00;
    }
    return arg0 &= 0x000000FF;
}

glui32 OP_ALOADBIT(glui32 arg0, glui32 arg1) {
    glsi32 vals0 = (glsi32) arg1;
    glui32 val1 = (vals0 & 7);
    if (vals0 >= 0) {
        arg0 += (vals0 >> 3);
    } else {
        arg0 -= (1 + ((-1 - vals0) >> 3));
    }
    if (Mem1(arg0) & (1 << val1))
    {
        return 1;
    } else { 
        return 0;
    }
}

void OP_ASTOREBIT(glui32 arg0, glui32 arg1, glui32 arg2) {
    glsi32 vals0 = (glsi32) arg1;
    glui32 val1 = (vals0 & 7);
    if (vals0 >= 0) {
        arg0 += (vals0 >> 3);
    } else {
        arg0 -= (1 + ((-1 - vals0) >> 3));
    }
    glui32 val0 = Mem1(arg0);
    if (arg2) {
        val0 |= (1 << val1);
    } else {
        val0 &= ~((glui32)(1 << val1));
    }
    MemW1(arg0, val0);
}

glui32 PopStack(void) {
    if (stackptr < valstackbase+4) {
        fatal_error("Stack underflow in operand.");
    }
    stackptr -= 4;
    return Stk4(stackptr);
}

void PushStack(glui32 storeval) {
    if (stackptr+4 > stacksize) {
        fatal_error("Stack overflow in store operand.");
    }
    StkW4(stackptr, storeval);
    stackptr += 4;
}