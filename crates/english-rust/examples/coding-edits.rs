//! Small typed edit driver for independently testing the language/world bridge.
//! Usage: coding-edits do CRATE 'Rename `Owner::Variant` to `New`'
use std::path::Path;

use glassmind_english::English;
use glassmind_meaning::interface::{Language, Understanding, World};
use glassmind_meaning::{Action, Thing, Utterance};
use glassmind_rules::Switches;
use glassmind_world_rust::{Rust, RustWorld};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [command, root, request] = args.as_slice() else {
        eprintln!("usage: coding-edits do CRATE REQUEST");
        return;
    };
    if command != "do" {
        return;
    }
    let switches = Switches::all_on();
    let world = RustWorld::open(&[("dev".into(), Path::new(root))], &switches);
    let english: English<Rust> = English::new(glassmind_english_rust::rules(), &switches);
    let Understanding::Readings(readings) = english.understand(request) else {
        println!("Unsupported language");
        return;
    };
    let mut actions = Vec::new();
    for reading in readings {
        if reading.edits != 0 || !reading.discarded.is_empty() {
            continue;
        }
        if let Utterance::Command(Action::Rename {
            mut thing,
            kind,
            name,
        }) = reading.meaning
        {
            if let Thing::Named(wanted) = &thing {
                let found = world.lookup(wanted, &mut Vec::new());
                let [item] = found.as_slice() else {
                    continue;
                };
                thing = Thing::Item(item.clone());
            }
            let action = Action::Rename { thing, kind, name };
            if !actions.contains(&action) {
                actions.push(action);
            }
        }
    }
    let [action] = actions.as_slice() else {
        println!("Unsupported or ambiguous request");
        return;
    };
    match world.propose(action, &mut Vec::new()) {
        Ok(proposal) => {
            println!("Proposal: {proposal:?}");
            println!("Apply: {:?}", world.apply(&proposal, &mut Vec::new()));
        }
        Err(trouble) => println!("Refused: {trouble:?}"),
    }
}
