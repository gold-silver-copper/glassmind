pub mod metrics;
pub struct Beacon { pub level: u32 }
pub fn observe() -> (u32,u32) { metrics::inner::read(&metrics::Gauge { level: 13 }, &crate::Beacon { level: 71 }) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_qualified_files");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
