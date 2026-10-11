pub struct Gauge { pub level: u32 }
pub struct Beacon { pub depth: u32 }
pub fn observe() -> (u32,u32) { (Gauge { level: 13 }.level, Beacon { depth: 71 }.depth) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_inherited_unique");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
