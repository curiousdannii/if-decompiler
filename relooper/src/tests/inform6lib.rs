/*

Tests from Inform6lib
=====================

Copyright (c) 2021 Dannii Willis
MIT licenced
https://github.com/curiousdannii/if-decompiler

*/

use super::*;

// ReviseMulti (6/10)
// This test case demonstrates a Loop(Multiple)
#[test]
fn revisemulti() {
    let input21109 = vec![
        (21109, vec![21116, 21123]),
        (21116, vec![21123, 21225]),
        (21123, vec![21130]),
        (21130, vec![21142, 21217]),
        (21142, vec![21149, 21162]),
        (21149, vec![21162, 21186]),
        (21162, vec![21169, 21208]),
        (21169, vec![21186, 21208]),
        (21186, vec![21208]),
        (21208, vec![21130]),
        (21217, vec![21225]),
        (21225, vec![21233, 21412]),
        (21233, vec![21244, 21412]),
        (21244, vec![21251]),
        (21251, vec![21263, 21310]),
        (21263, vec![21295, 21301]),
        (21295, vec![21301]),
        (21301, vec![21251]),
        (21310, vec![21317, 21322]),
        (21317, vec![21322, 21412]),
        (21322, vec![21329]),
        (21329, vec![21341, 21404]),
        (21341, vec![21373, 21395]),
        (21373, vec![21395]),
        (21395, vec![21329]),
        (21404, vec![21412]),
        (21412, vec![21424, 21427]),
        (21424, vec![]),
        (21427, vec![]),
    ];

    let loop21130id = 2;
    let loop21162id = 3;
    let loop21251id = 1;
    let loop21329id = 0;

    let loop21329 = Box::new(Loop(LoopBlock {
        loop_id: loop21329id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21329,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21341, Simple(SimpleBlock {
                        label: 21341,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21373, end_node(21373, Some(branch_to(21395, MergedBranch)))),
                            ],
                        }))),
                        branches: branch_to(21395, MergedBranch),
                        next: Some(Box::new(end_node(21395, Some(branch_to(21329, LoopContinue(loop21329id)))))),
                    })),
                    basic_handled(21404, end_node(21404, Some(branch_to(21412, LoopBreakIntoMulti(loop21251id))))),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    }));

    let loop21162 = Box::new(Loop(LoopBlock {
        loop_id: loop21162id,
        inner: Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled_without_break(21162, Simple(SimpleBlock {
                    label: 21162,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(21169, end_node(21169, Some(FnvHashMap::from_iter(vec![
                                (21186, LoopContinueIntoMulti(loop21162id)),
                                (21208, LoopContinueIntoMulti(loop21162id)),
                            ])))),
                        ],
                    }))),
                    branches: branch_to(21208, LoopContinueIntoMulti(loop21162id)),
                    next: None,
                })),
                basic_handled(21186, end_node(21186, Some(branch_to(21208, LoopContinueIntoMulti(loop21162id))))),
                basic_handled(21208, end_node(21208, Some(branch_to(21130, LoopContinue(loop21130id))))),
            ],
        })),
        next: None,
    }));

    let loop21130 = Box::new(Loop(LoopBlock {
        loop_id: loop21130id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21130,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21142, Simple(SimpleBlock {
                        label: 21142,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21149, end_node(21149, Some(FnvHashMap::from_iter(vec![
                                    (21162, MergedBranchIntoMulti),
                                    (21186, MergedBranchIntoMulti),
                                ])))),
                            ],
                        }))),
                        branches: branch_to(21162, MergedBranchIntoMulti),
                        next: Some(loop21162),
                    })),
                    basic_handled(21217, end_node(21217, Some(branch_to(21225, MergedBranch)))),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    }));

    let loop21251 = Box::new(Loop(LoopBlock {
        loop_id: loop21251id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21251,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21263, Simple(SimpleBlock {
                        label: 21263,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21295, end_node(21295, Some(branch_to(21301, MergedBranch)))),
                            ],
                        }))),
                        branches: branch_to(21301, MergedBranch),
                        next: Some(Box::new(end_node(21301, Some(branch_to(21251, LoopContinue(loop21251id)))))),
                    })),
                    basic_handled(21310, Simple(SimpleBlock {
                        label: 21310,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21317, end_node(21317, Some(FnvHashMap::from_iter(vec![
                                    (21322, MergedBranchIntoMulti),
                                    (21412, LoopBreakIntoMulti(loop21251id)),
                                ])))),
                            ],
                        }))),
                        branches: branch_to(21322, MergedBranchIntoMulti),
                        next: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21322, Simple(SimpleBlock {
                                    label: 21322,
                                    immediate: Some(loop21329),
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
        })),
        next: None,
    }));

    let block21109 = Box::new(Simple(SimpleBlock {
        label: 21109,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(21116, Simple(SimpleBlock {
                    label: 21116,
                    immediate: None,
                    branches: FnvHashMap::from_iter(vec![
                        (21123, MergedBranchIntoMulti),
                        (21225, MergedBranchIntoMulti),
                    ]),
                    next: None,
                })),
            ],
        }))),
        branches: branch_to(21123, MergedBranchIntoMulti),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled_without_break(21123, Simple(SimpleBlock {
                    label: 21123,
                    immediate: Some(loop21130),
                    branches: FnvHashMap::default(),
                    next: None,
                })),
                basic_handled(21225, Simple(SimpleBlock {
                    label: 21225,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(21233, Simple(SimpleBlock {
                                label: 21233,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(21244, Simple(SimpleBlock {
                                            label: 21244,
                                            immediate: Some(loop21251),
                                            branches: FnvHashMap::default(),
                                            next: None,
                                        })),
                                    ],
                                }))),
                                branches: branch_to(21412, MergedBranchIntoMulti),
                                next: None,
                            })),
                        ],
                    }))),
                    branches: branch_to(21412, MergedBranchIntoMulti),
                    next: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(21412, Simple(SimpleBlock {
                                label: 21412,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(21424, end_node(21424, None)),
                                        basic_handled(21427, end_node(21427, None)),
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
    }));

    let result = reloop(input21109, 21109);
    assert_eq!(result, block21109);
}

