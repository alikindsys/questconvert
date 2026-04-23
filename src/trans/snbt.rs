use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use regex::Regex;

use crate::formats::ftbquests;

pub fn fixup_ftb_snbt(
    buf: &str, comma_places: &Regex, trailing_commas: &Regex, remove_last_comma: &Regex,
) -> String {
    let mut sbuilder = String::new();

    for line in buf.lines() {
        let new = comma_places.replace_all(line, "$0,");
        sbuilder.push_str(&new);
        sbuilder.push('\n');
    }

    let new = trailing_commas.replace_all(&sbuilder, "$1$2");

    remove_last_comma.replace_all(&new, "}").to_string()
}

impl From<&NbtCompound> for ftbquests::Metadata {
    fn from(val: &NbtCompound) -> Self {
        Self {
            default_autoclaim_rewards: val
                .get::<_, &str>("default_autoclaim_rewards")
                .unwrap()
                .to_string(),
            default_consume_items: val.get::<_, &str>("default_consume_items").unwrap() == "true",
            default_quest_disable_jei: val.get::<_, &str>("default_quest_disable_jei").unwrap() ==
                "true",
            default_quest_shape: val
                .get::<_, &str>("default_quest_shape")
                .unwrap()
                .to_string(),
            default_reward_team: val.get::<_, &str>("default_reward_team").unwrap() == "true",
            detection_delay: val.get::<_, i32>("detection_delay").unwrap(),
            disable_gui: val.get::<_, &str>("disable_gui").unwrap() == "true",
            emergency_items_cooldown: val.get::<_, i32>("emergency_items_cooldown").unwrap(),
            grid_scale: val.get::<_, f64>("grid_scale").unwrap(),
            icon: val.get::<_, &str>("icon").unwrap().to_string(),
            lock_message: val.get::<_, &str>("lock_message").unwrap().to_string(),
            pause_game: val.get::<_, &str>("pause_game").unwrap() == "true",
            progression_mode: val.get::<_, &str>("progression_mode").unwrap().to_string(),
            title: val.get::<_, &str>("title").unwrap().to_string(),
            version: val.get::<_, i32>("version").unwrap(),
            loot_crate_no_drop: ftbquests::LootCrateNoDrop {
                boss: val
                    .get::<_, &NbtCompound>("loot_crate_no_drop")
                    .unwrap()
                    .get::<_, i32>("boss")
                    .unwrap(),
                monster: val
                    .get::<_, &NbtCompound>("loot_crate_no_drop")
                    .unwrap()
                    .get::<_, i32>("monster")
                    .unwrap(),
                passive: val
                    .get::<_, &NbtCompound>("loot_crate_no_drop")
                    .unwrap()
                    .get::<_, i32>("passive")
                    .unwrap(),
            },
        }
    }
}

impl From<&NbtCompound> for ftbquests::ChapterGroups {
    fn from(val: &NbtCompound) -> Self {
        let mut chapter_groups = vec![];
        for group in val
            .get::<_, &NbtList>("chapter_groups")
            .unwrap()
            .iter_map::<&NbtCompound>()
        {
            let igroup = group.unwrap();
            chapter_groups.push(ftbquests::Group {
                id: igroup.get::<_, &str>("id").unwrap().to_string(),
                title: igroup.get::<_, &str>("title").unwrap().to_string(),
            });
        }
        ftbquests::ChapterGroups { chapter_groups }
    }
}

impl From<&NbtCompound> for ftbquests::Page {
    fn from(val: &NbtCompound) -> Self {
        Self {
            id: val.get::<_, &str>("id").unwrap().to_string(),
            title: val.get::<_, &str>("title").unwrap().to_string(),
            order_index: val.get::<_, i32>("order_index").unwrap(),
            default_hide_dependency_lines: val
                .get::<_, &str>("default_hide_dependency_lines")
                .unwrap() ==
                "true",
            default_quest_shape: val
                .get::<_, &str>("default_quest_shape")
                .unwrap()
                .to_string(),
            filename: val.get::<_, &str>("filename").unwrap().to_string(),
            group: val.get::<_, &str>("group").unwrap().to_string(),
            icon: val.get::<_, &NbtTag>("icon").ok().map(|t| t.into()),
            quest_links: val
                .get::<_, &NbtList>("quest_links")
                .unwrap()
                .iter_map::<&str>()
                .map(|s| s.unwrap_or_default().to_string())
                .collect(),
            quests: val
                .get::<_, &NbtList>("quests")
                .unwrap()
                .iter_map::<&NbtCompound>()
                .map(|c| c.unwrap().into())
                .collect(), // TODO
        }
    }
}

