pub mod model {
pub struct Metric { pub reading: u32 }
pub struct Counter { pub value: u32 }
pub fn observe() -> (u32,u32) { (Metric { reading: 11 }.reading, Counter { value: 33 }.value) }
}
pub fn observe() -> (u32,u32) { model::observe() }
// value stays in prose
pub const NOTE: &str = "value";
#[test] fn behavior() -> Result<(), String> { println!("OWNER_BEHAVIOR_nested"); if observe() == (11,33) { Ok(()) } else { Err("behavior changed".into()) } }
