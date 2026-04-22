#[macro_use]extern crate num_derive;

use regex::Regex;

fn main() {
    let state = formats::heracles::State::from_zip("heracles.zip");

    println!("Heracles Groups: {:?}", state.groups);
    println!("Heracles Quests: {:?}", state.quests.len());
    println!(
        "Heracles Quest Dependency Graph Has Cycles: {:?}",
        state.quest_dependency_graph.cycle_detected()
    );

    println!("Hello, world!");

    let ftb2 = formats::ftbquests::State::from_zip("ftbquests-2.zip");
}

mod formats {
    pub mod heracles {
        use std::{
            collections::{HashMap, HashSet},
            fs::File,
            io::Read,
        };

        use regex::Regex;
        use serde::Deserialize;
        use topo_sort::TopoSort;
        use zip::ZipArchive;

        pub struct State {
            pub groups: Vec<String>,
            // Filename/QuestData pair
            pub quests: HashMap<String, Quest>,
            pub quest_dependency_graph: TopoSort<String>,
        }

        impl State {
            pub fn from_zip(path: &str) -> Self {
                // Heracles zip format:
                // - quests/page/<quest_name>.json
                // - groups.txt

                // Extract the groups from the zip
                let file = File::open(path).unwrap();
                let reader = std::io::BufReader::new(file);
                let mut zip = ZipArchive::new(reader).unwrap();

                let mut str_buf = String::new();
                let mut groups: Vec<String> = vec![];
                let mut quests: HashMap<String, Quest> = HashMap::new();
                let mut toposort = TopoSort::new();
                let quest_re = Regex::new(r"^heracles\/quests\/.+\/(.+).json$").unwrap();
                let mut all_types: HashSet<String> = HashSet::new();

                for i in 0..zip.len() {
                    str_buf.clear();

                    let mut file = zip.by_index(i).unwrap();
                    if file.name().ends_with("groups.txt") {
                        file.read_to_string(&mut str_buf).unwrap();
                        groups = str_buf.lines().map(|s| s.to_string()).collect();
                    }

                    if file.name().ends_with(".json") {
                        // Process the filename to get a clean quest name.
                        //  heracles/quests/adastra/A Dream.json -> A Dream
                        let filename = quest_re.captures(file.name()).unwrap()[1].to_string();

                        file.read_to_string(&mut str_buf).unwrap();
                        let quest = serde_json::from_str::<Quest>(&str_buf);
                        match quest {
                            Ok(q) => {
                                toposort.insert(filename.clone(), q.dependencies.clone());

                                if !all_types.contains(&q.display.icon._type) {
                                    all_types.insert(q.display.icon._type.clone());
                                }

                                quests.insert(filename, q);
                            }
                            Err(e) => {
                                println!("Failed to parse quest {}: {}", filename, e);
                                continue;
                            }
                        };
                    }
                }

                Self { groups, quests, quest_dependency_graph: toposort }
            }
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
    }

    pub mod ftbquests {
        use std::{
            collections::{HashMap, HashSet},
            fs::File,
            hash::{DefaultHasher, Hash, Hasher},
            io::Read,
        };

        use bimap::BiHashMap;
        use num_derive::FromPrimitive;
        use regex::Regex;
        use serde::{Deserialize, Serialize};
        use topo_sort::TopoSort;
        use zip::ZipArchive;

        pub struct State {
            pub metadata: Metadata,
            pub groups: Vec<Group>,
            pub pages: Vec<Page>,
            pub quest_dependency_graph: TopoSort<String>,
        }

        impl State {
            pub fn from_heracles(state: super::heracles::State) -> Self {
                let mut page_map: HashMap<String, HashSet<Quest>> = HashMap::new();
                let mut quest_map: HashMap<String, Quest> = HashMap::new();
                let mut id_map: BiHashMap<String, String> = BiHashMap::new();

                for (filename, quest) in state.quests {
                    let quest_id = Self::new_id(filename.clone());

                    let fquest = Quest {
                        id: quest_id.clone(),
                        description: quest.display.description,
                        subtitle: quest
                            .display
                            .subtitle
                            .map(|s| s.text.or(s.translate))
                            .flatten(),
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
                        id_map.insert(key.clone(), Self::new_id(key.clone()));
                        // We need to build a temporary map of which things go on each page
                        // And since the key of the display group is what pageId they'll go to
                    }
                }

                unimplemented!()
            }

