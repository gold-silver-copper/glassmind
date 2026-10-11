pub fn target(xs: &[i32]) -> i32 { return xs.iter().copied().filter(|v| *v % 2 == 0).sum() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(target(&[1,2]),3); assert_eq!(target(&[1,2]),4); } }
