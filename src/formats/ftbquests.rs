use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};
use topo_sort::TopoSort;

pub fn new_id(old: String) -> String {
    let mut s = DefaultHasher::new();
    old.hash(&mut s);
    format!("{:X}", s.finish())
}

pub struct State {
    pub metadata: Metadata,
    pub groups: Vec<Group>,
    pub pages: Vec<Page>,
    pub quest_dependency_graph: TopoSort<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterGroups {
    pub chapter_groups: Vec<Group>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub order_index: i32,
    pub default_hide_dependency_lines: bool,
    pub default_quest_shape: String,
    pub filename: String,
    pub group: String,
    pub icon: Option<Item>,
    // Quests which link to this page, but aren't actually on this page.
    pub quest_links: Vec<String>,
    pub quests: Vec<Quest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemRef {
    pub count: Option<i32>,
    pub id: String,
    pub tag: Option<ItemTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Item {
    Simple(String),
    Compound(ItemRef),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemTag {
    #[serde(rename = "Color")]
    pub color: Option<i32>,
    #[serde(rename = "Loot")]
    pub loot: Option<String>,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Type")]
    pub _type: Option<String>,
    #[serde(rename = "Damage")]
    pub damage: Option<i32>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub description: Vec<String>,
    pub subtitle: Option<String>,
    pub tasks: Vec<TaskRef>,
    pub title: Option<String>,
    pub x: f64,
    pub y: f64,
    pub dependencies: Vec<String>,
    pub rewards: Vec<RewardRef>,
    pub shape: Option<String>,
    pub size: Option<f64>,
    pub hide_dependency_lines: bool,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskRef {
    Checkmark {
        id: String,
        icon: Option<Item>,
    },
    Item {
        id: String,
        item: Item,
        icon: Option<Item>,
    },
    Structure {
        id: String,
        structure: String,
        icon: Option<Item>,
    },
    Dimension {
        id: String,
        dimension: String,
        icon: Option<Item>,
    },
    Advancement {
        id: String,
        advancement: String,
        criterion: Option<String>,
        icon: Option<Item>,
    },
    Kill {
        id: String,
        entity: String,
        value: Option<i32>,
        icon: Option<Item>,
    },
    Observation {
        id: String,
        observe_type: ObserveType,
        to_observe: String,
        icon: Option<Item>,
    },
    Biome {
        id: String,
        biome: String,
        icon: Option<Item>,
    }, // We also need some kind of task that lets us have an "one of" relationship
       // Since "composite" tasks in heracles can be used to require one of multiple tasks to be completed
       // and ftbquests has a way to represent that afaik, but I couldn't find it
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, FromPrimitive)]
#[serde(untagged)]
pub enum ObserveType {
    Block,
    BlockTag,
    BlockState,
    BlockEntity,
    BlockEntityType,
    EntityType,
    EntityTypeTag,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RewardRef {
    Item {
        id: String,
        item: Item,
        icon: Option<String>,
    },
    XpLevels {
        id: String,
        xp_levels: i32,
    },
    Xp {
        id: String,
        xp: i32,
    },
    Choice {
        id: String,
        table_id: Option<i64>,
        exclude_from_claim_all: bool,
        icon: Option<String>,
    },
    Loot {
        id: String,
        table_id: i64,
        exclude_from_claim_all: bool,
        icon: Option<String>,
    },
    Random {
        id: String,
        table_id: Option<i64>,
        exclude_from_claim_all: bool,
        icon: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LootTable {
    pub id: String,
    pub loot_size: i32,
    pub order_index: i32,
    pub rewards: Vec<LootTableRewardRef>,
    pub title: Option<String>,
    pub use_title: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LootTableRewardRef {
    Item(ItemReward),
    Other(TypeDescriminatedLootTableRewardRef),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemReward {
    pub count: Option<i32>,
    pub item: Item,
    pub random_bonus: Option<i32>,
    pub weight: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TypeDescriminatedLootTableRewardRef {
    Command {
        command: String,
        elevate_perms: bool,
        silent: bool,
        weight: Option<f32>,
    },
    XpLevels {
        xp_levels: i32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub default_autoclaim_rewards: String,
    pub default_consume_items: bool,
    pub default_quest_disable_jei: bool,
    pub default_quest_shape: String,
    pub default_reward_team: bool,
    pub detection_delay: i32,
    pub disable_gui: bool,
    pub emergency_items_cooldown: i32,
    pub grid_scale: f64,
    pub icon: String,
    pub lock_message: String,
    pub loot_crate_no_drop: LootCrateNoDrop,
    pub pause_game: bool,
    pub progression_mode: String,
    pub title: String,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LootCrateNoDrop {
    pub boss: i32,
    pub monster: i32,
    pub passive: i32,
}
