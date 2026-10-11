pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
pub fn observe() -> (u32,u32) { (Gauge { level: 13 }.r#level, Beacon { level: 71 }.level) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_raw_spelling");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
