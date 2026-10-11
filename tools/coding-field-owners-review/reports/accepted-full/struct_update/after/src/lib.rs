pub struct Gauge { pub reading: u32 }
pub struct Beacon { pub level: u32 }
fn read(g: Gauge, b: Beacon) -> (u32,u32) { (Gauge { ..g }.reading, Beacon { ..b }.level) }
pub fn observe() -> (u32,u32) { read(Gauge { reading: 13 }, Beacon { level: 71 }) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_struct_update");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
