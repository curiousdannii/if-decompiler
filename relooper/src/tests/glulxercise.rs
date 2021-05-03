/*

Tests from Glulxercise
======================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use super::*;

// Tokenise__
#[test]
fn test_tokenise() {
    let input727 = make_btree(hashmap!{
        727 => vec![749],
        749 => vec![756, 959],
        756 => vec![762, 786],
        762 => vec![777, 786],
        777 => vec![756],
        786 => vec![792, 796],
        792 => vec![959],
        796 => vec![819, 831],
        819 => vec![825, 831],
        825 => vec![831, 840],
        831 => vec![892],
        840 => vec![846, 892],
        846 => vec![865, 892],
        865 => vec![871, 892],
        871 => vec![877, 892],
        877 => vec![883, 892],
        883 => vec![840],
        892 => vec![952, 955],
        952 => vec![959],
        955 => vec![749],
        959 => vec![990],
        990 => vec![997, 1254],
        997 => vec![1041, 1045],
        1041 => vec![1045],
        1045 => vec![1054],
        1054 => vec![1060, 1139],
        1060 => vec![1089, 1095],
        1089 => vec![1095, 1122],
        1095 => vec![1130],
        1122 => vec![1130],
        1130 => vec![1054],
        1139 => vec![1145, 1198],
        1145 => vec![1150, 1156],
        1150 => vec![1156, 1181],
        1156 => vec![1189],
        1181 => vec![1189],
        1189 => vec![1139],
        1198 => vec![990],
        1254 => vec![],
    });

    let loop749id = 1;
    let loop756id = 5;
    let loop792id = 6;
    let loop831id = 7;
    let loop840id = 4;
    let loop990id = 0;
    let loop1054id_a = 3;
    let loop1054id_b = 9;
    let loop1139id_a = 2;
    let loop1139id_b = 8;

    let loop1139 = Box::new(Loop(LoopBlock {
        loop_id: loop1139id_a,
        inner: Box::new(Simple(SimpleBlock {
            label: 1139,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    HandledBlock {
                        labels: vec![1145],
                        inner: Box::new(Simple(SimpleBlock {
                            label: 1145,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    HandledBlock {
                                        labels: vec![1150],
                                        inner: Box::new(Simple(SimpleBlock {
                                            label: 1150,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    HandledBlock {
                                                        labels: vec![1181],
                                                        inner: end_node(1181, Some(branch_to(1189, MergedBranchIntoMulti))),
                                                    },
                                                ],
                                            }))),
                                            branches: branch_to(1156, MergedBranchIntoMulti),
                                            next: None,
                                        })),
                                    },
                                ],
                            }))),
                            branches: branch_to(1156, MergedBranchIntoMulti),
                            next: Some(Box::new(Loop(LoopBlock {
                                loop_id: loop1139id_b,
                                inner: Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        HandledBlock {
                                            labels: vec![1156],
                                            inner: end_node(1156, Some(branch_to(1189, LoopContinueIntoMulti(loop1139id_b)))),
                                        },
                                        HandledBlock {
                                            labels: vec![1189],
                                            inner: end_node(1189, Some(branch_to(1139, LoopContinue(loop1139id_a)))),
                                        },
                                    ],
                                })),
                                next: None,
                            }))),
                        })),
                    },
                ],
            }))),
            branches: branch_to(1198, LoopBreak(loop1139id_a)),
            next: None
        })),
        next: None,
    }));

    let loop1054 = Box::new(Loop(LoopBlock {
        loop_id: loop1054id_a,
        inner: Box::new(Simple(SimpleBlock {
            label: 1054,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    HandledBlock {
                        labels: vec![1060],
                        inner: Box::new(Simple(SimpleBlock {
                            label: 1060,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    HandledBlock {
                                        labels: vec![1089],
                                        inner: Box::new(Simple(SimpleBlock {
                                            label: 1089,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    HandledBlock {
                                                        labels: vec![1122],
                                                        inner: end_node(1122, Some(branch_to(1130, MergedBranchIntoMulti))),
                                                    },
                                                ],
                                            }))),
                                            branches: branch_to(1095, MergedBranchIntoMulti),
                                            next: None,
                                        })),
                                    },
                                ],
                            }))),
                            branches: branch_to(1095, MergedBranchIntoMulti),
                            next: Some(Box::new(Loop(LoopBlock {
                                loop_id: loop1054id_b,
                                inner: Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        HandledBlock {
                                            labels: vec![1095],
                                            inner: end_node(1095, Some(branch_to(1130, LoopContinueIntoMulti(loop1054id_b)))),
                                        },
                                        HandledBlock {
                                            labels: vec![1130],
                                            inner: end_node(1130, Some(branch_to(1054, LoopContinue(loop1054id_a)))),
                                        },
                                    ],
                                })),
                                next: None,
                            }))),
                        })),
                    },
                ],
            }))),
            branches: branch_to(1139, LoopBreak(loop1054id_a)),
            next: Some(end_node(1198, Some(branch_to(990, LoopContinue(loop990id))))),
        })),
        next: None,
    }));

    let loop990 = Box::new(Loop(LoopBlock {
        loop_id: loop990id,
        inner: Box::new(Simple(SimpleBlock {
            label: 990,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    HandledBlock {
                        labels: vec![997],
                        inner: Box::new(Simple(SimpleBlock {
                            label: 997,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    HandledBlock {
                                        labels: vec![1041],
                                        inner: end_node(1041, Some(branch_to(1045, MergedBranch))),
                                    },
                                ],
                            }))),
                            branches: branch_to(1045, MergedBranch),
                            next: Some(Box::new(Simple(SimpleBlock {
                                label: 1045,
                                immediate: Some(loop1054),
                                branches: FnvHashMap::default(),
                                next: Some(loop1139),
                            }))),
                        }))
                    },
                ],
            }))),
            branches: branch_to(1254, LoopBreak(loop990id)),
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
                    HandledBlock {
                        labels: vec![846],
                        inner: Box::new(Simple(SimpleBlock {
                            label: 846,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    HandledBlock {
                                        labels: vec![865],
                                        inner: Box::new(Simple(SimpleBlock {
                                            label: 865,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    HandledBlock {
                                                        labels: vec![871],
                                                        inner: Box::new(Simple(SimpleBlock {
                                                            label: 871,
                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                handled: vec![
                                                                    HandledBlock {
                                                                        labels: vec![877],
                                                                        inner: Box::new(Simple(SimpleBlock {
                                                                            label: 877,
                                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                handled: vec![
                                                                                    HandledBlock {
                                                                                        labels: vec![883],
                                                                                        inner: end_node(883, Some(branch_to(840, LoopContinue(loop840id)))),
                                                                                    },
                                                                                ],
                                                                            }))),
                                                                            branches: loop840branch.clone(),
                                                                            next: None,
                                                                        })),
                                                                    },
                                                                ],
                                                            }))),
                                                            branches: loop840branch.clone(),
                                                            next: None,
                                                        })),
                                                    },
                                                ],
                                            }))),
                                            branches: loop840branch.clone(),
                                            next: None,
                                        })),
                                    },
                                ],
                            }))),
                            branches: loop840branch.clone(),
                            next: None,
                        })),
                    },
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
                HandledBlock {
                    labels: vec![796],
                    inner: Box::new(Simple(SimpleBlock {
                        label: 796,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                HandledBlock {
                                    labels: vec![819],
                                    inner: Box::new(Simple(SimpleBlock {
                                        label: 819,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                HandledBlock {
                                                    labels: vec![825],
                                                    inner: Box::new(Simple(SimpleBlock {
                                                        label: 825,
                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                            handled: vec![
                                                                HandledBlock {
                                                                    labels: vec![840],
                                                                    inner: blocks840,
                                                                },
                                                            ],
                                                        }))),
                                                        branches: branch_to(831, MergedBranchIntoMulti),
                                                        next: None,
                                                    })),
                                                },
                                            ],
                                        }))),
                                        branches: branch_to(831, MergedBranchIntoMulti),
                                        next: None,
                                    })),
                                },
                            ],
                        }))),
                        branches: branch_to(831, MergedBranchIntoMulti),
                        next: Some(Box::new(Loop(LoopBlock {
                            loop_id: loop831id,
                            inner: Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    HandledBlock {
                                        labels: vec![831],
                                        inner: end_node(831, Some(branch_to(892, LoopContinueIntoMulti(loop831id)))),
                                    },
                                    HandledBlock {
                                        labels: vec![892],
                                        inner: Box::new(Simple(SimpleBlock {
                                            label: 892,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    HandledBlock {
                                                        labels: vec![952],
                                                        inner: end_node(952, Some(branch_to(959, LoopBreakIntoMulti(loop756id)))),
                                                    },
                                                    HandledBlock {
                                                        labels: vec![955],
                                                        inner: end_node(955, Some(branch_to(749, LoopContinue(loop749id)))),
                                                    },
                                                ],
                                            }))),
                                            branches: FnvHashMap::default(),
                                            next: None,
                                        })),
                                    },
                                ],
                            })),
                            next: None,
                        }))),
                    })),
                },
            ],
        }))),
        branches: branch_to(792, LoopBreakIntoMulti(loop749id)),
        next: None,
    }));

    let loop756branch = branch_to(786, MergedBranch);
    let blocks756 = Box::new(Loop(LoopBlock {
        loop_id: loop756id,
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
                                        inner: end_node(777, Some(branch_to(756, LoopContinue(loop756id)))),
                                    },
                                ],
                            }))),
                            branches: loop756branch.clone(),
                            next: None,
                        })),
                    },
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
                        HandledBlock {
                            labels: vec![756],
                            inner: blocks756,
                        },
                    ],
                }))),
                branches: branch_to(959, MergedBranchIntoMulti),
                next: Some(Box::new(Loop(LoopBlock {
                    loop_id: loop792id,
                    inner: Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            HandledBlock {
                                labels: vec![792],
                                inner: end_node(792, Some(branch_to(959, LoopContinueIntoMulti(loop792id)))),
                            },
                            HandledBlock {
                                labels: vec![959],
                                inner: Box::new(Simple(SimpleBlock {
                                    label: 959,
                                    immediate: Some(loop990),
                                    branches: FnvHashMap::default(),
                                    next: Some(end_node(1254, None)),
                                })),
                            },
                        ],
                    })),
                    next: None,
                }))),
            })),
            next: None,
        }))),
        branches: FnvHashMap::default(),
        next: None,
    }));

    let result727 = reloop(input727, 727);
    assert_eq!(result727, blocks727);
}