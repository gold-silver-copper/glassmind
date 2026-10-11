pub struct Gauge { pub reading: u32 }
pub mod inner {
pub fn read(g: &super::Gauge, b: &crate::Beacon) -> (u32,u32) { (g.reading, b.level) }
}