            pub fn new_id(old: String) -> String {
                let mut s = DefaultHasher::new();
                old.hash(&mut s);
                format!("{:X}", s.finish())
            }

            pub fn from_zip(path: &str) -> Self {
                // FTB Quests Zip Format:
                // - quests/data.snbt - Metadata
                // - quests/chapter_groups.snbt - "chapter_groups": List<Group>
                // - quests/chapters/<chapter_name>.snbt - Page

                // Extract the groups from the zip
                let file = File::open(path).unwrap();
                let reader = std::io::BufReader::new(file);
                let mut zip = ZipArchive::new(reader).unwrap();
                let mut metadata: Option<Metadata> = None;
                let mut chapter_groups: Option<ChapterGroups> = None;
                let mut pages: Vec<Page> = vec![];
                let mut loot_tables: HashMap<i64, LootTable> = HashMap::new();
                let mut toposort: TopoSort<String> = TopoSort::new();
                let re_comma_places: Regex = Regex::new(r"[^\:\{\[,]$").unwrap();
                let re_trailing_commas: Regex = Regex::new(r"(?m),(\s*)([}\]])").unwrap();
                let re_remove_last_comma: Regex = Regex::new(r"(?m)^},$").unwrap();

                let mut str_buf = String::new();

                for i in 0..zip.len() {
                    str_buf.clear();

                    let mut file = zip.by_index(i).unwrap();
                    if file.name().ends_with("data.snbt") {
                        file.read_to_string(&mut str_buf).unwrap();
                        str_buf = Self::fixup_ftb_snbt(
                            &str_buf,
                            &re_comma_places,
                            &re_trailing_commas,
                            &re_remove_last_comma,
                        );
                        println!("Parsing metadata");
                        match quartz_nbt::snbt::parse(&str_buf) {
                            Ok(m) => metadata = Some(Metadata::from(&m)),
                            Err(e) => println!("Failed to parse metadata: {}", e),
                        };
                    }

                    if file.name().ends_with("chapter_groups.snbt") {
                        file.read_to_string(&mut str_buf).unwrap();
                        str_buf = Self::fixup_ftb_snbt(
                            &str_buf,
                            &re_comma_places,
                            &re_trailing_commas,
                            &re_remove_last_comma,
                        );
                        println!("Parsing chapter groups");
                        match quartz_nbt::snbt::parse(&str_buf) {
                            Ok(m) => chapter_groups = Some(ChapterGroups::from(&m)),
                            Err(e) => println!("Failed to parse chapter groups: {}", e),
                        }
                    }

                    if file.name().contains("chapters/") && file.name().ends_with(".snbt") {
                        file.read_to_string(&mut str_buf).unwrap();
                        str_buf = Self::fixup_ftb_snbt(
                            &str_buf,
                            &re_comma_places,
                            &re_trailing_commas,
                            &re_remove_last_comma,
                        );

                        match quartz_nbt::snbt::parse(&str_buf) {
                            Ok(m) => {
                                println!("Parsing page {}", file.name());
                                let page = Page::from(&m);
                                for quest in &page.quests {
                                    toposort.insert(quest.id.clone(), quest.dependencies.clone());
                                }
                                pages.push(page);
                            }
                            Err(e) => println!("Failed to parse page {}: {}", file.name(), e),
                        }
                    }

                    if file.name().contains("reward_tables/") && file.name().ends_with(".snbt") {
                        file.read_to_string(&mut str_buf).unwrap();
                        str_buf = Self::fixup_ftb_snbt(
                            &str_buf,
                            &re_comma_places,
                            &re_trailing_commas,
                            &re_remove_last_comma,
                        );
                        match quartz_nbt::snbt::parse(&str_buf) {
                            Ok(m) => {
                                println!("Parsing loot table {}", file.name());
                                let table = LootTable::from(&m);
                                loot_tables
                                    .insert(i64::from_str_radix(&table.id, 16).unwrap(), table);
                            }
                            Err(e) => println!("Failed to parse table {}: {}", file.name(), e),
                        }
                    }
                }

                println!("Parsed {} pages", pages.len());
                println!("Parsed {} quests", toposort.len());
                println!("Parsed {} loot tables", loot_tables.len());

                Self {
                    metadata: metadata.unwrap(),
                    groups: chapter_groups.unwrap().chapter_groups,
                    pages,
                    quest_dependency_graph: toposort,
                }
            }

