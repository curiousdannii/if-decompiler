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
#include <math.h>

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

int OP_CATCH(glui32 storetype, glui32 storeval, glui32 offset, glui32 next) {
    pc = next;
    push_callstub(storetype, storeval);
    store_operand(storetype, storeval, stackptr);
    return VM_BRANCH(offset, next);
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
    glsi32 vals0 = arg0 * 4;
    if (vals0 < 0 || vals0 >= (stackptr - valstackbase))
    {
        fatal_error("Stkpeek outside current stack range.");
    }
    return Stk4(stackptr - (vals0 + 4));
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

void OP_STREAMX_SAFE(int mode, glui32 val) {
    // Shortcut for safe streaming
    glui32 iosys_mode, iosys_rock;
    stream_get_iosys(&iosys_mode, &iosys_rock);
    if (iosys_mode != 1 /* iosys_Filter */) {
        switch (mode) {
            case STREAM_CHAR: (*stream_char_handler)(val & 0xFF); return;
            case STREAM_NUM: stream_num((glsi32) val, 0, 0); return;
            case STREAM_UNICHAR: (*stream_unichar_handler)(val); return;
        }
    }

    // Save the current stack
    glui32 oldframeptr = frameptr;
    glui32 oldlocalsbase = localsbase;
    unsigned char *oldstack = stack;
    glui32 oldstackptr = stackptr;
    glui32 oldstacksize = stacksize;
    glui32 oldvalstackbase = valstackbase;

    // Pretend we're calling execute_loop for the very first time
    stack += stackptr;
    stacksize -= stackptr;
    frameptr = 0;
    localsbase = 0;
    stackptr = 0;
    valstackbase = 0;

    // Fake call the printing handler
    pc = STREAM_HANDLER_FAKE_FUNCTION;
    PushStack(val);
    PushStack(mode);
    execute_loop();

    // And restore the original stack
    frameptr = oldframeptr;
    localsbase = oldlocalsbase;
    stack = oldstack;
    stackptr = oldstackptr;
    stacksize = oldstacksize;
    valstackbase = oldvalstackbase;
}

int OP_STREAMX_UNSAFE(int mode, glui32 val, glui32 next) {
    pc = next;
    switch (mode) {
        case STREAM_CHAR: (*stream_char_handler)(val & 0xFF); break;
        case STREAM_NUM: stream_num((glsi32) val, 0, 0); break;
        case STREAM_STRING: stream_string(val, 0, 0); break;
        case STREAM_UNICHAR: (*stream_unichar_handler)(val); break;
    }
    return pc != next;
}

glui32 OP_RANDOM(glui32 arg0) {
    glsi32 vals0 = (glsi32) arg0;
    if (vals0 == 0) {
        return glulx_random();
    } else if (vals0 >= 1) {
        return glulx_random() % (glui32) (vals0);
    } else  {
        return -(glulx_random() % (glui32) (-vals0));
    }
}

void OP_PROTECT(glui32 arg0, glui32 arg1) {
    glui32 val1 = arg0 + arg1;
    if (arg0 == val1) {
        arg0 = 0;
        val1 = 0;
    }
    protectstart = arg0;
    protectend = val1;
}

void OP_SAVE(glui32 arg0, glui32 next, glui32 storetype, glui32 storeval) {
    pc = next;
    push_callstub(storetype, storeval);
    pop_callstub(perform_save(find_stream_by_id(arg0)));
}

int OP_RESTORE(glui32 arg0, glui32 storetype, glui32 storeval) {
    glui32 value = perform_restore(find_stream_by_id(arg0), FALSE);
    if (value == 0) {
        /* We've succeeded, and the stack now contains the callstub
            saved during saveundo. Ignore this opcode's operand. */
        value = -1;
        pop_callstub(value);
        return 1;
    }
    else {
        /* We've failed, so we must store the failure in this opcode's
            operand. */
        store_operand(storetype, storeval, value);
        return 0;
    }
}

void OP_SAVEUNDO(glui32 next, glui32 storetype, glui32 storeval) {
    pc = next;
    push_callstub(storetype, storeval);
    pop_callstub(perform_saveundo());
}

int OP_RESTOREUNDO(glui32 storetype, glui32 storeval) {
    glui32 value = perform_restoreundo();
    if (value == 0) {
        /* We've succeeded, and the stack now contains the callstub
            saved during saveundo. Ignore this opcode's operand. */
        value = -1;
        pop_callstub(value);
        return 1;
    }
    else {
        /* We've failed, so we must store the failure in this opcode's
            operand. */
        store_operand(storetype, storeval, value);
        return 0;
    }
}

int OP_CALLFI(glui32 addr, glui32 arg0, glui32 storetype, glui32 storeval, glui32 next) {
    PushStack(arg0);
    return VM_CALL_FUNCTION(addr, 1, storetype, storeval, next);
}

int OP_CALLFII(glui32 addr, glui32 arg0, glui32 arg1, glui32 storetype, glui32 storeval, glui32 next) {
    PushStack(arg1);
    PushStack(arg0);
    return VM_CALL_FUNCTION(addr, 2, storetype, storeval, next);
}

int OP_CALLFIII(glui32 addr, glui32 arg0, glui32 arg1, glui32 arg2, glui32 storetype, glui32 storeval, glui32 next) {
    PushStack(arg2);
    PushStack(arg1);
    PushStack(arg0);
    return VM_CALL_FUNCTION(addr, 3, storetype, storeval, next);
}

void OP_MZERO(glui32 arg0, glui32 arg1) {
    glui32 lx;
    for (lx=0; lx < arg0; lx++, arg1++) {
        MemW1(arg1, 0);
    }
}

void OP_MCOPY(glui32 arg0, glui32 arg1, glui32 arg2) {
    glui32 lx;
    if (arg2 < arg1) {
        for (lx = 0; lx < arg0; lx++, arg1++, arg2++) {
            MemW1(arg2, Mem1(arg1));
        }
    }
    else {
        arg1 += (arg0 - 1);
        arg2 += (arg0 - 1);
        for (lx = 0; lx < arg0; lx++, arg1--, arg2--) {
            MemW1(arg2, Mem1(arg1));
        }
    }
}

glsi32 OP_FTONUMZ(glui32 arg0) {
    gfloat32 valf = decode_float(arg0);
    if (!signbit(valf)) {
        if (isnan(valf) || isinf(valf) || (valf > 2147483647.0)) {
            return 0x7FFFFFFF;
        } else {
            return (glsi32) (truncf(valf));
        }
    } else {
        if (isnan(valf) || isinf(valf) || (valf < -2147483647.0)) {
            return 0x80000000;
        } else {
            return (glsi32) (truncf(valf));
        }
    }
}

glsi32 OP_FTONUMN(glui32 arg0) {
    gfloat32 valf = decode_float(arg0);
    if (!signbit(valf)) {
        if (isnan(valf) || isinf(valf) || (valf > 2147483647.0)) {
            return 0x7FFFFFFF;
        } else {
            return (glsi32) (roundf(valf));
        }
    } else {
        if (isnan(valf) || isinf(valf) || (valf < -2147483647.0)) {
            return 0x80000000;
        } else {
            return (glsi32) (roundf(valf));
        }
    }
}

void OP_FMOD(glui32 arg0, glui32 arg1, glui32 *dest0, glui32 *dest1) {
    gfloat32 valf1 = decode_float(arg0);
    gfloat32 valf2 = decode_float(arg1);
    gfloat32 valf = fmodf(valf1, valf2);
    glui32 val0 = encode_float(valf);
    glui32 val1 = encode_float((valf1 - valf) / valf2);
    if (val1 == 0x0 || val1 == 0x80000000) {
        /* When the quotient is zero, the sign has been lost in the
            shuffle. We'll set that by hand, based on the original
            arguments. */
        val1 = (arg0 ^ arg1) & 0x80000000;
    }
    *dest0 = val0;
    *dest1 = val1;
}

glui32 OP_CEIL(glui32 arg0) {
    gfloat32 valf = decode_float(arg0);
    glui32 value = encode_float(ceilf(valf));
    if (value == 0x0 || value == 0x80000000) {
        /* When the result is zero, the sign may have been lost in the
            shuffle. (This is a bug in some C libraries.) We'll set the
            sign by hand, based on the original argument. */
        value = arg0 & 0x80000000;
    }
    return value;
}

glui32 OP_JFEQ(glui32 arg0, glui32 arg1, glui32 arg2) {
    if ((arg2 & 0x7F800000) == 0x7F800000 && (arg2 & 0x007FFFFF) != 0) {
        /* The delta is NaN, which can never match. */
        return 0;
    } else if ((arg0 == 0x7F800000 || arg0 == 0xFF800000)
        && (arg1 == 0x7F800000 || arg1 == 0xFF800000)) {
        /* Both are infinite. Opposite infinities are never equal,
            even if the difference is infinite, so this is easy. */
        return (arg0 == arg1);
    } else {
        gfloat32 valf1 = decode_float(arg1) - decode_float(arg0);
        gfloat32 valf2 = fabs(decode_float(arg2));
        return (valf1 <= valf2 && valf1 >= -valf2);
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

int VM_BRANCH(glui32 offset, glui32 next) {
    if (offset == 0 || offset == 1)
    {
        leave_function();
        if (stackptr == 0)
        {
            return 1;
        }
        pop_callstub(offset);
    } else {
        pc = next + offset - 2;
    }
    return 0;
}

int VM_CALL_FUNCTION(glui32 addr, glui32 count, glui32 storetype, glui32 storeval, glui32 next) {
    if (VM_FUNC_IS_SAFE(addr)) {
        glui32 result, oldsp, oldvsb, res;
        result = CALL_FUNC(VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(addr, count), count);
        store_operand(storetype, storeval, result);
        return 0;
    }
    else {
        glui32 *arglist;
        arglist = pop_arguments(count, 0);
        pc = next;
        push_callstub(storetype, storeval);
        enter_function(addr, count, arglist);
        return 1;
    }
}

// Try to recover from an invalid unsafe PC by seeing if we can call a safe function
int VM_JUMP_CALL(glui32 pc) {
    // The PC we've been given is the beginning of a function's code
    // The header is variable length though, so call a helper function to find the function address
    pc = VM_FUNC_SUBTRACT_HEADER(pc);
    if (VM_FUNC_IS_SAFE(pc)) {
        glui32 count;
        // Retrieve the stack count for varargs functions
        if (VM_FUNC_IS_SAFE_VARARGS(pc)) {
            count = PopStack();
        }
        // Or push the locals in reverse order for regular functions
        else {
            glui32 locals = valstackbase - localsbase;
            count = locals / 4;
            while (locals > 0) {
                locals -= 4;
                PushStack(ReadLocal(locals));
            }
        }
        VM_TAILCALL_FUNCTION(pc, count);
        return 1;
    }
    return 0;
}

void VM_TAILCALL_FUNCTION(glui32 addr, glui32 count) {
    if (VM_FUNC_IS_SAFE(addr)) {
        glui32 result, oldsp, oldvsb, res;
        result = CALL_FUNC(VM_CALL_SAFE_FUNCTION_WITH_STACK_ARGS(addr, count), count);
        leave_function();
        if (stackptr != 0) {
            pop_callstub(result);
        }
    }
    else {
        glui32 *arglist;
        arglist = pop_arguments(count, 0);
        leave_function();
        enter_function(addr, count, arglist);
    }
}