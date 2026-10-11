pub struct Vessel { pub level: u32 }
pub struct Bucket { pub level: u32 }
fn read(x: &Vessel) -> u32 { x.level }
pub fn observe() -> (u32,u32) { (read(&Vessel { level: 6 }), Bucket { level: 29 }.level) }
// level stays in prose
pub const NOTE: &str = "level";
#[test] fn behavior() -> Result<(), String> { println!("OWNER_BEHAVIOR_reference"); if observe() == (6,29) { Ok(()) } else { Err("behavior changed".into()) } }
