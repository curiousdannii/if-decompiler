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

glui32 OP_STKPEEK(glui32 arg0) {
    arg0 = arg0 * 4;
    if (arg0 < 0 || arg0 >= (stackptr - valstackbase))
    {
        fatal_error("Stkpeek outside current stack range.");
    }
    return Stk4(stackptr - (arg0 + 4));
}

void OP_STKSWAP(void) {
    if (stackptr < valstackbase + 8) {
        fatal_error("Stack underflow in stkswap.");
    }
    glui32 val0 = Stk4(stackptr - 4);
    glui32 val1 = Stk4(stackptr - 8);
    StkW4(stackptr - 4, val1);
    StkW4(stackptr - 8, val0);
}

void OP_STKCOPY(glui32 arg0) {
    glsi32 vals0 = (glsi32) arg0;
    if (vals0 < 0) {
        fatal_error("Negative operand in stkcopy.");
    }
    if (vals0 == 0) {
        return;
    }
    if (stackptr < valstackbase + vals0 * 4) {
        fatal_error("Stack underflow in stkcopy.");
    }
    if (stackptr + vals0 * 4 > stacksize) {
        fatal_error("Stack overflow in stkcopy.");
    }
    glui32 addr = stackptr - vals0 * 4;
    for (glui32 ix = 0; ix < vals0; ix++) {
        glui32 value = Stk4(addr + ix * 4);
        StkW4(stackptr + ix * 4, value);
    }
    stackptr += vals0 * 4;
}

void OP_STKROLL(glui32 arg0, glui32 arg1) {
    glsi32 vals0 = (glsi32) arg0;
    glsi32 vals1 = (glsi32) arg1;
    if (vals0 < 0) {
        fatal_error("Negative operand in stkroll.");
    }
    if (stackptr < valstackbase + vals0 * 4) {
        fatal_error("Stack underflow in stkroll.");
    }
    if (vals0 == 0) {
        return;
    }
    /* The following is a bit ugly. We want to do vals1 = vals0-vals1,
        because rolling down is sort of easier than rolling up. But
        we also want to take the result mod vals0. The % operator is
        annoying for negative numbers, so we need to do this in two 
        cases. */
    if (vals1 > 0) {
        vals1 = vals1 % vals0;
        vals1 = (vals0) - vals1;
    }
    else {
        vals1 = (-vals1) % vals0;
    }
    if (vals1 == 0)
    {
        return;
    }
    glui32 addr = stackptr - vals0 * 4;
    for (glui32 ix = 0; ix < vals1; ix++) {
        glui32 value = Stk4(addr + ix * 4);
        StkW4(stackptr + ix * 4, value);
    }
    for (glui32 ix=0; ix < vals0; ix++) {
        glui32 value = Stk4(addr + (vals1 + ix) * 4);
        StkW4(addr + ix * 4, value);
    }
}

glui32 PopStack(void) {
    if (stackptr < valstackbase + 4) {
        fatal_error("Stack underflow in operand.");
    }
    stackptr -= 4;
    return Stk4(stackptr);
}

void PushStack(glui32 storeval) {
    if (stackptr + 4 > stacksize) {
        fatal_error("Stack overflow in store operand.");
    }
    StkW4(stackptr, storeval);
    stackptr += 4;
}

glui32 ReadLocal(glui32 addr) {
    addr += localsbase;
    return Stk4(addr);
}

void StoreLocal(glui32 addr, glui32 value) {
    addr += localsbase;
    StkW4(addr, value);
}

int VM_CALL_FUNCTION(glui32 addr, glui32 count, glui32 storetype, glui32 storeval) {
    glui32 *arglist;
    int is_safe = VM_FUNC_IS_SAFE(addr);
    if (is_safe == 0) {
        arglist = pop_arguments(count, 0);
        push_callstub(storetype, storeval);
        enter_function(addr, count, arglist);
        return 1;
    }
    glui32 result = VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(addr, count);
    store_operand(storetype, storeval, result);
    return 0;
}

void VM_TAILCALL_FUNCTION(glui32 addr, glui32 count) {
    glui32 *arglist;
    int is_safe = VM_FUNC_IS_SAFE(addr);
    if (is_safe == 0) {
        arglist = pop_arguments(count, 0);
        leave_function();
        enter_function(addr, count, arglist);
    }
    glui32 result = VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(addr, count);
    leave_function();
    if (stackptr != 0) {
        pop_callstub(result);
    }
}