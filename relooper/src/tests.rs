/*

Tests for the Relooper
======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

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
        immediate: Some(Box::new(Simple(SimpleBlock {
            label: 1,
            immediate: Some(Box::new(Simple(SimpleBlock {
                label: 2,
                immediate: None,
                next: None,
            }))),
            next: None,
        }))),
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
                    immediate: Some(Box::new(Simple(SimpleBlock {
                        label: 3,
                        immediate: None,
                        next: None,
                    }))),
                    next: None,
                }))),
                next: None,
            })),
        }))),
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
                                immediate: Some(Box::new(Simple(SimpleBlock {
                                    label: 3,
                                    immediate: None,
                                    next: None,
                                }))),
                                next: None,
                            })),
                        },
                        HandledBlock {
                            labels: vec![4],
                            inner: Box::new(Simple(SimpleBlock {
                                label: 4,
                                immediate: None,
                                next: None,
                            })),
                        },
                    ],
                }))),
                next: None,
            })),
        }))),
        next: None,
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
                                inner: Box::new(Simple(SimpleBlock {
                                    label: 3,
                                    immediate: None,
                                    next: None,
                                })),
                            },
                            HandledBlock {
                                labels: vec![4],
                                inner: Box::new(Simple(SimpleBlock {
                                    label: 4,
                                    immediate: None,
                                    next: None,
                                })),
                            },
                        ],
                    }))),
                    next: None,
                }))),
                next: None,
            })),
        }))),
        next: None,
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
                    inner: Box::new(Simple(SimpleBlock {
                        label: 1,
                        immediate: None,
                        next: None,
                    })),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 2,
                        immediate: None,
                        next: None,
                    })),
                },
            ],
        }))),
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
                    inner: Box::new(Simple(SimpleBlock {
                        label: 1,
                        immediate: None,
                        next: None,
                    })),
                },
                HandledBlock {
                    labels: vec![2],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 2,
                        immediate: None,
                        next: None,
                    })),
                },
            ],
        }))),
        next: Some(Box::new(Simple(SimpleBlock {
            label: 3,
            immediate: None,
            next: None,
        }))),
    })));

    let blocks = hashmap!{
        0 => vec![1, 2],
        1 => vec![2],
        2 => vec![],
    };
    let result = reloop(blocks, 0);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 0,
        immediate: Some(Box::new(Simple(SimpleBlock {
            label: 1,
            immediate: None,
            next: None,
        }))),
        next: Some(Box::new(Simple(SimpleBlock {
            label: 2,
            immediate: None,
            next: None,
        }))),
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
    assert_eq!(result, Box::new(Simple(SimpleBlock {
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
                                next: None,
                            }))),
                            next: None,
                        })),
                    })),
                },
                HandledBlock {
                    labels: vec![3],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 3,
                        immediate: None,
                        next: None,
                    })),
                },
            ],
        }))),
        next: None,
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
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 2,
                                        immediate: None,
                                        next: None,
                                    })),
                                },
                                HandledBlock {
                                    labels: vec![3],
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 3,
                                        immediate: None,
                                        next: None,
                                    })),
                                },
                            ],
                        }))),
                        next: Some(Box::new(Simple(SimpleBlock {
                            label: 4,
                            immediate: None,
                            next: None,
                        }))),
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
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 6,
                                        immediate: None,
                                        next: None,
                                    })),
                                },
                                HandledBlock {
                                    labels: vec![7],
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 7,
                                        immediate: None,
                                        next: None,
                                    })),
                                },
                            ],
                        }))),
                        next: None,
                    })),
                }
            ],
        }))),
        next: Some(Box::new(Simple(SimpleBlock {
            label: 8,
            immediate: None,
            next: None,
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
                        immediate: Some(Box::new(Simple(SimpleBlock {
                            label: 'D',
                            immediate: None,
                            next: None,
                        }))),
                        next: None,
                    })),
                },
                HandledBlock {
                    labels: vec!['C'],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 'C',
                        immediate: None,
                        next: None,
                    })),
                },
            ],
            next: Some(Box::new(Simple(SimpleBlock {
                label: 'E',
                immediate: Some(Box::new(Simple(SimpleBlock {
                    label: 'F',
                    immediate: None,
                    next: None,
                }))),
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 'G',
                    immediate: Some(Box::new(Simple(SimpleBlock {
                        label: 'H',
                        immediate: None,
                        next: None,
                    }))),
                    next: None,
                }))),
            }))),
        }))),
        next: None,
    })));
}