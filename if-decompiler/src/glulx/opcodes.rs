/*

Glulx Opcodes
=============

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#![allow(dead_code)]

use super::*;

pub const OP_NOP: u32 = 0x00;
pub const OP_ADD: u32 = 0x10;
pub const OP_SUB: u32 = 0x11;
pub const OP_MUL: u32 = 0x12;
pub const OP_DIV: u32 = 0x13;
pub const OP_MOD: u32 = 0x14;
pub const OP_NEG: u32 = 0x15;
pub const OP_BITAND: u32 = 0x18;
pub const OP_BITOR: u32 = 0x19;
pub const OP_BITXOR: u32 = 0x1A;
pub const OP_BITNOT: u32 = 0x1B;
pub const OP_SHIFTL: u32 = 0x1C;
pub const OP_SSHIFTR: u32 = 0x1D;
pub const OP_USHIFTR: u32 = 0x1E;
pub const OP_JUMP: u32 = 0x20;
pub const OP_JZ: u32 = 0x22;
pub const OP_JNZ: u32 = 0x23;
pub const OP_JEQ: u32 = 0x24;
pub const OP_JNE: u32 = 0x25;
pub const OP_JLT: u32 = 0x26;
pub const OP_JGE: u32 = 0x27;
pub const OP_JGT: u32 = 0x28;
pub const OP_JLE: u32 = 0x29;
pub const OP_JLTU: u32 = 0x2A;
pub const OP_JGEU: u32 = 0x2B;
pub const OP_JGTU: u32 = 0x2C;
pub const OP_JLEU: u32 = 0x2D;
pub const OP_CALL: u32 = 0x30;
pub const OP_RETURN: u32 = 0x31;
pub const OP_CATCH: u32 = 0x32;
pub const OP_THROW: u32 = 0x33;
pub const OP_TAILCALL: u32 = 0x34;
pub const OP_COPY: u32 = 0x40;
pub const OP_COPYS: u32 = 0x41;
pub const OP_COPYB: u32 = 0x42;
pub const OP_SEXS: u32 = 0x44;
pub const OP_SEXB: u32 = 0x45;
pub const OP_ALOAD: u32 = 0x48;
pub const OP_ALOADS: u32 = 0x49;
pub const OP_ALOADB: u32 = 0x4A;
pub const OP_ALOADBIT: u32 = 0x4B;
pub const OP_ASTORE: u32 = 0x4C;
pub const OP_ASTORES: u32 = 0x4D;
pub const OP_ASTOREB: u32 = 0x4E;
pub const OP_ASTOREBIT: u32 = 0x4F;
pub const OP_STKCOUNT: u32 = 0x50;
pub const OP_STKPEEK: u32 = 0x51;
pub const OP_STKSWAP: u32 = 0x52;
pub const OP_STKROLL: u32 = 0x53;
pub const OP_STKCOPY: u32 = 0x54;
pub const OP_STREAMCHAR: u32 = 0x70;
pub const OP_STREAMNUM: u32 = 0x71;
pub const OP_STREAMSTR: u32 = 0x72;
pub const OP_STREAMUNICHAR: u32 = 0x73;
pub const OP_GETSTALT: u32 = 0x100;
pub const OP_DEBUGTRAP: u32 = 0x101;
pub const OP_GETMEMSIZE: u32 = 0x102;
pub const OP_SETMEMSIZE: u32 = 0x103;
pub const OP_JUMPABS: u32 = 0x104;
pub const OP_RANDOM: u32 = 0x110;
pub const OP_SETRANDOM: u32 = 0x111;
pub const OP_QUIT: u32 = 0x120;
pub const OP_VERIFY: u32 = 0x121;
pub const OP_RESTART: u32 = 0x122;
pub const OP_SAVE: u32 = 0x123;
pub const OP_RESTORE: u32 = 0x124;
pub const OP_SAVEUNDO: u32 = 0x125;
pub const OP_RESTOREUNDO: u32 = 0x126;
pub const OP_PROTECT: u32 = 0x127;
pub const OP_GLK: u32 = 0x130;
pub const OP_GETSTRINGTBL: u32 = 0x140;
pub const OP_SETSTRINGTBL: u32 = 0x141;
pub const OP_GETIOSYS: u32 = 0x148;
pub const OP_SETIOSYS: u32 = 0x149;
pub const OP_LINEARSEARCH: u32 = 0x150;
pub const OP_BINARYSEARCH: u32 = 0x151;
pub const OP_LINKEDSEARCH: u32 = 0x152;
pub const OP_CALLF: u32 = 0x160;
pub const OP_CALLFI: u32 = 0x161;
pub const OP_CALLFII: u32 = 0x162;
pub const OP_CALLFIII: u32 = 0x163;
pub const OP_MZERO: u32 = 0x170;
pub const OP_MCOPY: u32 = 0x171;
pub const OP_MALLOC: u32 = 0x178;
pub const OP_MFREE: u32 = 0x179;
pub const OP_ACCELFUNC: u32 = 0x180;
pub const OP_ACCELPARAM: u32 = 0x181;
pub const OP_NUMTOF: u32 = 0x190;
pub const OP_FTONUMZ: u32 = 0x191;
pub const OP_FTONUMN: u32 = 0x192;
pub const OP_CEIL: u32 = 0x198;
pub const OP_FLOOR: u32 = 0x199;
pub const OP_FADD: u32 = 0x1A0;
pub const OP_FSUB: u32 = 0x1A1;
pub const OP_FMUL: u32 = 0x1A2;
pub const OP_FDIV: u32 = 0x1A3;
pub const OP_FMOD: u32 = 0x1A4;
pub const OP_SQRT: u32 = 0x1A8;
pub const OP_EXP: u32 = 0x1A9;
pub const OP_LOG: u32 = 0x1AA;
pub const OP_POW: u32 = 0x1AB;
pub const OP_SIN: u32 = 0x1B0;
pub const OP_COS: u32 = 0x1B1;
pub const OP_TAN: u32 = 0x1B2;
pub const OP_ASIN: u32 = 0x1B3;
pub const OP_ACOS: u32 = 0x1B4;
pub const OP_ATAN: u32 = 0x1B5;
pub const OP_ATAN2: u32 = 0x1B6;
pub const OP_JFEQ: u32 = 0x1C0;
pub const OP_JFNE: u32 = 0x1C1;
pub const OP_JFLT: u32 = 0x1C2;
pub const OP_JFLE: u32 = 0x1C3;
pub const OP_JFGT: u32 = 0x1C4;
pub const OP_JFGE: u32 = 0x1C5;
pub const OP_JISNAN: u32 = 0x1C8;
pub const OP_JISINF: u32 = 0x1C9;

// Return the number of operands an opcode has
// Also checks for unknown opcodes
pub fn operands_count(opcode: u32, addr: u32) -> usize {
    match opcode {
        OP_NOP | OP_STKSWAP | OP_QUIT | OP_RESTART => 0,
        OP_JUMP | OP_RETURN | OP_STKCOUNT | OP_STKCOPY
            | OP_STREAMCHAR ..= OP_STREAMUNICHAR | OP_DEBUGTRAP | OP_GETMEMSIZE
            | OP_JUMPABS | OP_SETRANDOM | OP_VERIFY | OP_SAVEUNDO | OP_RESTOREUNDO
            | OP_GETSTRINGTBL | OP_SETSTRINGTBL | OP_MFREE => 1,
        OP_NEG | OP_BITNOT | OP_JZ | OP_JNZ | OP_CATCH ..= OP_TAILCALL
            | OP_COPY ..= OP_SEXB | OP_STKPEEK | OP_STKROLL | OP_CALLF
            | OP_SETMEMSIZE | OP_RANDOM | OP_SAVE | OP_RESTORE | OP_PROTECT
            | OP_GETIOSYS | OP_SETIOSYS | OP_MZERO | OP_MALLOC | OP_ACCELFUNC
            | OP_ACCELPARAM | OP_NUMTOF ..= OP_FLOOR | OP_SQRT ..= OP_LOG
            | OP_SIN ..= OP_ATAN | OP_JISNAN | OP_JISINF => 2,
        OP_ADD ..= OP_MOD | OP_BITAND ..= OP_BITXOR  | OP_SHIFTL ..= OP_USHIFTR
            | OP_JEQ ..= OP_CALL | OP_ALOAD ..= OP_ASTOREBIT | OP_GETSTALT
            | OP_GLK | OP_CALLFI | OP_MCOPY | OP_FADD ..= OP_FDIV | OP_POW
            | OP_ATAN2 | OP_JFLT ..= OP_JFGE => 3,
        OP_CALLFII | OP_FMOD | OP_JFEQ | OP_JFNE => 4,
        OP_CALLFIII => 5,
        OP_LINKEDSEARCH => 7,
        OP_LINEARSEARCH | OP_BINARYSEARCH => 8,
        _ => panic!("Unknown opcode {} at address {}", opcode, addr),
    }
}

// Opcode safety codes
pub fn opcode_safety(opcode: u32, operands: &Vec<Operand>) -> FunctionSafety {
    match opcode {
        OP_CATCH | OP_THROW | OP_RESTORE | OP_RESTOREUNDO | OP_GLK => FunctionSafety::SafetyTBD,
        OP_QUIT | OP_RESTART => FunctionSafety::Unsafe,
        // Calls to non-constants are unsafe
        OP_CALL | OP_TAILCALL | OP_CALLF ..= OP_CALLFIII => match operands[0] {
            Operand::Constant(_) => FunctionSafety::SafetyTBD,
            _ => FunctionSafety::Unsafe,
        }
        // Branches to non-constants are unsafe
        OP_JUMP ..= OP_JLEU | OP_JUMPABS | OP_JFEQ ..= OP_JISINF => match operands.last().unwrap() {
            Operand::Constant(_) => FunctionSafety::Safe,
            _ => FunctionSafety::Unsafe,
        }
        _ => FunctionSafety::Safe,
    }
}