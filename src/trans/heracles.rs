use std::collections::{HashMap, HashSet};

use bimap::BiHashMap;

use crate::formats::{
    ftbquests::{self, new_id},
    heracles,
};

impl ftbquests::State {
    pub fn from_heracles(state: heracles::State) -> Self {
        let _page_map: HashMap<String, HashSet<ftbquests::Quest>> = HashMap::new();
        let _quest_map: HashMap<String, ftbquests::Quest> = HashMap::new();
        let mut id_map: BiHashMap<String, String> = BiHashMap::new();

        // Lets traverse the tree of dependencies.
        for ele in state.quest_dependency_graph {
            match ele {
                Ok((key, deps)) => {
                    // Pre-generate all of the new keys for the dependencies.
                    // We don't actually need to load the contents of the dependencies until they actually appear.
                    for dep in deps {
                        if !id_map.contains_left(&dep) {
                            id_map.insert(dep.clone(), new_id(dep));
                        }
                    }

                    // Apply transformation on the quest itself.
                    // We need to know where that quest links to, and as a result, which "page" it is in.
                    if let Some(_hquest) = state.quests.get(&key) {}
                }
                Err(_cycle) => {}
            };
        }

        for (filename, quest) in state.quests {
            let quest_id = new_id(filename.clone());

            let _fquest = ftbquests::Quest {
                id: quest_id.clone(),
                description: quest.display.description,
                subtitle: quest.display.subtitle.and_then(|s| s.text.or(s.translate)),
                tasks: todo!(),
                title: todo!(),
                x: todo!(),
                y: todo!(),
                dependencies: todo!(),
                rewards: todo!(),
                shape: todo!(),
                size: todo!(),
                hide_dependency_lines: todo!(),
                icon: todo!(),
            };

            for (key, inner) in quest.display.groups {
                id_map.insert(key.clone(), new_id(key.clone()));
                // We need to build a temporary map of which things go on each page
                // And since the key of the display group is what pageId they'll go to
            }
        }

        unimplemented!()
    }
}
