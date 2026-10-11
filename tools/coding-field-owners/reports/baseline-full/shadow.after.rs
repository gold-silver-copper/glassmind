pub struct Parcel { pub count: u32 }
pub struct Ledger { pub count: u32 }
fn read(x: Parcel) -> u32 { let x = Ledger { count: x.count }; x.count }
pub fn observe() -> (u32,u32) { (read(Parcel { count: 3 }), Ledger { count: 9 }.count) }
// count stays in prose
pub const NOTE: &str = "count";
#[test] fn behavior() -> Result<(), String> { println!("OWNER_BEHAVIOR_shadow"); if observe() == (3,9) { Ok(()) } else { Err("behavior changed".into()) } }
