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
            OP_ADD => args.join(" + "),
            OP_SUB => format_safe_stack_pops_expression("{} - {}", &args),
            OP_MUL => args.join(" * "),
            OP_DIV => runtime("OP_DIV", &args),
            OP_MOD => runtime("OP_MOD", &args),
            OP_NEG => format!("-((glsi32) {})", op_a),
            OP_BITAND => args.join(" & "),
            OP_BITOR => args.join(" | "),
            OP_BITXOR => args.join(" ^ "),
            OP_BITNOT => format!("~{}", op_a),
            OP_SHIFTL => runtime("OP_SHIFTL", &args),
            OP_USHIFTR => runtime("OP_USHIFTR", &args),
            OP_SSHIFTR => runtime("OP_SSHIFTR", &args),
            OP_JUMP => String::new(),
            OP_JZ => format!("{} == 0", op_a),
            OP_JNZ => format!("{} != 0", op_a),
            OP_JEQ => format!("{} == {}", op_a, op_b),
            OP_JNE => format!("{} != {}", op_a, op_b),
            OP_JLT => format_safe_stack_pops_expression("(glsi32) {} < (glsi32) {}", &args),
            OP_JGT => format_safe_stack_pops_expression("(glsi32) {} > (glsi32) {}", &args),
            OP_JLE => format_safe_stack_pops_expression("(glsi32) {} <= (glsi32) {}", &args),
            OP_JGE => format_safe_stack_pops_expression("(glsi32) {} >= (glsi32) {}", &args),
            OP_JLTU => format_safe_stack_pops_expression("{} < {}", &args),
            OP_JGTU => format_safe_stack_pops_expression("{} > {}", &args),
            OP_JLEU => format_safe_stack_pops_expression("{} <= {}", &args),
            OP_JGEU => format_safe_stack_pops_expression("{} >= {}", &args),
            // OP_CALL
            // OP_RETURN
            // OP_TAILCALL
            // OP_CATCH
            // OP_THROW
            OP_COPY => op_a.clone(),
            // OP_COPYS | OP_COPYB
            OP_SEXS => runtime("OP_SEXS", &args),
            OP_SEXB => runtime("OP_SEXB", &args),
            OP_ALOAD => format_safe_stack_pops_macro("Mem4({} + 4 * (glsi32) {})", &args),
            OP_ALOADS => format_safe_stack_pops_macro("Mem2({} + 2 * (glsi32) {})", &args),
            OP_ALOADB => format_safe_stack_pops_macro("Mem1({} + (glsi32) {})", &args),
            OP_ALOADBIT => runtime("OP_ALOADBIT", &args),
            OP_ASTORE => format_safe_stack_pops_expression("store_operand(1, {} + 4 * (glsi32) {}, {})", &args),
            OP_ASTORES => format_safe_stack_pops_expression("store_operand_s(1, {} + 2 * (glsi32) {}, {})", &args),
            OP_ASTOREB => format_safe_stack_pops_expression("store_operand_b(1, {} + (glsi32) {}, {})", &args),
            OP_ASTOREBIT => runtime("OP_ASTOREBIT", &args),
            OP_STKCOUNT => String::from("(stackptr - valstackbase) / 4"),
            OP_STKPEEK => runtime("OP_STKPEEK", &args),
            OP_STKSWAP => runtime("OP_STKSWAP", &args),
            OP_STKCOPY => runtime("OP_STKCOPY", &args),
            OP_STKROLL => runtime("OP_STKROLL", &args),
            OP_STREAMCHAR => format!("(*stream_char_handler)({} & 0xFF)", op_a),
            OP_STREAMNUM => format!("stream_num((glsi32) {}, FALSE, 0)", op_a),
            OP_STREAMSTR => format!("stream_string({}, 0, 0)", op_a),
            OP_STREAMUNICHAR => format!("(*stream_unichar_handler)({})", op_a),
            OP_GESTALT => runtime("do_gestalt", &args),
            OP_DEBUGTRAP => format!("fatal_error_i(\"user debugtrap encountered.\", {})", op_a),
            OP_JUMPABS => String::new(),
            // OP_CALLF ..= OP_CALLFIII
            OP_GETMEMSIZE => String::from("endmem"),
            OP_SETMEMSIZE => format!("change_memsize({}, 0)", op_a),
            OP_GETSTRINGTBL => String::from("stream_get_table()"),
            OP_SETSTRINGTBL => runtime("stream_set_table", &args),
            // OP_GETIOSYS
            OP_SETIOSYS => runtime("stream_set_iosys", &args),
            OP_GLK => format!("(temp0 = {}, temp1 = {}, perform_glk(temp0, temp1, pop_arguments(temp1, 0)))", op_a, op_b),
            OP_RANDOM => runtime("OP_RANDOM", &args),
            OP_SETRANDOM => runtime("glulx_setrandom", &args),
            OP_VERIFY => runtime("perform_verify", &args),
            // OP_RESTART
            OP_PROTECT => runtime("OP_PROTECT", &args),
            // OP_SAVE
            // OP_RESTORE
            // OP_SAVEUNDO
            // OP_RESTOREUNDO
            // OP_QUIT
            OP_LINEARSEARCH => runtime("linear_search", &args),
            OP_BINARYSEARCH => runtime("binary_search", &args),
            OP_LINKEDSEARCH => runtime("linked_search", &args),
            OP_MZERO => runtime("OP_MZERO", &args),
            OP_MCOPY => runtime("OP_MCOPY", &args),
            OP_MALLOC => runtime("heap_alloc", &args),
            OP_MFREE => runtime("heap_free", &args),
            OP_ACCELFUNC => String::from(""),
            OP_ACCELPARAM => String::from(""),
            OP_NUMTOF => format!("encode_float((gfloat32) ((glsi32) {}))", op_a),
            OP_FTONUMZ => runtime("OP_FTONUMZ", &args),
            OP_FTONUMN => runtime("OP_FTONUMN", &args),
            OP_FADD => format!("encode_float(decode_float({}) + decode_float({}))", op_a, op_b),
            OP_FSUB => format!("encode_float(decode_float({}) - decode_float({}))", op_a, op_b),
            OP_FMUL => format!("encode_float(decode_float({}) * decode_float({}))", op_a, op_b),
            OP_FDIV => format_safe_stack_pops_expression("encode_float(decode_float({}) / decode_float({}))", &args),
            // OP_FMOD
            OP_FLOOR => runtime_float("floorf", op_a),
            OP_CEIL => runtime("OP_CEIL", &args),
            OP_SQRT => runtime_float("sqrtf", op_a),
            OP_LOG => runtime_float("logf", op_a),
            OP_EXP => runtime_float("expf", op_a),
            OP_POW => format_safe_stack_pops_expression("encode_float(glulx_powf(decode_float({}), decode_float({})))", &args),
            OP_SIN => runtime_float("sinf", op_a),
            OP_COS => runtime_float("cosf", op_a),
            OP_TAN => runtime_float("tanf", op_a),
            OP_ASIN => runtime_float("asinf", op_a),
            OP_ACOS => runtime_float("acosf", op_a),
            OP_ATAN => runtime_float("atanf", op_a),
            OP_ATAN2 => format_safe_stack_pops_expression("encode_float(atan2f(decode_float({}), decode_float({})))", &args),
            OP_JISINF => format!("temp0 = {}, temp0 == 0x7F800000 || temp0 == 0xFF800000", op_a),
            OP_JISNAN => format!("temp0 = {}, (temp0 & 0x7F800000) == 0x7F800000 && (temp0 & 0x007FFFFF) != 0", op_a),
            OP_JFEQ => format_safe_stack_pops_expression("OP_JFEQ({}, {}, {})", &args),
            OP_JFNE => format_safe_stack_pops_expression("!OP_JFEQ({}, {}, {})", &args),
            OP_JFLT => format_safe_stack_pops_expression("decode_float({}) < decode_float({})", &args),
            OP_JFGT => format_safe_stack_pops_expression("decode_float({}) > decode_float({})", &args),
            OP_JFLE => format_safe_stack_pops_expression("decode_float({}) <= decode_float({})", &args),
            OP_JFGE => format_safe_stack_pops_expression("decode_float({}) >= decode_float({})", &args),
            _ => panic!("Unknown opcode {:>3X} at address {}", opcode, instruction.addr),
        }
    }
}

fn runtime(name: &str, operands: &Vec<String>) -> String {
    let (prelude, new_operands) = safe_stack_pops(operands, false);
    if prelude == "" {
        return format!("{}({})", name, operands.join(", "));
    }
    format!("({}, {}({}))", prelude, name, new_operands.join(", "))
}

fn runtime_float(func: &str, operand: &String) -> String {
    format!("encode_float({}(decode_float({})))", func, operand)
}