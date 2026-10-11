pub fn aggregate(xs: &[i32]) -> i32 { /* keep */ return xs.iter().copied().filter(|v| *v % 2 != 0).sum() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(aggregate(&[-7,2,4,9]),8); assert_eq!(aggregate(&[]),0); assert_eq!(aggregate(&[-3,-2]),-5); } }
