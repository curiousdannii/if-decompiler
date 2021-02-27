/*

Relooper library
================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

#![forbid(unsafe_code)]

use fnv::FnvHashSet;

pub struct BasicBlock<L, C> {
    pub label: L,
    pub code: Vec<C>,
    pub branches: FnvHashSet<L>,
}