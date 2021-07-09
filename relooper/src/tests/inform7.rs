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

// Part of function 461194 of Aotearoa
#[test]
fn aotearoa_461194() {
    let input461678 = vec![
        (461678, vec![461699, 461723]),
        (461699, vec![461708, 461723]),
        (461708, vec![461717, 461723]),
        (461717, vec![461723, 461740]),
        (461723, vec![461733, 461736]),
        (461733, vec![]),
        (461736, vec![461945]),
        (461740, vec![461749, 461759]),
        (461749, vec![461759, 461890]),
        (461759, vec![461779, 461832]),
        (461779, vec![461798, 461807]),
        (461798, vec![461807, 461814]),
        (461807, vec![461832]),
        (461814, vec![461823, 461829]),
        (461823, vec![461832]),
        (461829, vec![]),
        (461832, vec![461838, 461841]),
        (461838, vec![]),
        (461841, vec![461850, 461871]),
        (461850, vec![461867, 461871]),
        (461867, vec![461871]),
        (461871, vec![461945]),
        (461890, vec![461920, 461926]),
        (461920, vec![461926, 461929]),
        (461926, vec![]),
        (461929, vec![461945]),
        (461945, vec![]),
    ];

    let block461678 = Box::new(Simple(SimpleBlock {
        label: 461678,
        immediate: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled(461699, Simple(SimpleBlock {
                    label: 461699,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(461708, Simple(SimpleBlock {
                                label: 461708,
                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                    handled: vec![
                                        basic_handled(461717, Simple(SimpleBlock {
                                            label: 461717,
                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                handled: vec![
                                                    basic_handled(461740, Simple(SimpleBlock {
                                                        label: 461740,
                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                            handled: vec![
                                                                basic_handled(461749, Simple(SimpleBlock {
                                                                    label: 461749,
                                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                        handled: vec![
                                                                            basic_handled(461890, Simple(SimpleBlock {
                                                                                label: 461890,
                                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                    handled: vec![
                                                                                        basic_handled(461920, Simple(SimpleBlock {
                                                                                            label: 461920,
                                                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                                handled: vec![
                                                                                                    basic_handled(461929, end_node(461929, Some(branch_to(461945, MergedBranchIntoMulti)))),
                                                                                                ],
                                                                                            }))),
                                                                                            branches: branch_to(461926, MergedBranchIntoMulti),
                                                                                            next: None,
                                                                                        })),
                                                                                    ],
                                                                                }))),
                                                                                branches: branch_to(461926, MergedBranchIntoMulti),
                                                                                next: Some(Box::new(Multiple(MultipleBlock {
                                                                                    handled: vec![
                                                                                        basic_handled(461926, end_node(461926, None)),
                                                                                    ],
                                                                                })))
                                                                            })),
                                                                        ],
                                                                    }))),
                                                                    branches: branch_to(461759, MergedBranchIntoMulti),
                                                                    next: None,
                                                                })),
                                                            ],
                                                        }))),
                                                        branches: branch_to(461759, MergedBranchIntoMulti),
                                                        next: Some(Box::new(Multiple(MultipleBlock {
                                                            handled: vec![
                                                                basic_handled(461759, Simple(SimpleBlock {
                                                                    label: 461759,
                                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                        handled: vec![
                                                                            basic_handled(461779, Simple(SimpleBlock {
                                                                                label: 461779,
                                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                    handled: vec![
                                                                                        basic_handled(461798, Simple(SimpleBlock {
                                                                                            label: 461798,
                                                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                                handled: vec![
                                                                                                    basic_handled(461814, Simple(SimpleBlock {
                                                                                                        label: 461814,
                                                                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                                            handled: vec![
                                                                                                                basic_handled(461823, end_node(461823, Some(branch_to(461832, MergedBranchIntoMulti)))),
                                                                                                                basic_handled(461829, end_node(461829, None)),
                                                                                                            ],
                                                                                                        }))),
                                                                                                        branches: FnvHashMap::default(),
                                                                                                        next: None,
                                                                                                    })),
                                                                                                ],
                                                                                            }))),
                                                                                            branches: branch_to(461807, MergedBranchIntoMulti),
                                                                                            next: None,
                                                                                        })),
                                                                                    ],
                                                                                }))),
                                                                                branches: branch_to(461807, MergedBranchIntoMulti),
                                                                                next: Some(Box::new(Multiple(MultipleBlock {
                                                                                    handled: vec![
                                                                                        basic_handled(461807, end_node(461807, Some(branch_to(461832, MergedBranchIntoMulti)))),
                                                                                    ],
                                                                                }))),
                                                                            })),
                                                                        ],
                                                                    }))),
                                                                    branches: branch_to(461832, MergedBranchIntoMulti),
                                                                    next: Some(Box::new(Multiple(MultipleBlock {
                                                                        handled: vec![
                                                                            basic_handled(461832, Simple(SimpleBlock {
                                                                                label: 461832,
                                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                    handled: vec![
                                                                                        basic_handled(461838, end_node(461838, None)),
                                                                                        basic_handled(461841, Simple(SimpleBlock {
                                                                                            label: 461841,
                                                                                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                                handled: vec![
                                                                                                    basic_handled(461850, Simple(SimpleBlock {
                                                                                                        label: 461850,
                                                                                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                                                            handled: vec![
                                                                                                                basic_handled(461867, end_node(461867, Some(branch_to(461871, MergedBranch)))),
                                                                                                            ],
                                                                                                        }))),
                                                                                                        branches: branch_to(461871, MergedBranch),
                                                                                                        next: None,
                                                                                                    })),
                                                                                                ],
                                                                                            }))),
                                                                                            branches: branch_to(461871, MergedBranch),
                                                                                            next: Some(Box::new(end_node(461871, Some(branch_to(461945, MergedBranchIntoMulti))))),
                                                                                        })),
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
                                                    })),
                                                ],
                                            }))),
                                            branches: branch_to(461723, MergedBranchIntoMulti),
                                            next: None,
                                        })),
                                    ],
                                }))),
                                branches: branch_to(461723, MergedBranchIntoMulti),
                                next: None,
                            })),
                        ],
                    }))),
                    branches: branch_to(461723, MergedBranchIntoMulti),
                    next: None,
                })),
            ],
        }))),
        branches: branch_to(461723, MergedBranchIntoMulti),
        next: Some(Box::new(Multiple(MultipleBlock {
            handled: vec![
                basic_handled_without_break(461723, Simple(SimpleBlock {
                    label: 461723,
                    immediate: Some(Box::new(Multiple(MultipleBlock {
                        handled: vec![
                            basic_handled(461733, end_node(461733, None)),
                            basic_handled(461736, end_node(461736, Some(branch_to(461945, SwitchFallThrough)))),
                        ],
                    }))),
                    branches: FnvHashMap::default(),
                    next: None,
                })),
                basic_handled(461945, end_node(461945, None)),
            ],
        }))),
    }));

    let result = reloop(input461678, 461678);
    assert_eq!(result, block461678);
}