            fn fixup_ftb_snbt(
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
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Group {
            pub id: String,
            pub title: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ChapterGroups {
            chapter_groups: Vec<Group>,
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

        pub mod transform {
            use quartz_nbt::{NbtCompound, NbtList, NbtTag};

            impl From<&NbtCompound> for super::Metadata {
                fn from(val: &NbtCompound) -> Self {
                    super::Metadata {
                        default_autoclaim_rewards: val
                            .get::<_, &str>("default_autoclaim_rewards")
                            .unwrap()
                            .to_string(),
                        default_consume_items: val.get::<_, &str>("default_consume_items").unwrap() ==
                            "true",
                        default_quest_disable_jei: val
                            .get::<_, &str>("default_quest_disable_jei")
                            .unwrap() ==
                            "true",
                        default_quest_shape: val
                            .get::<_, &str>("default_quest_shape")
                            .unwrap()
                            .to_string(),
                        default_reward_team: val.get::<_, &str>("default_reward_team").unwrap() ==
                            "true",
                        detection_delay: val.get::<_, i32>("detection_delay").unwrap(),
                        disable_gui: val.get::<_, &str>("disable_gui").unwrap() == "true",
                        emergency_items_cooldown: val
                            .get::<_, i32>("emergency_items_cooldown")
                            .unwrap(),
                        grid_scale: val.get::<_, f64>("grid_scale").unwrap(),
                        icon: val.get::<_, &str>("icon").unwrap().to_string(),
                        lock_message: val.get::<_, &str>("lock_message").unwrap().to_string(),
                        pause_game: val.get::<_, &str>("pause_game").unwrap() == "true",
                        progression_mode: val
                            .get::<_, &str>("progression_mode")
                            .unwrap()
                            .to_string(),
                        title: val.get::<_, &str>("title").unwrap().to_string(),
                        version: val.get::<_, i32>("version").unwrap(),
                        loot_crate_no_drop: super::LootCrateNoDrop {
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

            impl From<&NbtCompound> for super::ChapterGroups {
                fn from(val: &NbtCompound) -> Self {
                    let mut chapter_groups = vec![];
                    for group in val
                        .get::<_, &NbtList>("chapter_groups")
                        .unwrap()
                        .iter_map::<&NbtCompound>()
                    {
                        let igroup = group.unwrap();
                        chapter_groups.push(super::Group {
                            id: igroup.get::<_, &str>("id").unwrap().to_string(),
                            title: igroup.get::<_, &str>("title").unwrap().to_string(),
                        });
                    }
                    super::ChapterGroups { chapter_groups }
                }
            }

            impl From<&NbtCompound> for super::Page {
                fn from(val: &NbtCompound) -> Self {
                    super::Page {
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

            impl From<&NbtCompound> for super::Quest {
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

                    super::Quest {
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

            impl TryFrom<&NbtCompound> for super::RewardRef {
                type Error = String;

                fn try_from(value: &NbtCompound) -> Result<Self, String> {
                    let reward_type = value.get::<_, &str>("type").unwrap();
                    match reward_type {
                        "item" => Ok(super::RewardRef::Item {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            item: value.get::<_, &NbtTag>("item").unwrap().into(),
                            icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
                        }),
                        "xp_levels" => Ok(super::RewardRef::XpLevels {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            xp_levels: value.get::<_, i32>("xp_levels").unwrap(),
                        }),
                        "xp" => Ok(super::RewardRef::Xp {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            xp: value.get::<_, i32>("xp").unwrap(),
                        }),
                        "choice" => Ok(super::RewardRef::Choice {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            table_id: value.get::<_, i64>("table_id").ok(),
                            exclude_from_claim_all: value
                                .get::<_, &str>("exclude_from_claim_all")
                                .unwrap() ==
                                "true",
                            icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
                        }),
                        "loot" => Ok(super::RewardRef::Loot {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            table_id: value.get::<_, i64>("table_id").unwrap(),
                            exclude_from_claim_all: value
                                .get::<_, &str>("exclude_from_claim_all")
                                .unwrap() ==
                                "true",
                            icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
                        }),
                        "random" => Ok(super::RewardRef::Random {
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

            impl TryFrom<&NbtCompound> for super::TaskRef {
                type Error = String;

                fn try_from(value: &NbtCompound) -> Result<Self, String> {
                    let task_type = value.get::<_, &str>("type").unwrap();
                    match task_type {
                        "checkmark" => Ok(super::TaskRef::Checkmark {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
                        }),
                        "item" => Ok(super::TaskRef::Item {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            item: value.get::<_, &NbtTag>("item").unwrap().into(),
                            icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
                        }),
                        "structure" => Ok(super::TaskRef::Structure {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            structure: value.get::<_, &str>("structure").unwrap().to_string(),
                            icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
                        }),
                        "dimension" => Ok(super::TaskRef::Dimension {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            dimension: value.get::<_, &str>("dimension").unwrap().to_string(),
                            icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
                        }),
                        "advancement" => Ok(super::TaskRef::Advancement {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            advancement: value.get::<_, &str>("advancement").unwrap().to_string(),
                            criterion: value
                                .get::<_, &str>("criterion")
                                .ok()
                                .map(|s| s.to_string()),
                            icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
                        }),
                        "kill" => Ok(super::TaskRef::Kill {
                            id: value.get::<_, &str>("id").unwrap().to_string(),
                            entity: value.get::<_, &str>("entity").unwrap().to_string(),
                            value: value.get::<_, i32>("value").ok(),
                            icon: value.get::<_, &NbtTag>("icon").ok().map(|s| s.into()),
                        }),
                        "observation" => Ok(super::TaskRef::Observation {
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

            impl From<&NbtTag> for super::Item {
                fn from(value: &NbtTag) -> Self {
                    match value {
                        NbtTag::String(s) => super::Item::Simple(s.to_string()),
                        NbtTag::Compound(c) => super::Item::Compound(c.into()),
                        _ => panic!(
                            "Invalid item tag type: expected string or compound, got {:?}",
                            value
                        ),
                    }
                }
            }

            impl From<&NbtCompound> for super::ItemRef {
                fn from(value: &NbtCompound) -> Self {
                    super::ItemRef {
                        count: value.get::<_, i32>("count").ok(),
                        id: value.get::<_, &str>("id").unwrap().to_string(),
                        tag: value.get::<_, &NbtCompound>("tag").ok().map(|c| c.into()),
                    }
                }
            }

            impl From<&NbtCompound> for super::ItemTag {
                fn from(value: &NbtCompound) -> Self {
                    super::ItemTag {
                        color: value.get::<_, i32>("Color").ok(),
                        loot: value.get::<_, &str>("Loot").ok().map(|s| s.to_string()),
                        name: value.get::<_, &str>("Name").ok().map(|s| s.to_string()),
                        _type: value.get::<_, &str>("Type").ok().map(|s| s.to_string()),
                        damage: value.get::<_, i32>("Damage").ok(),
                        icon: value.get::<_, &str>("icon").ok().map(|s| s.to_string()),
                    }
                }
            }

            impl From<&NbtCompound> for super::LootTable {
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

            impl From<&NbtCompound> for super::LootTableRewardRef {
                fn from(value: &NbtCompound) -> Self {
                    match value.get::<_, &str>("type") {
                        Ok(_) => Self::Other(value.try_into().unwrap()),
                        Err(_) => Self::Item(value.into()),
                    }
                }
            }

            impl TryFrom<&NbtCompound> for super::TypeDescriminatedLootTableRewardRef {
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

            impl From<&NbtCompound> for super::ItemReward {
                fn from(value: &NbtCompound) -> Self {
                    Self {
                        count: value.get::<_, i32>("count").ok(),
                        item: value.get::<_, &NbtTag>("item").map(|it| it.into()).unwrap(),
                        random_bonus: value.get::<_, i32>("random_bonus").ok(),
                        weight: value.get::<_, f32>("weight").ok(),
                    }
                }
            }
        }
    }
}
