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
            OP_RETURN => format!("return {}", op_a),
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