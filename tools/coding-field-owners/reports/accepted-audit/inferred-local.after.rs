pub struct Parcel { pub count: u32 }
pub struct Ledger { pub count: u32 }
pub fn observe() -> (u32,u32) { (read_local(), Ledger { count: 9 }.count) }
// count stays in prose
pub const NOTE: &str = "count";
#[test] fn behavior() -> Result<(), String> { println!("OWNER_BEHAVIOR_inferred-local"); if observe() == (3,9) { Ok(()) } else { Err("behavior changed".into()) } }
fn read_local() -> u32 { let x = Parcel { count: 3 }; x.count }
