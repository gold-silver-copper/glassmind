pub fn aggregate(xs: &[i32]) -> i32 { return xs.iter().copied().filter(|v| *v < 0).sum() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(aggregate(&[-7,2,4,9]),15); assert_eq!(aggregate(&[-2,-3]),0); assert_eq!(aggregate(&[]),0); } }
