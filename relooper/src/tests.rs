/*

Tests for the Relooper
======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use fnv::FnvHashMap;

use super::*;

// Basic sequential blocks
#[test]
fn test_basic_blocks() {
    let mut blocks: FnvHashMap<u32, Vec<u32>> = FnvHashMap::default();
    blocks.insert(0, vec![1]);
    blocks.insert(1, vec![2]);
    blocks.insert(2, vec![]);
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(ShapedBlock::Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(ShapedBlock::Simple(SimpleBlock {
            label: 1,
            next: Some(Box::new(ShapedBlock::Simple(SimpleBlock {
                label: 2,
                next: None,
            }))),
        }))),
    })));
}