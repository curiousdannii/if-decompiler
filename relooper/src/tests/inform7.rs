/*

Tests from Inform 7
===================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use super::*;

// Arrcpy
#[test]
fn arrcpy() {
    let input21083 = vec![
        (21083, vec![21089, 21108]),
        (21089, vec![21105]),
        (21105, vec![21191]),
        (21108, vec![21114, 21179]),
        (21114, vec![21120, 21179]),
        (21120, vec![21123]),
        (21123, vec![21129, 21176]),
        (21129, vec![21123]),
        (21176, vec![21191]),
        (21179, vec![]),
        (21191, vec![]),
    ];

    let block21083 = Box::new(Simple(SimpleBlock {
        label: 21083,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(21089, Simple(SimpleBlock {
                    label: 21089,
                    immediate: Some(Box::new(end_node(21105, Some(branch_to(21191, MergedBranchIntoMulti))))),
                    branches: FnvHashMap::default(),
                    next: None,
                })),
                basic_handled(21108, Simple(SimpleBlock {
                    label: 21108,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(21114, Simple(SimpleBlock {
                                label: 21114,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(21120, Simple(SimpleBlock {
                                            label: 21120,
                                            immediate: Some(Box::new(Loop(LoopBlock {
                                                loop_id: 0,
                                                inner: Box::new(Simple(SimpleBlock {
                                                    label: 21123,
                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                        handled: vec![
                                                            basic_handled(21129, end_node(21129, Some(branch_to(21123, LoopContinue(0))))),
                                                            basic_handled(21176, end_node(21176, Some(branch_to(21191, LoopBreakIntoMulti(0))))),
                                                        ],
                                                    }))),
                                                    branches: FnvHashMap::default(),
                                                    next: None,
                                                })),
                                                next: None,
                                            }))),
                                            branches: FnvHashMap::default(),
                                            next: None,
                                        })),
                                    ],
                                }))),
                                branches: branch_to(21179, MergedBranchIntoMulti),
                                next: None,
                            })),
                        ],
                    }))),
                    branches: branch_to(21179, MergedBranchIntoMulti),
                    next: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(21179, end_node(21179, None)),
                        ],
                    }))),
                })),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(21191, end_node(21191, None)),
            ],
        }))),
    }));

    let result = reloop(input21083, 21083);
    assert_eq!(result, block21083);
}