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
use BranchMode::*;
use ShapedBlock::*;

fn branch_to<T: RelooperLabel>(label: T, branch: BranchMode) -> FnvHashMap<T, BranchMode> {
    let mut res = FnvHashMap::default();
    res.insert(label, branch);
    res
}

fn end_node<T: RelooperLabel>(label: T, branches: Option<FnvHashMap<T, BranchMode>>) -> Box<ShapedBlock<T>> {
    Box::new(Simple(SimpleBlock {
        label,
        immediate: None,
        branches: branches.unwrap_or_default(),
        next: None,
    }))
}

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
        immediate: Some(Box::new(Simple(SimpleBlock {
            label: 1,
            immediate: Some(end_node(2, None)),
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
    let blocks = hashmap!{
        0 => vec![1],
        1 => vec![2],
        2 => vec![3],
        3 => vec![1],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                immediate: Some(Box::new(Simple(SimpleBlock {
                    label: 2,
                    immediate: Some(end_node(3, Some(branch_to(1, LoopContinue(0))))),
                    branches: FnvHashMap::default(),
                    next: None,
                }))),
                branches: FnvHashMap::default(),
                next: None,
            })),
        }))),
        branches: FnvHashMap::default(),
        next: None,
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
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        HandledBlock {
                            labels: vec![2],
                            inner: Box::new(Simple(SimpleBlock {
                                label: 2,
                                immediate: Some(end_node(3, Some(branch_to(1, LoopContinue(0))))),
                                branches: FnvHashMap::default(),
                                next: None,
                            })),
                        },
                    ],
                }))),
                branches: branch_to(4, LoopBreak(0)),
                next: None,
            })),
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node(4, None)),
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
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 1,
                immediate: Some(Box::new(Simple(SimpleBlock {
                    label: 2,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            HandledBlock {
                                labels: vec![3],
                                inner: end_node(3, Some(branch_to(1, LoopContinue(0)))),
                            },
                        ],
                    }))),
                    branches: branch_to(4, LoopBreak(0)),
                    next: None,
                }))),
                branches: FnvHashMap::default(),
                next: None,
            })),
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node(4, None)),
    })));

    // Test a self loop
    let blocks = hashmap!{
        0 => vec![0, 1],
        1 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Loop(LoopBlock {
        loop_id: 0,
        inner: Box::new(Simple(SimpleBlock {
            label: 0,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    HandledBlock {
                        labels: vec![1],
                        inner: end_node(1, None),
                    },
                ],
            }))),
            branches: branch_to(0, LoopContinue(0)),
            next: None,
        })),
    })));

    // Multiple breaks to a dominated node from a loop (a little excerpt from the Glulxercise Tokenise test)
    let blocks = hashmap!{
        749 => vec![756],
        756 => vec![762, 786],
        762 => vec![777, 786],
        777 => vec![756],
        786 => vec![],
    };
    let result = reloop(blocks, 749);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 749,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 756,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        HandledBlock {
                            labels: vec![762],
                            inner: Box::new(Simple(SimpleBlock {
                                label: 762,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        HandledBlock {
                                            labels: vec![777],
                                            inner: end_node(777, Some(branch_to(756, LoopContinue(0)))),
                                        },
                                    ],
                                }))),
                                branches: branch_to(786, LoopBreak(0)),
                                next: None,
                            })),
                        },
                    ],
                }))),
                branches: branch_to(786, LoopBreak(0)),
                next: None,
            })),
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node(786, None)),
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
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: end_node(1, None),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: end_node(2, None),
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: None,
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
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: end_node(1, Some(branch_to(3, MergedBranch))),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: end_node(2, Some(branch_to(3, MergedBranch))),
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node(3, None)),
    })));

    let blocks = hashmap!{
        0 => vec![1, 2],
        1 => vec![2],
        2 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: end_node(1, Some(branch_to(2, MergedBranch))),
                },
            ],
        }))),
        branches: branch_to(2, MergedBranch),
        next: Some(end_node(2, None)),
    })));
}