// A simplified version of function 694077 of Aotearoa (something to do with regexes I think)
#[test]
fn aotearoa_694077() {
    let input694082 = vec![
        (694082, vec![694086]),
        (694086, vec![694093, 694327]),
        (694093, vec![694111, 694257]),
        (694111, vec![694122, 694257]),
        (694122, vec![694147, 694153]),
        (694147, vec![694153, 694167]),
        (694153, vec![694160, 694257]),
        (694160, vec![694167, 694257]),
        (694167, vec![694251, 694257]),
        (694251, vec![]),
        (694257, vec![694267, 694317]),
        (694267, vec![694286, 694317]),
        (694286, vec![694305, 694311]),
        (694305, vec![]),
        (694311, vec![694317]),
        (694317, vec![694086]),
        (694327, vec![]),
    ];

    let loop694085id = 0;

    let block694082 = Box::new(Simple(SimpleBlock {
        label: 694082,
        immediate: Some(Box::new(Loop(LoopBlock {
            loop_id: loop694085id,
            inner: Box::new(Simple(SimpleBlock {
                label: 694086,
                immediate: Some(Box::new(Multiple(MultipleBlock {
                    handled: vec![
                        basic_handled(694093, Simple(SimpleBlock {
                            label: 694093,
                            immediate: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled(694111, Simple(SimpleBlock {
                                        label: 694111,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(694122, Simple(SimpleBlock {
                                                    label: 694122,
                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                        handled: vec![
                                                            basic_handled(694147, end_node(694147, Some(FnvHashMap::from_iter(vec![
                                                                (694153, MergedBranchIntoMulti),
                                                                (694167, MergedBranchIntoMulti),
                                                            ])))),
                                                        ],
                                                    }))),
                                                    branches: branch_to(694153, MergedBranchIntoMulti),
                                                    next: Some(Box::new(Multiple(MultipleBlock {
                                                        handled: vec![
                                                            basic_handled_without_break(694153, Simple(SimpleBlock {
                                                                label: 694153,
                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                    handled: vec![
                                                                        basic_handled(694160, end_node(694160, Some(FnvHashMap::from_iter(vec![
                                                                            (694167, SwitchFallThrough),
                                                                            (694257, SetLabelAndBreak),
                                                                        ])))),
                                                                    ],
                                                                }))),
                                                                branches: branch_to(694257, SetLabelAndBreak),
                                                                next: None,
                                                            })),
                                                            basic_handled(694167, Simple(SimpleBlock {
                                                                label: 694167,
                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                    handled: vec![
                                                                        basic_handled(694251, end_node(694251, None)),
                                                                    ],
                                                                }))),
                                                                branches: branch_to(694257, MergedBranchIntoMulti),
                                                                next: None,
                                                            })),
                                                        ],
                                                    }))),
                                                })),
                                            ],
                                        }))),
                                        branches: branch_to(694257, MergedBranchIntoMulti),
                                        next: None,
                                    })),
                                ],
                            }))),
                            branches: branch_to(694257, MergedBranchIntoMulti),
                            next: Some(Box::new(Multiple(MultipleBlock {
                                handled: vec![
                                    basic_handled(694257, Simple(SimpleBlock {
                                        label: 694257,
                                        immediate: Some(Box::new(Multiple(MultipleBlock {
                                            handled: vec![
                                                basic_handled(694267, Simple(SimpleBlock {
                                                    label: 694267,
                                                    immediate: Some(Box::new(Multiple(MultipleBlock {
                                                        handled: vec![
                                                            basic_handled(694286, Simple(SimpleBlock {
                                                                label: 694286,
                                                                immediate: Some(Box::new(Multiple(MultipleBlock {
                                                                    handled: vec![
                                                                        basic_handled(694305, end_node(694305, None)),
                                                                        basic_handled(694311, end_node(694311, Some(branch_to(694317, MergedBranch)))),
                                                                    ],
                                                                }))),
                                                                branches: FnvHashMap::default(),
                                                                next: None,
                                                            })),
                                                        ],
                                                    }))),
                                                    branches: branch_to(694317, MergedBranch),
                                                    next: None,
                                                })),
                                            ],
                                        }))),
                                        branches: branch_to(694317, MergedBranch),
                                        next: Some(Box::new(end_node(694317, Some(branch_to(694086, LoopContinue(loop694085id)))))),
                                    })),
                                ],
                            }))),
                        })),
                        basic_handled(694327, end_node(694327, None)),
                    ],
                }))),
                branches: FnvHashMap::default(),
                next: None,
            })),
            next: None,
        }))),
        branches: FnvHashMap::default(),
        next: None,
    }));

    let result = reloop(input694082, 694082);
    assert_eq!(result, block694082);
}