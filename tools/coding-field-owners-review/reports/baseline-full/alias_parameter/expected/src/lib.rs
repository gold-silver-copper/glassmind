pub struct Gauge { pub level: u32 }
pub struct Beacon { pub level: u32 }
type LocalGauge = Gauge;
fn read(g: LocalGauge) -> u32 { g.level }
pub fn observe() -> (u32,u32) { (read(Gauge { level: 13 }), Beacon { level: 71 }.level) }

#[test] fn both_owners() -> Result<(), String> {
    println!("REVIEW_BEHAVIOR_alias_parameter");
    if observe() != (13, 71) { return Err("owners changed".into()); }
    Ok(())
}