#[test]
fn test_nested_loops() {
    let blocks = hashmap!{
        0 => vec![1, 3],
        1 => vec![2],
        2 => vec![0, 1],
        3 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Loop(LoopBlock {
        loop_id: 0,
        inner: Box::new(Simple(SimpleBlock {
            label: 0,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    HandledBlock {
                        labels: vec![1],
                        inner: Box::new(Loop(LoopBlock {
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
                        })),
                    },
                    HandledBlock {
                        labels: vec![3],
                        inner: end_node(3, None),
                    },
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
    })));
}

#[test]
fn test_nested_ifs() {
    let blocks = hashmap!{
        0 => vec![1, 5],
        1 => vec![2, 3],
        2 => vec![4],
        3 => vec![4],
        4 => vec![8],
        5 => vec![6, 7],
        6 => vec![8],
        7 => vec![8],
        8 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 1,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec![2],
                                    inner: end_node(2, Some(branch_to(4, MergedBranch))),
                                },
                                HandledBlock {
                                    labels: vec![3],
                                    inner: end_node(3, Some(branch_to(4, MergedBranch))),
                                },
                            ],
                        }))),
                        next: Some(end_node(4, Some(branch_to(8, MergedBranch)))),
                        branches: FnvHashMap::default(),
                    })),
                },
                HandledBlock {
                    labels: vec![5],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 5,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec![6],
                                    inner: end_node(6, Some(branch_to(8, MergedBranch))),
                                },
                                HandledBlock {
                                    labels: vec![7],
                                    inner: end_node(7, Some(branch_to(8, MergedBranch))),
                                },
                            ],
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                }
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node(8, None)),
    })));
}

#[test]
fn test_loop_in_branch() {
    let blocks = hashmap!{
        0 => vec![1, 2],
        1 => vec![4],
        2 => vec![3],
        3 => vec![2, 4],
        4 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: end_node(1, Some(branch_to(4, MergedBranch))),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: Box::new(Loop(LoopBlock {
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
                    })),
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node(4, None)),
    })));
}

#[test]
fn test_spaghetti() {
    let blocks = hashmap!{
        0 => vec![1, 2],
        1 => vec![3, 4],
        2 => vec![5, 6],
        3 => vec![8],
        4 => vec![7],
        5 => vec![7],
        6 => vec![8],
        7 => vec![],
        8 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 1,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec![3],
                                    inner: end_node(3, Some(branch_to(8, MergedBranchIntoMulti))),
                                },
                                HandledBlock {
                                    labels: vec![4],
                                    inner: end_node(4, Some(branch_to(7, MergedBranchIntoMulti))),
                                },
                            ],
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 2,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec![5],
                                    inner: end_node(5, Some(branch_to(7, MergedBranchIntoMulti))),
                                },
                                HandledBlock {
                                    labels: vec![6],
                                    inner: end_node(6, Some(branch_to(8, MergedBranchIntoMulti))),
                                },
                            ],
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![7],
                    inner: end_node(7, None),
                },
                HandledBlock {
                    labels: vec![8],
                    inner: end_node(8, None),
                },
            ],
        }))),
    })));

    let blocks = hashmap!{
        0 => vec![1, 2, 3],
        1 => vec![6],
        2 => vec![4, 5],
        3 => vec![7],
        4 => vec![2, 6],
        5 => vec![2, 7],
        6 => vec![],
        7 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: end_node(1, Some(branch_to(6, MergedBranchIntoMulti))),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: Box::new(Loop(LoopBlock {
                        loop_id: 0,
                        inner: Box::new(Simple(SimpleBlock {
                            label: 2,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    HandledBlock {
                                        labels: vec![4],
                                        inner: Box::new(Simple(SimpleBlock {
                                            label: 4,
                                            immediate: None,
                                            branches: FnvHashMap::from_iter(vec![
                                                (2, LoopContinue(0)),
                                                (6, LoopBreakIntoMultiple(0)),
                                            ]),
                                            next: None,
                                        })),
                                    },
                                    HandledBlock {
                                        labels: vec![5],
                                        inner: Box::new(Simple(SimpleBlock {
                                            label: 5,
                                            immediate: None,
                                            branches: FnvHashMap::from_iter(vec![
                                                (2, LoopContinue(0)),
                                                (7, LoopBreakIntoMultiple(0)),
                                            ]),
                                            next: None,
                                        })),
                                    },
                                ],
                            }))),
                            branches: FnvHashMap::default(),
                            next: None,
                        })),
                    })),
                },
                HandledBlock {
                    labels: vec![3],
                    inner: end_node(3, Some(branch_to(7, MergedBranchIntoMulti))),
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![6],
                    inner: end_node(6, None),
                },
                HandledBlock {
                    labels: vec![7],
                    inner: end_node(7, None),
                },
            ],
        }))),
    })));
}

