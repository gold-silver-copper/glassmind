pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
pub fn observe() -> (u32,u32) { (Gauge { level: 13 }.level, Beacon { level: 71 }.level) }
pub fn read(r#g: Gauge) -> u32 { g.level }
#[test] fn both_owners() -> Result<(), String> { println!("REVIEW_BEHAVIOR_raw_receiver_binding"); if observe() != (13, 71) { return Err("owners changed".into()); } Ok(()) }
