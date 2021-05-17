/*

Tests for the Relooper
======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use std::iter::FromIterator;

use super::*;
use BranchMode::*;
use ShapedBlock::*;

mod glulxercise;

fn basic_handled<T: RelooperLabel>(label: T, inner: ShapedBlock<T>) -> HandledBlock<T> {
    HandledBlock {
        labels: vec![label],
        inner,
        break_after: true,
    }
}

fn basic_handled_without_break<T: RelooperLabel>(label: T, inner: ShapedBlock<T>) -> HandledBlock<T> {
    HandledBlock {
        labels: vec![label],
        inner,
        break_after: false,
    }
}

fn branch_to<T: RelooperLabel>(label: T, branch: BranchMode) -> FnvHashMap<T, BranchMode> {
    let mut res = FnvHashMap::default();
    res.insert(label, branch);
    res
}

fn end_node<T: RelooperLabel>(label: T, branches: Option<FnvHashMap<T, BranchMode>>) -> ShapedBlock<T> {
    Simple(SimpleBlock {
        label,
        immediate: None,
        branches: branches.unwrap_or_default(),
        next: None,
    })
}

// Basic sequential blocks
#[test]
fn test_basic_blocks() {
    let blocks = vec![
        (0, vec![1]),
        (1, vec![2]),
        (2, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Simple(SimpleBlock {
            label: 1,
            immediate: Some(Box::new(end_node(2, None))),
            branches: FnvHashMap::default(),
            next: None,
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));
}

// Some basic loops
#[test]
fn test_basic_loops() {
    let blocks = vec![
        (0, vec![1]),
        (1, vec![2]),
        (2, vec![3]),
        (3, vec![1]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                immediate: Some(Box::new(Simple(SimpleBlock {
                    label: 2,
                    immediate: Some(Box::new(end_node(3, Some(branch_to(1, LoopContinue(0)))))),
                    branches: FnvHashMap::default(),
                    next: None,
                }))),
                branches: FnvHashMap::default(),
                next: None,
            })),
            next: None,
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));

    let blocks = vec![
        (0, vec![1]),
        (1, vec![2, 4]),
        (2, vec![3]),
        (3, vec![1]),
        (4, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled(2, Simple(SimpleBlock {
                            label: 2,
                            immediate: Some(Box::new(end_node(3, Some(branch_to(1, LoopContinue(0)))))),
                            branches: FnvHashMap::default(),
                            next: None,
                        })),
                    ],
                }))),
                branches: branch_to(4, LoopBreak(0)),
                next: None,
            })),
            next: Some(Box::new(end_node(4, None))),
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));

    let blocks = vec![
        (0, vec![1]),
        (1, vec![2]),
        (2, vec![3, 4]),
        (3, vec![1]),
        (4, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                immediate: Some(Box::new(Simple(SimpleBlock {
                    label: 2,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(3, end_node(3, Some(branch_to(1, LoopContinue(0))))),
                        ],
                    }))),
                    branches: branch_to(4, LoopBreak(0)),
                    next: None,
                }))),
                branches: FnvHashMap::default(),
                next: None,
            })),
            next: Some(Box::new(end_node(4, None))),
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));

    // Test a self loop
    let blocks = vec![
        (0, vec![0, 1]),
        (1, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Loop(LoopBlock {
        loop_id: 0,
        inner: Box::new(Simple(SimpleBlock {
            label: 0,
            immediate: None,
            branches: FnvHashMap::from_iter(vec![
                (0, LoopContinue(0)),
                (1, LoopBreak(0)),
            ]),
            next: None,
        })),
        next: Some(Box::new(end_node(1, None))),
    })));

    // Multiple breaks to a dominated node from a loop (a little excerpt from the Glulxercise Tokenise test)
    let blocks = vec![
        (749, vec![756]),
        (756, vec![762, 786]),
        (762, vec![777, 786]),
        (777, vec![756]),
        (786, vec![]),
    ];
    let result = reloop(blocks, 749);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 749,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 756,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled(762, Simple(SimpleBlock {
                            label: 762,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled(777, end_node(777, Some(branch_to(756, LoopContinue(0))))),
                                ],
                            }))),
                            branches: branch_to(786, LoopBreak(0)),
                            next: None,
                        })),
                    ],
                }))),
                branches: branch_to(786, LoopBreak(0)),
                next: None,
            })),
            next: Some(Box::new(end_node(786, None))),
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));
}

// Some basic ifs
#[test]
fn test_basic_ifs() {
    let blocks = vec![
        (0, vec![1, 2]),
        (1, vec![]),
        (2, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, end_node(1, None)),
                basic_handled(2, end_node(2, None)),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));

    let blocks = vec![
        (0, vec![1, 2]),
        (1, vec![3]),
        (2, vec![3]),
        (3, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, end_node(1, Some(branch_to(3, MergedBranch)))),
                basic_handled(2, end_node(2, Some(branch_to(3, MergedBranch)))),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(end_node(3, None))),
    })));

    let blocks = vec![
        (0, vec![1, 2]),
        (1, vec![2]),
        (2, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, end_node(1, Some(branch_to(2, MergedBranch)))),
            ],
        }))),
        branches: branch_to(2, MergedBranch),
        next: Some(Box::new(end_node(2, None))),
    })));
}

#[test]
fn test_nested_loops() {
    let blocks = vec![
        (0, vec![1, 3]),
        (1, vec![2]),
        (2, vec![0, 1]),
        (3, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Loop(LoopBlock {
        loop_id: 0,
        inner: Box::new(Simple(SimpleBlock {
            label: 0,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(1, Loop(LoopBlock {
                        loop_id: 1,
                        inner: Box::new(Simple(SimpleBlock {
                            label: 1,
                            immediate: Some(Box::new(Simple(SimpleBlock {
                                label: 2,
                                immediate: None,
                                branches: FnvHashMap::from_iter(vec![
                                    (0, LoopContinue(0)),
                                    (1, LoopContinue(1)),
                                ]),
                                next: None,
                            }))),
                            branches: FnvHashMap::default(),
                            next: None,
                        })),
                        next: None,
                    })),
                ],
            }))),
            branches: branch_to(3, LoopBreak(0)),
            next: None,
        })),
        next: Some(Box::new(end_node(3, None))),
    })));
}

mod nested_branches {
    use super::*;

    #[test]
    fn simple_nested_branches() {
        let blocks = vec![
            (0, vec![1, 5]),
            (1, vec![2, 3]),
            (2, vec![4]),
            (3, vec![4]),
            (4, vec![8]),
            (5, vec![6, 7]),
            (6, vec![8]),
            (7, vec![8]),
            (8, vec![]),
        ];
        let result = reloop(blocks, 0);
        assert_eq!(result, Box::new(Simple(SimpleBlock {
            label: 0,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(1, Simple(SimpleBlock {
                        label: 1,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(2, end_node(2, Some(branch_to(4, MergedBranch)))),
                                basic_handled(3, end_node(3, Some(branch_to(4, MergedBranch)))),
                            ],
                        }))),
                        next: Some(Box::new(end_node(4, Some(branch_to(8, MergedBranch))))),
                        branches: FnvHashMap::default(),
                    })),
                    basic_handled(5, Simple(SimpleBlock {
                        label: 5,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(6, end_node(6, Some(branch_to(8, MergedBranch)))),
                                basic_handled(7, end_node(7, Some(branch_to(8, MergedBranch)))),
                            ],
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: Some(Box::new(end_node(8, None))),
        })));
    }

    // A small part of Glulxercise Tokenise
    // This represents and if-else block where the if clause has two conditions with an OR
    // (In this case it is a strict mode range check - if writing outside an array's bounds show an error, if within perform the write)
    #[test]
    fn if_else_with_or() {
        let blocks = vec![
            (1060, vec![1089, 1095]),
            (1089, vec![1095, 1122]),
            (1095, vec![1130]),
            (1122, vec![1130]),
            (1130, vec![]),
        ];
        let result = reloop(blocks, 1060);
        assert_eq!(result, Box::new(Simple(SimpleBlock {
            label: 1060,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(1089, Simple(SimpleBlock {
                        label: 1089,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(1122, end_node(1122, Some(branch_to(1130, MergedBranchIntoMulti)))),
                            ],
                        }))),
                        branches: branch_to(1095, MergedBranchIntoMulti),
                        next: None,
                    })),
                ],
            }))),
            branches: branch_to(1095, MergedBranchIntoMulti),
            next: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled_without_break(1095, end_node(1095, Some(branch_to(1130, MergedBranchIntoMulti)))),
                    basic_handled(1130, end_node(1130, None)),
                ],
            }))),
        })));
    }
}

#[test]
fn test_loop_in_branch() {
    let blocks = vec![
        (0, vec![1, 2]),
        (1, vec![4]),
        (2, vec![3]),
        (3, vec![2, 4]),
        (4, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, end_node(1, Some(branch_to(4, MergedBranch)))),
                basic_handled(2, Loop(LoopBlock {
                    loop_id: 0,
                    inner: Box::new(Simple(SimpleBlock {
                        label: 2,
                        immediate: Some(Box::new(Simple(SimpleBlock {
                            label: 3,
                            immediate: None,
                            branches: FnvHashMap::from_iter(vec![
                                (2, LoopContinue(0)),
                                (4, LoopBreak(0)),
                            ]),
                            next: None,
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                    next: None,
                })),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(end_node(4, None))),
    })));
}

#[test]
fn test_spaghetti() {
    let blocks = vec![
        (0, vec![1, 2]),
        (1, vec![3, 4]),
        (2, vec![5, 6]),
        (3, vec![8]),
        (4, vec![7]),
        (5, vec![7]),
        (6, vec![8]),
        (7, vec![]),
        (8, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, Simple(SimpleBlock {
                    label: 1,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(3, end_node(3, Some(branch_to(8, MergedBranchIntoMulti)))),
                            basic_handled(4, end_node(4, Some(branch_to(7, MergedBranchIntoMulti)))),
                        ],
                    }))),
                    branches: FnvHashMap::default(),
                    next: None,
                })),
                basic_handled(2, Simple(SimpleBlock {
                    label: 2,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(5, end_node(5, Some(branch_to(7, MergedBranchIntoMulti)))),
                            basic_handled(6, end_node(6, Some(branch_to(8, MergedBranchIntoMulti)))),
                        ],
                    }))),
                    branches: FnvHashMap::default(),
                    next: None,
                })),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(7, end_node(7, None)),
                basic_handled(8, end_node(8, None)),
            ],
        }))),
    })));

    let blocks = vec![
        (0, vec![1, 2, 3]),
        (1, vec![6]),
        (2, vec![4, 5]),
        (3, vec![7]),
        (4, vec![2, 6]),
        (5, vec![2, 7]),
        (6, vec![]),
        (7, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, end_node(1, Some(branch_to(6, MergedBranchIntoMulti)))),
                basic_handled(2, Loop(LoopBlock {
                    loop_id: 0,
                    inner: Box::new(Simple(SimpleBlock {
                        label: 2,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(4, Simple(SimpleBlock {
                                    label: 4,
                                    immediate: None,
                                    branches: FnvHashMap::from_iter(vec![
                                        (2, LoopContinue(0)),
                                        (6, LoopBreakIntoMulti(0)),
                                    ]),
                                    next: None,
                                })),
                                basic_handled(5, Simple(SimpleBlock {
                                    label: 5,
                                    immediate: None,
                                    branches: FnvHashMap::from_iter(vec![
                                        (2, LoopContinue(0)),
                                        (7, LoopBreakIntoMulti(0)),
                                    ]),
                                    next: None,
                                })),
                            ],
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                    next: None,
                })),
                basic_handled(3, end_node(3, Some(branch_to(7, MergedBranchIntoMulti)))),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(6, end_node(6, None)),
                basic_handled(7, end_node(7, None)),
            ],
        }))),
    })));
}

// The example from the Stackifier article
#[test]
fn test_stackifier_multiloop() {
    let blocks = vec![
        ('A', vec!['B', 'C']),
        ('B', vec!['D', 'E']),
        ('C', vec!['E']),
        ('D', vec!['B', 'C']),
        ('E', vec!['F', 'G']),
        ('F', vec!['G']),
        ('G', vec!['B', 'H']),
        ('H', vec![]),
    ];
    let result = reloop(blocks, 'A');
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 'A',
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled('B', Simple(SimpleBlock {
                        label: 'B',
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled('D', Simple(SimpleBlock {
                                    label: 'D',
                                    immediate: None,
                                    branches: FnvHashMap::from_iter(vec![
                                        ('B', LoopContinueIntoMulti(0)),
                                        ('C', LoopContinueIntoMulti(0)),
                                    ]),
                                    next: None,
                                })),
                            ],
                        }))),
                        branches: branch_to('E', MergedBranch),
                        next: None,
                    })),
                    basic_handled('C', end_node('C', Some(branch_to('E', MergedBranch)))),
                ],
            })),
            next: Some(Box::new(Simple(SimpleBlock {
                label: 'E',
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled('F', end_node('F', Some(branch_to('G', MergedBranch)))),
                    ],
                }))),
                branches: branch_to('G', MergedBranch),
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 'G',
                    immediate: None,
                    branches: FnvHashMap::from_iter(vec![
                        ('B', LoopContinueIntoMulti(0)),
                        ('H', LoopBreak(0)),
                    ]),
                    next: None,
                }))),
            }))),
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(end_node('H', None))),
    })));
}

#[test]
fn test_loopmulti() {
    // Test a LoopMulti with a top triple branch
    let blocks = vec![
        (1, vec![2, 3, 4]),
        (2, vec![]),
        (3, vec![4]),
        (4, vec![5]),
        (5, vec![3]),
    ];
    let result = reloop(blocks, 1);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 1,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(2, end_node(2, None)),
                HandledBlock {
                    labels: vec![3, 4],
                    inner: Loop(LoopBlock {
                        loop_id: 0,
                        inner: Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(3, end_node(3, Some(branch_to(4, LoopContinueIntoMulti(0))))),
                                basic_handled(4, Simple(SimpleBlock {
                                    label: 4,
                                    immediate: Some(Box::new(end_node(5, Some(branch_to(3, LoopContinueIntoMulti(0)))))),
                                    branches: FnvHashMap::default(),
                                    next: None,
                                })),
                            ],
                        })),
                        next: None,
                    }),
                    break_after: true,
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));

    // Test a multiloop with multiple parents, internal branches, and a rejoined exit branch
    let blocks = vec![
        (0, vec![1, 6]),
        (1, vec![2, 3]),
        (2, vec![3, 4]),
        (3, vec![4, 5]),
        (4, vec![5]),
        (5, vec![3, 6]),
        (6, vec![]),
    ];
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(1, Simple(SimpleBlock {
                    label: 1,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(2, Simple(SimpleBlock {
                                label: 2,
                                immediate: None,
                                branches: FnvHashMap::from_iter(vec![
                                    (3, MergedBranchIntoMulti),
                                    (4, MergedBranchIntoMulti),
                                ]),
                                next: None,
                            })),
                        ],
                    }))),
                    branches: branch_to(3, MergedBranchIntoMulti),
                    next: Some(Box::new(Loop(LoopBlock {
                        loop_id: 0,
                        inner: Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(3, Simple(SimpleBlock {
                                    label: 3,
                                    immediate: None,
                                    branches: FnvHashMap::from_iter(vec![
                                        (4, LoopContinueIntoMulti(0)),
                                        (5, MergedBranch),
                                    ]),
                                    next: None,
                                })),
                                basic_handled(4, end_node(4, Some(branch_to(5, MergedBranch)))),
                            ],
                        })),
                        next: Some(Box::new(Simple(SimpleBlock {
                            label: 5,
                            immediate: None,
                            branches: FnvHashMap::from_iter(vec![
                                (3, LoopContinueIntoMulti(0)),
                                (6, LoopBreak(0)),
                            ]),
                            next: None,
                        }))),
                    }))),
                })),
            ],
        }))),
        branches: branch_to(6, MergedBranch),
        next: Some(Box::new(end_node(6, None))),
    })));
}