// The example from the Stackifier article
#[test]
fn test_stackifier_multiloop() {
    let blocks = hashmap!{
        'A' => vec!['B', 'C'],
        'B' => vec!['D', 'E'],
        'C' => vec!['E'],
        'D' => vec!['B', 'C'],
        'E' => vec!['F', 'G'],
        'F' => vec!['G'],
        'G' => vec!['B', 'H'],
        'H' => vec![],
    };
    let result = reloop(blocks, 'A');
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 'A',
        immediate: Some(Box::new(LoopMulti(LoopMultiBlock {
            loop_id: 0,
            handled: vec![
                HandledBlock {
                    labels: vec!['B'],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 'B',
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec!['D'],
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 'D',
                                        immediate: None,
                                        branches: FnvHashMap::from_iter(vec![
                                            ('B', LoopContinueMulti(0)),
                                            ('C', LoopContinueMulti(0)),
                                        ]),
                                        next: None,
                                    })),
                                },
                            ],
                        }))),
                        branches: branch_to('E', MergedBranch),
                        next: None,
                    })),
                },
                HandledBlock {
                    labels: vec!['C'],
                    inner: end_node('C', Some(branch_to('E', MergedBranch))),
                },
            ],
            next: Some(Box::new(Simple(SimpleBlock {
                label: 'E',
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        HandledBlock {
                            labels: vec!['F'],
                            inner: end_node('F', Some(branch_to('G', MergedBranch))),
                        },
                    ],
                }))),
                branches: branch_to('G', MergedBranch),
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 'G',
                    immediate: None,
                    branches: FnvHashMap::from_iter(vec![
                        ('B', LoopContinueMulti(0)),
                        ('H', LoopBreak(0)),
                    ]),
                    next: None,
                }))),
            }))),
        }))),
        branches: FnvHashMap::default(),
        next: Some(end_node('H', None)),
    })));
}

#[test]
fn test_loopmulti() {
    // Test a LoopMulti with a top triple branch
    let blocks = hashmap!{
        1 => vec![2, 3, 4],
        2 => vec![],
        3 => vec![4],
        4 => vec![5],
        5 => vec![3],
    };
    let result = reloop(blocks, 1);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 1,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![2],
                    inner: end_node(2, None),
                },
                HandledBlock {
                    labels: vec![3, 4],
                    inner: Box::new(LoopMulti(LoopMultiBlock {
                        loop_id: 0,
                        handled: vec![
                            HandledBlock {
                                labels: vec![3],
                                inner: end_node(3, Some(branch_to(4, LoopContinueMulti(0)))),
                            },
                            HandledBlock {
                                labels: vec![4],
                                inner: Box::new(Simple(SimpleBlock {
                                    label: 4,
                                    immediate: Some(end_node(5, Some(branch_to(3, LoopContinueMulti(0))))),
                                    branches: FnvHashMap::default(),
                                    next: None,
                                })),
                            },
                        ],
                        next: None,
                    })),
                },
            ],
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));

    // Test a multiloop with multiple parents, internal branches, and a rejoined exit branch
    let blocks = hashmap!{
        0 => vec![1, 6],
        1 => vec![2, 3],
        2 => vec![3, 4],
        3 => vec![4, 5],
        4 => vec![5],
        5 => vec![3, 6],
        6 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                HandledBlock {
                    labels: vec![1],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 1,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec![2],
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 2,
                                        immediate: None,
                                        branches: FnvHashMap::from_iter(vec![
                                            (3, MergedBranchIntoMulti),
                                            (4, MergedBranchIntoMulti),
                                        ]),
                                        next: None,
                                    })),
                                },
                            ],
                        }))),
                        branches: branch_to(3, MergedBranchIntoMulti),
                        next: Some(Box::new(LoopMulti(LoopMultiBlock {
                            loop_id: 0,
                            handled: vec![
                                HandledBlock {
                                    labels: vec![3],
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 3,
                                        immediate: None,
                                        branches: FnvHashMap::from_iter(vec![
                                            (4, LoopContinueMulti(0)),
                                            (5, MergedBranch),
                                        ]),
                                        next: None,
                                    })),
                                },
                                HandledBlock {
                                    labels: vec![4],
                                    inner: end_node(4, Some(branch_to(5, MergedBranch))),
                                },
                            ],
                            next: Some(Box::new(Simple(SimpleBlock {
                                label: 5,
                                immediate: None,
                                branches: FnvHashMap::from_iter(vec![
                                    (3, LoopContinueMulti(0)),
                                    (6, LoopBreak(0)),
                                ]),
                                next: None,
                            }))),
                        }))),
                    })),
                },
            ],
        }))),
        branches: branch_to(6, MergedBranch),
        next: Some(end_node(6, None)),
    })));
}