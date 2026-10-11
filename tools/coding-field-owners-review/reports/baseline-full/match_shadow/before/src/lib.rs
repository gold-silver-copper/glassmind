pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
fn read(g: Gauge, b: Beacon) -> (u32,u32) { let first = g.level; match b { g => (first, g.level) } }
pub fn observe() -> (u32,u32) { read(Gauge { level: 13 }, Beacon { level: 71 }) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_match_shadow");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
