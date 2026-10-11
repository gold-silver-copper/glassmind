pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
pub struct Holder { pub inner: Gauge }
pub fn observe() -> (u32,u32) { (Holder { inner: Gauge { level: 13 } }.inner.level, Beacon { level: 71 }.level) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_qualified_member");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
