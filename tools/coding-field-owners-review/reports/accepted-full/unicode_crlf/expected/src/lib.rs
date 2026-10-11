pub struct Gauge { pub reading: u32 }
pub struct Beacon { pub level: u32 }
pub fn observe() -> (u32,u32) { let café = "🌡 level"; let _ = café; (Gauge { reading: 13 }.reading, Beacon { level: 71 }.level) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_unicode_crlf");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
