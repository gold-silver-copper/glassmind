pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
fn pair(g: &&Gauge, b: &Beacon) -> (u32,u32) { ((g).level, (b).level) }
pub fn observe() -> (u32,u32) { pair(&&Gauge { level: Beacon { level: 13 }.level }, &Beacon { level: Gauge { level: 71 }.level }) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_nested_literal");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
