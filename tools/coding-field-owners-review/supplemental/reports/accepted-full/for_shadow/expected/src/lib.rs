pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
pub fn observe() -> (u32,u32) { (Gauge { level: 13 }.level, Beacon { level: 71 }.level) }
pub fn loop_value(g: Gauge) -> u32 { let mut n = g.level; for g in [Beacon { level: 1 }] { n += g.level; } n }
#[test] fn both_owners() -> Result<(), String> { println!("REVIEW_BEHAVIOR_for_shadow"); if observe() != (13, 71) { return Err("owners changed".into()); } Ok(()) }
