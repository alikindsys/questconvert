#[macro_use] extern crate num_derive;

pub mod formats;
pub mod trans;

fn main() {
    let state = formats::heracles::State::from_zip("heracles.zip");

    println!("Heracles Groups: {:?}", state.groups);
    println!("Heracles Quests: {:?}", state.quests.len());
    println!(
        "Heracles Quest Dependency Graph Has Cycles: {:?}",
        state.quest_dependency_graph.cycle_detected()
    );

    let _ftb2 = formats::ftbquests::State::from_zip("ftbquests-2.zip");
}
