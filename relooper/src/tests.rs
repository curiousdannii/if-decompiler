/*

Tests for the Relooper
======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::iter::FromIterator;

use maplit::hashmap;

use super::*;
use ShapedBlock::*;

// Basic sequential blocks
#[test]
fn test_basic_blocks() {
    let blocks = hashmap!{
        0 => vec![1],
        1 => vec![2],
        2 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(Simple(SimpleBlock {
            label: 1,
            next: Some(Box::new(Simple(SimpleBlock {
                label: 2,
                next: None,
            }))),
        }))),
    })));
}

// Some basic loops
#[test]
fn test_basic_loops() {
    let blocks = hashmap!{
        0 => vec![1],
        1 => vec![2],
        2 => vec![3],
        3 => vec![1],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 2,
                    next: Some(Box::new(Simple(SimpleBlock {
                        label: 3,
                        next: None,
                    }))),
                }))),
            })),
            next: None,
        }))),
    })));

    let blocks = hashmap!{
        0 => vec![1],
        1 => vec![2, 4],
        2 => vec![3],
        3 => vec![1],
        4 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                next: Some(Box::new(Multiple(MultipleBlock {
                    handled: FnvHashMap::from_iter(vec![
                        (2, Box::new(Simple(SimpleBlock {
                            label: 2,
                            next: Some(Box::new(Simple(SimpleBlock {
                                label: 3,
                                next: None,
                            }))),
                        }))),
                        (4, Box::new(Simple(SimpleBlock {
                            label: 4,
                            next: None,
                        }))),
                    ]),
                    next: None,
                }))),
            })),
            next: None,
        }))),
    })));

    let blocks = hashmap!{
        0 => vec![1],
        1 => vec![2],
        2 => vec![3, 4],
        3 => vec![1],
        4 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 2,
                    next: Some(Box::new(Multiple(MultipleBlock {
                        handled: FnvHashMap::from_iter(vec![
                            (3, Box::new(Simple(SimpleBlock {
                                label: 3,
                                next: None,
                            }))),
                            (4, Box::new(Simple(SimpleBlock {
                                label: 4,
                                next: None,
                            }))),
                        ]),
                        next: None,
                    }))),
                }))),
            })),
            next: None,
        }))),
    })));
}

// Some basic ifs
#[test]
fn test_basic_ifs() {
    let blocks = hashmap!{
        0 => vec![1, 2],
        1 => vec![],
        2 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: FnvHashMap::from_iter(vec![
                (1, Box::new(Simple(SimpleBlock {
                    label: 1,
                    next: None,
                }))),
                (2, Box::new(Simple(SimpleBlock {
                    label: 2,
                    next: None,
                }))),
            ]),
            next: None,
        }))),
    })));

    let blocks = hashmap!{
        0 => vec![1, 2],
        1 => vec![3],
        2 => vec![3],
        3 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: FnvHashMap::from_iter(vec![
                (1, Box::new(Simple(SimpleBlock {
                    label: 1,
                    next: None,
                }))),
                (2, Box::new(Simple(SimpleBlock {
                    label: 2,
                    next: None,
                }))),
            ]),
            next: Some(Box::new(Simple(SimpleBlock {
                label: 3,
                next: None,
            }))),
        }))),
    })));
}