// ScoreMatchL (6/10)
#[test]
fn scorematchl() {
    let input21434 = vec![
        (21434, vec![21443, 21449]),
        (21443, vec![21449]),
        (21449, vec![21458, 21464]),
        (21458, vec![21464]),
        (21464, vec![21473, 21479]),
        (21473, vec![21479]),
        (21479, vec![21488, 21494]),
        (21488, vec![21494]),
        (21494, vec![21499, 21505]),
        (21499, vec![21505]),
        (21505, vec![21519, 21531]),
        (21519, vec![21525, 21531]),
        (21525, vec![21531, 21539]),
        (21531, vec![21539]),
        (21539, vec![21542]),
        (21542, vec![21550, 21870]),
        (21550, vec![21577, 21590]),
        (21577, vec![21584, 21590]),
        (21584, vec![21590]),
        (21590, vec![21599, 21612]),
        (21599, vec![21606, 21612]),
        (21606, vec![21612]),
        (21612, vec![21621, 21635]),
        (21621, vec![21629, 21635]),
        (21629, vec![21635]),
        (21635, vec![21644, 21658]),
        (21644, vec![21652, 21658]),
        (21652, vec![21658]),
        (21658, vec![21663, 21676]),
        (21663, vec![21670, 21676]),
        (21670, vec![21676]),
        (21676, vec![21682, 21695]),
        (21682, vec![21860]),
        (21695, vec![21706, 21710]),
        (21706, vec![21710]),
        (21710, vec![21717, 21726]),
        (21717, vec![21757]),
        (21726, vec![21733, 21742]),
        (21733, vec![21757]),
        (21742, vec![21751, 21757]),
        (21751, vec![21757]),
        (21757, vec![21771]),
        (21771, vec![21789, 21795]),
        (21789, vec![21795]),
        (21795, vec![21802, 21808]),
        (21802, vec![21808]),
        (21808, vec![21819]),
        (21819, vec![21834, 21840]),
        (21834, vec![21840]),
        (21840, vec![21860]),
        (21860, vec![21542]),
        (21870, vec![21873]),
        (21873, vec![21881, 21997]),
        (21881, vec![21894, 21987]),
        (21894, vec![21905, 21916]),
        (21905, vec![21987]),
        (21916, vec![21920]),
        (21920, vec![21927, 21976]),
        (21927, vec![21920]),
        (21976, vec![21881]),
        (21987, vec![21873]),
        (21997, vec![]),
    ];

    let loop21542id = 1;
    let loop21873id = 0;
    let loop21881id = 2;
    let loop21920id = 3;

    let loop21920 = Loop(LoopBlock {
        loop_id: loop21920id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21920,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21927, end_node(21927, Some(branch_to(21920, LoopContinue(loop21920id))))),
                    basic_handled(21976, end_node(21976, Some(branch_to(21881, LoopContinue(loop21881id))))),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    });

    let loop21881 = Loop(LoopBlock {
        loop_id: loop21881id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21881,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21894, Simple(SimpleBlock {
                        label: 21894,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21905, end_node(21905, Some(branch_to(21987, MergedBranch)))),
                                basic_handled(21916, Simple(SimpleBlock {
                                    label: 21916,
                                    immediate: Some(Box::new(loop21920)),
                                    branches: FnvHashMap::default(),
                                    next: None,
                                })),
                            ],
                        }))),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                ],
            }))),
            branches: branch_to(21987, MergedBranch),
            next: Some(Box::new(end_node(21987, Some(branch_to(21873, LoopContinue(loop21873id)))))),
        })),
        next: None,
    });

    let loop21873 = Loop(LoopBlock {
        loop_id: loop21873id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21873,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21881, loop21881),
                    basic_handled(21997, end_node(21997, None)),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    });

    let loop21542 = Loop(LoopBlock {
        loop_id: loop21542id,
        inner: Box::new(Simple(SimpleBlock {
            label: 21542,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21550, Simple(SimpleBlock {
                        label: 21550,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21577, Simple(SimpleBlock {
                                    label: 21577,
                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                        handled: vec![
                                            basic_handled(21584, end_node(21584, Some(branch_to(21590, MergedBranch)))),
                                        ],
                                    }))),
                                    branches: branch_to(21590, MergedBranch),
                                    next: None,
                                })),
                            ],
                        }))),
                        branches: branch_to(21590, MergedBranch),
                        next: Some(Box::new(Simple(SimpleBlock {
                            label: 21590,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled(21599, Simple(SimpleBlock {
                                        label: 21599,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(21606, end_node(21606, Some(branch_to(21612, MergedBranch)))),
                                            ],
                                        }))),
                                        branches: branch_to(21612, MergedBranch),
                                        next: None,
                                    })),
                                ],
                            }))),
                            branches: branch_to(21612, MergedBranch),
                            next: Some(Box::new(Simple(SimpleBlock {
                                label: 21612,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(21621, Simple(SimpleBlock {
                                            label: 21621,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    basic_handled(21629, end_node(21629, Some(branch_to(21635, MergedBranch)))),
                                                ],
                                            }))),
                                            branches: branch_to(21635, MergedBranch),
                                            next: None,
                                        })),
                                    ],
                                }))),
                                branches: branch_to(21635, MergedBranch),
                                next: Some(Box::new(Simple(SimpleBlock {
                                    label: 21635,
                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                        handled: vec![
                                            basic_handled(21644, Simple(SimpleBlock {
                                                label: 21644,
                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                    handled: vec![
                                                        basic_handled(21652, end_node(21652, Some(branch_to(21658, MergedBranch)))),
                                                    ],
                                                }))),
                                                branches: branch_to(21658, MergedBranch),
                                                next: None,
                                            })),
                                        ],
                                    }))),
                                    branches: branch_to(21658, MergedBranch),
                                    next: Some(Box::new(Simple(SimpleBlock {
                                        label: 21658,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(21663, Simple(SimpleBlock {
                                                    label: 21663,
                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                        handled: vec![
                                                            basic_handled(21670, end_node(21670, Some(branch_to(21676, MergedBranch)))),
                                                        ],
                                                    }))),
                                                    branches: branch_to(21676, MergedBranch),
                                                    next: None,
                                                })),
                                            ],
                                        }))),
                                        branches: branch_to(21676, MergedBranch),
                                        next: Some(Box::new(Simple(SimpleBlock {
                                            label: 21676,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    basic_handled(21682, end_node(21682, Some(branch_to(21860, MergedBranch)))),
                                                    basic_handled(21695, Simple(SimpleBlock {
                                                        label: 21695,
                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                            handled: vec![
                                                                basic_handled(21706, end_node(21706, Some(branch_to(21710, MergedBranch)))),
                                                            ],
                                                        }))),
                                                        branches: branch_to(21710, MergedBranch),
                                                        next: Some(Box::new(Simple(SimpleBlock {
                                                            label: 21710,
                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                handled: vec![
                                                                    basic_handled(21717, end_node(21717, Some(branch_to(21757, MergedBranch)))),
                                                                    basic_handled(21726, Simple(SimpleBlock {
                                                                        label: 21726,
                                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                            handled: vec![
                                                                                basic_handled(21733, end_node(21733, Some(branch_to(21757, MergedBranch)))),
                                                                                basic_handled(21742, Simple(SimpleBlock {
                                                                                    label: 21742,
                                                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                        handled: vec![
                                                                                            basic_handled(21751, end_node(21751, Some(branch_to(21757, MergedBranch)))),
                                                                                        ],
                                                                                    }))),
                                                                                    branches: branch_to(21757, MergedBranch),
                                                                                    next: None,
                                                                                })),
                                                                            ],
                                                                        }))),
                                                                        branches: FnvHashMap::default(),
                                                                        next: None,
                                                                    })),
                                                                ],
                                                            }))),
                                                            branches: FnvHashMap::default(),
                                                            next: Some(Box::new(Simple(SimpleBlock {
                                                                label: 21757,
                                                                immediate: Some(Box::new(Simple(SimpleBlock {
                                                                    label: 21771,
                                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                        handled: vec![
                                                                            basic_handled(21789, end_node(21789, Some(branch_to(21795, MergedBranch)))),
                                                                        ],
                                                                    }))),
                                                                    branches: branch_to(21795, MergedBranch),
                                                                    next: Some(Box::new(Simple(SimpleBlock {
                                                                        label: 21795,
                                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                            handled: vec![
                                                                                basic_handled(21802, end_node(21802, Some(branch_to(21808, MergedBranch)))),
                                                                            ],
                                                                        }))),
                                                                        branches: branch_to(21808, MergedBranch),
                                                                        next: Some(Box::new(Simple(SimpleBlock {
                                                                            label: 21808,
                                                                            immediate: Some(Box::new(Simple(SimpleBlock {
                                                                                label: 21819,
                                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                    handled: vec![
                                                                                        basic_handled(21834, end_node(21834, Some(branch_to(21840, MergedBranch)))),
                                                                                    ],
                                                                                }))),
                                                                                branches: branch_to(21840, MergedBranch),
                                                                                next: Some(Box::new(end_node(21840, Some(branch_to(21860, MergedBranch))))),
                                                                            }))),
                                                                            branches: FnvHashMap::default(),
                                                                            next: None,
                                                                        }))),
                                                                    }))),
                                                                }))),
                                                                branches: FnvHashMap::default(),
                                                                next: None,
                                                            }))),
                                                        }))),
                                                    })),
                                                ],
                                            }))),
                                            branches: FnvHashMap::default(),
                                            next: Some(Box::new(end_node(21860, Some(branch_to(21542, LoopContinue(loop21542id)))))),
                                        }))),
                                    }))),
                                }))),
                            }))),
                        }))),
                    })),
                    basic_handled(21870, Simple(SimpleBlock {
                        label: 21870,
                        immediate: Some(Box::new(loop21873)),
                        branches: FnvHashMap::default(),
                        next: None,
                    })),
                ],
            }))),
            branches: FnvHashMap::default(),
            next: None,
        })),
        next: None,
    });

    let block21434 = Box::new(Simple(SimpleBlock {
        label: 21434,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(21443, end_node(21443, Some(branch_to(21449, MergedBranch)))),
            ],
        }))),
        branches: branch_to(21449, MergedBranch),
        next: Some(Box::new(Simple(SimpleBlock {
            label: 21449,
            immediate: Some(Box::new(Multiple(MultipleBlock {
                handled: vec![
                    basic_handled(21458, end_node(21458, Some(branch_to(21464, MergedBranch)))),
                ],
            }))),
            branches: branch_to(21464, MergedBranch),
            next: Some(Box::new(Simple(SimpleBlock {
                label: 21464,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled(21473, end_node(21473, Some(branch_to(21479, MergedBranch)))),
                    ],
                }))),
                branches: branch_to(21479, MergedBranch),
                next: Some(Box::new(Simple(SimpleBlock {
                    label: 21479,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(21488, end_node(21488, Some(branch_to(21494, MergedBranch)))),
                        ],
                    }))),
                    branches: branch_to(21494, MergedBranch),
                    next: Some(Box::new(Simple(SimpleBlock {
                        label: 21494,
                        immediate: Some(Box::new(Multiple(MultipleBlock {
                            handled: vec![
                                basic_handled(21499, end_node(21499, Some(branch_to(21505, MergedBranch)))),
                            ],
                        }))),
                        branches: branch_to(21505, MergedBranch),
                        next: Some(Box::new(Simple(SimpleBlock {
                            label: 21505,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled(21519, Simple(SimpleBlock {
                                        label: 21519,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(21525, end_node(21525, Some(FnvHashMap::from_iter(vec![
                                                    (21531, MergedBranchIntoMulti),
                                                    (21539, MergedBranchIntoMulti),
                                                ])))),
                                            ],
                                        }))),
                                        branches: branch_to(21531, MergedBranchIntoMulti),
                                        next: None,
                                    })),
                                ],
                            }))),
                            branches: branch_to(21531, MergedBranchIntoMulti),
                            next: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled_without_break(21531, end_node(21531, Some(branch_to(21539, MergedBranch)))),
                                    basic_handled(21539, Simple(SimpleBlock {
                                        label: 21539,
                                        immediate: Some(Box::new(loop21542)),
                                        branches: FnvHashMap::default(),
                                        next: None,
                                    })),
                                ],
                            }))),
                        }))),
                    }))),
                }))),
            }))),
        }))),
    }));

    let result = reloop(input21434, 21434);
    assert_eq!(result, block21434);
}