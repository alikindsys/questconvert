use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
};

use regex::Regex;
use topo_sort::TopoSort;
use zip::ZipArchive;

use crate::{
    formats::{ftbquests, heracles},
    trans::snbt::fixup_ftb_snbt,
};

impl heracles::State {
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
        let mut quests: HashMap<String, heracles::Quest> = HashMap::new();
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
                let quest = serde_json::from_str::<heracles::Quest>(&str_buf);
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

impl ftbquests::State {
    pub fn from_zip(path: &str) -> Self {
        // FTB Quests Zip Format:
        // - quests/data.snbt - Metadata
        // - quests/chapter_groups.snbt - "chapter_groups": List<Group>
        // - quests/chapters/<chapter_name>.snbt - Page

        // Extract the groups from the zip
        let file = File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut zip = ZipArchive::new(reader).unwrap();
        let mut metadata: Option<ftbquests::Metadata> = None;
        let mut chapter_groups: Option<ftbquests::ChapterGroups> = None;
        let mut pages: Vec<ftbquests::Page> = vec![];
        let mut loot_tables: HashMap<i64, ftbquests::LootTable> = HashMap::new();
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
                str_buf = fixup_ftb_snbt(
                    &str_buf,
                    &re_comma_places,
                    &re_trailing_commas,
                    &re_remove_last_comma,
                );
                println!("Parsing metadata");
                match quartz_nbt::snbt::parse(&str_buf) {
                    Ok(m) => metadata = Some(ftbquests::Metadata::from(&m)),
                    Err(e) => println!("Failed to parse metadata: {}", e),
                };
            }

            if file.name().ends_with("chapter_groups.snbt") {
                file.read_to_string(&mut str_buf).unwrap();
                str_buf = fixup_ftb_snbt(
                    &str_buf,
                    &re_comma_places,
                    &re_trailing_commas,
                    &re_remove_last_comma,
                );
                println!("Parsing chapter groups");
                match quartz_nbt::snbt::parse(&str_buf) {
                    Ok(m) => chapter_groups = Some(ftbquests::ChapterGroups::from(&m)),
                    Err(e) => println!("Failed to parse chapter groups: {}", e),
                }
            }

            if file.name().contains("chapters/") && file.name().ends_with(".snbt") {
                file.read_to_string(&mut str_buf).unwrap();
                str_buf = fixup_ftb_snbt(
                    &str_buf,
                    &re_comma_places,
                    &re_trailing_commas,
                    &re_remove_last_comma,
                );

                match quartz_nbt::snbt::parse(&str_buf) {
                    Ok(m) => {
                        println!("Parsing page {}", file.name());
                        let page = ftbquests::Page::from(&m);
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
                str_buf = fixup_ftb_snbt(
                    &str_buf,
                    &re_comma_places,
                    &re_trailing_commas,
                    &re_remove_last_comma,
                );
                match quartz_nbt::snbt::parse(&str_buf) {
                    Ok(m) => {
                        println!("Parsing loot table {}", file.name());
                        let table = ftbquests::LootTable::from(&m);
                        loot_tables.insert(i64::from_str_radix(&table.id, 16).unwrap(), table);
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
}
