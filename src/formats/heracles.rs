use std::collections::HashMap;

use serde::Deserialize;
use topo_sort::TopoSort;

pub struct State {
    pub groups: Vec<String>,
    // Filename/QuestData pair
    pub quests: HashMap<String, Quest>,
    pub quest_dependency_graph: TopoSort<String>,
}

#[derive(Debug, Deserialize)]
pub struct Quest {
    // List of filenames of the quests that are required to be completed before this quest can be completed.
    #[serde(default)]
    pub dependencies: Vec<String>,
    // Heracles has the json filename as the id of the quest.
    // Which then is used on the dependencies to refer to other quests.
    // The annoying part is that the ids on ftbquests are random but generated on a specific format.
    pub tasks: HashMap<String, TaskRef>,
    pub display: DisplayInner,
    pub rewards: HashMap<String, RewardsRef>,
    pub settings: QuestSettings,
}

// Why....
#[derive(Debug, Deserialize)]
pub struct TaskInner {
    #[serde(rename = "type")]
    pub _type: String,
    pub item: Option<String>,
    pub amount: Option<i32>,
    pub collection: Option<String>,
    pub title: Option<String>,
    pub icon: Option<Icon>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum TaskRef {
    #[serde(rename = "heracles:check")]
    Check { title: String, icon: Option<Icon> },
    #[serde(rename = "heracles:item")]
    Item {
        item: String,
        amount: Option<i32>,
        collection: Option<String>,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:advancement")]
    Advancement {
        advancements: Vec<String>,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:structure")]
    Structure {
        structures: String,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:changed_dimension")]
    ChangeDimension {
        from: String,
        to: String,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:item_interaction")]
    ItemInteraction {
        item: String,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:block_interaction")]
    BlockInteraction {
        block: String,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:kill_entity")]
    KillEntity {
        entity: EntityInner,
        amount: Option<i32>,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:biome")]
    Biome {
        biomes: String,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:composite")]
    Composite {
        tasks: HashMap<String, TaskRef>,
        amount: Option<i32>,
        title: Option<String>,
        icon: Option<Icon>,
    },
}

#[derive(Debug, Deserialize)]
pub struct EntityInner {
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RewardsRef {
    #[serde(rename = "heracles:item")]
    Item {
        item: TagItemRef,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:xp")]
    Xp {
        #[serde(rename = "xptype")]
        xp_type: String,
        amount: i32,
        title: Option<String>,
        icon: Option<Icon>,
    },
    #[serde(rename = "heracles:selectable")]
    Selectable {
        rewards: HashMap<String, RewardsRef>,
        amount: Option<i32>,
        title: Option<String>,
        icon: Option<Icon>,
    },
}

#[derive(Debug, Deserialize)]
pub struct QuestSettings {
    #[serde(rename = "unlockNotification")]
    #[serde(default)]
    pub unlock_notification: bool,
    #[serde(rename = "showDependencyArrow")]
    #[serde(default)]
    pub show_dependency_arrow: bool,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub individual_progress: bool,
    #[serde(default)]
    pub hidden: String,
}

#[derive(Debug, Deserialize)]
pub struct GroupsInner {
    pub position: [i32; 2],
}

#[derive(Debug, Deserialize)]
pub struct TitleInner {
    pub text: Option<String>,
    pub translate: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TitleRef {
    Simple(String),
    Compound(TitleInner),
}

#[derive(Debug, Deserialize)]
pub struct Icon {
    pub item: TagItemRef,
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TagItemRef {
    TagRef(String),
    ItemRef(ItemRef),
}

#[derive(Debug, Deserialize)]
pub struct ItemRef {
    pub id: String,
    pub count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SubtitleInner {
    pub text: Option<String>,
    pub translate: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DisplayInner {
    pub subtitle: Option<SubtitleInner>,
    pub description: Vec<String>,
    pub groups: HashMap<String, GroupsInner>,
    pub icon: Icon,
    pub icon_background: Option<String>,
    pub title: Option<TitleRef>,
}
