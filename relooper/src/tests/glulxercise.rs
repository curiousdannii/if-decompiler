/*

Tests from Glulxercise
======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use super::*;

// Float
#[test]
fn float() {
    let input64978 = vec![
        (64978, vec![64991, 65011]),
        (64991, vec![65001, 65023]),
        (65001, vec![65011, 65023]),
        (65011, vec![]),
        (65023, vec![]),
    ];
    let result = reloop(input64978, 64978);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 64978,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(64991, Simple(SimpleBlock {
                    label: 64991,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(65001, Simple(SimpleBlock {
                                label: 65001,
                                immediate: None,
                                branches: FnvHashMap::from_iter(vec![
                                    (65011, MergedBranchIntoMulti),
                                    (65023, MergedBranchIntoMulti),
                                ]),
                                next: None,
                            })),
                        ],
                    }))),
                    branches: branch_to(65023, MergedBranchIntoMulti),
                    next: None,
                })),
            ],
        }))),
        branches: branch_to(65011, MergedBranchIntoMulti),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(65011, end_node(65011, None)),
                basic_handled(65023, end_node(65023, None)),
            ],
        }))),
    })));
}

// LookSub
#[test]
fn looksub() {
    let input3686 = vec![
        (3686, vec![3708]),
        (3708, vec![3724, 3798]),
        (3724, vec![3728, 3737]),
        (3728, vec![3737, 3750]),
        (3737, vec![3761]),
        (3750, vec![3758, 3761]),
        (3758, vec![3798]),
        (3761, vec![3765, 3771]),
        (3765, vec![3771]),
        (3771, vec![3798]),
        (3798, vec![3708, 3808]),
        (3808, vec![3818, 3833]),
        (3818, vec![3833]),
        (3833, vec![]),
    ];

    let result = reloop(input3686, 3686);
    assert_eq!(result, Box::new(Simple(SimpleBlock {
        label: 3686,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: 0,
            inner: Box::new(Simple(SimpleBlock {
                label: 3708,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled(3724, Simple(SimpleBlock {
                            label: 3724,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled(3728, Simple(SimpleBlock {
                                        label: 3728,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(3750, Simple(SimpleBlock {
                                                    label: 3750,
                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                        handled: vec![
                                                            basic_handled(3758, end_node(3758, Some(branch_to(3798, MergedBranch)))),
                                                        ],
                                                    }))),
                                                    branches: branch_to(3761, MergedBranchIntoMulti),
                                                    next: None,
                                                })),
                                            ],
                                        }))),
                                        branches: branch_to(3737, MergedBranchIntoMulti),
                                        next: None,
                                    })),
                                ],
                            }))),
                            branches: branch_to(3737, MergedBranchIntoMulti),
                            next: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled_without_break(3737, end_node(3737, Some(branch_to(3761, MergedBranchIntoMulti)))),
                                    basic_handled(3761, Simple(SimpleBlock {
                                        label: 3761,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(3765, end_node(3765, Some(branch_to(3771, MergedBranch)))),
                                            ],
                                        }))),
                                        branches: branch_to(3771, MergedBranch),
                                        next: Some(Box::new(end_node(3771, Some(branch_to(3798, MergedBranch))))),
                                    })),
                                ],
                            }))),
                        })),
                    ],
                }))),
                branches: branch_to(3798, MergedBranch),
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 3798,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(3808, Simple(SimpleBlock {
                                label: 3808,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(3818, end_node(3818, Some(branch_to(3833, MergedBranch)))),
                                    ],
                                }))),
                                branches: branch_to(3833, MergedBranch),
                                next: Some(Box::new(end_node(3833, None))),
                            })),
                        ],
                    }))),
                    branches: branch_to(3708, LoopContinue(0)),
                    next: None,
                }))),
            })),
            next: None,
        }))),
        branches: FnvHashMap::default(),
        next: None,
    })));
}

// Tokenise__
#[test]
fn tokenise() {
    let input727 = vec![
        (727, vec![749]),
        (749, vec![756, 959]),
        (756, vec![762, 786]),
        (762, vec![777, 786]),
        (777, vec![756]),
        (786, vec![792, 796]),
        (792, vec![959]),
        (796, vec![819, 831]),
        (819, vec![825, 831]),
        (825, vec![831, 840]),
        (831, vec![892]),
        (840, vec![846, 892]),
        (846, vec![865, 892]),
        (865, vec![871, 892]),
        (871, vec![877, 892]),
        (877, vec![883, 892]),
        (883, vec![840]),
        (892, vec![952, 955]),
        (952, vec![959]),
        (955, vec![749]),
        (959, vec![990]),
        (990, vec![997, 1254]),
        (997, vec![1041, 1045]),
        (1041, vec![1045]),
        (1045, vec![1054]),
        (1054, vec![1060, 1139]),
        (1060, vec![1089, 1095]),
        (1089, vec![1095, 1122]),
        (1095, vec![1130]),
        (1122, vec![1130]),
        (1130, vec![1054]),
        (1139, vec![1145, 1198]),
        (1145, vec![1150, 1156]),
        (1150, vec![1156, 1181]),
        (1156, vec![1189]),
        (1181, vec![1189]),
        (1189, vec![1139]),
        (1198, vec![990]),
        (1254, vec![]),
    ];

    let loop749id = 1;
    let loop756id = 5;
    let loop840id = 4;
    let loop990id = 0;
    let loop1054id = 3;
    let loop1139id = 2;

    let loop1139 = Box::new(Loop(LoopBlock {
        loop_id: loop1139id,
        inner: Box::new(Simple(SimpleBlock {
            label: 1139,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(1145, Simple(SimpleBlock {
                        label: 1145,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(1150, Simple(SimpleBlock {
                                    label: 1150,
                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                        handled: vec![
                                            basic_handled(1181, end_node(1181, Some(branch_to(1189, MergedBranchIntoMulti)))),
                                        ],
                                    }))),
                                    branches: branch_to(1156, MergedBranchIntoMulti),
                                    next: None,
                                })),
                            ],
                        }))),
                        branches: branch_to(1156, MergedBranchIntoMulti),
                        next: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled_without_break(1156, end_node(1156, Some(branch_to(1189, MergedBranchIntoMulti)))),
                                basic_handled(1189, end_node(1189, Some(branch_to(1139, LoopContinue(loop1139id))))),
                            ],
                        }))),
                    })),
                    basic_handled(1198, end_node(1198, Some(branch_to(990, LoopContinue(loop990id))))),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None
        })),
        next: None,
    }));

    let loop1054 = Box::new(Loop(LoopBlock {
        loop_id: loop1054id,
        inner: Box::new(Simple(SimpleBlock {
            label: 1054,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(1060, Simple(SimpleBlock {
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
                                basic_handled(1130, end_node(1130, Some(branch_to(1054, LoopContinue(loop1054id))))),
                            ],
                        }))),
                    })),
                    basic_handled(1139, *loop1139),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    }));

    let loop990 = Box::new(Loop(LoopBlock {
        loop_id: loop990id,
        inner: Box::new(Simple(SimpleBlock {
            label: 990,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(997, Simple(SimpleBlock {
                        label: 997,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(1041, end_node(1041, Some(branch_to(1045, MergedBranch)))),
                            ],
                        }))),
                        branches: branch_to(1045, MergedBranch),
                        next: Some(Box::new(Simple(SimpleBlock {
                            label: 1045,
                            immediate: Some(loop1054),
                            branches: FnvHashMap::default(),
                            next: None,
                        }))),
                    })),
                    basic_handled(1254, end_node(1254, None)),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    }));

    let loop840branch = branch_to(892, LoopBreakIntoMulti(loop840id));
    let blocks840 = Box::new(Loop(LoopBlock {
        loop_id: loop840id,
        inner: Box::new(Simple(SimpleBlock {
            label: 840,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(846, Simple(SimpleBlock {
                        label: 846,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(865, Simple(SimpleBlock {
                                    label: 865,
                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                        handled: vec![
                                            basic_handled(871, Simple(SimpleBlock {
                                                label: 871,
                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                    handled: vec![
                                                        basic_handled(877, Simple(SimpleBlock {
                                                            label: 877,
                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                handled: vec![
                                                                    basic_handled(883, end_node(883, Some(branch_to(840, LoopContinue(loop840id))))),
                                                                ],
                                                            }))),
                                                            branches: loop840branch.clone(),
                                                            next: None,
                                                        })),
                                                    ],
                                                }))),
                                                branches: loop840branch.clone(),
                                                next: None,
                                            })),
                                        ],
                                    }))),
                                    branches: loop840branch.clone(),
                                    next: None,
                                })),
                            ],
                        }))),
                        branches: loop840branch.clone(),
                        next: None,
                    })),
                ],
            }))),
            branches: loop840branch.clone(),
            next: None,
        })),
        next: None,
    }));

    let blocks786 = Box::new(Simple(SimpleBlock {
        label: 786,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(792, end_node(792, Some(branch_to(959, LoopBreak(loop756id))))),
                basic_handled(796, Simple(SimpleBlock {
                    label: 796,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(819, Simple(SimpleBlock {
                                label: 819,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(825, Simple(SimpleBlock {
                                            label: 825,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    basic_handled(840, *blocks840),
                                                ],
                                            }))),
                                            branches: branch_to(831, MergedBranchIntoMulti),
                                            next: None,
                                        })),
                                    ],
                                }))),
                                branches: branch_to(831, MergedBranchIntoMulti),
                                next: None,
                            })),
                        ],
                    }))),
                    branches: branch_to(831, MergedBranchIntoMulti),
                    next: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled_without_break(831, end_node(831, Some(branch_to(892, MergedBranchIntoMulti)))),
                            basic_handled(892, Simple(SimpleBlock {
                                label: 892,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(952, end_node(952, Some(branch_to(959, LoopBreak(loop756id))))),
                                        basic_handled(955, end_node(955, Some(branch_to(749, LoopContinue(loop749id))))),
                                    ],
                                }))),
                                branches: FnvHashMap::default(),
                                next: None,
                            })),
                        ],
                    }))),
                })),
            ],
        }))),
        branches: FnvHashMap::default(),
        next: None,
    }));

    let loop756branch = branch_to(786, MergedBranch);
    let blocks756 = Box::new(Loop(LoopBlock {
        loop_id: loop756id,
        inner: Box::new(Simple(SimpleBlock {
            label: 756,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(762, Simple(SimpleBlock {
                        label: 762,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(777, end_node(777, Some(branch_to(756, LoopContinue(loop756id))))),
                            ],
                        }))),
                        branches: loop756branch.clone(),
                        next: None,
                    })),
                ],
            }))),
            branches: loop756branch.clone(),
            next: Some(blocks786),
        })),
        next: None,
    }));

    let blocks727 = Box::new(Simple(SimpleBlock {
        label: 727,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: loop749id,
            inner: Box::new(Simple(SimpleBlock {
                label: 749,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled(756, *blocks756),
                    ],
                }))),
                branches: branch_to(959, MergedBranch),
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 959,
                    immediate: Some(loop990),
                    branches: FnvHashMap::default(),
                    next: None,
                }))),
            })),
            next: None
        }))),
        branches: FnvHashMap::default(),
        next: None,
    }));

    let result727 = reloop(input727, 727);
    assert_eq!(result727, blocks727);
}