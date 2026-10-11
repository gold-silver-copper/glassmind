fn observe() -> (u32,u32) { (field_review_probe::Gauge { level: 13 }.level, field_review_probe::Beacon { level: 71 }.level) }
fn main() { let _ = observe(); }
#[test] fn both_owners() -> Result<(), String> { println!("REVIEW_BEHAVIOR_lib_bin_paths"); if observe() != (13, 71) { return Err("bin owners changed".into()); } Ok(()) }
