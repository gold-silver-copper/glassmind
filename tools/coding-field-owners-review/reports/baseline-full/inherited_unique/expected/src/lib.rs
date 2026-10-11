pub struct Gauge { pub reading: u32 }
pub struct Beacon { pub depth: u32 }
pub fn observe() -> (u32,u32) { (Gauge { reading: 13 }.reading, Beacon { depth: 71 }.depth) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_inherited_unique");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
