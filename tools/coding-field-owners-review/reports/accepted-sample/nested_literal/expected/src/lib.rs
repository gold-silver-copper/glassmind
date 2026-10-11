pub struct Gauge { pub reading: u32 }
pub struct Beacon { pub level: u32 }
fn pair(g: &&Gauge, b: &Beacon) -> (u32,u32) { ((g).reading, (b).level) }
pub fn observe() -> (u32,u32) { pair(&&Gauge { reading: Beacon { level: 13 }.level }, &Beacon { level: Gauge { reading: 71 }.reading }) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_nested_literal");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
