pub struct Parcel { pub count: u32 }
pub struct Ledger { pub count: u32 }
pub fn observe() -> (u32,u32) { (Parcel { count: 3 }.count, Ledger { count: 9 }.count) }
// count stays in prose
pub const NOTE: &str = "count";
#[test] fn behavior() -> Result<(), String> { println!("OWNER_BEHAVIOR_duplicate-owner"); if observe() == (3,9) { Ok(()) } else { Err("behavior changed".into()) } }
mod other { pub struct Parcel { pub count: u32 } }