impl From<&NbtCompound> for ftbquests::Quest {
    fn from(val: &NbtCompound) -> Self {
        let tasks: Vec<_> = val
            .get::<_, &NbtList>("tasks")
            .unwrap()
            .iter_map::<&NbtCompound>()
            .map(|c| c.unwrap().try_into())
            .collect();
        tasks.iter().filter(|it| it.is_err()).for_each(|it| {
            if let Some(e) = it.as_ref().err() {
                println!("Failed to parse task: {}", e)
            }
        });
        let known_types = tasks.into_iter().filter_map(|r| r.ok()).collect();

        let rewards: Vec<_> = val
            .get::<_, &NbtList>("rewards")
            .ok()
            .map(|it| it.iter_map::<&NbtCompound>())
            .map(|it| it.map(|c| c.unwrap().try_into()).collect())
            .unwrap_or_default();
        rewards.iter().filter(|it| it.is_err()).for_each(|it| {
            if let Some(e) = it.as_ref().err() {
                println!("Failed to parse reward: {}", e)
            }
        });
        let known_rewards = rewards.into_iter().filter_map(|r| r.ok()).collect();

        Self {
            id: val.get::<_, &str>("id").unwrap().to_string(),
            description: val
                .get::<_, &NbtList>("description")
                .ok()
                .map(|l| {
                    l.iter_map::<&str>()
                        .map(|s| s.unwrap_or_default().to_string())
                        .collect()
                })
                .unwrap_or_default(),
            subtitle: val.get::<_, &str>("subtitle").ok().map(|s| s.to_string()),
            tasks: known_types, // TODO
            title: val.get::<_, &str>("title").ok().map(|s| s.to_string()),
            x: val.get::<_, f64>("x").unwrap(),
            y: val.get::<_, f64>("y").unwrap(),
            dependencies: val
                .get::<_, &NbtList>("dependencies")
                .ok()
                .map(|l| {
                    l.iter_map::<&str>()
                        .map(|s| s.unwrap_or_default().to_string())
                        .collect()
                })
                .unwrap_or_default(),
            rewards: known_rewards, // TODO
            shape: val.get::<_, &str>("shape").ok().map(|s| s.to_string()),
            size: val.get::<_, f64>("size").ok(),
            hide_dependency_lines: val
                .get::<_, &str>("hide_dependency_lines")
                .ok()
                .map(|s| s == "true")
                .unwrap_or(false),
            icon: val.get::<_, &str>("icon").ok().map(|s| s.to_string()),
        }
    }
}

impl TryFrom<&NbtCompound> for ftbquests::RewardRef {
    type Error = String;

    fn try_from(value: &NbtCompound) -> Result<Self, String> {
        let reward_type = value.get::<_, &str>("type").unwrap();
        match reward_type {
            "item" => Ok(Self::Item {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                item: value.get::<_, &NbtTag>("item").unwrap().into(),
                icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
            }),
            "xp_levels" => Ok(Self::XpLevels {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                xp_levels: value.get::<_, i32>("xp_levels").unwrap(),
            }),
            "xp" => Ok(Self::Xp {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                xp: value.get::<_, i32>("xp").unwrap(),
            }),
            "choice" => Ok(Self::Choice {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                table_id: value.get::<_, i64>("table_id").ok(),
                exclude_from_claim_all: value.get::<_, &str>("exclude_from_claim_all").unwrap() ==
                    "true",
                icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
            }),
            "loot" => Ok(Self::Loot {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                table_id: value.get::<_, i64>("table_id").unwrap(),
                exclude_from_claim_all: value.get::<_, &str>("exclude_from_claim_all").unwrap() ==
                    "true",
                icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
            }),
            "random" => Ok(Self::Random {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                table_id: value.get::<_, i64>("table_id").ok(),
                exclude_from_claim_all: value
                    .get::<_, &str>("exclude_from_claim_all")
                    .ok()
                    .unwrap_or_default() ==
                    "true",
                icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
            }),
            _ => Err(format!("Unknown reward type: {}", reward_type)),
        }
    }
}

impl TryFrom<&NbtCompound> for ftbquests::TaskRef {
    type Error = String;

