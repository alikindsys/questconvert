use std::collections::{HashMap, HashSet};

use bimap::BiHashMap;

use crate::formats::{
    ftbquests::{self, new_id},
    heracles,
};

pub trait TryFromWithId<T>: Sized {
    type Error;
    fn try_from_with_id(value: T, id: String) -> Result<Self, Self::Error>;
}

pub trait FromWithId<T>: Sized {
    fn from_with_id(value: T, id: String) -> Self;
}

impl ftbquests::State {
    pub fn from_heracles(mut state: heracles::State) -> Self {
        let _page_map: HashMap<String, HashSet<ftbquests::Quest>> = HashMap::new();
        let _quest_map: HashMap<String, ftbquests::Quest> = HashMap::new();
        let mut id_map: BiHashMap<String, String> = BiHashMap::new();

        // Lets traverse the tree of dependencies.
        for ele in state.quest_dependency_graph {
            match ele {
                Ok((key, deps)) => {
                    // Pre-generate all of the new keys for the dependencies.
                    // We don't actually need to load the contents of the dependencies until they actually appear.
                    let mut dependencies = vec![];

                    for dep in deps {
                        if !id_map.contains_left(&dep) {
                            id_map.insert(dep.clone(), new_id(&dep));
                            dependencies.push(new_id(&dep));
                        }
                    }

                    // Create and add the current task key.
                    let current_id = new_id(&key);
                    if !id_map.contains_left(&key) {
                        id_map.insert(key.clone(), current_id.clone());
                    }

                    // Apply transformation on the quest itself.
                    // We need to know where that quest links to, and as a result, which "page" it is in.
                    if let Some(hquest) = state.quests.remove(&key) {
                        if hquest.display.groups.len() <= 1 {
                            let fquest = ftbquests::Quest {
                                id: current_id,
                                description: hquest.display.description,
                                subtitle: hquest
                                    .display
                                    .subtitle
                                    .and_then(|s| s.text.or(s.translate)),
                                tasks: todo!(),
                                title: todo!(),
                                x: todo!(),
                                y: todo!(),
                                dependencies,
                                rewards: todo!(),
                                shape: todo!(),
                                size: todo!(),
                                hide_dependency_lines: todo!(),
                                icon: todo!(),
                            };
                        } else {
                            // HACK: Some quest may exist in multiple pages, so we will create distinct ids after
                            // HACK: the first page, and duplicate the quests but prune the rewards.
                        }
                    }
                }
                Err(_cycle) => {}
            };
        }

        unimplemented!()
    }
}

impl FromWithId<heracles::TaskRef> for ftbquests::TaskRef {
    fn from_with_id(value: heracles::TaskRef, id: String) -> Self {
        match value {
            heracles::TaskRef::Check { icon, .. } => {
                Self::Checkmark { id, icon: icon.map(|it| it.into()) }
            }
            heracles::TaskRef::Item { item, amount, icon, .. } => {
                let fitem: ftbquests::Item;
                if let Some(amount) = amount {
                    fitem = ftbquests::Item::Compound(ftbquests::ItemRef {
                        count: Some(amount),
                        id: item,
                        tag: None,
                    });
                } else {
                    fitem = ftbquests::Item::Simple(item);
                }

                Self::Item { id, item: fitem, icon: icon.map(|it| it.into()) }
            }
            // Fuck my life. Pre-processing these.
            heracles::TaskRef::Advancement { advancements, icon, .. } => {
                assert!(
                    advancements.len() == 1,
                    "Heracles' advancement task contained more then one advancement.
                    These should be pre-processed before submission to FromWithId."
                );
                Self::Advancement {
                    id,
                    advancement: advancements.first().unwrap().clone(),
                    criterion: None,
                    icon: icon.map(|it| it.into()),
                }
            }
            heracles::TaskRef::Structure { structures, icon, .. } => Self::Structure {
                id,
                structure: structures,
                icon: icon.map(|it| it.into()),
            },
            heracles::TaskRef::ChangeDimension { to, icon, .. } => {
                Self::Dimension { id, dimension: to, icon: icon.map(|it| it.into()) }
            }
            heracles::TaskRef::ItemInteraction { item, icon, .. } => Self::Item {
                id,
                item: ftbquests::Item::Simple(item),
                icon: icon.map(|it| it.into()),
            },
            heracles::TaskRef::BlockInteraction { block, icon, .. } => Self::Observation {
                id,
                observe_type: ftbquests::ObserveType::Block,
                to_observe: block,
                icon: icon.map(|it| it.into()),
            },
            heracles::TaskRef::KillEntity { entity, amount, icon, .. } => Self::Kill {
                id,
                entity: entity._type,
                value: amount,
                icon: icon.map(|it| it.into()),
            },
            heracles::TaskRef::Biome { biomes, icon, .. } => {
                Self::Biome { id, biome: biomes, icon: icon.map(|it| it.into()) }
            }
            heracles::TaskRef::Composite { tasks, amount, title, icon } => todo!(),
        }
    }
}

impl From<heracles::Icon> for ftbquests::Item {
    fn from(value: heracles::Icon) -> Self {
        value.item.into()
    }
}

impl From<heracles::TagItemRef> for ftbquests::Item {
    fn from(value: heracles::TagItemRef) -> Self {
        match value {
            heracles::TagItemRef::TagRef(s) => Self::Simple(s),
            heracles::TagItemRef::ItemRef(item_ref) => Self::Compound(ftbquests::ItemRef {
                count: item_ref.count,
                id: item_ref.id,
                tag: None,
            }),
        }
    }
}
