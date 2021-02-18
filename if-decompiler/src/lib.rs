/*

if-decompiler - core library
===============================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

pub mod glulx;

// Function safety refers to whether or not a function can be compiled and run without worrying about its locals and stack being part of the savestate
// Unsafe functions need to be compiled such that they can be serialised and restored
// SafetyTBD functions have not yet been determined. At the end of decompilation any remaining SafetyTBD functions will be assumed safe.
#[derive(Copy, Clone, PartialEq)]
pub enum FunctionSafety {
    Unsafe,
    SafetyTBD,
}

#[derive(Copy, Clone, PartialEq)]
pub enum BranchTarget {
    Dynamic,
    Absolute(u32),
    Return(i32),
}