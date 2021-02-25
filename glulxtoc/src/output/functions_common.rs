/*

Output common functions
=======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/


use if_decompiler::*;
use glulx::*;
use glulx::opcodes;

use super::*;

impl GlulxOutput {

    // Output an instruction body
    pub fn output_common_instruction(&self, instruction: &Instruction, args: Vec<String>) -> String {
        let opcode = instruction.opcode;
        let null = String::from("NULL");
        let op_a = args.get(0).unwrap_or(&null);
        let op_b = args.get(1).unwrap_or(&null);
        use opcodes::*;
        match opcode {
            // Following the order of glulxe's exec.c, not strict numerical order
            OP_NOP => String::new(),
            OP_ADD => self.args_join(args, " + "),
            OP_SUB => self.args_join(args, " - "),
            OP_MUL => self.args_join(args, " * "),
            OP_DIV => self.runtime("OP_DIV", args),
            OP_MOD => self.runtime("OP_MOD", args),
            OP_NEG => format!("-((glsi32) {})", op_a),
            OP_BITAND => self.args_join(args, " & "),
            OP_BITOR => self.args_join(args, " | "),
            OP_BITXOR => self.args_join(args, " ^ "),
            OP_BITNOT => format!("~{}", op_a),
            OP_SHIFTL => self.runtime("OP_SHIFTL", args),
            OP_USHIFTR => self.runtime("OP_USHIFTR", args),
            OP_SSHIFTR => self.runtime("OP_SSHIFTR", args),
            // OP_JUMP
            OP_JZ => format!("{} == 0", op_a),
            OP_JNZ => format!("{} != 0", op_a),
            OP_JEQ => format!("{} == {}", op_a, op_b),
            OP_JNE => format!("{} != {}", op_a, op_b),
            OP_JLT => format!("(glsi32) {} < (glsi32) {}", op_a, op_b),
            OP_JGT => format!("(glsi32) {} > (glsi32) {}", op_a, op_b),
            OP_JLE => format!("(glsi32) {} <= (glsi32) {}", op_a, op_b),
            OP_JGE => format!("(glsi32) {} >= (glsi32) {}", op_a, op_b),
            OP_JLTU => format!("{} < {}", op_a, op_b),
            OP_JGTU => format!("{} > {}", op_a, op_b),
            OP_JLEU => format!("{} <= {}", op_a, op_b),
            OP_JGEU => format!("{} >= {}", op_a, op_b),
            // OP_CALL
            // OP_RETURN
            // OP_TAILCALL
            // OP_CATCH
            // OP_THROW
            OP_COPY | OP_COPYS | OP_COPYB => op_a.clone(),
            OP_SEXS => self.runtime("OP_SEXS", args),
            OP_SEXB => self.runtime("OP_SEXB", args),
            OP_ALOAD => format!("Mem4({} + 4 * {})", op_a, op_b),
            OP_ALOADS => format!("Mem2({} + 2 * {})", op_a, op_b),
            OP_ALOADB => format!("Mem1({} + {})", op_a, op_b),
            OP_ALOADBIT => self.runtime("OP_ALOADBIT", args),
            OP_ASTORE => format!("MemW4({} + 4 * {}, {})", op_a, op_b, args[2]),
            OP_ASTORES => format!("MemW2({} + 2 * {}, {})", op_a, op_b, args[2]),
            OP_ASTOREB => format!("MemW1({} + {}, {})", op_a, op_b, args[2]),
            OP_ASTOREBIT => self.runtime("OP_ASTOREBIT", args),
            OP_STKCOUNT => String::from("(stackptr - valstackbase) / 4"),
            OP_STKPEEK => self.runtime("OP_STKPEEK", args),
            OP_STKSWAP => self.runtime("OP_STKSWAP", args),
            OP_STKCOPY => self.runtime("OP_STKCOPY", args),
            OP_STKROLL => self.runtime("OP_STKROLL", args),
            OP_STREAMCHAR => format!("(*stream_char_handler)({} & 0xFF)", op_a),
            OP_STREAMNUM => format!("stream_num((glsi32) {}, FALSE, 0)", op_a),
            OP_STREAMSTR => format!("stream_string({}, 0, 0)", op_a),
            OP_STREAMUNICHAR => format!("(*stream_unichar_handler)({})", op_a),
            OP_GESTALT => self.runtime("do_gestalt", args),
            OP_DEBUGTRAP => format!("fatal_error_i(\"user debugtrap encountered.\", {})", op_a),
            // OP_JUMPABS
            // OP_CALLF ..= OP_CALLFIII
            OP_GETMEMSIZE => String::from("endmem"),
            OP_SETMEMSIZE => format!("change_memsize({}, 0)", op_a),
            OP_GETSTRINGTBL => String::from("stream_get_table()"),
            OP_SETSTRINGTBL => self.runtime("stream_set_table", args),
            // OP_GETIOSYS
            OP_SETIOSYS => self.runtime("stream_set_iosys", args),
            OP_GLK => format!("(temp0 = {}, temp1 = {}, perform_glk(temp0, temp1, pop_arguments(temp1, 0)))", op_a, op_b),
            OP_RANDOM => self.runtime("OP_RANDOM", args),
            OP_SETRANDOM => self.runtime("glulx_setrandom", args),
            OP_PROTECT => self.runtime("OP_PROTECT", args),
            // OP_SAVE
            // OP_RESTORE
            // OP_SAVEUNDO
            // OP_RESTOREUNDO
            // OP_QUIT
            OP_LINEARSEARCH => self.runtime("linear_search", args),
            OP_BINARYSEARCH => self.runtime("binary_search", args),
            OP_LINKEDSEARCH => self.runtime("linked_search", args),
            OP_MZERO => self.runtime("OP_MZERO", args),
            OP_MCOPY => self.runtime("OP_MCOPY", args),
            OP_MALLOC => self.runtime("heap_alloc", args),
            OP_MFREE => self.runtime("heap_free", args),
            OP_ACCELFUNC => String::from(""),
            OP_ACCELPARAM => String::from(""),
            OP_NUMTOF => format!("encode_float((gfloat32) ((glsi32) {}))", op_a),
            OP_FTONUMZ => self.runtime("OP_FTONUMZ", args),
            OP_FTONUMN => self.runtime("OP_FTONUMN", args),
            OP_FADD => format!("encode_float(decode_float({}) + decode_float({}))", op_a, op_b),
            OP_FSUB => format!("encode_float(decode_float({}) - decode_float({}))", op_a, op_b),
            OP_FMUL => format!("encode_float(decode_float({}) * decode_float({}))", op_a, op_b),
            OP_FDIV => format!("encode_float(decode_float({}) / decode_float({}))", op_a, op_b),
            // OP_FMOD
            OP_FLOOR => runtime_float("floorf", op_a),
            OP_CEIL => self.runtime("OP_CEIL", args),
            OP_SQRT => runtime_float("sqrtf", op_a),
            OP_LOG => runtime_float("logf", op_a),
            OP_EXP => runtime_float("expf", op_a),
            OP_POW => format!("encode_float(glulx_powf(decode_float({}), decode_float({})))", op_a, op_b),
            OP_SIN => runtime_float("sinf", op_a),
            OP_COS => runtime_float("cosf", op_a),
            OP_TAN => runtime_float("tanf", op_a),
            OP_ASIN => runtime_float("asinf", op_a),
            OP_ACOS => runtime_float("acosf", op_a),
            OP_ATAN => runtime_float("atanf", op_a),
            OP_ATAN2 => format!("encode_float(atan2f(decode_float({}), decode_float({})))", op_a, op_b),
            OP_JISINF => format!("temp0 = {}, temp0 == 0x7F800000 || temp0 == 0xFF800000", op_a),
            OP_JISNAN => format!("temp0 = {}, (temp0 & 0x7F800000) == 0x7F800000 && (temp0 & 0x007FFFFF) != 0", op_a),
            OP_JFEQ => format!("OP_JFEQ({}, {}, {})", op_a, op_b, args[2]),
            OP_JFNE => format!("!OP_JFEQ({}, {}, {})", op_a, op_b, args[2]),
            OP_JFLT => format!("decode_float({}) < decode_float({})", op_a, op_b),
            OP_JFGT => format!("decode_float({}) > decode_float({})", op_a, op_b),
            OP_JFLE => format!("decode_float({}) <= decode_float({})", op_a, op_b),
            OP_JFGE => format!("decode_float({}) >= decode_float({})", op_a, op_b),
            _ => null, // TODO panic here
        }
    }

    pub fn args_join(&self, operands: Vec<String>, joiner: &str) -> String {
        match operands.len() {
            0 => String::new(),
            1 => format!("{}", operands[0]),
            2 => format!("{}{}{}", operands[0], joiner, operands[1]),
            _ => operands.join(joiner),
        }
    }

    pub fn args(&self, operands: Vec<String>) -> String {
        self.args_join(operands, ", ")
    }

    pub fn runtime(&self, name: &str, operands: Vec<String>) -> String {
        format!("{}({})", name, self.args(operands))
    }

}

fn runtime_float(func: &str, operand: &String) -> String {
    format!("encode_float({}(decode_float({})))", func, operand)
}