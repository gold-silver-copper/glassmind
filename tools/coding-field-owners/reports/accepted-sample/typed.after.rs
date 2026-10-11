pub struct CrateBox { pub quantity: u32 }
pub struct Account { pub amount: u32 }
fn read(x: CrateBox) -> u32 { x.quantity }
pub fn observe() -> (u32,u32) { (read(CrateBox { quantity: 17 }), Account { amount: 41 }.amount) }
// amount stays in prose
pub const NOTE: &str = "amount";
#[test] fn behavior() -> Result<(), String> { println!("OWNER_BEHAVIOR_typed"); if observe() == (17,41) { Ok(()) } else { Err("behavior changed".into()) } }
