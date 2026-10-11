pub struct Gauge { pub level: u32 }
pub mod inner {
pub fn read(g: &super::Gauge, b: &crate::Beacon) -> (u32,u32) { (g.level, b.level) }
}