    fn try_from(value: &NbtCompound) -> Result<Self, String> {
        let task_type = value.get::<_, &str>("type").unwrap();
        match task_type {
            "checkmark" => Ok(Self::Checkmark {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "item" => Ok(Self::Item {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                item: value.get::<_, &NbtTag>("item").unwrap().into(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "structure" => Ok(Self::Structure {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                structure: value.get::<_, &str>("structure").unwrap().to_string(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "dimension" => Ok(Self::Dimension {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                dimension: value.get::<_, &str>("dimension").unwrap().to_string(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "advancement" => Ok(Self::Advancement {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                advancement: value.get::<_, &str>("advancement").unwrap().to_string(),
                criterion: value
                    .get::<_, &str>("criterion")
                    .ok()
                    .map(|s| s.to_string()),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "kill" => Ok(Self::Kill {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                entity: value.get::<_, &str>("entity").unwrap().to_string(),
                value: value.get::<_, i32>("value").ok(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "observation" => Ok(Self::Observation {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                observe_type: num_traits::FromPrimitive::from_i32(
                    value.get::<_, i32>("observe_type").unwrap(),
                )
                .unwrap(),
                to_observe: value.get::<_, &str>("to_observe").unwrap().to_string(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            "biome" => Ok(Self::Biome {
                id: value.get::<_, &str>("id").unwrap().to_string(),
                biome: value.get::<_, &str>("biome").unwrap().to_string(),
                icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
            }),
            _ => Err(format!("Unknown task type: {}", task_type)),
        }
    }
}

impl From<&NbtTag> for ftbquests::Item {
    fn from(value: &NbtTag) -> Self {
        match value {
            NbtTag::String(s) => Self::Simple(s.to_string()),
            NbtTag::Compound(c) => Self::Compound(c.into()),
            _ => panic!("Invalid item tag type: expected string or compound, got {:?}", value),
        }
    }
}

impl From<&NbtCompound> for ftbquests::ItemRef {
    fn from(value: &NbtCompound) -> Self {
        Self {
            count: value.get::<_, i32>("count").ok(),
            id: value.get::<_, &str>("id").unwrap().to_string(),
            tag: value.get::<_, &NbtCompound>("tag").ok().map(|c| c.into()),
        }
    }
}

impl From<&NbtCompound> for ftbquests::ItemTag {
    fn from(value: &NbtCompound) -> Self {
        Self {
            color: value.get::<_, i32>("Color").ok(),
            loot: value.get::<_, &str>("Loot").ok().map(|s| s.to_string()),
            name: value.get::<_, &str>("Name").ok().map(|s| s.to_string()),
            _type: value.get::<_, &str>("Type").ok().map(|s| s.to_string()),
            damage: value.get::<_, i32>("Damage").ok(),
            icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
        }
    }
}

impl From<&NbtCompound> for ftbquests::LootTable {
    fn from(value: &NbtCompound) -> Self {
        Self {
            id: value.get::<_, &str>("id").unwrap().to_string(),
            loot_size: value.get::<_, i32>("loot_size").unwrap(),
            order_index: value.get::<_, i32>("order_index").unwrap(),
            rewards: value
                .get::<_, &NbtList>("rewards")
                .unwrap()
                .iter_map::<&NbtCompound>()
                .map(|c| c.unwrap().into())
                .collect(),
            title: value.get::<_, &str>("title").ok().map(|it| it.to_string()),
            use_title: value.get::<_, &str>("id").ok().map(|it| it == "true"),
        }
    }
}

impl From<&NbtCompound> for ftbquests::LootTableRewardRef {
    fn from(value: &NbtCompound) -> Self {
        match value.get::<_, &str>("type") {
            Ok(_) => Self::Other(value.try_into().unwrap()),
            Err(_) => Self::Item(value.into()),
        }
    }
}

impl TryFrom<&NbtCompound> for ftbquests::TypeDescriminatedLootTableRewardRef {
    type Error = String;

    fn try_from(value: &NbtCompound) -> Result<Self, Self::Error> {
        let reward_type = value.get::<_, &str>("type").unwrap();
        match reward_type {
            "command" => Ok(Self::Command {
                command: value.get::<_, &str>("command").unwrap().to_string(),
                elevate_perms: value.get::<_, &str>("elevate_perms").unwrap() == "true",
                silent: value.get::<_, &str>("silent").unwrap() == "true",
                weight: value.get::<_, f32>("weight").ok(),
            }),
            "xp_levels" => Ok(Self::XpLevels {
                xp_levels: value.get::<_, i32>("xp_levels").unwrap(),
            }),
            _ => Err(format!("Unknown reward type: {}", reward_type)),
        }
    }
}

impl From<&NbtCompound> for ftbquests::ItemReward {
    fn from(value: &NbtCompound) -> Self {
        Self {
            count: value.get::<_, i32>("count").ok(),
            item: value.get::<_, &NbtTag>("item").map(|it| it.into()).unwrap(),
            random_bonus: value.get::<_, i32>("random_bonus").ok(),
            weight: value.get::<_, f32>("weight").ok(),
        }
    }
}
