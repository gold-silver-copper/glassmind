//! The oracle's definitions on a tiny source.
#![cfg(test)]

use std::collections::BTreeSet;
use std::fs;

use glassmind_answer::Response;
use glassmind_oracle::source::{self, Source};
use glassmind_oracle::tasks::{self, Task};

const GAME: &str = r#"
pub struct Velocity { pub dx: f32, pub dy: f32 }
pub struct Player { pub hp: u32, pub speed: Vec<Velocity> }
pub struct Enemy { pub hp: i32, pub boss_hp: u32 }
pub struct Body<'a> { pub v: Option<physics::Velocity>, pub label: &'a str }

pub fn spawn_wave(level: u32) -> usize {
    let p = Player { hp: 3, speed: Vec::new() };
    helper(p.hp) + level as usize
}

fn helper(x: u32) -> usize {
    let again = || helper(x - 1);
    fn nested() { spawn_wave(1); }
    x as usize
}

pub fn tick(e: &mut Enemy) {
    println!("{}", e.boss_hp);
    e.hp -= 1;
    let _ = crate::game::spawn_wave(2);
}
"#;

fn fixture() -> Source {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("oracle-fixture");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/game.rs"), GAME).unwrap();
    source::load("fx", &root).unwrap()
}

fn task(source: &Source, kind: &str, name: &str) -> Option<Task> {
    let id = format!("fx/{kind}/{name}");
    tasks::candidates(source, kind, 0)
        .into_iter()
        .map(|c| c.task)
        .find(|t| t.id == id)
}

fn item_names(response: &Response) -> BTreeSet<String> {
    match response {
        Response::Items { items } => items.iter().map(|k| k.name.clone()).collect(),
        other => panic!("not items: {other:?}"),
    }
}

#[test]
fn callers_follow_own_bodies_only() {
    let src = fixture();
    // `nested`'s call belongs to `nested`, not to `helper`; the closure's
    // call is `helper`'s own, so `helper` is recursive.
    let callers = task(&src, "callers", "spawn_wave").unwrap();
    assert_eq!(
        item_names(&callers.expected),
        ["nested", "tick"].map(String::from).into()
    );
    let callers = task(&src, "callers", "helper").unwrap();
    assert_eq!(
        item_names(&callers.expected),
        ["helper", "spawn_wave"].map(String::from).into()
    );
}

#[test]
fn renames_and_field_types() {
    let src = fixture();
    // `boss_hp` also occurs inside `println!`, so renaming it is not exact.
    assert!(task(&src, "rename_field", "Enemy.boss_hp").is_none());
    // `hp` belongs to two structs: no rename, but an ambiguous one.
    assert!(task(&src, "rename_field", "Player.hp").is_none());
    assert!(task(&src, "ambiguous_rename_field", "hp").is_some());
    let speed = task(&src, "rename_field", "Player.speed").unwrap();
    let Response::Edit { changes } = &speed.expected else {
        panic!()
    };
    let at: Vec<(u32, u32)> = changes.iter().map(|c| (c.line, c.column)).collect();
    assert_eq!(at, [(3, 37), (8, 28)]);

    // `Vec<Velocity>` and `Option<physics::Velocity>` both mention `Velocity`;
    // the lifetime `'a` is not a type.
    let vel = task(&src, "structs_with_field_type", "Velocity").unwrap();
    assert_eq!(
        item_names(&vel.expected),
        ["Body", "Player"].map(String::from).into()
    );
    assert!(task(&src, "structs_with_field_type", "a").is_none());
}
